use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};

/// Displays the character's passive traits and perks.
pub struct Traits;

impl Traits {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Traits {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .text("Traits")
            .text_align(Align2::RIGHT_BOTTOM)
            .ui(ui)
    }
}
