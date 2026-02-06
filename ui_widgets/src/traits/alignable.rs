use crate::egui::Align2;

/// A trait for widgets that support alignment within their available space.
pub trait Alignable {
    /// Sets the alignment of the widget within the available space.
    /// Builder-style: returns `&mut Self` for chaining.
    fn set_align(&mut self, align: Align2) -> &mut Self;
}
