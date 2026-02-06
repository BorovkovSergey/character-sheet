use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

/// Displays the character's active abilities.
pub struct Abilities;

impl Abilities {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Abilities {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(120, 60, 100))
            .text("Abilities")
            .text_align(Align2::CENTER_CENTER)
            .text_angle(std::f32::consts::FRAC_PI_4)
            .ui(ui)
    }
}
