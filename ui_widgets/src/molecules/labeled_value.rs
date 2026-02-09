use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{SECONDARY_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, CornerRadius, Rect, Stroke, StrokeKind};
use crate::traits::{Roundable, WithText};

/// A row with a label on the left, a tag (e.g. stat abbreviation)
/// right-aligned before a value box on the right.
///
/// Used for initiative in the status bar and skill entries in the skills panel.
pub struct LabeledValue {
    label: String,
    tag: String,
    value: String,
    text_size: Option<f32>,
    stroke: Stroke,
    rounding: CornerRadius,
    box_rounding: CornerRadius,
}

impl LabeledValue {
    pub fn new(label: impl Into<String>, tag: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tag: tag.into(),
            value: value.into(),
            text_size: None,
            stroke: Stroke::NONE,
            rounding: CornerRadius::ZERO,
            box_rounding: CornerRadius::same(8),
        }
    }

    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn rounding(mut self, rounding: CornerRadius) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn box_rounding(mut self, rounding: CornerRadius) -> Self {
        self.box_rounding = rounding;
        self
    }

    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        if self.stroke.width > 0.0 {
            painter.rect_stroke(rect, self.rounding, self.stroke, StrokeKind::Inside);
        }

        let text_size = self.text_size.unwrap_or(rect.height() * 0.45);
        let box_height = rect.height() * 0.75;
        let box_width = box_height * 1.64;
        let box_pad = (rect.height() - box_height) / 2.0;
        let text_pad = (rect.height() - text_size) / 2.0;

        let box_rect = Rect::from_center_size(
            egui::pos2(rect.max.x - box_width / 2.0 - box_pad, rect.center().y),
            egui::vec2(box_width, box_height),
        );

        ShapeBox::new(Shape::Rectangle)
            .fill(SECONDARY_COLOR)
            .stroke(Stroke::NONE)
            .set_rounding(self.box_rounding)
            .set_text(&self.value)
            .set_text_color(TEXT_COLOR)
            .set_text_size(text_size)
            .paint(painter, box_rect);

        let tag_rect = Rect::from_min_max(
            egui::pos2(rect.min.x + text_pad, box_rect.min.y),
            egui::pos2(box_rect.min.x - 6.0, box_rect.max.y),
        );
        Text::new(&self.tag)
            .color(TEXT_COLOR)
            .size(text_size)
            .align(Align2::RIGHT_CENTER)
            .paint(painter, tag_rect);

        let label_rect = Rect::from_min_max(
            egui::pos2(rect.min.x + text_pad, box_rect.min.y),
            egui::pos2(tag_rect.max.x, box_rect.max.y),
        );
        Text::new(&self.label)
            .color(TEXT_COLOR)
            .size(text_size)
            .align(Align2::LEFT_CENTER)
            .paint(painter, label_rect);
    }
}
