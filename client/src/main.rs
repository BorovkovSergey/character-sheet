mod character_select;
mod components;
mod events;
mod network;
mod state;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use character_select::CharacterSelectPlugin;
use components::{despawn_active_character, recalculate_effects};
use network::NetworkPlugin;
use state::AppScreen;
use ui::UiPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "D&D Character Sheet".to_string(),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::INFO,
                filter: "wgpu=error,naga=warn".to_string(),
                ..Default::default()
            }),
    )
    .add_plugins(EguiPlugin::default())
    .init_state::<AppScreen>()
    .add_plugins(CharacterSelectPlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins(UiPlugin)
    .add_systems(PreStartup, setup)
    .add_systems(Update, recalculate_effects)
    .add_systems(OnExit(AppScreen::CharacterSheet), despawn_active_character)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
