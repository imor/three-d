use crate::renderer::*;

/// A line segment whose line thickness remains the same even at different zoom levels.
/// This is only useful for 2D applications because it is drawn in the xy plane.
pub struct Line2D {
    context: Context,
    start: PhysicalPoint,
    end: PhysicalPoint,
    thickness: u32,
    positions: VertexBuffer,
    prev_positions: VertexBuffer,
}

// We use a z value of something greater than zero for Line2D
// because it is usually drawn over other shapes which have a z value of zero
const Z: f32 = 0.001;

impl Line2D {
    /// Construct a new line segment
    pub fn new(
        context: &Context,
        start: impl Into<PhysicalPoint>,
        end: impl Into<PhysicalPoint>,
        thickness: u32,
    ) -> Self {
        assert_ne!(
            thickness, 0,
            "Line segment thickness should be greater than zero"
        );

        let start = start.into();
        let end = end.into();

        Self {
            context: context.clone(),
            start,
            end,
            thickness,
            positions: Self::positions(context, &start, &end),
            prev_positions: Self::prev_positions(context, &start, &end),
        }
    }

    /// Get the start point of the line.
    pub fn start(&self) -> PhysicalPoint {
        self.start
    }

    /// Get the end point of the line.
    pub fn end(&self) -> PhysicalPoint {
        self.end
    }

    fn draw(&self, program: &Program, render_states: RenderStates, camera: &Camera) {
        let viewport = camera.viewport();
        program.use_uniform("model", Mat4::identity());
        program.use_uniform("viewProjection", camera.projection() * camera.view());
        program.use_uniform(
            "resolution",
            vec2(viewport.width as f32, viewport.height as f32),
        );
        program.use_uniform("thickness", self.thickness as f32);
        program.use_vertex_attribute("position", &self.positions);
        program.use_vertex_attribute("prev", &self.prev_positions);
        program.draw_arrays(render_states, viewport, self.positions.vertex_count());
    }

    /// Returns the vertex positions of the two triangles making a rectangular line
    fn positions(context: &Context, start: &PhysicalPoint, end: &PhysicalPoint) -> VertexBuffer {
        VertexBuffer::new_with_data(
            context,
            &[
                vec2(end.x, end.y),     // bottom right
                vec2(start.x, start.y), // bottom left
                vec2(start.x, start.y), // top left
                vec2(start.x, start.y), // top left
                vec2(end.x, end.y),     // top right
                vec2(end.x, end.y),     // bottom right
            ]
            .map(|v| v.extend(Z)),
        )
    }

    /// Returns the previous vertex positions of the two triangles making a rectangular line
    fn prev_positions(
        context: &Context,
        start: &PhysicalPoint,
        end: &PhysicalPoint,
    ) -> VertexBuffer {
        let start_vec: Vec2 = (*start).into();
        let end_vec: Vec2 = (*end).into();
        let line_seg_vec = end_vec - start_vec;
        let line_seg_vec = line_seg_vec.normalize();
        VertexBuffer::new_with_data(
            context,
            &[
                end_vec + line_seg_vec,
                start_vec + line_seg_vec,
                start_vec - line_seg_vec,
                start_vec - line_seg_vec,
                end_vec - line_seg_vec,
                end_vec + line_seg_vec,
            ]
            .map(|i| i.extend(Z)),
        )
    }
}

impl Geometry for Line2D {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        let fragment_shader = material.fragment_shader(lights);
        self.context
            .program(
                include_str!("shaders/line2d.vert").to_owned(),
                fragment_shader.source,
                |program| {
                    material.use_uniforms(program, camera, lights);
                    self.draw(program, material.render_states(), camera);
                },
            )
            .expect("Failed to compile line segment program");
    }

    fn render_with_post_material(
        &self,
        material: &dyn PostMaterial,
        camera: &Camera,
        lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        let fragment_shader = material.fragment_shader(lights, color_texture, depth_texture);
        self.context
            .program(
                include_str!("shaders/line2d.vert").to_owned(),
                fragment_shader.source,
                |program| {
                    material.use_uniforms(program, camera, lights, color_texture, depth_texture);
                    self.draw(program, material.render_states(), camera);
                },
            )
            .expect("Failed to compile line segment program");
    }

    ///
    /// Returns the [AxisAlignedBoundingBox] for this geometry in the global coordinate system.
    ///
    fn aabb(&self) -> AxisAlignedBoundingBox {
        let start: Vec2 = self.start.into();
        let end: Vec2 = self.end.into();
        AxisAlignedBoundingBox::new_with_positions(&[start.extend(0.0), end.extend(0.0)])
    }
}

impl<'a> IntoIterator for &'a Line2D {
    type Item = &'a dyn Geometry;
    type IntoIter = std::iter::Once<&'a dyn Geometry>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}
