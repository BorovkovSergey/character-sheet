use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

/// Displays equipped weapon information.
pub struct Weapon;

impl Weapon {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Weapon {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(120, 80, 40))
            .ui(ui)
    }
}
