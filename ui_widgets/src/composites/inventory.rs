use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

/// Displays the character's inventory of carried items.
pub struct Inventory;

impl Inventory {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Inventory {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(60, 90, 110))
            .ui(ui)
    }
}
