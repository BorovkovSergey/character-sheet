use crate::colors::SECONDARY_COLOR;
use crate::egui::{self, CornerRadius, TextureId, Widget};
use crate::molecules::{InventoryTable, TitledBox};

/// Displays the character's currently equipped gear slots as a 3x4 grid.
pub struct EquippedGear {
    image: TextureId,
}

impl EquippedGear {
    pub fn new(image: TextureId) -> Self {
        Self { image }
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
                InventoryTable::new(self.image, 3, 4).paint(ui, rect);
            })
    }
}
