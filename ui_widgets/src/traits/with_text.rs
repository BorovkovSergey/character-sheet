use crate::egui::{Align2, Color32};

/// A trait for widgets that support rendering text content within their bounds.
pub trait WithText: Sized {
    /// Sets the text content to display.
    fn set_text(self, text: impl Into<String>) -> Self;

    /// Sets the text color.
    fn set_text_color(self, color: Color32) -> Self;

    /// Sets the font size in points.
    fn set_text_size(self, size: f32) -> Self;

    /// Sets the alignment of text within the widget bounds.
    fn set_text_align(self, align: Align2) -> Self;

    /// Sets the rotation angle of the text in radians (clockwise).
    fn set_text_angle(self, angle: f32) -> Self;
}
