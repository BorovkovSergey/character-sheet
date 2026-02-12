use std::collections::BTreeMap;

use crate::atoms::{Shape, ShapeBox};
use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Color32, CornerRadius, Stroke, TextureId, Widget};
use crate::molecules::{TitlePosition, TitledBox};
use crate::traits::{Roundable, WithText};

/// Displays the character's defense and resistance stats.
pub struct Stats {
    resists: BTreeMap<String, (TextureId, i32)>,
    protections: BTreeMap<String, (TextureId, i32)>,
}

impl Stats {
    pub fn new(
        resists: BTreeMap<String, (TextureId, i32)>,
        protections: BTreeMap<String, (TextureId, i32)>,
    ) -> Self {
        Self {
            resists,
            protections,
        }
    }
}

impl Widget for Stats {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width = ui.available_width();
        let height = ui.available_height();

        let top_h = height * 0.54;
        let bottom_h = height * 0.44;
        let gap = height * 0.02;

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
                            inner_titled_boxes_with_icons(ui, &self.protections, 16);
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
                            inner_titled_boxes_with_icons(ui, &self.resists, 12);
                        });
                },
            );
        })
        .response
    }
}

/// Lays out a row of equally-spaced inner [`TitledBox`] widgets with per-item icons.
fn inner_titled_boxes_with_icons(
    ui: &mut egui::Ui,
    values: &BTreeMap<String, (TextureId, i32)>,
    rounding: u8,
) {
    let count = values.len() as f32;
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
            for (label, (icon, value)) in values {
                let text = format_signed(*value);

                ui.allocate_ui_with_layout(
                    egui::vec2(item_width, inner_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        TitledBox::new(label.as_str())
                            .title_position(TitlePosition::Top)
                            .fill(Color32::TRANSPARENT)
                            .rounding(8)
                            .show(ui, |ui| {
                                let shape = ShapeBox::new(Shape::Rectangle)
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::new(1.0, STROKE_COLOR))
                                    .icon(*icon)
                                    .set_text(text.clone())
                                    .set_text_color(TEXT_COLOR)
                                    .set_rounding(CornerRadius::same(rounding));
                                ui.add(shape);
                            });
                    },
                );
            }
        });
    });
}

fn format_signed(value: i32) -> String {
    if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}
