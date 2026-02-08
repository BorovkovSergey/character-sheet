use crate::egui::{self, Align, Align2, Color32, Rect, TextureId, Vec2};

/// A texture-based icon that can be painted inside a bounding rect.
///
/// Supports alignment, explicit sizing, and color tinting.
#[derive(Debug, Clone)]
pub struct Icon {
    texture_id: TextureId,
    align: Align2,
    size: Vec2,
    tint: Color32,
}

impl Icon {
    /// Creates a new `Icon` with sensible defaults:
    /// - center-bottom alignment
    /// - 20x20 size
    /// - no tint (white)
    pub fn new(texture_id: TextureId) -> Self {
        Self {
            texture_id,
            align: Align2::CENTER_BOTTOM,
            size: egui::vec2(20.0, 20.0),
            tint: Color32::WHITE,
        }
    }

    /// Sets the alignment within the parent rect (consumes and returns `Self`).
    pub fn align(mut self, align: Align2) -> Self {
        self.align = align;
        self
    }

    /// Sets the icon size in points (consumes and returns `Self`).
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Sets the tint color (consumes and returns `Self`).
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = tint;
        self
    }

    pub fn set_align(&mut self, align: Align2) -> &mut Self {
        self.align = align;
        self
    }

    pub fn set_size(&mut self, size: Vec2) -> &mut Self {
        self.size = size;
        self
    }

    pub fn set_tint(&mut self, tint: Color32) -> &mut Self {
        self.tint = tint;
        self
    }

    /// Returns the shape rect inset to reserve space for the icon's protrusion.
    ///
    /// The protruding half of the icon (outside the shape border) is accounted
    /// for by shrinking the rect on the aligned edge.
    pub fn inset_rect(&self, rect: Rect) -> Rect {
        let half_w = self.size.x * 0.5;
        let half_h = self.size.y * 0.5;

        let top = if matches!(self.align.y(), Align::Min) {
            half_h
        } else {
            0.0
        };
        let bottom = if matches!(self.align.y(), Align::Max) {
            half_h
        } else {
            0.0
        };
        let left = if matches!(self.align.x(), Align::Min) {
            half_w
        } else {
            0.0
        };
        let right = if matches!(self.align.x(), Align::Max) {
            half_w
        } else {
            0.0
        };

        Rect::from_min_max(
            egui::pos2(rect.min.x + left, rect.min.y + top),
            egui::pos2(rect.max.x - right, rect.max.y - bottom),
        )
    }

    /// Computes the icon rect so the icon straddles the edge of the parent rect.
    ///
    /// The icon's center is placed on the aligned edge. For example,
    /// `CENTER_BOTTOM` means the icon is centered horizontally and its
    /// vertical center sits on the bottom border — half inside, half outside.
    fn compute_rect(&self, rect: Rect) -> Rect {
        let half_w = self.size.x * 0.5;
        let half_h = self.size.y * 0.5;

        let x = match self.align.x() {
            Align::Min => rect.left() - half_w,
            Align::Center => rect.center().x - half_w,
            Align::Max => rect.right() - half_w,
        };
        let y = match self.align.y() {
            Align::Min => rect.top() - half_h,
            Align::Center => rect.center().y - half_h,
            Align::Max => rect.bottom() - half_h,
        };

        Rect::from_min_size(egui::pos2(x, y), self.size)
    }

    /// Paints the icon directly onto the painter without affecting UI layout.
    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let icon_rect = self.compute_rect(rect);
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(self.texture_id, icon_rect, uv, self.tint);
    }
}
