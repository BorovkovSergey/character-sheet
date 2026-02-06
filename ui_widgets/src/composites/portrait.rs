use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

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
            .ui(ui)
    }
}
