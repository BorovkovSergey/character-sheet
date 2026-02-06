use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

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
            .ui(ui)
    }
}
