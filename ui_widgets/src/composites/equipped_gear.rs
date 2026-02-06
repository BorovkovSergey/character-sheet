use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

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
            .fill(Color32::from_rgb(90, 110, 60))
            .ui(ui)
    }
}
