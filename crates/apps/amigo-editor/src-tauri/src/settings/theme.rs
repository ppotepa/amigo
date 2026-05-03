use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSettingsDto {
    pub active_theme_id: String,
}

impl Default for ThemeSettingsDto {
    fn default() -> Self {
        Self {
            active_theme_id: "amigo-dark-navy".to_owned(),
        }
    }
}

pub fn validate_theme_id(theme_id: &str) -> Result<(), String> {
    match theme_id {
        "amigo-dark-navy" | "amigo-light-paper" | "amigo-mexico" => Ok(()),
        _ => Err(format!("unknown theme id `{theme_id}`")),
    }
}
