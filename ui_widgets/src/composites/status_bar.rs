use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{
    AP_COLOR, AP_SPENT_COLOR, HP_COLOR, HP_SPENT_COLOR, MAIN_COLOR, MP_COLOR, MP_SPENT_COLOR,
    SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR,
};
use crate::egui::{self, Align2, CornerRadius, Rect, Stroke};
use crate::molecules::ProgressBar;
use crate::traits::{Roundable, WithText};

/// Result of rendering a `StatusBar`. Each field is `Some(new_value)` when
/// the corresponding progress bar was clicked.
pub struct StatusBarResponse {
    pub hp: Option<u32>,
    pub mp: Option<u32>,
    pub ap: Option<u32>,
}

/// Displays the character's HP, MP, and AP as three horizontal progress bars.
pub struct StatusBar {
    hp_current: u32,
    hp_max: u32,
    mp_current: u32,
    mp_max: u32,
    ap_current: u32,
    ap_max: u32,
    initiative: i32,
}

impl StatusBar {
    pub fn new(
        hp_current: u32,
        hp_max: u32,
        mp_current: u32,
        mp_max: u32,
        ap_current: u32,
        ap_max: u32,
        initiative: i32,
    ) -> Self {
        Self {
            hp_current,
            hp_max,
            mp_current,
            mp_max,
            ap_current,
            ap_max,
            initiative,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> StatusBarResponse {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(available_width, available_height),
            egui::Sense::hover(),
        );

        // Draw background
        {
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::same(12), MAIN_COLOR);
            let stroke = Stroke::new(1.0, STROKE_COLOR);
            if stroke.width > 0.0 {
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(12),
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }

        // Inner rect with 7% padding from top/left/right, max 65% of widget height
        let pad = rect.height() * 0.07;
        let content_height = rect.height() * 0.65;
        let inner_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + pad, rect.min.y + pad),
            egui::vec2(rect.width() - pad * 2.0, content_height),
        );

        // Render progress bars, collecting click results
        let bar_spacing = inner_rect.height() * 0.05;
        let bar_height = (inner_rect.height() - bar_spacing * 2.0) / 3.0;

        let hp_rect = Rect::from_min_size(
            egui::pos2(inner_rect.min.x, inner_rect.min.y),
            egui::vec2(inner_rect.width(), bar_height),
        );
        let mp_rect = Rect::from_min_size(
            egui::pos2(
                inner_rect.min.x,
                inner_rect.min.y + bar_height + bar_spacing,
            ),
            egui::vec2(inner_rect.width(), bar_height),
        );
        let ap_rect = Rect::from_min_size(
            egui::pos2(
                inner_rect.min.x,
                inner_rect.min.y + 2.0 * (bar_height + bar_spacing),
            ),
            egui::vec2(inner_rect.width(), bar_height),
        );

        let hp = {
            let mut bar_ui = ui.new_child(egui::UiBuilder::new().max_rect(hp_rect));
            ProgressBar::new("HP", self.hp_current, self.hp_max, HP_COLOR, HP_SPENT_COLOR)
                .show(&mut bar_ui)
        };
        let mp = {
            let mut bar_ui = ui.new_child(egui::UiBuilder::new().max_rect(mp_rect));
            ProgressBar::new("MP", self.mp_current, self.mp_max, MP_COLOR, MP_SPENT_COLOR)
                .show(&mut bar_ui)
        };
        let ap = {
            let mut bar_ui = ui.new_child(egui::UiBuilder::new().max_rect(ap_rect));
            ProgressBar::new("AP", self.ap_current, self.ap_max, AP_COLOR, AP_SPENT_COLOR)
                .show(&mut bar_ui)
        };

        // Initiative sub-widget below progress bars
        {
            let painter = ui.painter();
            let text_size = rect.height() * 0.10;
            let box_height = rect.height() * 0.20;
            let box_width = rect.width() * 0.14;

            let initiative_center_y = (inner_rect.max.y + rect.max.y) / 2.0;

            let box_rect = Rect::from_center_size(
                egui::pos2(inner_rect.max.x - box_width / 2.0, initiative_center_y),
                egui::vec2(box_width, box_height),
            );

            ShapeBox::new(Shape::Rectangle)
                .fill(SECONDARY_COLOR)
                .stroke(Stroke::NONE)
                .set_rounding(CornerRadius::same(8))
                .set_text(&self.initiative.to_string())
                .set_text_color(TEXT_COLOR)
                .set_text_size(text_size)
                .paint(painter, box_rect);

            let per_rect = Rect::from_min_max(
                egui::pos2(inner_rect.min.x, box_rect.min.y),
                egui::pos2(box_rect.min.x - 6.0, box_rect.max.y),
            );
            Text::new("Per")
                .color(TEXT_COLOR)
                .size(text_size)
                .align(Align2::RIGHT_CENTER)
                .paint(painter, per_rect);

            let initiative_rect = Rect::from_min_max(
                egui::pos2(inner_rect.min.x, box_rect.min.y),
                egui::pos2(per_rect.max.x, box_rect.max.y),
            );
            Text::new("Initiative")
                .color(TEXT_COLOR)
                .size(text_size)
                .align(Align2::LEFT_CENTER)
                .paint(painter, initiative_rect);
        }

        StatusBarResponse { hp, mp, ap }
    }
}
