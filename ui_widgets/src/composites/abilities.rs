use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};

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
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .text("Abilities")
            .text_align(Align2::CENTER_CENTER)
            .text_angle(std::f32::consts::FRAC_PI_4)
            .ui(ui)
    }
}
