use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};
use crate::traits::WithText;

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
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .set_text("Skills")
            .set_text_align(Align2::CENTER_BOTTOM)
            .ui(ui)
    }
}
