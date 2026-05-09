//! Shared 2D post-processing effect models.
//! Renderer integrations consume these data types; authored domains decide where to attach them.

mod model;
mod plugin;
mod service;

pub use model::*;
pub use plugin::*;
pub use service::*;
