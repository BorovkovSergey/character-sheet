use crate::colors::SECONDARY_COLOR;
use crate::egui::{self, CornerRadius, TextureId, Widget};
use crate::molecules::{InventoryTable, TitledBox};

/// Displays the character's inventory as a 5x8 grid of [`InventoryCell`] items
/// inside a [`TitledBox`].
pub struct Inventory {
    image: TextureId,
}

impl Inventory {
    pub fn new(image: TextureId) -> Self {
        Self { image }
    }
}

impl Widget for Inventory {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        TitledBox::new("Inventory")
            .fill(SECONDARY_COLOR)
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .show(ui, |ui| {
                let rect = ui.max_rect();
                InventoryTable::new(self.image, 5, 8).paint(ui, rect);
            })
    }
}
