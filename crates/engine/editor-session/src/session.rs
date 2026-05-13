use amigo_core::AmigoResult;
use amigo_editor_api::{
    EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry, GizmoProvider,
    ValidationProvider,
};
use amigo_session::RuntimeSession;

use crate::{EditorPreviewRuntime, SceneDocumentState, SelectionState, UndoRedoStack};

pub struct EditorSession {
    pub document: SceneDocumentState,
    pub selection: SelectionState,
    pub undo_redo: UndoRedoStack,
    pub capabilities: EditorCapabilityRegistry,
    pub preview_runtime: Option<EditorPreviewRuntime>,
}

impl EditorSession {
    pub fn new(document: SceneDocumentState) -> Self {
        Self {
            document,
            selection: SelectionState::default(),
            undo_redo: UndoRedoStack::default(),
            capabilities: EditorCapabilityRegistry::default(),
            preview_runtime: None,
        }
    }

    pub fn set_preview_runtime(&mut self, runtime: RuntimeSession) {
        self.preview_runtime = Some(EditorPreviewRuntime { runtime });
    }

    pub fn register_capability<C>(&self, capability: C)
    where
        C: EditorCapability + 'static,
    {
        self.capabilities.register_capability(capability);
    }

    pub fn register_validation_provider<P>(&self, provider: P)
    where
        P: ValidationProvider + 'static,
    {
        self.capabilities.register_validation_provider(provider);
    }

    pub fn register_gizmo_provider<P>(&self, provider: P)
    where
        P: GizmoProvider + 'static,
    {
        self.capabilities.register_gizmo_provider(provider);
    }

    pub fn register_capability_provider<P>(&self, provider: P) -> AmigoResult<()>
    where
        P: EditorCapabilityProvider + 'static,
    {
        provider.register(&self.capabilities)
    }
}

