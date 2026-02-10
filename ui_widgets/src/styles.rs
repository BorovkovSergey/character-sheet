use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Stroke};

/// Global UI style settings shared across widgets.
pub struct UiStyle;

impl UiStyle {
    /// Content padding ratio relative to screen height.
    /// Applied as top, bottom, and right padding inside `TitledBox` content areas.
    const CONTENT_PADDING_RATIO: f32 = 0.002;

    /// Returns the standard content padding (top/bottom/right) for `TitledBox`,
    /// computed as 0.2 % of the available screen height.
    pub fn content_padding(ui: &egui::Ui) -> f32 {
        ui.ctx().content_rect().height() * Self::CONTENT_PADDING_RATIO
    }

    /// Sets the global egui visuals on the [`egui::Context`] to match the app's
    /// light color scheme. Call once at startup so that all egui-managed overlays
    /// (context menus, popup windows, tooltips) use the correct frame and colors.
    pub fn apply_global_style(ctx: &egui::Context) {
        let mut v = egui::Visuals::light();
        v.window_fill = MAIN_COLOR;
        v.panel_fill = MAIN_COLOR;
        v.window_stroke = Stroke::new(1.0, STROKE_COLOR);
        v.window_shadow = egui::Shadow::NONE;
        v.popup_shadow = egui::Shadow::NONE;
        v.faint_bg_color = SECONDARY_COLOR;
        v.extreme_bg_color = SECONDARY_COLOR;
        v.override_text_color = Some(TEXT_COLOR);

        v.widgets.noninteractive.bg_fill = MAIN_COLOR;
        v.widgets.noninteractive.weak_bg_fill = MAIN_COLOR;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, STROKE_COLOR);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_COLOR);

        v.widgets.inactive.bg_fill = SECONDARY_COLOR;
        v.widgets.inactive.weak_bg_fill = SECONDARY_COLOR;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, STROKE_COLOR);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_COLOR);

        v.widgets.hovered.bg_fill = STROKE_COLOR;
        v.widgets.hovered.weak_bg_fill = STROKE_COLOR;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, STROKE_COLOR);
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_COLOR);

        v.widgets.active.bg_fill = STROKE_COLOR;
        v.widgets.active.weak_bg_fill = STROKE_COLOR;
        v.widgets.active.bg_stroke = Stroke::new(1.0, STROKE_COLOR);
        v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_COLOR);

        ctx.set_visuals(v);
    }
}
