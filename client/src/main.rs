mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

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
    .add_plugins(UiPlugin)
    .add_systems(PreStartup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
