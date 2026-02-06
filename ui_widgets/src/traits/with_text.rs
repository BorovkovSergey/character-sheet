use crate::egui::{Align2, Color32};

/// A trait for widgets that support rendering text content within their bounds.
pub trait WithText {
    /// Sets the text content to display.
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_text(&mut self, text: impl Into<String>) -> &mut Self;

    /// Sets the text color.
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_text_color(&mut self, color: Color32) -> &mut Self;

    /// Sets the font size in points.
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_text_size(&mut self, size: f32) -> &mut Self;

    /// Sets the alignment of text within the widget bounds.
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_text_align(&mut self, align: Align2) -> &mut Self;

    /// Sets the rotation angle of the text in radians (clockwise).
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_text_angle(&mut self, angle: f32) -> &mut Self;
}
