use crate::egui::Vec2;

/// A trait for widgets that support configurable minimum and maximum sizes.
pub trait Sizeable {
    /// Sets the maximum size for the widget, clamping it to at most this size.
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_max_size(&mut self, size: Vec2) -> &mut Self;

    /// Sets the minimum size for the widget, ensuring it is at least this size.
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_min_size(&mut self, size: Vec2) -> &mut Self;
}
