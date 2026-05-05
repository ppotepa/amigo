use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSettingsDto {
    pub active_theme_id: String,
    pub active_font_id: String,
}

impl Default for ThemeSettingsDto {
    fn default() -> Self {
        Self {
            active_theme_id: "night-in-mexico".to_owned(),
            active_font_id: "source-sans-3".to_owned(),
        }
    }
}

pub fn validate_theme_id(theme_id: &str) -> Result<(), String> {
    match theme_id {
        "night-in-mexico" | "siesta-in-mexico" | "amigo-light-paper" | "krakowskie-przedmiescie" => Ok(()),
        _ => Err(format!("unknown theme id `{theme_id}`")),
    }
}

pub fn validate_font_id(font_id: &str) -> Result<(), String> {
    match font_id {
        "source-sans-3" | "geist-sans" | "segoe-ui" | "system-ui" => Ok(()),
        _ => Err(format!("unknown font id `{font_id}`")),
    }
}

pub fn normalize_font_id(font_id: &str) -> Option<&'static str> {
    match font_id {
        "source-sans-3" => Some("source-sans-3"),
        "geist-sans" => Some("geist-sans"),
        "segoe-ui" => Some("segoe-ui"),
        "system-ui" => Some("system-ui"),
        _ => None,
    }
}

pub fn normalize_theme_id(theme_id: &str) -> Option<&'static str> {
    match theme_id {
        "night-in-mexico" => Some("night-in-mexico"),
        "siesta-in-mexico" => Some("siesta-in-mexico"),
        "amigo-light-paper" => Some("amigo-light-paper"),
        "krakowskie-przedmiescie" => Some("krakowskie-przedmiescie"),
        _ => None,
    }
}
