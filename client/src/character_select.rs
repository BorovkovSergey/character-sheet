use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use shared::CharacterSummary;
use ui_widgets::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};

use crate::create_character::CreateCharacterOpen;
use crate::network::{ClientSkillRegistry, ClientTraitRegistry, PendingClientMessages};
use crate::portrait::{CropEditorSlot, PendingCreationPortrait, PortraitPickerResult};

use crate::state::AppScreen;

/// Holds the list of character summaries received from the server.
#[derive(Debug, Clone, Resource, Default)]
pub struct CharacterList {
    pub characters: Vec<CharacterSummary>,
}

pub struct CharacterSelectPlugin;

impl Plugin for CharacterSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterList>()
            .init_resource::<CreateCharacterOpen>()
            .add_systems(
                EguiPrimaryContextPass,
                render_character_select.run_if(in_state(AppScreen::CharacterSelect)),
            );
    }
}

fn render_character_select(
    mut contexts: EguiContexts,
    character_list: Res<CharacterList>,
    mut pending_messages: ResMut<PendingClientMessages>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut create_open: ResMut<CreateCharacterOpen>,
    skill_registry: Res<ClientSkillRegistry>,
    trait_registry: Res<ClientTraitRegistry>,
    portrait_picker: Res<PortraitPickerResult>,
    mut pending_creation_portrait: ResMut<PendingCreationPortrait>,
    mut crop_editor: ResMut<CropEditorSlot>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |_ui| {});

    let screen_rect = ctx.viewport_rect();
    let panel_width = (screen_rect.width() * 0.4).max(340.0).min(500.0);
    let scroll_height = (screen_rect.height() * 0.6).max(350.0);

    let mut selected: Option<CharacterSummary> = None;

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
                    for summary in &character_list.characters {
                        if render_character_entry(ui, summary) {
                            selected = Some(summary.clone());
                        }
                        ui.add_space(6.0);
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.vertical_centered(|ui| {
                let button =
                    egui::Button::new(egui::RichText::new("Create").size(16.0).color(TEXT_COLOR))
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                        .fill(MAIN_COLOR)
                        .min_size(egui::vec2(panel_width * 0.5, 36.0));

                if ui.add(button).clicked() {
                    create_open.0 = true;
                }
            });
            ui.add_space(4.0);
        });

    if let Some(summary) = selected {
        pending_messages
            .0
            .push(shared::ClientMessage::RequestVersionList { id: summary.id });
        next_state.set(AppScreen::VersionSelect);
    }

    if create_open.0 {
        crate::create_character::render_create_character_overlay(
            ctx,
            &mut create_open,
            &skill_registry,
            &trait_registry,
            &mut pending_messages,
            &portrait_picker,
            &mut pending_creation_portrait,
            &mut crop_editor,
        );
    }

    // Poll picker & render crop popup when image is ready.
    crate::portrait::poll_and_render_crop_popup(ctx, &mut crop_editor, &portrait_picker);

    Ok(())
}

/// Renders a single character summary entry as a clickable card.
/// Returns `true` if the card was clicked.
fn render_character_entry(ui: &mut egui::Ui, summary: &CharacterSummary) -> bool {
    let id = ui.id().with(summary.id);
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
                egui::RichText::new(&summary.name)
                    .size(16.0)
                    .color(TEXT_COLOR),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!(
                    "{}  |  {}  |  Level {}",
                    summary.race, summary.class, summary.level
                ))
                .size(13.0)
                .color(egui::Color32::from_rgb(0x88, 0x88, 0x99)),
            );
        });

    let response = frame_response.response.interact(egui::Sense::click());
    ui.data_mut(|d| d.insert_temp(id, response.hovered()));

    response.clicked()
}
