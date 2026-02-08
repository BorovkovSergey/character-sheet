use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::TEXT_COLOR;
use crate::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, Widget};
use crate::traits::Roundable;

/// A horizontal bar of `ShapeBox` rectangles representing a resource (e.g. HP, MP, AP).
///
/// Boxes from the left up to `current` are filled with `active_color`;
/// the remaining boxes use `spent_color`. A label is shown to the left
/// and a "current/max" counter to the right.
pub struct ProgressBar {
    label: String,
    current: u32,
    max: u32,
    active_color: Color32,
    spent_color: Color32,
}

impl ProgressBar {
    pub fn new(
        label: impl Into<String>,
        current: u32,
        max: u32,
        active_color: Color32,
        spent_color: Color32,
    ) -> Self {
        Self {
            label: label.into(),
            current,
            max,
            active_color,
            spent_color,
        }
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_width, available_height),
            egui::Sense::hover(),
        );

        let painter = ui.painter();

        // Paint ShapeBox cells across the full width
        if self.max > 0 {
            let gap = 1.0;
            let total_gaps = if self.max > 1 {
                (self.max - 1) as f32 * gap
            } else {
                0.0
            };
            let box_width = (rect.width() - total_gaps) / self.max as f32;

            for i in 0..self.max {
                let x = rect.min.x + i as f32 * (box_width + gap);
                let box_rect =
                    Rect::from_min_size(egui::pos2(x, rect.min.y), egui::vec2(box_width, rect.height()));

                let color = if i < self.current {
                    self.active_color
                } else {
                    self.spent_color
                };

                let shape = ShapeBox::new(Shape::Rectangle)
                    .fill(color)
                    .stroke(Stroke::NONE)
                    .set_rounding(CornerRadius::same(4));
                shape.paint(painter, box_rect);
            }
        }

        // Center overlay with label and value
        let overlay_color = Color32::from_rgba_unmultiplied(0xEB, 0xEB, 0xF5, 0xCC);
        let overlay_height = rect.height() * 0.66;
        let text_size = overlay_height * 0.75;
        let pad = overlay_height * 0.08;

        // Measure both texts to compute overlay width
        let font_id = egui::FontId::proportional(text_size);
        let label_galley = painter.layout_no_wrap(self.label.clone(), font_id.clone(), TEXT_COLOR);
        let value_text = format!("{}/{}", self.current, self.max);
        let value_galley = painter.layout_no_wrap(value_text.clone(), font_id, TEXT_COLOR);
        let overlay_width = pad + label_galley.size().x + 6.0 + value_galley.size().x + pad;

        let overlay_rect = Rect::from_center_size(
            rect.center(),
            egui::vec2(overlay_width, overlay_height),
        );

        let overlay = ShapeBox::new(Shape::Rectangle)
            .fill(overlay_color)
            .stroke(Stroke::NONE)
            .set_rounding(CornerRadius::same(4));
        overlay.paint(painter, overlay_rect);

        let text_rect = Rect::from_min_max(
            egui::pos2(overlay_rect.min.x + pad, overlay_rect.min.y),
            egui::pos2(overlay_rect.max.x - pad, overlay_rect.max.y),
        );

        Text::new(&self.label)
            .color(TEXT_COLOR)
            .size(text_size)
            .align(Align2::LEFT_CENTER)
            .paint(painter, text_rect);

        Text::new(value_text)
            .color(TEXT_COLOR)
            .size(text_size)
            .bold()
            .align(Align2::RIGHT_CENTER)
            .paint(painter, text_rect);

        response
    }
}
