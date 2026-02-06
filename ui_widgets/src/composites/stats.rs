use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

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
            .ui(ui)
    }
}
