#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PluginKind {
    RenderableSource,
    SemanticSource,
    TargetConsumer,
    SourceAndConsumer,
    Bundle,
    Adapter,
    Tooling,
    Noop,
}

impl PluginKind {
    pub fn as_manifest_str(self) -> &'static str {
        match self {
            PluginKind::RenderableSource => "renderable-source",
            PluginKind::SemanticSource => "semantic-source",
            PluginKind::TargetConsumer => "target-consumer",
            PluginKind::SourceAndConsumer => "source-and-consumer",
            PluginKind::Bundle => "bundle",
            PluginKind::Adapter => "adapter",
            PluginKind::Tooling => "tooling",
            PluginKind::Noop => "noop",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderParticipation {
    None,
    SourceRenderer,
    TargetWriter,
    TargetConsumer,
    RenderBackend,
}

impl RenderParticipation {
    pub fn as_manifest_str(self) -> &'static str {
        match self {
            RenderParticipation::None => "none",
            RenderParticipation::SourceRenderer => "source-renderer",
            RenderParticipation::TargetWriter => "target-writer",
            RenderParticipation::TargetConsumer => "target-consumer",
            RenderParticipation::RenderBackend => "render-backend",
        }
    }
}
