use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};

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
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .text("Characteristics")
            .text_align(Align2::RIGHT_TOP)
            .ui(ui)
    }
}
