use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub enum SceneDocumentError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: Option<PathBuf>,
        source: serde_yaml::Error,
    },
    Hydration {
        scene_id: String,
        entity_id: String,
        component_kind: String,
        message: String,
    },
}

impl Display for SceneDocumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to read scene document from `{}`: {source}",
                    path.display()
                )
            }
            Self::Parse {
                path: Some(path),
                source,
            } => write!(
                f,
                "failed to parse scene document from `{}`: {source}",
                path.display()
            ),
            Self::Parse { path: None, source } => {
                write!(f, "failed to parse scene document: {source}")
            }
            Self::Hydration {
                scene_id,
                entity_id,
                component_kind,
                message,
            } => write!(
                f,
                "failed to hydrate scene document `{scene_id}` entity `{entity_id}` component `{component_kind}`: {message}"
            ),
        }
    }
}

impl Error for SceneDocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::Hydration { .. } => None,
        }
    }
}

pub type SceneDocumentResult<T> = Result<T, SceneDocumentError>;
