use crate::atoms::Text;
use crate::colors::{SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, CornerRadius, Pos2, Rect, Stroke, StrokeKind, Vec2, Widget};

/// Displays the character's name and core identity information.
///
/// Layout (top to bottom):
/// - Top row: single ShapeBox occupying 52% of total height, with rounded top corners.
/// - Gap: 4% of total height.
/// - Bottom row: two side-by-side ShapeBoxes occupying 44% of total height,
///   separated by a 1% horizontal gap.
pub struct IdentityBar {
    name: String,
    race: String,
    class: String,
}

impl IdentityBar {
    pub fn new(name: impl Into<String>, race: impl Into<String>, class: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            race: race.into(),
            class: class.into(),
        }
    }
}

impl Widget for IdentityBar {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::hover());

        let total_w = rect.width();
        let total_h = rect.height();

        let top_h = total_h * 0.52;
        let gap_h = total_h * 0.04;
        let bottom_h = total_h * 0.44;
        let col_gap = total_w * 0.01;

        // Top row rect
        let top_rect = Rect::from_min_size(rect.min, Vec2::new(total_w, top_h));

        // Bottom row origin
        let bottom_y = rect.min.y + top_h + gap_h;
        let bottom_col_w = (total_w - col_gap) * 0.5;

        let bottom_left_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, bottom_y),
            Vec2::new(bottom_col_w, bottom_h),
        );

        let bottom_right_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + bottom_col_w + col_gap, bottom_y),
            Vec2::new(bottom_col_w, bottom_h),
        );

        let painter = ui.painter();
        let stroke = Stroke::new(1.0, STROKE_COLOR);

        // Top box: top corners 16, bottom corners 4
        let top_rounding = CornerRadius {
            nw: 16,
            ne: 16,
            sw: 4,
            se: 4,
        };
        painter.rect(
            top_rect,
            top_rounding,
            SECONDARY_COLOR,
            stroke,
            StrokeKind::Inside,
        );
        Text::new(&self.name)
            .color(TEXT_COLOR)
            .size(20.0)
            .bold()
            .paint(painter, top_rect);

        // Bottom-left box: bottom-left corner 12, rest 4
        let bl_rounding = CornerRadius {
            nw: 4,
            ne: 4,
            sw: 12,
            se: 4,
        };
        painter.rect(
            bottom_left_rect,
            bl_rounding,
            SECONDARY_COLOR,
            stroke,
            StrokeKind::Inside,
        );
        Text::new(&self.race)
            .color(TEXT_COLOR)
            .paint(painter, bottom_left_rect);

        // Bottom-right box: bottom-right corner 12, rest 4
        let br_rounding = CornerRadius {
            nw: 4,
            ne: 4,
            sw: 4,
            se: 12,
        };
        painter.rect(
            bottom_right_rect,
            br_rounding,
            SECONDARY_COLOR,
            stroke,
            StrokeKind::Inside,
        );
        Text::new(&self.class)
            .color(TEXT_COLOR)
            .paint(painter, bottom_right_rect);

        response
    }
}
