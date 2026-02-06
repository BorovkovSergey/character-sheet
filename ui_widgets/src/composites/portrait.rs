use crate::atoms::{Shape, ShapeBox};
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, Align2, Stroke, Widget};

/// Character portrait display area.
pub struct Portrait;

impl Portrait {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Portrait {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::new(1.0, STROKE_COLOR))
            .text("Portrait")
            .text_align(Align2::CENTER_CENTER)
            .ui(ui)
    }
}
