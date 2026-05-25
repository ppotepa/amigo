use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_scene::SceneComponentDocument as ComponentDocument;
use amigo_scene::{
    Camera2dModeDocument, CameraAperture2dDocument, CameraExposure2dDocument, CameraFilm2dDocument,
    CameraLens2dDocument, CameraLensSurface2dDocument, CameraLook2dDocument,
    CameraShutter2dDocument, RenderContributionsDocument, SceneComponentDocument,
    SceneComponentPayload, SceneComponentSchemaProvider, SceneDocumentError, SceneDocumentResult,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Camera2dDocument {
    #[serde(default = "default_camera2d_id")]
    pub id: String,
    #[serde(default)]
    pub mode: Camera2dModeDocument,
    #[serde(default)]
    pub render_contributions: RenderContributionsDocument,
    #[serde(default)]
    pub exposure: CameraExposure2dDocument,
    #[serde(default)]
    pub shutter: CameraShutter2dDocument,
    #[serde(default)]
    pub lens: CameraLens2dDocument,
    #[serde(default)]
    pub lens_surface: CameraLensSurface2dDocument,
    #[serde(default)]
    pub film: CameraFilm2dDocument,
    #[serde(default)]
    pub look: CameraLook2dDocument,
    #[serde(default)]
    pub aperture: CameraAperture2dDocument,
}

impl Default for Camera2dDocument {
    fn default() -> Self {
        Self {
            id: default_camera2d_id(),
            mode: Camera2dModeDocument::default(),
            render_contributions: RenderContributionsDocument::default(),
            exposure: CameraExposure2dDocument::default(),
            shutter: CameraShutter2dDocument::default(),
            lens: CameraLens2dDocument::default(),
            lens_surface: CameraLensSurface2dDocument::default(),
            film: CameraFilm2dDocument::default(),
            look: CameraLook2dDocument::default(),
            aperture: CameraAperture2dDocument::default(),
        }
    }
}

impl Camera2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            ComponentDocument::Camera2d {
                id,
                mode,
                render_contributions,
                exposure,
                shutter,
                lens,
                lens_surface,
                film,
                look,
                aperture,
            } => Some(Self {
                id: id.clone(),
                mode: *mode,
                render_contributions: render_contributions.clone(),
                exposure: exposure.clone(),
                shutter: shutter.clone(),
                lens: lens.clone(),
                lens_surface: lens_surface.clone(),
                film: film.clone(),
                look: look.clone(),
                aperture: aperture.clone(),
            }),
            _ => None,
        }
    }
}

impl SceneComponentPayload for Camera2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.camera.camera-core.Camera2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_camera_2d_plugin_payload(payload: &Value) -> SceneDocumentResult<Camera2dDocument> {
    serde_yaml::from_value::<Camera2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy)]
pub struct Camera2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for Camera2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.camera.camera-core.Camera2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["Camera2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<Camera2dDocument>(Value::Mapping(
            payload,
        ))?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_camera_2d_plugin_payload(payload)?))
    }
}

fn default_camera2d_id() -> String {
    "main".to_owned()
}
