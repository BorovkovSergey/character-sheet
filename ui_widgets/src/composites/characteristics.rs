use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, CornerRadius, Rect, Stroke, Widget};
use crate::traits::{Roundable, WithText};

/// Displays the character's primary characteristics as 8 boxes
/// arranged in 2 rows of 4.
///
/// Each box has the characteristic name on the left and a square ShapeBox
/// with the value on the right.
///
/// Uses `Vec<(String, u32)>` rather than `BTreeMap` to preserve the caller's
/// intended display order (e.g. STR, DEX, END, ...), which does not match
/// alphabetical sorting.
pub struct Characteristics {
    values: Vec<(String, u32)>,
}

impl Characteristics {
    pub fn new(values: Vec<(String, u32)>) -> Self {
        Self { values }
    }
}

impl Widget for Characteristics {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let cols = 4.0;
        let rows = 2.0;
        let spacing_x = 4.0;
        let spacing_y = 4.0;
        let item_width = (available_width - spacing_x * (cols - 1.0)) / cols;
        let item_height = (available_height - spacing_y * (rows - 1.0)) / rows;

        let total = self.values.len();

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            let mut idx = 0;
            for row in self.values.chunks(4) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(spacing_x, 0.0);
                    for (label, value) in row {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(item_width, item_height),
                            egui::Sense::hover(),
                        );

                        let painter = ui.painter();

                        // Base rounding 8, with 12 on the four outer corners
                        let mut rounding = CornerRadius::same(8);
                        if idx == 0 {
                            rounding.nw = 20;
                        }
                        if idx == 3 {
                            rounding.ne = 20;
                        }
                        if idx == total - 4 {
                            rounding.sw = 20;
                        }
                        if idx == total - 1 {
                            rounding.se = 20;
                        }

                        // Background
                        let clipped = painter.with_clip_rect(rect);
                        clipped.rect_filled(rect, rounding, SECONDARY_COLOR);

                        // Label text on the left, 7% padding from left edge
                        let text_x = rect.min.x + rect.width() * 0.07;
                        let text_rect =
                            Rect::from_min_max(egui::pos2(text_x, rect.min.y), rect.max);
                        Text::new(label)
                            .color(TEXT_COLOR)
                            .size(12.0)
                            .align(Align2::LEFT_CENTER)
                            .paint(painter, text_rect);

                        // Square ShapeBox on the right, 2.5% of width padding
                        let pad = rect.width() * 0.025;
                        let box_side = rect.height() - pad * 2.0;
                        let box_rect = Rect::from_min_size(
                            egui::pos2(rect.max.x - pad - box_side, rect.min.y + pad),
                            egui::vec2(box_side, box_side),
                        );

                        let mut shape = ShapeBox::new(Shape::Rectangle)
                            .fill(MAIN_COLOR)
                            .stroke(Stroke::NONE)
                            .set_text(value.to_string())
                            .set_text_color(TEXT_COLOR);
                        shape.set_rounding(CornerRadius::same(12));
                        shape.paint(painter, box_rect);

                        idx += 1;
                    }
                });
                ui.add_space(spacing_y);
            }
        })
        .response
    }
}
