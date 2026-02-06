use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

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
            .text("Weapon")
            .text_align(Align2::CENTER_TOP)
            .text_angle(std::f32::consts::FRAC_PI_6)
            .ui(ui)
    }
}
