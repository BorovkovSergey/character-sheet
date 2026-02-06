use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

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
            .text("EquippedGear")
            .text_align(Align2::LEFT_CENTER)
            .text_angle(std::f32::consts::FRAC_PI_2)
            .ui(ui)
    }
}
