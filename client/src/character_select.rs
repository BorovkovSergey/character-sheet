use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use shared::Character;
use ui_widgets::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};

use crate::components::spawn_character;
use crate::network::ClientTraitRegistry;
use crate::state::AppScreen;

/// Holds the list of available characters received from the server.
#[derive(Debug, Clone, Resource, Default)]
pub struct CharacterList {
    pub characters: Vec<Character>,
}

pub struct CharacterSelectPlugin;

impl Plugin for CharacterSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterList>().add_systems(
            EguiPrimaryContextPass,
            render_character_select.run_if(in_state(AppScreen::CharacterSelect)),
        );
    }
}

fn render_character_select(
    mut contexts: EguiContexts,
    mut commands: Commands,
    character_list: Res<CharacterList>,
    trait_registry: Res<ClientTraitRegistry>,
    mut next_state: ResMut<NextState<AppScreen>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |_ui| {});

    let screen_rect = ctx.viewport_rect();
    let panel_width = (screen_rect.width() * 0.4).max(340.0).min(500.0);
    let scroll_height = (screen_rect.height() * 0.6).max(350.0);

    let mut selected: Option<Character> = None;

    egui::Window::new("Select Character")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(SECONDARY_COLOR)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                .inner_margin(egui::Margin::same(20)),
        )
        .min_width(panel_width)
        .max_width(panel_width)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Select Character")
                        .size(24.0)
                        .color(TEXT_COLOR)
                        .strong(),
                );
                ui.add_space(12.0);
            });

            ui.separator();
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .show(ui, |ui| {
                    for character in &character_list.characters {
                        if render_character_entry(ui, character) {
                            selected = Some(character.clone());
                        }
                        ui.add_space(6.0);
                    }
                });
        });

    if let Some(mut character) = selected {
        character.recalculate_effects(&trait_registry.0);
        spawn_character(&mut commands, &character);
        next_state.set(AppScreen::CharacterSheet);
    }

    Ok(())
}

/// Renders a single character entry as a clickable card.
/// Returns `true` if the card was clicked.
fn render_character_entry(ui: &mut egui::Ui, character: &Character) -> bool {
    let id = ui.id().with(character.id);
    let was_hovered = ui.data(|d| d.get_temp::<bool>(id).unwrap_or(false));

    let fill = if was_hovered {
        MAIN_COLOR
    } else {
        SECONDARY_COLOR
    };

    let frame_response = egui::Frame::new()
        .corner_radius(6.0)
        .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .fill(fill)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(&character.name)
                    .size(16.0)
                    .color(TEXT_COLOR),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!(
                    "{}  |  {}  |  Level {}",
                    character.race, character.class, character.level
                ))
                .size(13.0)
                .color(egui::Color32::from_rgb(0x88, 0x88, 0x99)),
            );
        });

    let response = frame_response.response.interact(egui::Sense::click());
    ui.data_mut(|d| d.insert_temp(id, response.hovered()));

    response.clicked()
}
