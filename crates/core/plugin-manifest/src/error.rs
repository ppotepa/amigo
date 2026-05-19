#[derive(Debug)]
pub enum PluginManifestParseError {
    Toml(toml::de::Error),
    UnknownPluginKind(String),
    UnknownRenderParticipation(String),
}

impl From<toml::de::Error> for PluginManifestParseError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}
