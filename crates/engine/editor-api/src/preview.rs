#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    Edit,
    Play,
}

#[derive(Debug, Clone)]
pub struct PreviewRequest {
    pub mode: PreviewMode,
}
