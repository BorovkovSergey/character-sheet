use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

/// Displays the character's primary characteristics (STR, DEX, etc.).
pub struct Characteristics;

impl Characteristics {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Characteristics {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(80, 80, 120))
            .ui(ui)
    }
}
