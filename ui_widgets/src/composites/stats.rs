use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};

/// Displays the character's derived stats (HP, MP, etc.).
pub struct Stats;

impl Stats {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Stats {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .text("Stats")
            .text_align(Align2::LEFT_TOP)
            .ui(ui)
    }
}
