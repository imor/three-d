use crate::renderer::*;

///
/// A 2D rectangular outline for the xy plane.
///
pub struct Outline {
    width: f32,
    height: f32,
    center: PhysicalPoint,
    rotation: Radians,

    top_left: Vec2,
    bottom_right: Vec2,
    bottom_left: Vec2,

    top: Line2D,
    right: Line2D,
    bottom: Line2D,
    left: Line2D,
}

impl Outline {
    ///
    /// Constructs a new outline.
    ///
    pub fn new(
        context: &Context,
        center: impl Into<PhysicalPoint>,
        rotation: impl Into<Radians>,
        width: f32,
        height: f32,
        thickness: u32,
    ) -> Self {
        let center = center.into();
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let top_left = vec2(-half_width, half_height);
        let bottom_right = vec2(half_width, -half_height);
        let bottom_left = vec2(-half_width, -half_height);
        let zero = Vec2::zero();
        let one_x = vec2(1.0, 0.0);
        let mut outline = Self {
            width,
            height,
            center,
            rotation: rotation.into(),
            top_left,
            bottom_right,
            bottom_left,
            top: Line2D::new(context, zero, one_x, thickness),
            right: Line2D::new(context, zero, one_x, thickness),
            bottom: Line2D::new(context, zero, one_x, thickness),
            left: Line2D::new(context, zero, one_x, thickness),
        };
        outline
    }

    /// Set the rotation of the outline.
    pub fn set_rotation(&mut self, rotation: impl Into<Radians>) {
        self.rotation = rotation.into();
        self.update();
    }

    fn update(&mut self) {
        let scale_by_width = Mat3::from_nonuniform_scale(self.width, 1.0);
        let scale_by_height = Mat3::from_nonuniform_scale(self.height, 1.0);
        let translation_to_center = Mat3::from_translation(self.center.into());
        let rotation = Mat3::from_angle_z(self.rotation);
        let rotation_90 = Mat3::from_angle_z(Rad(std::f32::consts::PI / 2.0));

        // Update top line
        let translation_to_corner = Mat3::from_translation(self.top_left);
        let transformation = to_3d_transformation(
            translation_to_center * rotation * translation_to_corner * scale_by_width,
        );
        self.top.set_transformation(transformation);

        // Update right line
        let translation_to_corner = Mat3::from_translation(self.bottom_right);
        let transformation = to_3d_transformation(
            translation_to_center
                * rotation
                * translation_to_corner
                * rotation_90
                * scale_by_height,
        );
        self.right.set_transformation(transformation);

        // Update bottom line
        let translation_to_corner = Mat3::from_translation(self.bottom_left);
        let transformation = to_3d_transformation(
            translation_to_center * rotation * translation_to_corner * scale_by_width,
        );
        self.bottom.set_transformation(transformation);

        // Update left line
        let translation_to_corner = Mat3::from_translation(self.bottom_left);
        let transformation = to_3d_transformation(
            translation_to_center
                * rotation
                * translation_to_corner
                * rotation_90
                * scale_by_height,
        );
        self.left.set_transformation(transformation);
    }
}

impl Geometry for Outline {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        self.top.render_with_material(material, camera, lights);
        self.right.render_with_material(material, camera, lights);
        self.bottom.render_with_material(material, camera, lights);
        self.left.render_with_material(material, camera, lights);
    }

    fn render_with_post_material(
        &self,
        material: &dyn PostMaterial,
        camera: &Camera,
        lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        self.top
            .render_with_post_material(material, camera, lights, color_texture, depth_texture);
        self.right.render_with_post_material(
            material,
            camera,
            lights,
            color_texture,
            depth_texture,
        );
        self.bottom.render_with_post_material(
            material,
            camera,
            lights,
            color_texture,
            depth_texture,
        );
        self.left
            .render_with_post_material(material, camera, lights, color_texture, depth_texture);
    }

    ///
    /// Returns the [AxisAlignedBoundingBox] for this geometry in the global coordinate system.
    ///
    fn aabb(&self) -> AxisAlignedBoundingBox {
        let center: Vec2 = self.center.into();
        AxisAlignedBoundingBox::new_with_positions(&[
            (center - 0.5 * vec2(self.width, self.height)).extend(0.0),
            (center + 0.5 * vec2(self.width, self.height)).extend(0.0),
        ])
    }
}

impl<'a> IntoIterator for &'a Outline {
    type Item = &'a dyn Geometry;
    type IntoIter = std::iter::Once<&'a dyn Geometry>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}
