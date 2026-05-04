//! Asset catalog and preparation layer for the engine runtime.
//! It tracks manifests, load state, and prepared asset payloads that downstream domains consume.

mod catalog;
mod model;
mod plugin;
mod prepare;

pub use catalog::*;
pub use model::*;
pub use plugin::*;
pub use prepare::*;

#[cfg(test)]
mod tests;
