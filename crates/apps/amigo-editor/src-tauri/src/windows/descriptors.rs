#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum EditorWindowKind {
    Startup,
    Workspace { session_id: String, title: String },
    Theme,
    Settings,
    ModSettings { session_id: String },
}

impl EditorWindowKind {
    pub fn label(&self) -> String {
        match self {
            Self::Startup => "startup".to_owned(),
            Self::Workspace { session_id, .. } => format!("workspace-{session_id}"),
            Self::Theme => "theme".to_owned(),
            Self::Settings => "settings".to_owned(),
            Self::ModSettings { session_id } => format!("mod-settings-{session_id}"),
        }
    }

    pub fn route(&self) -> String {
        match self {
            Self::Startup => "index.html?window=startup".to_owned(),
            Self::Workspace { session_id, .. } => {
                format!("index.html?window=workspace&sessionId={session_id}")
            }
            Self::Theme => "index.html?window=theme".to_owned(),
            Self::Settings => "index.html?window=settings".to_owned(),
            Self::ModSettings { session_id } => {
                format!("index.html?window=mod-settings&sessionId={session_id}")
            }
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::Startup => "Amigo Editor".to_owned(),
            Self::Workspace { title, .. } => format!("Amigo Editor - {title}"),
            Self::Theme => "Theme Controller - Amigo Editor".to_owned(),
            Self::Settings => "Settings - Amigo Editor".to_owned(),
            Self::ModSettings { .. } => "Mod Settings - Amigo Editor".to_owned(),
        }
    }
}
