use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSettingsDto {
    pub active_theme_id: String,
}

impl Default for ThemeSettingsDto {
    fn default() -> Self {
        Self {
            active_theme_id: "mexico-at-night".to_owned(),
        }
    }
}

pub fn validate_theme_id(theme_id: &str) -> Result<(), String> {
    match theme_id {
        "mexico-sand" | "mexico-at-night" | "amigo-light-paper" => Ok(()),
        _ => Err(format!("unknown theme id `{theme_id}`")),
    }
}

pub fn normalize_theme_id(theme_id: &str) -> Option<&'static str> {
    match theme_id {
        "mexico-sand" | "amigo-mexico" => Some("mexico-sand"),
        "mexico-at-night" | "amigo-dark-navy" | "amigo-mexico-dark" => Some("mexico-at-night"),
        "amigo-light-paper" => Some("amigo-light-paper"),
        _ => None,
    }
}
