//! Shared scripting service contracts and value types.
//! It defines runtime, events, commands, and component metadata used by scripting backends.

mod runtime;
mod services;
mod types;

#[cfg(test)]
mod tests;

pub use runtime::*;
pub use services::*;
pub use types::*;
