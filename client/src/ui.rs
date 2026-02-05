use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use shared::{ClientMessage, Resource};

use crate::networking::{ReconnectRequest, SendMessage};
use crate::state::{AppState, ConnectionStatus};

/// Maximum allowed character name length to prevent memory issues
const MAX_CHARACTER_NAME_LENGTH: usize = 100;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, render_ui);
    }
}

fn render_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
    mut send_writer: MessageWriter<SendMessage>,
    mut reconnect_writer: MessageWriter<ReconnectRequest>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("D&D Character Sheet");

        // Connection status
        ui.horizontal(|ui| {
            ui.label("Status:");
            match &state.connection_status {
                ConnectionStatus::Connected => {
                    ui.colored_label(egui::Color32::GREEN, "Connected");
                }
                ConnectionStatus::Connecting => {
                    ui.colored_label(egui::Color32::YELLOW, "Connecting...");
                }
                ConnectionStatus::Disconnected => {
                    ui.colored_label(egui::Color32::RED, "Disconnected");
                    if ui.button("Reconnect").clicked() {
                        reconnect_writer.write(ReconnectRequest);
                    }
                }
                ConnectionStatus::Error(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
                    if ui.button("Retry").clicked() {
                        reconnect_writer.write(ReconnectRequest);
                    }
                }
            }
        });

        ui.separator();

        // Create new character section
        ui.horizontal(|ui| {
            ui.label("New character name:");
            ui.text_edit_singleline(&mut state.new_character_name);

            // Enforce character name length limit
            if state.new_character_name.len() > MAX_CHARACTER_NAME_LENGTH {
                state.new_character_name.truncate(MAX_CHARACTER_NAME_LENGTH);
            }

            if ui.button("Create").clicked() && !state.new_character_name.trim().is_empty() {
                send_writer.write(SendMessage(ClientMessage::CreateCharacter {
                    name: state.new_character_name.clone(),
                }));
                state.new_character_name.clear();
            }
        });

        // Show character count if approaching limit
        if state.new_character_name.len() > MAX_CHARACTER_NAME_LENGTH - 20 {
            ui.label(format!(
                "{}/{} characters",
                state.new_character_name.len(),
                MAX_CHARACTER_NAME_LENGTH
            ));
        }

        ui.separator();

        // Character list
        ui.heading("Characters");

        if state.characters.is_empty() {
            ui.label("No characters yet. Create one above!");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_delete = None;
                let mut to_save = None;

                for character in &mut state.characters {
                    egui::Frame::new()
                        .inner_margin(8.0)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading(&character.name);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Delete").clicked() {
                                        to_delete = Some(character.id);
                                    }
                                    if ui.button("Save").clicked() {
                                        to_save = Some(character.clone());
                                    }
                                });
                            });

                            ui.add_space(4.0);

                            // HP bar - changes apply locally
                            render_resource_bar(ui, "HP", &mut character.hp, egui::Color32::RED);

                            // Mana bar
                            render_resource_bar(ui, "Mana", &mut character.mana, egui::Color32::BLUE);

                            // Action Points bar
                            render_resource_bar(ui, "AP", &mut character.action_points, egui::Color32::GOLD);
                        });

                    ui.add_space(8.0);
                }

                // Handle save
                if let Some(character) = to_save {
                    send_writer.write(SendMessage(ClientMessage::UpdateCharacter { character }));
                }

                // Handle deletion
                if let Some(id) = to_delete {
                    send_writer.write(SendMessage(ClientMessage::DeleteCharacter { id }));
                }
            });
        }

        // Refresh button
        ui.separator();
        if ui.button("Refresh List").clicked() {
            send_writer.write(SendMessage(ClientMessage::RequestCharacterList));
        }
    });

    Ok(())
}

/// Renders a resource bar with +/- buttons. Modifies resource in place.
fn render_resource_bar(ui: &mut egui::Ui, label: &str, resource: &mut Resource, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(format!("{:>4}:", label));

        if ui.button("-").clicked() && resource.current > 0 {
            resource.current -= 1;
        }

        let progress = resource.current as f32 / resource.max as f32;
        let bar = egui::ProgressBar::new(progress)
            .fill(color)
            .text(format!("{}/{}", resource.current, resource.max));
        ui.add_sized([150.0, 20.0], bar);

        if ui.button("+").clicked() && resource.current < resource.max {
            resource.current += 1;
        }

        if ui.button("Full").clicked() {
            resource.current = resource.max;
        }
    });
}
