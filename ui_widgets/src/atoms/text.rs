use crate::colors::STROKE_COLOR;
use crate::egui::{self, Align, Align2, Rect};
use crate::traits::WithText;

/// A styled text element that can be painted inside a bounding rect.
///
/// Text is clipped to the rect, can be aligned within it, and optionally rotated.
#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    color: egui::Color32,
    size: f32,
    align: Align2,
    angle: f32,
    bold: bool,
}

impl Text {
    /// Creates a new `Text` with sensible defaults:
    /// - stroke color (dark gray-blue)
    /// - 14pt proportional font
    /// - center-center alignment
    /// - no rotation
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            color: STROKE_COLOR,
            size: 14.0,
            align: Align2::CENTER_CENTER,
            angle: 0.0,
            bold: false,
        }
    }

    /// Sets the text color (consumes and returns `Self` for builder chaining).
    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    /// Sets the font size in points (consumes and returns `Self` for builder chaining).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the alignment within the bounding rect (consumes and returns `Self` for builder chaining).
    pub fn align(mut self, align: Align2) -> Self {
        self.align = align;
        self
    }

    /// Sets the rotation angle in radians (consumes and returns `Self` for builder chaining).
    pub fn angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    /// Enables bold font rendering (consumes and returns `Self` for builder chaining).
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Paints the text inside `rect`, clipped to its bounds.
    ///
    /// When bold is enabled, the text is painted twice with a 1px horizontal
    /// offset to produce a faux-bold effect without requiring a dedicated bold font.
    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let font_id = egui::FontId::proportional(self.size);
        let galley = painter.layout_no_wrap(self.content.clone(), font_id.clone(), self.color);
        let galley_size = galley.size();

        let anchor_x = match self.align.x() {
            Align::Min => rect.left(),
            Align::Center => rect.center().x - galley_size.x * 0.5,
            Align::Max => rect.right() - galley_size.x,
        };
        let anchor_y = match self.align.y() {
            Align::Min => rect.top(),
            Align::Center => rect.center().y - galley_size.y * 0.5,
            Align::Max => rect.bottom() - galley_size.y,
        };

        let pos = egui::pos2(anchor_x, anchor_y);
        let clipped = painter.with_clip_rect(rect);

        if self.bold {
            // Faux-bold: paint text twice with 1px horizontal offset
            let bold_offset = egui::pos2(pos.x + 1.0, pos.y);
            let galley_copy = painter.layout_no_wrap(self.content.clone(), font_id, self.color);
            let mut text_shape_offset =
                egui::epaint::TextShape::new(bold_offset, galley_copy, self.color);
            text_shape_offset.angle = self.angle;
            clipped.add(text_shape_offset);
        }

        let mut text_shape = egui::epaint::TextShape::new(pos, galley, self.color);
        text_shape.angle = self.angle;
        clipped.add(text_shape);
    }
}

impl WithText for Text {
    fn set_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.content = text.into();
        self
    }

    fn set_text_color(&mut self, color: egui::Color32) -> &mut Self {
        self.color = color;
        self
    }

    fn set_text_size(&mut self, size: f32) -> &mut Self {
        self.size = size;
        self
    }

    fn set_text_align(&mut self, align: Align2) -> &mut Self {
        self.align = align;
        self
    }

    fn set_text_angle(&mut self, angle: f32) -> &mut Self {
        self.angle = angle;
        self
    }
}
