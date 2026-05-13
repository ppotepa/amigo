use amigo_core::AmigoResult;

pub trait GizmoProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn build_gizmos(&self, ctx: &GizmoContext, out: &mut GizmoOutput) -> AmigoResult<()>;
}

#[derive(Debug, Default)]
pub struct GizmoContext;

#[derive(Debug, Default)]
pub struct GizmoOutput {
    pub gizmos: Vec<GizmoDescriptor>,
}

#[derive(Debug, Clone)]
pub struct GizmoDescriptor {
    pub id: String,
    pub label: String,
    pub kind: GizmoKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoKind {
    Translate2D,
    Translate3D,
    Rotate2D,
    Rotate3D,
    Scale2D,
    Scale3D,
}

