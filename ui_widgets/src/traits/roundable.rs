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
    fn set_rounding(self, rounding: CornerRadius) -> Self;

    /// Sets the same radius for all four corners.
    fn set_rounding_all(self, radius: u8) -> Self
    where
        Self: Sized,
    {
        self.set_rounding(CornerRadius::same(radius))
    }

    /// Sets the radius for a single corner, leaving others unchanged.
    fn set_corner_rounding(self, corner: Corner, radius: u8) -> Self;

    /// Sets the same radius for all corners except the specified one(s).
    fn set_rounding_except(mut self, corners: &[Corner], radius: u8) -> Self
    where
        Self: Sized,
    {
        self = self.set_rounding_all(radius);
        for &corner in corners {
            self = self.set_corner_rounding(corner, 0);
        }
        self
    }

    /// Sets the radius only for the specified corners, others stay at 0.
    fn set_rounding_only(mut self, corners: &[Corner], radius: u8) -> Self
    where
        Self: Sized,
    {
        self = self.set_rounding(CornerRadius::ZERO);
        for &corner in corners {
            self = self.set_corner_rounding(corner, radius);
        }
        self
    }
}
