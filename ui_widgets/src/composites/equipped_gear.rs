use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};
use crate::traits::WithText;

/// Displays the character's currently equipped gear slots.
pub struct EquippedGear;

impl EquippedGear {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for EquippedGear {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .set_text("EquippedGear")
            .set_text_align(Align2::LEFT_CENTER)
            .set_text_angle(std::f32::consts::FRAC_PI_2)
            .ui(ui)
    }
}
