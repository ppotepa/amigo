//! Shared scripting service contracts and value types.
//! It defines runtime, events, commands, and component metadata used by scripting backends.

mod binding_provider;
mod command_handler;
mod dev_console_input;
mod runtime;
mod services;
mod types;

#[cfg(test)]
mod tests;

pub use binding_provider::*;
pub use command_handler::*;
pub use dev_console_input::*;
pub use runtime::*;
pub use services::*;
pub use types::*;
