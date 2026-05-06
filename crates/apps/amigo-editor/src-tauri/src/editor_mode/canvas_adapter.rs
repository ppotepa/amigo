use super::dto::{EditorHitTestResultDto, EditorPointerEventDto, EditorSceneCanvasKindDto};
use super::session::EditorModeSession;

pub trait EditorCanvasAdapter {
    fn kind(&self) -> EditorSceneCanvasKindDto;

    fn handle_pointer_event(
        &self,
        session: &mut EditorModeSession,
        event: &EditorPointerEventDto,
    ) -> Result<(), String>;

    fn hit_test(
        &self,
        session: &EditorModeSession,
        event: &EditorPointerEventDto,
    ) -> Result<EditorHitTestResultDto, String>;
}

pub struct EditorCanvas2DAdapter;
pub struct EditorCanvas25DAdapter;
pub struct EditorCanvas3DAdapter;

impl EditorCanvasAdapter for EditorCanvas2DAdapter {
    fn kind(&self) -> EditorSceneCanvasKindDto {
        EditorSceneCanvasKindDto::TwoD
    }

    fn handle_pointer_event(
        &self,
        session: &mut EditorModeSession,
        _event: &EditorPointerEventDto,
    ) -> Result<(), String> {
        session.bump_revision();
        Ok(())
    }

    fn hit_test(
        &self,
        _session: &EditorModeSession,
        _event: &EditorPointerEventDto,
    ) -> Result<EditorHitTestResultDto, String> {
        Err("EDITOR_CANVAS_2D_HIT_TEST_NOT_IMPLEMENTED".to_owned())
    }
}
