use crate::{EditorPropertyAccess, EditorPropertyDescriptor};

impl EditorPropertyDescriptor {
    /// Access used by editor/runtime consumers after accounting for metadata that
    /// explicitly marks a field as unavailable for live editing.
    pub fn effective_access(&self) -> EditorPropertyAccess {
        if self.readonly_reason.is_some() || self.tags.iter().any(|tag| *tag == "Unsupported") {
            EditorPropertyAccess::ReadOnly
        } else {
            self.access
        }
    }

    pub fn is_effectively_editable(&self) -> bool {
        self.effective_access() == EditorPropertyAccess::Editable
    }
}
