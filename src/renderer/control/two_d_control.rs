use super::*;
use crate::core::*;

///
/// A control that makes the camera move perpendicular to the xy plane. Useful for 2D editors.
///
pub struct TwoDControl {
    frustum_height: f32,
}

impl TwoDControl {
    /// Creates a new 2d control with the given frustum height.
    pub fn new(frustum_height: f32) -> Self {
        Self { frustum_height }
    }

    /// Handles the events. Must be called each frame.
    pub fn handle_events(&mut self, camera: &mut Camera, events: &mut [Event]) -> bool {
        let mut handled = false;
        for event in events.iter() {
            match event {
                Event::MouseMotion { delta, button, .. } => {
                    if *button == Some(MouseButton::Left) {
                        let pan_factor = self.frustum_height / camera.viewport().height as f32;
                        let speed = pan_factor * camera.position().z.abs();
                        let right = camera.right_direction();
                        let up = right.cross(camera.view_direction());
                        let delta = -right * speed * delta.0 + up * speed * delta.1;
                        camera.translate(&delta);
                        handled = true;
                    }
                }
                Event::MouseWheel {
                    delta, position, ..
                } => {
                    let mut target = camera.position_at_pixel(position);
                    target.z = 0.0;
                    camera.zoom_towards_2d(&target, delta.1.into());
                    handled = true;
                }
                _ => {}
            }
        }
        handled
    }
}
