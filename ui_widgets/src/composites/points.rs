use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};
use crate::traits::WithText;

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
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .set_text("Points")
            .set_text_align(Align2::LEFT_BOTTOM)
            .set_text_angle(-std::f32::consts::FRAC_PI_6)
            .ui(ui)
    }
}
