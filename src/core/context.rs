use super::*;
use crate::context::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

///
/// Contains information about the graphics context to use for rendering and other "global" variables.
///
#[derive(Clone)]
pub struct Context {
    context: Rc<GlContext>,
    programs: Rc<RefCell<HashMap<String, Program>>>,
    effects: Rc<RefCell<HashMap<String, ImageEffect>>>,
    camera2d: Rc<RefCell<Option<Camera>>>,
}

impl Context {
    pub fn from_gl_context(context: Rc<GlContext>) -> ThreeDResult<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            // Create one Vertex Array Object which is then reused all the time.
            let vao = context
                .create_vertex_array()
                .map_err(|e| CoreError::ContextCreation(e))?;
            context.bind_vertex_array(Some(vao));
            // Enable seamless cube map textures
            context.enable(glow::TEXTURE_CUBE_MAP_SEAMLESS);
        }
        let c = Self {
            context,
            programs: Rc::new(RefCell::new(HashMap::new())),
            effects: Rc::new(RefCell::new(HashMap::new())),
            camera2d: Rc::new(RefCell::new(None)),
        };
        c.error_check()?;
        Ok(c)
    }

    ///
    /// Compiles a [Program] with the given vertex and fragment shader source and stores it for later use.
    /// If it has already been created, then it is just returned.
    ///
    pub fn program(
        &self,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
        callback: impl FnOnce(&Program) -> ThreeDResult<()>,
    ) -> ThreeDResult<()> {
        let key = format!("{}{}", vertex_shader_source, fragment_shader_source);
        if !self.programs.borrow().contains_key(&key) {
            self.programs.borrow_mut().insert(
                key.clone(),
                Program::from_source(self, vertex_shader_source, fragment_shader_source)?,
            );
        };
        callback(self.programs.borrow().get(&key).unwrap())
    }

    ///
    /// Compiles an [ImageEffect] with the given fragment shader source and stores it for later use.
    /// If it has already been created, then it is just returned.
    ///
    pub fn effect(
        &self,
        fragment_shader_source: &str,
        callback: impl FnOnce(&ImageEffect) -> ThreeDResult<()>,
    ) -> ThreeDResult<()> {
        if !self.effects.borrow().contains_key(fragment_shader_source) {
            self.effects.borrow_mut().insert(
                fragment_shader_source.to_string(),
                ImageEffect::new(self, fragment_shader_source)?,
            );
        };
        callback(self.effects.borrow().get(fragment_shader_source).unwrap())
    }

    ///
    /// Returns a camera for viewing 2D content.
    ///
    pub fn camera2d(
        &self,
        viewport: Viewport,
        callback: impl FnOnce(&Camera) -> ThreeDResult<()>,
    ) -> ThreeDResult<()> {
        if self.camera2d.borrow().is_none() {
            *self.camera2d.borrow_mut() = Some(Camera::new_orthographic(
                self,
                viewport,
                vec3(0.0, 0.0, -1.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, -1.0, 0.0),
                1.0,
                0.0,
                10.0,
            )?)
        }
        let mut camera2d = self.camera2d.borrow_mut();
        camera2d.as_mut().unwrap().set_viewport(viewport)?;
        camera2d.as_mut().unwrap().set_orthographic_projection(
            viewport.height as f32,
            0.0,
            10.0,
        )?;
        camera2d.as_mut().unwrap().set_view(
            vec3(
                viewport.width as f32 * 0.5,
                viewport.height as f32 * 0.5,
                -1.0,
            ),
            vec3(
                viewport.width as f32 * 0.5,
                viewport.height as f32 * 0.5,
                0.0,
            ),
            vec3(0.0, -1.0, 0.0),
        )?;
        callback(camera2d.as_ref().unwrap())
    }

    pub(super) fn error_check(&self) -> ThreeDResult<()> {
        #[cfg(debug_assertions)]
        unsafe {
            let e = self.get_error();
            if e != glow::NO_ERROR {
                Err(CoreError::ContextError(
                    match e {
                        glow::INVALID_ENUM => "Invalid enum",
                        glow::INVALID_VALUE => "Invalid value",
                        glow::INVALID_OPERATION => "Invalid operation",
                        glow::INVALID_FRAMEBUFFER_OPERATION => "Invalid framebuffer operation",
                        glow::OUT_OF_MEMORY => "Out of memory",
                        glow::STACK_OVERFLOW => "Stack overflow",
                        glow::STACK_UNDERFLOW => "Stack underflow",
                        _ => "Unknown",
                    }
                    .to_string(),
                ))?;
            }
        }
        Ok(())
    }

    pub(super) fn framebuffer_check(&self) -> ThreeDResult<()> {
        #[cfg(debug_assertions)]
        unsafe {
            match self.check_framebuffer_status(glow::FRAMEBUFFER) {
                glow::FRAMEBUFFER_COMPLETE => Ok(()),
                glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => Err(CoreError::RenderTargetCreation(
                    "FRAMEBUFFER_INCOMPLETE_ATTACHMENT".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => Err(CoreError::RenderTargetCreation(
                    "FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                    Err(CoreError::RenderTargetCreation(
                        "FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT".to_string(),
                    ))
                }
                glow::FRAMEBUFFER_UNSUPPORTED => Err(CoreError::RenderTargetCreation(
                    "FRAMEBUFFER_UNSUPPORTED".to_string(),
                )),
                glow::FRAMEBUFFER_UNDEFINED => Err(CoreError::RenderTargetCreation(
                    "FRAMEBUFFER_UNDEFINED".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => Err(CoreError::RenderTargetCreation(
                    "FRAMEBUFFER_INCOMPLETE_READ_BUFFER".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => Err(CoreError::RenderTargetCreation(
                    "FRAMEBUFFER_INCOMPLETE_MULTISAMPLE".to_string(),
                )),
                glow::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => Err(CoreError::RenderTargetCreation(
                    "FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS".to_string(),
                )),
                _ => Err(CoreError::RenderTargetCreation(
                    "Unknown framebuffer error".to_string(),
                )),
            }?;
        }
        Ok(())
    }
}

impl std::ops::Deref for Context {
    type Target = glow::Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
