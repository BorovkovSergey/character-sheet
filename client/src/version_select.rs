use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use shared::VersionSummary;
use ui_widgets::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use uuid::Uuid;

use crate::network::PendingClientMessages;
use crate::state::AppScreen;

/// Holds the version list for the currently selected character.
#[derive(Debug, Clone, Resource, Default)]
pub struct VersionList {
    pub character_id: Uuid,
    pub character_name: String,
    pub versions: Vec<VersionSummary>,
}

/// Pending delete confirmation state.
#[derive(Resource, Default)]
struct DeleteConfirm {
    version: Option<u32>,
    character: bool,
}

pub struct VersionSelectPlugin;

impl Plugin for VersionSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VersionList>()
            .init_resource::<DeleteConfirm>()
            .add_systems(
                EguiPrimaryContextPass,
                render_version_select.run_if(in_state(AppScreen::VersionSelect)),
            );
    }
}

fn render_version_select(
    mut contexts: EguiContexts,
    version_list: Res<VersionList>,
    mut pending_messages: ResMut<PendingClientMessages>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut delete_confirm: ResMut<DeleteConfirm>,
    auth_state: Res<crate::network::AuthState>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |_ui| {});

    let screen_rect = ctx.viewport_rect();
    let panel_width = (screen_rect.width() * 0.4).max(340.0).min(500.0);
    let scroll_height = (screen_rect.height() * 0.6).max(350.0);

    let mut selected_version: Option<u32> = None;
    let mut go_back = false;
    let mut request_delete: Option<u32> = None;

    egui::Window::new("Select Version")
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
                    egui::RichText::new(&version_list.character_name)
                        .size(24.0)
                        .color(TEXT_COLOR)
                        .strong(),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Select Version")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(0x88, 0x88, 0x99)),
                );
                ui.add_space(12.0);
            });

            ui.separator();
            ui.add_space(4.0);

            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("< Back to Characters")
                            .size(14.0)
                            .color(TEXT_COLOR),
                    )
                    .fill(SECONDARY_COLOR)
                    .stroke(egui::Stroke::NONE),
                )
                .clicked()
            {
                go_back = true;
            }

            ui.add_space(8.0);

            let can_delete = version_list.versions.len() > 1 && auth_state.authenticated;
            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .show(ui, |ui| {
                    // Iterate in reverse to show newest first
                    for version in version_list.versions.iter().rev() {
                        let action = render_version_entry(ui, version, can_delete);
                        if action.selected {
                            selected_version = Some(version.version);
                        }
                        if action.delete {
                            request_delete = Some(version.version);
                        }
                        ui.add_space(6.0);
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            if auth_state.authenticated {
                ui.vertical_centered(|ui| {
                    let delete_char_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("\u{00D7}  Delete Character")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(0xCC, 0x33, 0x33)),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(0xAA, 0x44, 0x44),
                        ))
                        .corner_radius(4.0)
                        .min_size(egui::vec2(panel_width * 0.5, 32.0)),
                    );
                    if delete_char_btn.clicked() {
                        delete_confirm.character = true;
                    }
                });
            }
            ui.add_space(4.0);
        });

    if go_back {
        delete_confirm.version = None;
        delete_confirm.character = false;
        next_state.set(AppScreen::CharacterSelect);
    }

    if let Some(version) = selected_version {
        pending_messages
            .0
            .push(shared::ClientMessage::RequestCharacterVersion {
                id: version_list.character_id,
                version: Some(version),
            });
    }

    if let Some(version) = request_delete {
        delete_confirm.version = Some(version);
    }

    // Confirmation dialog
    if let Some(version) = delete_confirm.version {
        let mut close = false;
        egui::Window::new("Confirm Delete")
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
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("Delete version {}?", version))
                            .size(18.0)
                            .color(TEXT_COLOR),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        let delete_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Delete")
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(0xCC, 0x33, 0x33))
                            .corner_radius(4.0),
                        );
                        if delete_btn.clicked() {
                            pending_messages
                                .0
                                .push(shared::ClientMessage::DeleteVersion {
                                    id: version_list.character_id,
                                    version,
                                });
                            close = true;
                        }
                        ui.add_space(8.0);
                        let cancel_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Cancel").size(14.0).color(TEXT_COLOR),
                            )
                            .fill(MAIN_COLOR)
                            .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                            .corner_radius(4.0),
                        );
                        if cancel_btn.clicked() {
                            close = true;
                        }
                    });
                    ui.add_space(4.0);
                });
            });
        if close {
            delete_confirm.version = None;
        }
    }

    // Character deletion confirmation dialog
    if delete_confirm.character {
        let mut close = false;
        egui::Window::new("Confirm Delete Character")
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
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("Delete \"{}\"?", version_list.character_name))
                            .size(18.0)
                            .color(TEXT_COLOR),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("All versions will be permanently deleted.")
                            .size(13.0)
                            .color(egui::Color32::from_rgb(0xAA, 0x44, 0x44)),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        let delete_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Delete")
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(0xCC, 0x33, 0x33))
                            .corner_radius(4.0),
                        );
                        if delete_btn.clicked() {
                            pending_messages
                                .0
                                .push(shared::ClientMessage::DeleteCharacter {
                                    id: version_list.character_id,
                                });
                            pending_messages
                                .0
                                .push(shared::ClientMessage::RequestCharacterList);
                            next_state.set(AppScreen::CharacterSelect);
                            close = true;
                        }
                        ui.add_space(8.0);
                        let cancel_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Cancel").size(14.0).color(TEXT_COLOR),
                            )
                            .fill(MAIN_COLOR)
                            .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                            .corner_radius(4.0),
                        );
                        if cancel_btn.clicked() {
                            close = true;
                        }
                    });
                    ui.add_space(4.0);
                });
            });
        if close {
            delete_confirm.character = false;
        }
    }

    Ok(())
}

struct VersionEntryAction {
    selected: bool,
    delete: bool,
}

/// Renders a single version entry as a clickable card with a delete button.
fn render_version_entry(
    ui: &mut egui::Ui,
    version: &VersionSummary,
    can_delete: bool,
) -> VersionEntryAction {
    let id = ui.id().with(("version", version.version));
    let was_hovered = ui.data(|d| d.get_temp::<bool>(id).unwrap_or(false));

    let fill = if was_hovered {
        MAIN_COLOR
    } else {
        SECONDARY_COLOR
    };

    let mut delete_clicked = false;

    let frame_response = egui::Frame::new()
        .corner_radius(6.0)
        .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .fill(fill)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Version {}", version.version))
                        .size(16.0)
                        .color(TEXT_COLOR),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if can_delete {
                        let x_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("\u{00D7}")
                                    .size(18.0)
                                    .color(egui::Color32::from_rgb(0xAA, 0x44, 0x44)),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(28.0, 28.0)),
                        );
                        if x_btn.clicked() {
                            delete_clicked = true;
                        }
                    }
                    ui.label(
                        egui::RichText::new(format!("Level {}", version.level))
                            .size(13.0)
                            .color(egui::Color32::from_rgb(0x88, 0x88, 0x99)),
                    );
                });
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format_timestamp(version.saved_at))
                    .size(13.0)
                    .color(egui::Color32::from_rgb(0x88, 0x88, 0x99)),
            );
        });

    let response = &frame_response.response;
    let hovered = response.contains_pointer();
    let card_clicked = hovered && !delete_clicked && ui.input(|i| i.pointer.primary_clicked());
    ui.data_mut(|d| d.insert_temp(id, hovered));

    VersionEntryAction {
        selected: card_clicked,
        delete: delete_clicked,
    }
}

/// Format a Unix timestamp (seconds UTC) into a human-readable date string.
fn format_timestamp(ts: i64) -> String {
    // Manual UTC date/time computation from Unix timestamp
    // This avoids adding chrono as a dependency
    let secs = ts;
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    // Compute year, month, day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_date(days);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02} UTC",
        year, month, day, hours, minutes
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    // Algorithm based on civil_from_days from Howard Hinnant
    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
