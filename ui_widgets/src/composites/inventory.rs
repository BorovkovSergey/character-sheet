use std::cell::Cell;

use crate::colors::SECONDARY_COLOR;
use crate::egui::{self, CornerRadius, TextureId};
use crate::molecules::{CellAction, InventoryTable, InventoryTooltip, TitledBox};

/// Displays the character's inventory as a 5x8 grid of [`InventoryCell`] items
/// inside a [`TitledBox`]. Hovering over a filled cell shows a tooltip popup.
/// Right-clicking an item shows a context menu with "Equip" and "Remove".
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

    /// Renders the inventory and returns the action the user triggered, if any.
    pub fn show(self, ui: &mut egui::Ui) -> Option<CellAction> {
        let action: Cell<Option<CellAction>> = Cell::new(None);
        TitledBox::new("Inventory")
            .fill(SECONDARY_COLOR)
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let result = InventoryTable::new(self.image, 5, 8)
                    .id_salt("inventory")
                    .context_label("Equip")
                    .show_remove(true)
                    .items(self.items)
                    .paint(ui, rect);
                action.set(result);
            });
        action.get()
    }
}
