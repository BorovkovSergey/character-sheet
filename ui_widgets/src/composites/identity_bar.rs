use crate::atoms::{Shape, ShapeBox};
use crate::egui::{self, Align2, Color32, Widget};

/// Displays the character's name and core identity information.
pub struct IdentityBar;

impl IdentityBar {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for IdentityBar {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::from_rgb(60, 100, 60))
            .text("IdentityBar")
            .text_align(Align2::LEFT_CENTER)
            .ui(ui)
    }
}
