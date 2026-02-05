use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use shared::ClientMessage;

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

                for character in &state.characters {
                    ui.horizontal(|ui| {
                        ui.label(&character.name);
                        ui.label(format!("({})", character.id));
                        if ui.button("Delete").clicked() {
                            to_delete = Some(character.id);
                        }
                    });
                }

                // Handle deletion outside the loop to avoid borrow issues
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
