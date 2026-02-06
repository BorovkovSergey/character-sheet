use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};

/// Displays the character's name and core identity information.
pub struct IdentityBar;

impl IdentityBar {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for IdentityBar {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .text("IdentityBar")
            .text_align(Align2::LEFT_CENTER)
            .ui(ui)
    }
}
