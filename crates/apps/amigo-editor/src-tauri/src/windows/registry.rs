use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::dto::{EditorWindowInfoDto, EditorWindowRegistryDto};

#[derive(Debug, Default)]
pub struct EditorWindowRegistry {
    windows: Mutex<BTreeMap<String, EditorWindowInfoDto>>,
}

impl EditorWindowRegistry {
    pub fn register_window(
        &self,
        label: String,
        kind: String,
        session_id: Option<String>,
    ) -> Result<(), String> {
        let focused = false;
        let info = EditorWindowInfoDto {
            label: label.clone(),
            kind,
            session_id,
            focused,
            last_seen_at: unix_seconds(),
        };
        self.windows
            .lock()
            .map_err(|_| "editor window registry lock was poisoned".to_owned())?
            .insert(label, info);
        Ok(())
    }

    pub fn mark_focused(&self, label: &str) -> Result<(), String> {
        let mut windows = self
            .windows
            .lock()
            .map_err(|_| "editor window registry lock was poisoned".to_owned())?;
        for window in windows.values_mut() {
            window.focused = window.label == label;
            if window.focused {
                window.last_seen_at = unix_seconds();
            }
        }
        Ok(())
    }

    pub fn remove_window(&self, label: &str) -> Result<(), String> {
        self.windows
            .lock()
            .map_err(|_| "editor window registry lock was poisoned".to_owned())?
            .remove(label);
        Ok(())
    }

    pub fn snapshot(&self) -> Result<EditorWindowRegistryDto, String> {
        let windows = self
            .windows
            .lock()
            .map_err(|_| "editor window registry lock was poisoned".to_owned())?
            .values()
            .cloned()
            .collect::<Vec<_>>();
        Ok(EditorWindowRegistryDto { windows })
    }
}

fn unix_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}
