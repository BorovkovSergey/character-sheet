use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Color32, Widget};

/// Displays the character's passive traits and perks.
pub struct Traits;

impl Traits {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Traits {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(100, 60, 120))
            .ui(ui)
    }
}
