use crate::colors::SECONDARY_COLOR;
use crate::egui::{self, CornerRadius, TextureId, Widget};
use crate::molecules::{InventoryTable, InventoryTooltip, TitledBox};

/// Displays the character's currently equipped gear slots as a 3x4 grid.
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
}

impl Widget for EquippedGear {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        TitledBox::new("Equipped")
            .fill(SECONDARY_COLOR)
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .show(ui, |ui| {
                let rect = ui.max_rect();
                InventoryTable::new(self.image, 3, 4)
                    .id_salt("equipped_gear")
                    .items(self.items)
                    .paint(ui, rect);
            })
    }
}
