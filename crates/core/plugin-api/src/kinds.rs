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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderParticipation {
    None,
    SourceRenderer,
    TargetWriter,
    TargetConsumer,
    RenderBackend,
}
