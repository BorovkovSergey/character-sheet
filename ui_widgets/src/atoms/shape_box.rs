use crate::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, StrokeKind, Vec2, Widget};
use crate::traits::{Alignable, Corner, Roundable, Sizeable, WithText};

use super::Text;

/// The geometric shape to draw inside a `ShapeBox`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle,
    Rectangle,
}

/// A simple flat single-color shape widget with a border/stroke.
///
/// By default it fills all available space in the UI. Use `Sizeable` and
/// `Alignable` trait methods (or the builder helpers) to constrain and
/// position the shape.
#[derive(Debug, Clone)]
pub struct ShapeBox {
    fill: Color32,
    stroke: Stroke,
    shape: Shape,
    rounding: CornerRadius,
    max_size: Option<Vec2>,
    min_size: Option<Vec2>,
    align: Align2,
    text: Option<Text>,
}

impl ShapeBox {
    /// Creates a new `ShapeBox` with sensible defaults:
    /// - gray fill
    /// - 1px dark stroke
    /// - no size constraints
    /// - center alignment
    pub fn new(shape: Shape) -> Self {
        Self {
            fill: Color32::GRAY,
            stroke: Stroke::new(1.0, Color32::DARK_GRAY),
            shape,
            rounding: CornerRadius::ZERO,
            max_size: None,
            min_size: None,
            align: Align2::CENTER_CENTER,
            text: None,
        }
    }

    /// Sets the fill color (consumes and returns `Self` for builder chaining).
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = fill;
        self
    }

    /// Sets the stroke (consumes and returns `Self` for builder chaining).
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Sets the shape kind (consumes and returns `Self` for builder chaining).
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the text content (consumes and returns `Self` for builder chaining).
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let s = text.into();
        self.text = Some(match self.text {
            Some(mut t) => {
                t.set_text(s);
                t
            }
            None => Text::new(s),
        });
        self
    }

    /// Sets the text color (consumes and returns `Self` for builder chaining).
    pub fn text_color(mut self, color: Color32) -> Self {
        if let Some(ref mut t) = self.text {
            t.set_text_color(color);
        }
        self
    }

    /// Sets the font size in points (consumes and returns `Self` for builder chaining).
    pub fn text_size(mut self, size: f32) -> Self {
        if let Some(ref mut t) = self.text {
            t.set_text_size(size);
        }
        self
    }

    /// Sets the text alignment within the widget bounds (consumes and returns `Self` for builder chaining).
    pub fn text_align(mut self, align: Align2) -> Self {
        if let Some(ref mut t) = self.text {
            t.set_text_align(align);
        }
        self
    }

    /// Sets the text rotation angle in radians (consumes and returns `Self` for builder chaining).
    pub fn text_angle(mut self, angle: f32) -> Self {
        if let Some(ref mut t) = self.text {
            t.set_text_angle(angle);
        }
        self
    }

    /// Computes the final desired size from the available space, respecting
    /// min/max constraints. When both min and max are set, max takes priority.
    fn compute_desired_size(&self, available: Vec2) -> Vec2 {
        let mut size = available;

        // Apply min_size: component-wise max
        if let Some(min) = self.min_size {
            size.x = size.x.max(min.x);
            size.y = size.y.max(min.y);
        }

        // Apply max_size last so it wins on conflict with min_size
        if let Some(max) = self.max_size {
            size.x = size.x.min(max.x);
            size.y = size.y.min(max.y);
        }

        size
    }

    /// Paints the shape into the given rect using the provided painter.
    fn paint(&self, painter: &egui::Painter, rect: Rect) {
        match self.shape {
            Shape::Rectangle => {
                painter.rect_filled(rect, self.rounding, self.fill);
                painter.rect_stroke(rect, self.rounding, self.stroke, StrokeKind::Inside);
            }
            Shape::Circle => {
                let diameter = rect.width().min(rect.height());
                let radius = diameter * 0.5;
                let center = rect.center();
                painter.circle_filled(center, radius, self.fill);
                painter.circle_stroke(center, radius, self.stroke);
            }
        }

        if let Some(ref text) = self.text {
            text.paint(painter, rect);
        }
    }
}

impl Sizeable for ShapeBox {
    fn set_max_size(&mut self, size: Vec2) -> &mut Self {
        self.max_size = Some(size);
        self
    }

    fn set_min_size(&mut self, size: Vec2) -> &mut Self {
        self.min_size = Some(size);
        self
    }
}

impl Alignable for ShapeBox {
    fn set_align(&mut self, align: Align2) -> &mut Self {
        self.align = align;
        self
    }
}

impl Roundable for ShapeBox {
    fn set_rounding(&mut self, rounding: CornerRadius) -> &mut Self {
        self.rounding = rounding;
        self
    }

    fn set_corner_rounding(&mut self, corner: Corner, radius: u8) -> &mut Self {
        match corner {
            Corner::TopLeft => self.rounding.nw = radius,
            Corner::TopRight => self.rounding.ne = radius,
            Corner::BottomLeft => self.rounding.sw = radius,
            Corner::BottomRight => self.rounding.se = radius,
        }
        self
    }
}

impl WithText for ShapeBox {
    fn set_text(&mut self, text: impl Into<String>) -> &mut Self {
        let s = text.into();
        self.text = Some(match self.text.take() {
            Some(mut t) => {
                t.set_text(s);
                t
            }
            None => Text::new(s),
        });
        self
    }

    fn set_text_color(&mut self, color: Color32) -> &mut Self {
        if let Some(ref mut t) = self.text {
            t.set_text_color(color);
        }
        self
    }

    fn set_text_size(&mut self, size: f32) -> &mut Self {
        if let Some(ref mut t) = self.text {
            t.set_text_size(size);
        }
        self
    }

    fn set_text_align(&mut self, align: Align2) -> &mut Self {
        if let Some(ref mut t) = self.text {
            t.set_text_align(align);
        }
        self
    }

    fn set_text_angle(&mut self, angle: f32) -> &mut Self {
        if let Some(ref mut t) = self.text {
            t.set_text_angle(angle);
        }
        self
    }
}

impl Widget for ShapeBox {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let available = ui.available_size();
        let desired = self.compute_desired_size(available);

        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());

        self.paint(ui.painter(), rect);

        response
    }
}
