#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderSpace {
    World3D,
    World2D,
    Screen2D,
    Ui,
    Gizmos,
    DebugOverlay,
}
