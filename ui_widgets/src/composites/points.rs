use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

/// Displays the character's point pools (experience, karma, etc.).
pub struct Points;

impl Points {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Points {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(120, 100, 60))
            .ui(ui)
    }
}
