//! A bounding box that aligns with the object in the xy plane.

use cgmath::Rad;
use three_d_asset::{PixelPoint, Radians};

///
/// A bounding box that aligns with the object in the xy plane.
///
#[derive(Debug, Copy, Clone)]
pub struct OrientedBoundingBox2D {
    /// width of the bounding box
    pub width: f32,
    /// height of the bounding box
    pub height: f32,
    /// center of the bounding box
    pub center: PixelPoint,
    /// rotation of the bounding box
    pub rotation: Radians,
}

impl OrientedBoundingBox2D {
    ///
    /// Creates an new instance of [OrientedBoundingBox2D]
    ///
    pub fn new(width: f32, height: f32, center: PixelPoint, rotation: impl Into<Radians>) -> Self {
        Self {
            width,
            height,
            center,
            rotation: rotation.into(),
        }
    }
}

impl Default for OrientedBoundingBox2D {
    fn default() -> Self {
        OrientedBoundingBox2D::new(1.0, 1.0, PixelPoint { x: 0.0, y: 0.0 }, Rad(0.0))
    }
}
