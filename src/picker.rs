//! This module contains functionality for picking objects in a scene.

use three_d_asset::{Camera, PixelPoint, Vec3};

use crate::{ColorMaterial, Context, DepthMaterial, Geometry};

///
/// A trait that allows for objects to be picked in a collection of gemetries
///
pub trait Pick {
    ///
    /// The result of the pick operation
    ///
    type PickResult;

    ///
    /// This function will return the picked value
    ///
    fn pick(
        &self,
        camera: &Camera,
        pixel: impl Into<PixelPoint> + Copy,
        geometries: &[&dyn Geometry],
    ) -> Option<Self::PickResult>;
}

///
/// A picker which returns the location in the 3D scene shown at a pixel on the screen.
/// This picker can be used to get a point on the surface of a 3D model for example.
///
pub struct LocationPicker {
    context: Context,
}

impl LocationPicker {
    ///
    /// Create a new instance of the [LocationPicker].
    ///
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),
        }
    }

    ///
    /// Finds the closest intersection between a ray starting at the given position in the given direction and the given geometries.
    /// Returns ```None``` if no geometry was hit before the given maximum depth.
    ///
    fn ray_intersect(
        &self,
        position: Vec3,
        direction: Vec3,
        max_depth: f32,
        geometries: impl IntoIterator<Item = impl Geometry>,
    ) -> Option<Vec3> {
        use crate::core::*;
        let viewport = Viewport::new_at_origin(1, 1);
        let up = if direction.dot(vec3(1.0, 0.0, 0.0)).abs() > 0.99 {
            direction.cross(vec3(0.0, 1.0, 0.0))
        } else {
            direction.cross(vec3(1.0, 0.0, 0.0))
        };
        let camera = Camera::new_orthographic(
            viewport,
            position,
            position + direction * max_depth,
            up,
            0.01,
            0.0,
            max_depth,
        );
        let mut texture = Texture2D::new_empty::<f32>(
            &self.context,
            viewport.width,
            viewport.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let mut depth_texture = DepthTexture2D::new::<f32>(
            &self.context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let depth_material = DepthMaterial {
            render_states: RenderStates {
                write_mask: WriteMask {
                    red: true,
                    ..WriteMask::DEPTH
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let depth = RenderTarget::new(
            texture.as_color_target(None),
            depth_texture.as_depth_target(),
        )
        .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
        .write(|| {
            for geometry in geometries {
                geometry.render_with_material(&depth_material, &camera, &[]);
            }
        })
        .read_color()[0];
        if depth < 1.0 {
            Some(position + direction * depth * max_depth)
        } else {
            None
        }
    }
}

impl Pick for LocationPicker {
    type PickResult = Vec3;
    ///
    /// Finds the closest intersection between a ray from the given camera in the given pixel coordinate and the given geometries.
    /// The pixel coordinate must be in physical pixels, where (viewport.x, viewport.y) indicate the bottom left corner of the viewport
    /// and (viewport.x + viewport.width, viewport.y + viewport.height) indicate the top right corner.
    /// Returns ```None``` if no geometry was hit between the near (`z_near`) and far (`z_far`) plane for this camera.
    ///
    fn pick(
        &self,
        camera: &Camera,
        pixel: impl Into<PixelPoint> + Copy,
        geometries: &[&dyn Geometry],
    ) -> Option<Vec3> {
        let pos = camera.position_at_pixel(pixel);
        let dir = camera.view_direction_at_pixel(pixel);
        self.ray_intersect(
            pos + dir * camera.z_near(),
            dir,
            camera.z_far() - camera.z_near(),
            geometries,
        )
    }
}

///
/// A picker that returns the index of the picked object from the slice of geomerties passed to the pick method
///
pub struct ObjectPicker {
    context: Context,
}

impl ObjectPicker {
    ///
    /// Creates a new instance of the ObjectPicker
    ///
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),
        }
    }

    ///
    /// Finds the closest intersection between a ray starting at the given position in the given direction and the given geometries.
    /// Returns ```None``` if no geometry was hit before the given maximum depth.
    ///
    fn ray_intersect(
        &self,
        position: Vec3,
        direction: Vec3,
        max_depth: f32,
        geometries: &[&dyn Geometry],
    ) -> Option<usize> {
        use crate::core::*;
        let viewport = Viewport::new_at_origin(1, 1);
        let up = if direction.dot(vec3(1.0, 0.0, 0.0)).abs() > 0.99 {
            direction.cross(vec3(0.0, 1.0, 0.0))
        } else {
            direction.cross(vec3(1.0, 0.0, 0.0))
        };
        let camera = Camera::new_orthographic(
            viewport,
            position,
            position + direction * max_depth,
            up,
            0.01,
            0.0,
            max_depth,
        );
        let mut texture = Texture2D::new_empty::<Vec4>(
            &self.context,
            viewport.width,
            viewport.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let mut depth_texture = DepthTexture2D::new::<f32>(
            &self.context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let color = RenderTarget::new(
            texture.as_color_target(None),
            depth_texture.as_depth_target(),
        )
        .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
        .write(|| {
            for (i, geometry) in geometries.iter().enumerate() {
                // TODO:Fix color precision issues which occur because color is normalized
                // when sent to shaders which may not return the original color. This could
                // lead to wrong object being picked.
                let color = i.try_into().expect("Too many objects");
                let color_material = ColorMaterial {
                    color,
                    ..Default::default()
                };
                geometry.render_with_material(&color_material, &camera, &[]);
            }
        })
        .read_color::<Vec4>()[0];
        let picked_color = Color::from_rgba_slice(&[color.x, color.y, color.z, color.w]);
        if picked_color == Color::WHITE {
            return None;
        } else {
            return Some(picked_color.into());
        }
    }
}

impl Pick for ObjectPicker {
    type PickResult = usize;

    fn pick(
        &self,
        camera: &Camera,
        pixel: impl Into<PixelPoint> + Copy,
        geometries: &[&dyn Geometry],
    ) -> Option<Self::PickResult> {
        let pos = camera.position_at_pixel(pixel);
        let dir = camera.view_direction_at_pixel(pixel);
        self.ray_intersect(
            pos + dir * camera.z_near(),
            dir,
            camera.z_far() - camera.z_near(),
            geometries,
        )
    }
}
