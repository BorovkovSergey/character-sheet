use crate::egui::CornerRadius;

/// Identifies a single corner of a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// A trait for widgets that support corner rounding (rectangles).
pub trait Roundable {
    /// Sets rounding for all four corners at once using an egui `CornerRadius`.
    fn set_rounding(&mut self, rounding: CornerRadius) -> &mut Self;

    /// Sets the same radius for all four corners.
    fn set_rounding_all(&mut self, radius: u8) -> &mut Self {
        self.set_rounding(CornerRadius::same(radius))
    }

    /// Sets the radius for a single corner, leaving others unchanged.
    fn set_corner_rounding(&mut self, corner: Corner, radius: u8) -> &mut Self;

    /// Sets the same radius for all corners except the specified one(s).
    fn set_rounding_except(&mut self, corners: &[Corner], radius: u8) -> &mut Self {
        self.set_rounding_all(radius);
        for &corner in corners {
            self.set_corner_rounding(corner, 0);
        }
        self
    }

    /// Sets the radius only for the specified corners, others stay at 0.
    fn set_rounding_only(&mut self, corners: &[Corner], radius: u8) -> &mut Self {
        self.set_rounding(CornerRadius::ZERO);
        for &corner in corners {
            self.set_corner_rounding(corner, radius);
        }
        self
    }
}
