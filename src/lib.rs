#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]
#![warn(missing_docs)]
#![warn(unsafe_code)]
//!
//! A 3D renderer which enables out-of-the-box build to both desktop and web with the same code.
//! See the [README](https://crates.io/crates/three-d) for more information and
//! the [examples](https://github.com/asny/three-d/tree/master/examples) for how to use it.
//!

pub mod context;

pub mod core;

pub mod picker;
pub use picker::*;

pub mod obb2d;
pub use obb2d::*;

pub mod renderer;
pub use renderer::*;

pub mod window;
pub use window::*;

mod gui;
pub use gui::*;
