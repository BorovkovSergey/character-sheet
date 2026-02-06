use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

/// Displays the character's learned skills and their levels.
pub struct Skills;

impl Skills {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Skills {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(60, 120, 100))
            .ui(ui)
    }
}
