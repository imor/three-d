use three_d_asset::{Camera, PixelPoint, Vec3};

use crate::{Context, DepthMaterial, Geometry};

///
/// Finds the closest intersection between a ray from the given camera in the given pixel coordinate and the given geometries.
/// The pixel coordinate must be in physical pixels, where (viewport.x, viewport.y) indicate the bottom left corner of the viewport
/// and (viewport.x + viewport.width, viewport.y + viewport.height) indicate the top right corner.
/// Returns ```None``` if no geometry was hit between the near (`z_near`) and far (`z_far`) plane for this camera.
///
pub fn pick(
    context: &Context,
    camera: &Camera,
    pixel: impl Into<PixelPoint> + Copy,
    geometries: impl IntoIterator<Item = impl Geometry>,
) -> Option<Vec3> {
    let pos = camera.position_at_pixel(pixel);
    let dir = camera.view_direction_at_pixel(pixel);
    ray_intersect(
        context,
        pos + dir * camera.z_near(),
        dir,
        camera.z_far() - camera.z_near(),
        geometries,
    )
}

///
/// Finds the closest intersection between a ray starting at the given position in the given direction and the given geometries.
/// Returns ```None``` if no geometry was hit before the given maximum depth.
///
pub fn ray_intersect(
    context: &Context,
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
        context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );
    let mut depth_texture = DepthTexture2D::new::<f32>(
        context,
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
