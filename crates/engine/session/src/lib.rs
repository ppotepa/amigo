//! High-level runtime session API for Amigo.
//!
//! This crate is the reusable session boundary between low-level runtime
//! services and concrete hosts such as `amigo-app`, the future editor,
//! headless validators and scene preview tools.

pub mod bootstrap;
pub mod frame;
pub mod options;
pub mod scheduler_session;
pub mod render_session;
pub mod script_session;
pub mod script_command_registry;
pub mod scene_command_registry;
pub mod scene_session;
pub mod runtime_session;
pub mod runtime_capabilities;
pub mod scheduling;
mod session_runtime_capabilities;

pub use bootstrap::*;
pub use runtime_capabilities::*;
pub use scheduling::*;
pub use frame::*;
pub use render_session::*;
pub use session_runtime_capabilities::*;
pub use script_session::*;
pub use script_command_registry::*;
pub use options::*;
pub use scheduler_session::*;
pub use scene_command_registry::*;
pub use scene_session::*;
pub use runtime_session::*;
