use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::dto::EditorSessionDto;

#[derive(Debug, Default)]
pub struct EditorSessionRegistry {
    sessions: Mutex<BTreeMap<String, EditorSessionDto>>,
}

impl EditorSessionRegistry {
    pub fn create_session(
        &self,
        mod_id: String,
        root_path: String,
        selected_scene_id: Option<String>,
    ) -> Result<EditorSessionDto, String> {
        let created_at = unix_seconds();
        let session = EditorSessionDto {
            session_id: format!("editor-session-{created_at}"),
            mod_id,
            root_path,
            created_at,
            selected_scene_id,
        };

        self.sessions
            .lock()
            .map_err(|_| "editor session registry lock was poisoned".to_owned())?
            .insert(session.session_id.clone(), session.clone());

        Ok(session)
    }

    pub fn get_session(&self, session_id: &str) -> Result<EditorSessionDto, String> {
        self.sessions
            .lock()
            .map_err(|_| "editor session registry lock was poisoned".to_owned())?
            .get(session_id)
            .cloned()
            .ok_or_else(|| format!("editor session `{session_id}` was not found"))
    }

    pub fn close_session(&self, session_id: &str) -> Result<(), String> {
        self.sessions
            .lock()
            .map_err(|_| "editor session registry lock was poisoned".to_owned())?
            .remove(session_id);
        Ok(())
    }
}

fn unix_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}
