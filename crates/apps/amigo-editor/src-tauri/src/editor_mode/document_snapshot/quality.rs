use std::collections::BTreeMap;

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};
use crate::editor_mode::dto::{EditorSceneObjectDto, EditorSceneSnapshotQualityDto};

use super::{
    DIAG_COMPONENT_BOUNDS_UNSUPPORTED, DIAG_ENTITY_LOCKED_BY_PLACEMENT, DIAG_ENTITY_NO_BOUNDS2,
    DIAG_ENTITY_NO_TRANSFORM2,
};

pub fn append_object_quality_diagnostics(
    objects: &[EditorSceneObjectDto],
    diagnostics: &mut Vec<EditorDiagnosticDto>,
) {
    for object in objects {
        if object.transform_2.is_none() && object.transform_3.is_none() {
            push_diagnostic(
                diagnostics,
                DiagnosticLevel::Info,
                DIAG_ENTITY_NO_TRANSFORM2,
                format!(
                    "Entity `{}` has no transform2/transform3 and is not editable in the viewport.",
                    object.entity_id
                ),
            );
            continue;
        }

        if object.transform_2.is_some() && object.bounds_2.is_none() {
            let code = if object.category == "script" || object.category == "other" {
                DIAG_COMPONENT_BOUNDS_UNSUPPORTED
            } else {
                DIAG_ENTITY_NO_BOUNDS2
            };
            push_diagnostic(
                diagnostics,
                DiagnosticLevel::Info,
                code,
                format!(
                    "Entity `{}` has transform2 but no supported 2D bounds provider.",
                    object.entity_id
                ),
            );
        }

        if object.locked && object.selectable {
            push_diagnostic(
                diagnostics,
                DiagnosticLevel::Info,
                DIAG_ENTITY_LOCKED_BY_PLACEMENT,
                format!(
                    "Entity `{}` is selectable but locked: {}",
                    object.entity_id,
                    object
                        .locked_reason
                        .clone()
                        .unwrap_or_else(|| "no editor command is available".to_owned())
                ),
            );
        }
    }
}

pub fn snapshot_quality(
    indexed_entities: usize,
    objects: &[EditorSceneObjectDto],
    diagnostics: &[EditorDiagnosticDto],
) -> EditorSceneSnapshotQualityDto {
    let mut diagnostics_by_code = BTreeMap::new();
    for diagnostic in diagnostics {
        *diagnostics_by_code
            .entry(diagnostic.code.clone())
            .or_insert(0) += 1;
    }

    EditorSceneSnapshotQualityDto {
        indexed_entities,
        objects: objects.len(),
        editable_objects: objects
            .iter()
            .filter(|object| object.selectable && !object.locked && object.bounds_2.is_some())
            .count(),
        objects_without_transform: objects
            .iter()
            .filter(|object| object.transform_2.is_none() && object.transform_3.is_none())
            .count(),
        objects_without_bounds: objects
            .iter()
            .filter(|object| object.bounds_2.is_none())
            .count(),
        unsupported_bounds_providers: diagnostics_by_code
            .get(DIAG_COMPONENT_BOUNDS_UNSUPPORTED)
            .copied()
            .unwrap_or(0),
        diagnostics_by_code,
    }
}

pub fn push_diagnostic(
    diagnostics: &mut Vec<EditorDiagnosticDto>,
    level: DiagnosticLevel,
    code: &str,
    message: impl Into<String>,
) {
    diagnostics.push(EditorDiagnosticDto {
        level,
        code: code.to_owned(),
        message: message.into(),
        path: None,
    });
}
