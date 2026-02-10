use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, CornerRadius, Rect, Stroke, Widget};
use crate::traits::{Roundable, WithText};

/// Displays two point pools side by side: Characteristic points and Skill points.
pub struct Points {
    characteristic_points: u32,
    skill_points: u32,
}

impl Points {
    pub fn new(characteristic_points: u32, skill_points: u32) -> Self {
        Self {
            characteristic_points,
            skill_points,
        }
    }
}

impl Widget for Points {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let spacing_x = 4.0;
        let item_width = (available_width - spacing_x) / 2.0;

        let items: [(&str, u32, CornerRadius); 2] = [
            (
                "Characteristic points",
                self.characteristic_points,
                CornerRadius::same(12),
            ),
            ("Skill points", self.skill_points, CornerRadius::same(12)),
        ];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(spacing_x, 0.0);

            for (label, value, bg_rounding) in items {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(item_width, available_height),
                    egui::Sense::hover(),
                );

                let painter = ui.painter();

                // Background
                let clipped = painter.with_clip_rect(rect);
                clipped.rect_filled(rect, bg_rounding, SECONDARY_COLOR);

                // Label text on the left
                let text_x = rect.min.x + rect.width() * 0.07;
                let text_rect = Rect::from_min_max(egui::pos2(text_x, rect.min.y), rect.max);
                Text::new(label)
                    .color(TEXT_COLOR)
                    .size(12.0)
                    .align(Align2::LEFT_CENTER)
                    .paint(painter, text_rect);

                // Square ShapeBox on the right with the value
                let pad = rect.width() * 0.025;
                let box_side = rect.height() - pad * 2.0;
                let box_rect = Rect::from_min_size(
                    egui::pos2(rect.max.x - pad - box_side, rect.min.y + pad),
                    egui::vec2(box_side, box_side),
                );

                let shape = ShapeBox::new(Shape::Rectangle)
                    .fill(MAIN_COLOR)
                    .stroke(Stroke::new(1.0, STROKE_COLOR))
                    .set_text(value.to_string())
                    .set_text_color(TEXT_COLOR)
                    .set_rounding(CornerRadius::same(8));
                shape.paint(painter, box_rect);
            }
        })
        .response
    }
}
