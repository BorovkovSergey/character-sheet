use std::f32::consts::FRAC_PI_2;

use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{SECONDARY_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, Widget};
use crate::traits::Roundable;

/// A rectangular box with a rotated title strip on the left and a content area on the right.
///
/// The title text is rotated 90 degrees counter-clockwise (reads bottom-to-top) and painted
/// in a narrow vertical header strip. The content area fills the remaining space to the right.
pub struct TitledBox {
    title: String,
    fill: Color32,
    rounding: CornerRadius,
    content_fill: Option<Color32>,
    content_rounding: CornerRadius,
}

impl TitledBox {
    /// Creates a new `TitledBox` with the given title text.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            fill: SECONDARY_COLOR,
            rounding: CornerRadius::ZERO,
            content_fill: None,
            content_rounding: CornerRadius::ZERO,
        }
    }

    /// Sets the fill color for the outer box.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = fill;
        self
    }

    /// Sets the corner rounding for the outer box.
    pub fn rounding(mut self, rounding: impl Into<CornerRadius>) -> Self {
        self.rounding = rounding.into();
        self
    }

    /// Sets the fill color for the content area (painted as a ShapeBox).
    pub fn content_fill(mut self, fill: Color32) -> Self {
        self.content_fill = Some(fill);
        self
    }

    /// Sets the corner rounding for the content area.
    pub fn content_rounding(mut self, rounding: impl Into<CornerRadius>) -> Self {
        self.content_rounding = rounding.into();
        self
    }

    /// Splits the allocated rect into a header strip (left) and content area (right).
    fn split_rect(rect: Rect) -> (Rect, Rect) {
        let header_width = (rect.width() * 0.08).max(20.0);
        let header_rect =
            Rect::from_min_max(rect.min, egui::pos2(rect.min.x + header_width, rect.max.y));
        let content_rect =
            Rect::from_min_max(egui::pos2(rect.min.x + header_width, rect.min.y), rect.max);
        (header_rect, content_rect)
    }

    fn paint(&self, painter: &egui::Painter, rect: Rect) {
        if self.rounding != CornerRadius::ZERO {
            let clipped = painter.with_clip_rect(rect);
            clipped.rect_filled(rect.expand(1.0), self.rounding, self.fill);
        } else {
            painter.rect_filled(rect, CornerRadius::ZERO, self.fill);
        }

        let (header_rect, content_rect) = Self::split_rect(rect);

        // Rotated title text in the header strip
        Text::new(&self.title)
            .color(TEXT_COLOR)
            .size(12.0)
            .align(Align2::CENTER_CENTER)
            .angle(-FRAC_PI_2)
            .paint(painter, header_rect);

        if let Some(content_fill) = self.content_fill {
            let padding = 4.0;
            let inner_rect = content_rect.shrink(padding);
            let mut shape = ShapeBox::new(Shape::Rectangle)
                .fill(content_fill)
                .stroke(Stroke::NONE);
            shape.set_rounding(self.content_rounding);
            shape.paint(painter, inner_rect);
        }
    }
}

impl Widget for TitledBox {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::hover());
        self.paint(ui.painter(), rect);
        response
    }
}
