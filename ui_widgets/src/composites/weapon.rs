use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};
use crate::traits::WithText;

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
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .set_text("Weapon")
            .set_text_align(Align2::CENTER_TOP)
            .set_text_angle(std::f32::consts::FRAC_PI_6)
            .ui(ui)
    }
}
