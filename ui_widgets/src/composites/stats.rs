use std::collections::BTreeMap;

use crate::atoms::{Shape, ShapeBox};
use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Color32, CornerRadius, Stroke, TextureId, Widget};
use crate::molecules::{TitlePosition, TitledBox};
use crate::traits::Roundable;

/// Displays the character's defense and resistance stats.
pub struct Stats {
    icon: TextureId,
    resists: BTreeMap<String, u32>,
}

impl Stats {
    pub fn new(icon: TextureId, resists: BTreeMap<String, u32>) -> Self {
        Self { icon, resists }
    }
}

impl Widget for Stats {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width = ui.available_width();
        let height = ui.available_height();

        let top_h = height * 0.54;
        let bottom_h = height * 0.44;
        let gap = height * 0.02;

        let defense_labels = ["Melee", "Range", "Magic", "Body", "Mind"];
        let resist_labels = ["Fire", "Ice", "Lightning", "Poison", "Spirit", "Dark"];

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            // Defense section
            ui.allocate_ui_with_layout(
                egui::vec2(width, top_h),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    TitledBox::new("Defense")
                        .fill(SECONDARY_COLOR)
                        .rounding(16)
                        .content_fill(MAIN_COLOR)
                        .content_rounding(14)
                        .show(ui, |ui| {
                            inner_titled_boxes(ui, &defense_labels, false, self.icon, None);
                        });
                },
            );

            ui.add_space(gap);

            // Resist section
            ui.allocate_ui_with_layout(
                egui::vec2(width, bottom_h),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    TitledBox::new("Resist")
                        .fill(SECONDARY_COLOR)
                        .rounding(16)
                        .content_fill(MAIN_COLOR)
                        .content_rounding(14)
                        .show(ui, |ui| {
                            inner_titled_boxes(
                                ui,
                                &resist_labels,
                                true,
                                self.icon,
                                Some(&self.resists),
                            );
                        });
                },
            );
        })
        .response
    }
}

/// Lays out a row of equally-spaced inner [`TitledBox`] widgets.
fn inner_titled_boxes(
    ui: &mut egui::Ui,
    labels: &[&str],
    is_resist: bool,
    icon: TextureId,
    values: Option<&BTreeMap<String, u32>>,
) {
    let count = labels.len() as f32;
    let spacing = 4.0;
    let available_width = ui.available_width();
    let available_height = ui.available_height();
    let pad_x = available_width * 0.015;
    let pad_y = available_height * 0.015;
    let inner_width = available_width - pad_x * 2.0;
    let inner_height = available_height - pad_y * 2.0;
    let item_width = (inner_width - spacing * (count - 1.0)) / count;

    ui.vertical(|ui| {
        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            ui.spacing_mut().item_spacing = egui::vec2(spacing, 0.0);
            for label in labels {
                let text = if let Some(vals) = values {
                    let v = *vals.get(*label).unwrap_or(&0) as i32;
                    format_signed(v)
                } else {
                    let value = pseudo_value(label, is_resist);
                    if is_resist {
                        format_signed(value)
                    } else {
                        value.to_string()
                    }
                };

                ui.allocate_ui_with_layout(
                    egui::vec2(item_width, inner_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        TitledBox::new(*label)
                            .title_position(TitlePosition::Top)
                            .fill(Color32::TRANSPARENT)
                            .rounding(8)
                            .show(ui, |ui| {
                                let mut shape = ShapeBox::new(Shape::Rectangle)
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::new(1.0, STROKE_COLOR))
                                    .text(text.clone())
                                    .text_color(TEXT_COLOR)
                                    .icon(icon);
                                let radius = if is_resist { 12 } else { 16 };
                                shape.set_rounding(CornerRadius::same(radius));
                                ui.add(shape);
                            });
                    },
                );
            }
        });
    });
}

fn pseudo_value(label: &str, signed: bool) -> i32 {
    let hash: u32 = label
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    if signed {
        (hash % 11) as i32 - 5
    } else {
        (hash % 20) as i32 + 1
    }
}

fn format_signed(value: i32) -> String {
    if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}
