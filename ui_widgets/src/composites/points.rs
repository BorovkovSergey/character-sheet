use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

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
            .fill(Color32::from_rgb(120, 100, 60))
            .text("Points")
            .text_align(Align2::LEFT_BOTTOM)
            .text_angle(-std::f32::consts::FRAC_PI_6)
            .ui(ui)
    }
}
