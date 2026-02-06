use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

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
            .fill(Color32::from_rgb(80, 120, 80))
            .text("Stats")
            .text_align(Align2::LEFT_TOP)
            .ui(ui)
    }
}
