#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprDebugView {
    #[default]
    Final,
    FeatureClasses,
    StrokeIds,
}
