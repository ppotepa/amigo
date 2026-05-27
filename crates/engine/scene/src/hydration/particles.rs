use amigo_fx::{ColorInterpolation, ColorRamp, ColorStop};

use super::style::parse_color_rgba_hex;
use crate::*;

pub(super) fn color_ramp_from_document(
    document: &ColorRampSceneDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRamp> {
    Ok(ColorRamp {
        interpolation: match document.interpolation {
            ColorInterpolationSceneDocument::LinearRgb => ColorInterpolation::LinearRgb,
            ColorInterpolationSceneDocument::Step => ColorInterpolation::Step,
        },
        stops: document
            .stops
            .iter()
            .map(|stop| {
                Ok(ColorStop {
                    t: stop.t,
                    color: parse_color_rgba_hex(&stop.color, scene_id, entity_id, component_kind)?,
                })
            })
            .collect::<SceneDocumentResult<Vec<_>>>()?,
    })
}
