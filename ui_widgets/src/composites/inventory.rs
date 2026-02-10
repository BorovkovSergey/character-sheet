use crate::colors::SECONDARY_COLOR;
use crate::egui::{self, CornerRadius, TextureId, Widget};
use crate::molecules::{InventoryTable, InventoryTooltip, TitledBox};

/// Displays the character's inventory as a 5x8 grid of [`InventoryCell`] items
/// inside a [`TitledBox`]. Hovering over a filled cell shows a tooltip popup.
pub struct Inventory {
    image: TextureId,
    items: Vec<Option<InventoryTooltip>>,
}

impl Inventory {
    pub fn new(image: TextureId) -> Self {
        Self {
            image,
            items: Vec::new(),
        }
    }

    pub fn items(mut self, items: Vec<Option<InventoryTooltip>>) -> Self {
        self.items = items;
        self
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
                InventoryTable::new(self.image, 5, 8)
                    .id_salt("inventory")
                    .items(self.items)
                    .paint(ui, rect);
            })
    }
}
