//!
//! High-level features for easy rendering of different types of objects with different types of shading.
//! Can be combined seamlessly with the mid-level features in the [core](crate::core) module as well as functionality in the [context](crate::context) module.
//!
//! This module contains five main traits
//! - [Geometry] - a geometric representation in 3D space
//! - [Material] - a material that can be applied to a geometry
//! - [PostMaterial] - a material that can be applied to a geometry and rendered after the rest of the scene has been rendered
//! - [Object] - an object in 3D space which has both geometry and material information (use the [Gm] struct to combine any [Material] and [Geometry] into an object)
//! - [Light] - a light that shines onto objects in the scene (some materials are affected by lights, others are not)
//!
//! Common implementations of these traits are found in their respective modules but it is also possible to do a custom implementation by implementing one of the four traits.
//!
//! There are several ways to render something.
//! Objects can be rendered directly using [Object::render] or used in a render call, for example [RenderTarget::render].
//! Geometries can be rendered with a given material using [Geometry::render_with_material] or combined into an object using the [Gm] struct and again used in a render call.
//!

pub use crate::core::*;

use thiserror::Error;
///
/// Error in the [renderer](crate::renderer) module.
///
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum RendererError {
    #[error("{0} buffer length must be {1}, actual length is {2}")]
    InvalidBufferLength(String, usize, usize),
    #[error("the material {0} is required by the geometry {1} but could not be found")]
    MissingMaterial(String, String),
}

pub mod material;
pub use material::*;

pub mod effect;
pub use effect::*;

pub mod light;
pub use light::*;

pub mod geometry;
pub use geometry::*;

pub mod object;
pub use object::*;

pub mod control;
pub use control::*;

macro_rules! impl_render_target_extensions_body {
    () => {
        ///
        /// Render the objects using the given camera and lights into this render target.
        /// Use an empty array for the `lights` argument, if the objects does not require lights to be rendered.
        /// Also, objects outside the camera frustum are not rendered and the objects are rendered in the order given by [cmp_render_order].
        ///
        pub fn render(
            &self,
            camera: &Camera,
            objects: impl IntoIterator<Item = impl Object>,
            lights: &[&dyn Light],
        ) -> &Self {
            self.render_partially(self.scissor_box(), camera, objects, lights)
        }

        ///
        /// Render the objects using the given camera and lights into the part of this render target defined by the scissor box.
        /// Use an empty array for the `lights` argument, if the objects does not require lights to be rendered.
        /// Also, objects outside the camera frustum are not rendered and the objects are rendered in the order given by [cmp_render_order].
        ///
        pub fn render_partially(
            &self,
            scissor_box: ScissorBox,
            camera: &Camera,
            objects: impl IntoIterator<Item = impl Object>,
            lights: &[&dyn Light],
        ) -> &Self {
            let (mut deferred_objects, mut forward_objects): (Vec<_>, Vec<_>) = objects
                .into_iter()
                .filter(|o| camera.in_frustum(&o.aabb()))
                .partition(|o| o.material_type() == MaterialType::Deferred);

            // Deferred
            if deferred_objects.len() > 0 {
                // Geometry pass
                let mut geometry_pass_camera = camera.clone();
                let viewport =
                    Viewport::new_at_origin(camera.viewport().width, camera.viewport().height);
                geometry_pass_camera.set_viewport(viewport);
                deferred_objects.sort_by(|a, b| cmp_render_order(&geometry_pass_camera, a, b));
                let mut geometry_pass_texture = Texture2DArray::new_empty::<[u8; 4]>(
                    &self.context,
                    viewport.width,
                    viewport.height,
                    3,
                    Interpolation::Nearest,
                    Interpolation::Nearest,
                    None,
                    Wrapping::ClampToEdge,
                    Wrapping::ClampToEdge,
                );
                let mut geometry_pass_depth_texture = DepthTexture2D::new::<f32>(
                    &self.context,
                    viewport.width,
                    viewport.height,
                    Wrapping::ClampToEdge,
                    Wrapping::ClampToEdge,
                );
                let gbuffer_layers = [0, 1, 2];
                RenderTarget::new(
                    geometry_pass_texture.as_color_target(&gbuffer_layers, None),
                    geometry_pass_depth_texture.as_depth_target(),
                )
                .clear(ClearState::default())
                .write(|| {
                    for object in deferred_objects {
                        object.render(&geometry_pass_camera, lights);
                    }
                });

                // Lighting pass
                self.write_partially(scissor_box, || {
                    DeferredPhysicalMaterial::lighting_pass(
                        &self.context,
                        camera,
                        ColorTexture::Array {
                            texture: &geometry_pass_texture,
                            layers: &gbuffer_layers,
                        },
                        DepthTexture::Single(&geometry_pass_depth_texture),
                        lights,
                    )
                });
            }

            // Forward
            forward_objects.sort_by(|a, b| cmp_render_order(camera, a, b));
            self.write_partially(scissor_box, || {
                for object in forward_objects {
                    object.render(camera, lights);
                }
            });
            self
        }

        ///
        /// Render the geometries with the given [Material] using the given camera and lights into this render target.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_with_material(
            &self,
            material: &dyn Material,
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
        ) -> &Self {
            self.render_partially_with_material(
                self.scissor_box(),
                material,
                camera,
                geometries,
                lights,
            )
        }

        ///
        /// Render the geometries with the given [Material] using the given camera and lights into the part of this render target defined by the scissor box.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_partially_with_material(
            &self,
            scissor_box: ScissorBox,
            material: &dyn Material,
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
        ) -> &Self {
            self.write_partially(scissor_box, || {
                for object in geometries
                    .into_iter()
                    .filter(|o| camera.in_frustum(&o.aabb()))
                {
                    object.render_with_material(material, camera, lights);
                }
            });
            self
        }

        ///
        /// Render the geometries with the given [PostMaterial] using the given camera and lights into this render target.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_with_post_material(
            &self,
            material: &dyn PostMaterial,
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
            color_texture: Option<ColorTexture>,
            depth_texture: Option<DepthTexture>,
        ) -> &Self {
            self.render_partially_with_post_material(
                self.scissor_box(),
                material,
                camera,
                geometries,
                lights,
                color_texture,
                depth_texture,
            )
        }

        ///
        /// Render the geometries with the given [PostMaterial] using the given camera and lights into the part of this render target defined by the scissor box.
        /// Use an empty array for the `lights` argument, if the material does not require lights to be rendered.
        ///
        pub fn render_partially_with_post_material(
            &self,
            scissor_box: ScissorBox,
            material: &dyn PostMaterial,
            camera: &Camera,
            geometries: impl IntoIterator<Item = impl Geometry>,
            lights: &[&dyn Light],
            color_texture: Option<ColorTexture>,
            depth_texture: Option<DepthTexture>,
        ) -> &Self {
            self.write_partially(scissor_box, || {
                for object in geometries
                    .into_iter()
                    .filter(|o| camera.in_frustum(&o.aabb()))
                {
                    object.render_with_post_material(
                        material,
                        camera,
                        lights,
                        color_texture,
                        depth_texture,
                    );
                }
            });
            self
        }
    };
}

macro_rules! impl_render_target_extensions {
    // 2 generic arguments with bounds
    ($name:ident < $a:ident : $ta:tt , $b:ident : $tb:tt >) => {
        impl<$a: $ta, $b: $tb> $name<$a, $b> {
            impl_render_target_extensions_body!();
        }
    };
    // 1 generic argument with bound
    ($name:ident < $a:ident : $ta:tt >) => {
        impl<$a: $ta> $name<$a> {
            impl_render_target_extensions_body!();
        }
    };
    // 1 liftetime argument
    ($name:ident < $lt:lifetime >) => {
        impl<$lt> $name<$lt> {
            impl_render_target_extensions_body!();
        }
    };
    // without any arguments
    ($name:ty) => {
        impl $name {
            impl_render_target_extensions_body!();
        }
    };
}

impl_render_target_extensions!(RenderTarget<'a>);
impl_render_target_extensions!(ColorTarget<'a>);
impl_render_target_extensions!(DepthTarget<'a>);
impl_render_target_extensions!(RenderTargetMultisample<C: TextureDataType, D: DepthTextureDataType>);
impl_render_target_extensions!(ColorTargetMultisample<C: TextureDataType>);
impl_render_target_extensions!(DepthTargetMultisample<D: DepthTextureDataType>);

///
/// Returns an orthographic camera for viewing 2D content.
/// The camera is placed at the center of the given viewport.
/// The (0, 0) position is at the bottom left corner and the
/// (`viewport.width`, `viewport.height`) position is at the top right corner.
///
pub fn camera2d(viewport: Viewport) -> Camera {
    Camera::new_orthographic(
        viewport,
        vec3(
            viewport.width as f32 * 0.5,
            viewport.height as f32 * 0.5,
            1.0,
        ),
        vec3(
            viewport.width as f32 * 0.5,
            viewport.height as f32 * 0.5,
            0.0,
        ),
        vec3(0.0, 1.0, 0.0),
        viewport.height as f32,
        0.0,
        10.0,
    )
}

///
/// Compare function for sorting objects based on distance from the camera.
/// The order is opaque objects from nearest to farthest away from the camera,
/// then transparent objects from farthest away to closest to the camera.
///
pub fn cmp_render_order(
    camera: &Camera,
    obj0: impl Object,
    obj1: impl Object,
) -> std::cmp::Ordering {
    if obj0.material_type() == MaterialType::Transparent
        && obj1.material_type() != MaterialType::Transparent
    {
        std::cmp::Ordering::Greater
    } else if obj0.material_type() != MaterialType::Transparent
        && obj1.material_type() == MaterialType::Transparent
    {
        std::cmp::Ordering::Less
    } else {
        let distance_a = camera.position().distance2(obj0.aabb().center());
        let distance_b = camera.position().distance2(obj1.aabb().center());
        if distance_a.is_nan() || distance_b.is_nan() {
            distance_a.is_nan().cmp(&distance_b.is_nan()) // whatever - just save us from panicing on unwrap below
        } else if obj0.material_type() == MaterialType::Transparent {
            distance_b.partial_cmp(&distance_a).unwrap()
        } else {
            distance_a.partial_cmp(&distance_b).unwrap()
        }
    }
}
