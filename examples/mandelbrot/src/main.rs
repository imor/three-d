use three_d::*;
use three_d_asset::ZoomConfig;

struct MandelbrotMaterial {}

impl Material for MandelbrotMaterial {
    fn fragment_shader(&self, _lights: &[&dyn Light]) -> FragmentShader {
        FragmentShader {
            source: include_str!("mandelbrot.frag").to_string(),
            attributes: FragmentAttributes {
                position: true,
                ..FragmentAttributes::NONE
            },
        }
    }
    fn use_uniforms(&self, _program: &Program, _camera: &Camera, _lights: &[&dyn Light]) {}
    fn render_states(&self) -> RenderStates {
        RenderStates {
            depth_test: DepthTest::Always,
            write_mask: WriteMask::COLOR,
            cull: Cull::Back,
            ..Default::default()
        }
    }
    fn material_type(&self) -> MaterialType {
        MaterialType::Opaque
    }
}

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Mandelbrot!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let height = 2.5;
    // Renderer
    let mut camera = Camera::new_orthographic_with_zoom_config(
        window.viewport(),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        height,
        0.0,
        100.0,
        ZoomConfig {
            max_zoom_ins: 20,
            max_zoom_outs: 10,
            ..ZoomConfig::default()
        },
    );

    let mut control = TwoDControl::new(height);

    let mut mesh = Gm::new(
        Mesh::new(
            &context,
            &CpuMesh {
                positions: Positions::F32(vec![
                    vec3(-2.0, -2.0, 0.0),
                    vec3(2.0, -2.0, 0.0),
                    vec3(2.0, 2.0, 0.0),
                    vec3(2.0, 2.0, 0.0),
                    vec3(-2.0, 2.0, 0.0),
                    vec3(-2.0, -2.0, 0.0),
                ]),
                ..Default::default()
            },
        ),
        MandelbrotMaterial {},
    );
    mesh.set_transformation(Mat4::from_scale(10.0));

    // main loop
    window.render_loop(move |mut frame_input| {
        let mut redraw = frame_input.first_frame;
        redraw |= camera.set_viewport(frame_input.viewport);

        redraw |= control.handle_events(&mut camera, &mut frame_input.events);

        if redraw {
            frame_input
                .screen()
                .clear(ClearState::color(0.0, 1.0, 1.0, 1.0))
                .render(&camera, &mesh, &[]);
        }

        FrameOutput {
            swap_buffers: redraw,
            wait_next_event: true,
            ..Default::default()
        }
    });
}
