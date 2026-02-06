use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

/// Displays the character's currency and wealth.
pub struct Wallet;

impl Wallet {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Wallet {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(110, 90, 50))
            .text("Wallet")
            .text_align(Align2::RIGHT_CENTER)
            .text_angle(-std::f32::consts::FRAC_PI_4)
            .ui(ui)
    }
}
