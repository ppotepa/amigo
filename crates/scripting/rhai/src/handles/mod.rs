//! Script-friendly handle wrappers exposed to Rhai.
//! They provide stable accessors around engine entities and assets without leaking internal service types.

mod asset_ref;
mod entity_ref;

pub use asset_ref::AssetRef;
pub use entity_ref::EntityRef;
