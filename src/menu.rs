use crate::AppState;
use bevy::prelude::*;

const TITLE_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);

#[derive(Component)]
struct MenuEntity;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(Update, menu_input.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), cleanup_menu);
    }
}

fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MenuEntity,
            BackgroundColor(BG_COLOR),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("404 JUMPER"),
                TextFont { font_size: 60.0, ..default() },
                TextColor(TITLE_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
                MenuEntity,
            ));
            parent.spawn((
                Text::new("Press ENTER to Start"),
                TextFont { font_size: 30.0, ..default() },
                MenuEntity,
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn menu_input(keyboard_input: Res<ButtonInput<KeyCode>>, mouse_input: Res<ButtonInput<MouseButton>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::NumpadEnter) || mouse_input.just_pressed(MouseButton::Left) {
        next_state.set(AppState::InGame);
    }
}
fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for e in &query {
        commands.entity(e).try_despawn();
    }
}
