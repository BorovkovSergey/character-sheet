use std::f32::consts::FRAC_PI_2;

use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{SECONDARY_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, Widget};
use crate::traits::Roundable;

/// Controls where the title strip is placed within a [`TitledBox`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TitlePosition {
    /// Vertical header strip on the left with rotated text (bottom-to-top).
    #[default]
    Left,
    /// Horizontal header strip on top with normal text.
    Top,
}

/// A rectangular box with a title strip and a content area.
///
/// When [`TitlePosition::Left`] (the default), the title text is rotated 90 degrees
/// counter-clockwise (reads bottom-to-top) and painted in a narrow vertical header strip.
/// The content area fills the remaining space to the right.
///
/// When [`TitlePosition::Top`], the title text is drawn horizontally in a narrow strip
/// across the top. The content area fills the remaining space below.
pub struct TitledBox {
    title: String,
    title_position: TitlePosition,
    title_angle: Option<f32>,
    header_ratio: Option<f32>,
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
            title_position: TitlePosition::default(),
            title_angle: None,
            header_ratio: None,
            fill: SECONDARY_COLOR,
            rounding: CornerRadius::ZERO,
            content_fill: None,
            content_rounding: CornerRadius::ZERO,
        }
    }

    /// Sets the position of the title strip.
    pub fn title_position(mut self, position: TitlePosition) -> Self {
        self.title_position = position;
        self
    }

    /// Overrides the title text rotation angle (in radians).
    ///
    /// By default, [`TitlePosition::Left`] uses `-PI/2` and [`TitlePosition::Top`]
    /// uses `0.0`. This method lets you override that (e.g. keep text horizontal
    /// in a left-side strip by passing `0.0`).
    pub fn title_angle(mut self, angle: f32) -> Self {
        self.title_angle = Some(angle);
        self
    }

    /// Overrides the header strip size as a fraction of the total box dimension.
    ///
    /// For [`TitlePosition::Left`] this controls width; for [`TitlePosition::Top`]
    /// this controls height. Default is `0.08` (8%).
    pub fn header_ratio(mut self, ratio: f32) -> Self {
        self.header_ratio = Some(ratio);
        self
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

    /// Splits the allocated rect into a header strip and content area based on `position`.
    ///
    /// - [`TitlePosition::Left`]: vertical strip on the left (8% of width, min 20px).
    /// - [`TitlePosition::Top`]: horizontal strip on the top (8% of height, min 20px).
    fn split_rect(rect: Rect, position: TitlePosition, ratio: Option<f32>) -> (Rect, Rect) {
        let default_ratio = 0.08;
        match position {
            TitlePosition::Left => {
                let header_width = (rect.width() * ratio.unwrap_or(default_ratio)).max(20.0);
                let header_rect =
                    Rect::from_min_max(rect.min, egui::pos2(rect.min.x + header_width, rect.max.y));
                let content_rect =
                    Rect::from_min_max(egui::pos2(rect.min.x + header_width, rect.min.y), rect.max);
                (header_rect, content_rect)
            }
            TitlePosition::Top => {
                let header_height = (rect.height() * ratio.unwrap_or(default_ratio)).max(20.0);
                let header_rect = Rect::from_min_max(
                    rect.min,
                    egui::pos2(rect.max.x, rect.min.y + header_height),
                );
                let content_rect = Rect::from_min_max(
                    egui::pos2(rect.min.x, rect.min.y + header_height),
                    rect.max,
                );
                (header_rect, content_rect)
            }
        }
    }

    /// Paints the outer background and title strip, returning the content rect.
    fn paint_chrome(&self, painter: &egui::Painter, rect: Rect) -> Rect {
        painter.rect_filled(rect, self.rounding, self.fill);

        let (header_rect, content_rect) =
            Self::split_rect(rect, self.title_position, self.header_ratio);

        let default_angle = match self.title_position {
            TitlePosition::Left => -FRAC_PI_2,
            TitlePosition::Top => 0.0,
        };
        let angle = self.title_angle.unwrap_or(default_angle);

        let title = Text::new(&self.title)
            .color(TEXT_COLOR)
            .size(12.0)
            .align(Align2::CENTER_CENTER)
            .angle(angle);

        title.paint(painter, header_rect);

        content_rect
    }

    fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let content_rect = self.paint_chrome(painter, rect);

        if let Some(content_fill) = self.content_fill {
            let inner_rect = content_rect.shrink(4.0);
            let shape = ShapeBox::new(Shape::Rectangle)
                .fill(content_fill)
                .stroke(Stroke::NONE)
                .set_rounding(self.content_rounding);
            shape.paint(painter, inner_rect);
        }
    }

    /// Renders the titled box and places child widgets inside the content area.
    ///
    /// Unlike the [`Widget`] impl, this method accepts a closure that receives
    /// a `&mut Ui` scoped to the padded content rect, so you can place
    /// arbitrary child widgets inside the box.
    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::hover());

        let content_rect = self.paint_chrome(ui.painter(), rect);

        let pad_top = content_rect.height() * 0.02;
        let pad_bottom = pad_top;
        let pad_right = pad_top;
        let pad_left = content_rect.width() * 0.005;
        let inner_rect = Rect::from_min_max(
            egui::pos2(content_rect.min.x + pad_left, content_rect.min.y + pad_top),
            egui::pos2(
                content_rect.max.x - pad_right,
                content_rect.max.y - pad_bottom,
            ),
        );

        if let Some(content_fill) = self.content_fill {
            let shape = ShapeBox::new(Shape::Rectangle)
                .fill(content_fill)
                .stroke(Stroke::NONE)
                .set_rounding(self.content_rounding);
            shape.paint(ui.painter(), inner_rect);
        }

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Min)),
        );
        child_ui.set_clip_rect(inner_rect.intersect(ui.clip_rect()));
        content(&mut child_ui);

        response
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
