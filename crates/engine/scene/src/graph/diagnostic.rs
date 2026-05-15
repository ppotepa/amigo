use super::SceneGraphNodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneGraphDiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneGraphDiagnostic {
    pub severity: SceneGraphDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub node: Option<SceneGraphNodeId>,
}

impl SceneGraphDiagnostic {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        node: Option<SceneGraphNodeId>,
    ) -> Self {
        Self {
            severity: SceneGraphDiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            node,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        node: Option<SceneGraphNodeId>,
    ) -> Self {
        Self {
            severity: SceneGraphDiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            node,
        }
    }

    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        node: Option<SceneGraphNodeId>,
    ) -> Self {
        Self {
            severity: SceneGraphDiagnosticSeverity::Info,
            code: code.into(),
            message: message.into(),
            node,
        }
    }
}
