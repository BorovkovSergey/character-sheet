use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, CornerRadius, Stroke, Widget};
use crate::molecules::TitledBox;
use crate::styles::UiStyle;
use crate::traits::Roundable;

/// Displays the character's learned abilities as a 2-column grid of cards.
pub struct Abilities {
    entries: Vec<String>,
}

impl Abilities {
    pub fn new(entries: Vec<String>) -> Self {
        Self { entries }
    }
}

impl Widget for Abilities {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        TitledBox::new("Abilities")
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .show(ui, |ui| {
                if self.entries.is_empty() {
                    return;
                }

                let available = ui.available_size();
                let pad = UiStyle::content_padding(ui);
                let cols = 2usize;
                let rows = (self.entries.len() + cols - 1) / cols;
                let card_width = (available.x - pad * (cols as f32 + 1.0)) / cols as f32;
                let card_height = (available.y - pad * (rows as f32 + 1.0)) / rows as f32;
                let origin = ui.min_rect().min;
                let painter = ui.painter();

                for (i, _entry) in self.entries.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let x = origin.x + pad + (card_width + pad) * col as f32;
                    let y = origin.y + pad + (card_height + pad) * row as f32;
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(card_width, card_height),
                    );

                    ShapeBox::new(Shape::Rectangle)
                        .fill(Color32::WHITE)
                        .stroke(Stroke::NONE)
                        .set_rounding(CornerRadius::same(12))
                        .paint(painter, rect);
                }
            })
    }
}
