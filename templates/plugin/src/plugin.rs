/// Stable semantic identity for the generated plugin.
pub const PLUGIN_ID: &str = "amigo.family.plugin-name";

/// Keeps ownership checks explicit while the generated plugin is filled in.
pub fn owns_semantic_id(id: &str) -> bool {
    id == PLUGIN_ID || id.starts_with(&format!("{PLUGIN_ID}."))
}
