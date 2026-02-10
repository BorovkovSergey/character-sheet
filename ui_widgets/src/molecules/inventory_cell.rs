use crate::atoms::{Shape, ShapeBox};
use crate::colors::MAIN_COLOR;
use crate::egui::{self, Color32, CornerRadius, Rect, Stroke, TextureId};
use crate::traits::Roundable;

/// A single inventory slot: a rounded white box with an item image inside.
pub struct InventoryCell {
    image: TextureId,
}

impl InventoryCell {
    pub fn new(image: TextureId) -> Self {
        Self { image }
    }

    /// Paints the inventory cell into the given rect.
    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        ShapeBox::new(Shape::Rectangle)
            .fill(MAIN_COLOR)
            .stroke(Stroke::NONE)
            .set_rounding(CornerRadius::same(14))
            .paint(painter, rect);

        let img_size = rect.size() * 0.46;
        let img_rect = Rect::from_center_size(rect.center(), img_size);
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(self.image, img_rect, uv, Color32::WHITE);
    }
}
