use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

/// Displays the character's current status effects and conditions.
pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for StatusBar {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(60, 60, 100))
            .ui(ui)
    }
}
