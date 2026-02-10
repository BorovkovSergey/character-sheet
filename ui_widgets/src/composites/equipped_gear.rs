use std::cell::Cell;

use crate::colors::SECONDARY_COLOR;
use crate::egui::{self, CornerRadius, TextureId};
use crate::molecules::{InventoryTable, InventoryTooltip, TitledBox};

/// Displays the character's currently equipped gear slots as a 3x4 grid.
/// Right-clicking an equipped item shows an "Unequip" context menu.
pub struct EquippedGear {
    image: TextureId,
    items: Vec<Option<InventoryTooltip>>,
}

impl EquippedGear {
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

    /// Renders the equipped gear and returns the index of the item the user chose to unequip.
    pub fn show(self, ui: &mut egui::Ui) -> Option<usize> {
        let action = Cell::new(None);
        TitledBox::new("Equipped")
            .fill(SECONDARY_COLOR)
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let result = InventoryTable::new(self.image, 3, 4)
                    .id_salt("equipped_gear")
                    .context_label("Unequip")
                    .items(self.items)
                    .paint(ui, rect);
                // Only primary action (Unequip) is relevant here.
                if let Some(crate::molecules::CellAction::Primary(i)) = result {
                    action.set(Some(i));
                }
            });
        action.get()
    }
}
