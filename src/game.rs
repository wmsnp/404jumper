use crate::AppState;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/litemoon.png");
        app.insert_resource(JumpTimer::default())
            .insert_resource(SpawnNextPlatform::default())
            .insert_resource(LastLandedPlatform::default())
            .add_systems(OnEnter(AppState::InGame), setup_game)
            .add_systems(Update, (jump, respawn, update_jump, update_height, spawn_platform, landing, camera).run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), cleanup);
    }
}

#[derive(Component)]
struct Player {
    on_ground: bool,
}

#[derive(Component)]
struct Platform;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct HeightDisplay;

#[derive(Resource, Default)]
struct SpawnNextPlatform {
    should_spawn: bool,
}

#[derive(Resource, Default)]
struct JumpTimer {
    pressed_time: f32,
}
#[derive(Resource, Default)]
struct LastLandedPlatform {
    entity: Option<Entity>,
}

const PLAYER_SIZE: Vec2 = Vec2::new(64.0, 64.0);
const PLATFORM_SIZE: Vec2 = Vec2::new(100.0, 20.0);
fn update_height(player_query: Query<&Transform, With<Player>>, mut text_query: Query<&mut Text, With<HeightDisplay>>, windows: Query<&Window, With<PrimaryWindow>>) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok(mut text) = text_query.single_mut() {
            if let Ok(window) = windows.single() {
                let initial_platform_y = -window.height() / 2.0 + 50.0;
                let initial_player_y = initial_platform_y + PLATFORM_SIZE.y / 2.0 + PLAYER_SIZE.y / 2.0;
                let height = player_transform.translation.y - initial_player_y;
                text.0 = format!("{:.0}", height);
            }
        }
    }
}

fn camera(player_query: Query<&Transform, With<Player>>, mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>, windows: Query<&Window, With<PrimaryWindow>>) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            if let Ok(window) = windows.single() {
                let half_height = window.height() / 2.0;
                let half_width = window.width() / 2.0;
                let player_y_in_camera = player_transform.translation.y - camera_transform.translation.y;
                let player_x_in_camera = player_transform.translation.x - camera_transform.translation.x;
                let top_threshold = half_height - 50.0;
                let bottom_threshold = -half_height + 50.0;
                let left_threshold = -half_width + 50.0;
                let right_threshold = half_width - 50.0;
                if player_y_in_camera > top_threshold {
                    camera_transform.translation.y += player_y_in_camera - top_threshold;
                } else if player_y_in_camera < bottom_threshold {
                    camera_transform.translation.y += player_y_in_camera - bottom_threshold;
                }
                if player_x_in_camera < left_threshold {
                    camera_transform.translation.x += player_x_in_camera - left_threshold;
                } else if player_x_in_camera > right_threshold {
                    camera_transform.translation.x += player_x_in_camera - right_threshold;
                }
            }
        }
    }
}

fn setup_game(
    mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window, With<PrimaryWindow>>, mut last_landed: ResMut<LastLandedPlatform>, mut spawn_flag: ResMut<SpawnNextPlatform>,
) {
    if let Ok(window) = windows.single() {
        let platform_y = -window.height() / 2.0 + 50.0;
        let player_y = platform_y + PLATFORM_SIZE.y / 2.0 + PLAYER_SIZE.y / 2.0;
        commands.spawn((
            Sprite {
                image: asset_server.load("embedded://bevy_404jumper/assets/litemoon.png"),
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            Player { on_ground: true },
            Velocity(Vec3::ZERO),
            Transform::from_xyz(0.0, player_y, 0.0),
            GlobalTransform::default(),
        ));
        let platform_entity = commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.3, 0.3, 0.3),
                    custom_size: Some(PLATFORM_SIZE),
                    ..default()
                },
                Platform,
                Transform::from_xyz(0.0, platform_y, 0.0),
                GlobalTransform::default(),
            ))
            .id();
        commands
            .spawn((Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },))
            .with_children(|parent| {
                parent.spawn((Text::new("Height: "), TextFont { font_size: 24.0, ..default() }));
                parent.spawn((Text::new("0"), TextFont { font_size: 24.0, ..default() }, TextColor(Color::srgb(0.5, 0.5, 0.0)), HeightDisplay));
            });
        last_landed.entity = Some(platform_entity);
        spawn_flag.should_spawn = true;
    }
}

fn respawn(
    mut params: ParamSet<(
        Query<(&mut Transform, &mut Velocity, &mut Player)>,
        Query<&mut Transform, (With<Camera>, Without<Player>)>,
        Query<(Entity, &Transform), With<Platform>>,
    )>,
    last_landed: Res<LastLandedPlatform>, windows: Query<&Window, With<PrimaryWindow>>,
) {
    let platforms: Vec<(Entity, Vec3)> = params.p2().iter().map(|(e, t)| (e, t.translation)).collect();

    if let Ok((mut player_transform, mut velocity, mut player)) = params.p0().single_mut() {
        if let Ok(window) = windows.single() {
            let bottom_limit = -window.height() / 2.0 - PLAYER_SIZE.y;
            if player_transform.translation.y < bottom_limit {
                if let Some(platform_entity) = last_landed.entity {
                    if let Some((_, platform_pos)) = platforms.iter().find(|(entity, _)| *entity == platform_entity) {
                        player_transform.translation.y = platform_pos.y + PLATFORM_SIZE.y / 2.0 + PLAYER_SIZE.y / 2.0;
                        player_transform.translation.x = platform_pos.x;
                        velocity.0 = Vec3::ZERO;
                        player.on_ground = true;
                    }
                } else {
                    player_transform.translation = Vec3::new(0.0, 0.0, 0.0);
                    velocity.0 = Vec3::ZERO;
                    player.on_ground = true;
                }
                if let Ok(mut camera_transform) = params.p1().single_mut() {
                    camera_transform.translation.y = platforms.iter().map(|(_, pos)| pos.y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
                    camera_transform.translation.x = 0.0;
                }
            }
        }
    }
}

fn jump(
    mouse_input: Res<ButtonInput<MouseButton>>, time: Res<Time>, windows: Query<&Window, With<PrimaryWindow>>, touches: Res<Touches>, mut jump_timer: ResMut<JumpTimer>,
    mut player_query: Query<(&mut Player, &mut Velocity, &Transform)>,
) {
    if let Ok((mut player, mut velocity, transform)) = player_query.single_mut() {
        if player.on_ground {
            if mouse_input.just_pressed(MouseButton::Left) || touches.any_just_pressed() {
                jump_timer.pressed_time = 0.0;
            }
            if mouse_input.pressed(MouseButton::Left) || touches.iter().count() > 0 {
                jump_timer.pressed_time += time.delta_secs();
            }
            let mut should_jump = false;
            let mut target_x = transform.translation.x;
            if mouse_input.just_released(MouseButton::Left) {
                if let Ok(window) = windows.single() {
                    if let Some(cursor_pos) = window.cursor_position() {
                        let window_size = Vec2::new(window.width(), window.height());
                        target_x = cursor_pos.x - window_size.x / 2.0;
                        should_jump = true;
                    }
                }
            }
            if touches.any_just_released() {
                if let Some(touch) = touches.iter_just_released().next() {
                    let touch_pos = touch.position();
                    let window = windows.single().unwrap();
                    let window_size = Vec2::new(window.width(), window.height());
                    target_x = touch_pos.x - window_size.x / 2.0;
                    should_jump = true;
                }
            }
            if should_jump {
                velocity.0 = Vec3::new(target_x - transform.translation.x, jump_timer.pressed_time * 2000.0, 0.0);
                player.on_ground = false;
                jump_timer.pressed_time = 0.0;
            }
        }
    }
}

fn update_jump(mut player_query: Query<(&mut Transform, &mut Velocity, &mut Player), With<Player>>, time: Res<Time>) {
    if let Ok((mut transform, mut velocity, player)) = player_query.single_mut() {
        if !player.on_ground {
            velocity.0.y -= 1500.0 * time.delta_secs();
            transform.translation += velocity.0 * time.delta_secs();
        }
    }
}

fn landing(
    mut params: ParamSet<(Query<(&mut Transform, &mut Velocity, &mut Player)>, Query<(Entity, &Transform), With<Platform>>)>, mut spawn_flag: ResMut<SpawnNextPlatform>,
    mut last_landed: ResMut<LastLandedPlatform>, time: Res<Time>,
) {
    let platforms: Vec<(Entity, Vec3)> = params.p1().iter().map(|(e, t)| (e, t.translation)).collect();

    if let Ok((mut player_transform, mut velocity, mut player)) = params.p0().single_mut() {
        if player.on_ground {
            return;
        }
        let player_bottom = player_transform.translation.y - PLAYER_SIZE.y / 2.0;
        let highest_platform_y = platforms.iter().map(|(_, pos)| pos.y).fold(f32::MIN, |a, b| a.max(b));
        for (entity, platform_transform) in platforms {
            let dx = (player_transform.translation.x - platform_transform.x).abs();
            let platform_top = platform_transform.y + PLATFORM_SIZE.y / 2.0;
            let player_bottom_next = player_bottom + velocity.0.y * time.delta_secs();
            if dx < (PLATFORM_SIZE.x + PLAYER_SIZE.x) / 2.0 && player_bottom >= platform_top && player_bottom_next <= platform_top {
                player_transform.translation.y = platform_top + PLAYER_SIZE.y / 2.0;
                player.on_ground = true;
                velocity.0 = Vec3::ZERO;
                if platform_transform.y == highest_platform_y {
                    spawn_flag.should_spawn = true;
                    last_landed.entity = Some(entity);
                }
                break;
            }
        }
    }
}

fn spawn_platform(
    mut commands: Commands, mut spawn_flag: ResMut<SpawnNextPlatform>, platform_query: Query<&Transform, With<Platform>>, last_landed: Res<LastLandedPlatform>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !spawn_flag.should_spawn {
        return;
    }
    if let Ok(window) = windows.single() {
        let highest_y = platform_query.iter().map(|t| t.translation.y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
        let last_x = last_landed.entity.and_then(|e| platform_query.get(e).ok()).map(|t| t.translation.x).unwrap_or(0.0);
        let half_width = window.width() / 2.0;
        let min_x = -half_width + PLATFORM_SIZE.x / 2.0;
        let max_x = half_width - PLATFORM_SIZE.x / 2.0;

        let mut x;
        loop {
            x = rand::random::<f32>() * (max_x - min_x) + min_x;
            if (x - last_x).abs() > PLATFORM_SIZE.x {
                break;
            }
        }

        let min_y_offset = 50.0;
        let max_y_offset = 150.0;
        let y = highest_y + min_y_offset + rand::random::<f32>() * (max_y_offset - min_y_offset);

        commands.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(PLATFORM_SIZE),
                ..default()
            },
            Platform,
            Transform::from_xyz(x, y, 0.0),
            GlobalTransform::default(),
        ));

        spawn_flag.should_spawn = false;
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, Or<(With<Player>, With<Platform>)>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
