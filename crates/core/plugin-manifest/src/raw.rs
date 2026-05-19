use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawPluginManifest {
    pub id: String,
    pub family: String,
    pub kind: String,
    #[serde(default)]
    pub renderable: bool,
    #[serde(default)]
    pub render_participation: Option<String>,
    #[serde(default)]
    pub capabilities: RawCapabilities,
    #[serde(default)]
    pub slots: RawSlots,
    #[serde(default)]
    pub targets: RawTargets,
    #[serde(default)]
    pub contributions: RawContributions,
    #[serde(default)]
    pub diagnostics: RawDiagnostics,
    #[serde(default)]
    pub docs: RawDocs,
    #[serde(default)]
    pub tests: RawTests,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawCapabilities {
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawSlots {
    #[serde(default)]
    pub implements: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub replaces: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawTargets {
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
    #[serde(default)]
    pub contributes: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawContributions {
    #[serde(default)]
    pub emits: Vec<RawContribution>,
    #[serde(default)]
    pub consumes: Vec<RawContribution>,
}

#[derive(Debug, Deserialize)]
pub struct RawContribution {
    pub domain: String,
    #[serde(rename = "type")]
    pub contribution_type: String,
    #[serde(default = "default_contribution_policy")]
    pub policy: String,
}

fn default_contribution_policy() -> String {
    "ExplicitOnly".to_string()
}

#[derive(Debug, Default, Deserialize)]
pub struct RawDiagnostics {
    #[serde(default)]
    pub channels: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawDocs {
    pub pipeline: Option<String>,
    pub contributions: Option<String>,
    pub diagnostics: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawTests {
    pub hydration: Option<String>,
    pub participation: Option<String>,
    pub candidate: Option<String>,
    pub waterfall: Option<String>,
    pub diagnostics: Option<String>,
}
