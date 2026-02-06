use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

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
            .fill(Color32::from_rgb(100, 60, 60))
            .text("Portrait")
            .text_align(Align2::CENTER_CENTER)
            .ui(ui)
    }
}
