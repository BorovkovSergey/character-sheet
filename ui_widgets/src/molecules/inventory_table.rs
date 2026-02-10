use crate::egui::{self, Rect, TextureId};
use crate::styles::UiStyle;

use super::InventoryCell;

/// A grid of [`InventoryCell`] items with configurable column and row counts.
pub struct InventoryTable {
    image: TextureId,
    cols: usize,
    rows: usize,
}

impl InventoryTable {
    pub fn new(image: TextureId, cols: usize, rows: usize) -> Self {
        Self { image, cols, rows }
    }

    /// Paints the grid into the given rect.
    pub fn paint(&self, ui: &egui::Ui, rect: Rect) {
        let pad = UiStyle::content_padding(ui);
        let cell_width = (rect.width() - pad * (self.cols as f32 + 1.0)) / self.cols as f32;
        let cell_height = (rect.height() - pad * (self.rows as f32 + 1.0)) / self.rows as f32;
        let painter = ui.painter();

        for i in 0..(self.cols * self.rows) {
            let col = i % self.cols;
            let row = i / self.cols;
            let x = rect.min.x + pad + (cell_width + pad) * col as f32;
            let y = rect.min.y + pad + (cell_height + pad) * row as f32;
            let cell_rect =
                Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_width, cell_height));

            InventoryCell::new(self.image).paint(painter, cell_rect);
        }
    }
}
