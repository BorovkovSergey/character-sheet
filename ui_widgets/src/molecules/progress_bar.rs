use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::TEXT_COLOR;
use crate::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke};
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

    /// Renders the progress bar and returns `Some(new_value)` if a cell was clicked,
    /// where `new_value` is the 1-indexed position of the clicked cell.
    pub fn show(self, ui: &mut egui::Ui) -> Option<u32> {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_width, available_height),
            egui::Sense::click(),
        );

        let painter = ui.painter();

        let gap = 1.0;
        let box_width = if self.max > 0 {
            let total_gaps = if self.max > 1 {
                (self.max - 1) as f32 * gap
            } else {
                0.0
            };
            let bw = (rect.width() - total_gaps) / self.max as f32;

            for i in 0..self.max {
                let x = rect.min.x + i as f32 * (bw + gap);
                let box_rect =
                    Rect::from_min_size(egui::pos2(x, rect.min.y), egui::vec2(bw, rect.height()));

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
            bw
        } else {
            0.0
        };

        // Center overlay with label and value
        let overlay_color = Color32::from_rgba_unmultiplied(0xEB, 0xEB, 0xF5, 0xCC);
        let overlay_height = rect.height() * 0.66;
        let text_size = overlay_height * 0.75;
        let pad = overlay_height * 0.08;

        let font_id = egui::FontId::proportional(text_size);
        let label_galley = painter.layout_no_wrap(self.label.clone(), font_id.clone(), TEXT_COLOR);
        let value_text = format!("{}/{}", self.current, self.max);
        let value_galley = painter.layout_no_wrap(value_text.clone(), font_id, TEXT_COLOR);
        let overlay_width = pad + label_galley.size().x + 6.0 + value_galley.size().x + pad;

        let overlay_rect =
            Rect::from_center_size(rect.center(), egui::vec2(overlay_width, overlay_height));

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

        if response.clicked() && self.max > 0 {
            if let Some(pos) = response.interact_pointer_pos() {
                let relative_x = pos.x - rect.min.x;
                let cell = (relative_x / (box_width + gap)).floor() as u32;
                if cell < self.max {
                    return Some(cell + 1);
                }
            }
        }

        None
    }
}
