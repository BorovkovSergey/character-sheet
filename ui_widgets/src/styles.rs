use crate::egui;

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
}
