mod game;
mod menu;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#bevy-canvas".into()),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_camera)
        .init_state::<AppState>()
        .add_plugins(game::GamePlugin)
        .add_plugins(menu::MenuPlugin)
        .run();
}
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
