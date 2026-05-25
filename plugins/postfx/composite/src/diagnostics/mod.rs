use std::collections::BTreeMap;

use crate::{PostFx2d, PostFxPipelineKind, PostFxRole2d, PostFxScope2d, ScopedPostFx2dStack};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostFxDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostFxDiagnostic2d {
    pub severity: PostFxDiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
    pub host_id: Option<String>,
    pub effect_id: Option<String>,
    pub family: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct FamilyUse {
    role: PostFxRole2d,
    kind: &'static str,
    host_id: String,
    effect_id: String,
}

pub fn diagnose_post_fx_stacks(stacks: &[ScopedPostFx2dStack]) -> Vec<PostFxDiagnostic2d> {
    let mut diagnostics = Vec::new();
    let mut families: BTreeMap<&'static str, Vec<FamilyUse>> = BTreeMap::new();

    for stack in stacks {
        let supported_scope_and_pipeline = matches!(
            (&stack.scope, stack.pipeline),
            (PostFxScope2d::Frame, PostFxPipelineKind::FrameGraph)
                | (
                    PostFxScope2d::DrawLayer { .. },
                    PostFxPipelineKind::OffscreenDrawLayer
                )
                | (
                    PostFxScope2d::SceneObjectPixels { .. },
                    PostFxPipelineKind::OffscreenObject
                )
                | (
                    PostFxScope2d::GroupSubtree { .. },
                    PostFxPipelineKind::OffscreenGroup
                )
                | (
                    PostFxScope2d::SourceImage { .. },
                    PostFxPipelineKind::CachedImage
                )
                | (
                    PostFxScope2d::ImagePart { .. },
                    PostFxPipelineKind::CachedImage
                )
        );
        if !supported_scope_and_pipeline {
            let active_effects = stack
                .effects
                .iter()
                .filter(|effect| effect.effect.is_active())
                .count();
            diagnostics.push(PostFxDiagnostic2d {
                severity: PostFxDiagnosticSeverity::Warning,
                code: "unsupported_scoped_post_fx",
                message: format!(
                    "scoped post-fx execution is not implemented for scope={} pipeline={:?} host={} active_effects={}",
                    stack.scope.label(),
                    stack.pipeline,
                    stack.host_id.as_str(),
                    active_effects
                ),
                host_id: Some(stack.host_id.as_str().to_owned()),
                effect_id: None,
                family: None,
            });
            continue;
        }

        if stack.scope == PostFxScope2d::Frame && stack.pipeline != PostFxPipelineKind::FrameGraph {
            diagnostics.push(PostFxDiagnostic2d {
                severity: PostFxDiagnosticSeverity::Warning,
                code: "unsupported_frame_post_fx_pipeline",
                message: format!(
                    "frame post-fx pipeline {:?} is not executable",
                    stack.pipeline
                ),
                host_id: Some(stack.host_id.as_str().to_owned()),
                effect_id: None,
                family: None,
            });
            continue;
        }

        for instance in &stack.effects {
            let role = instance.effect.default_role();
            let Some(family) = instance.effect.photographic_family() else {
                continue;
            };
            families.entry(family).or_default().push(FamilyUse {
                role,
                kind: effect_kind_label(&instance.effect),
                host_id: stack.host_id.as_str().to_owned(),
                effect_id: instance.id.as_str().to_owned(),
            });
        }
    }

    for (family, uses) in families {
        let Some(camera_use) = uses.iter().find(|entry| {
            entry.role == PostFxRole2d::CameraCapture
                || (family == "look" && entry.host_id.starts_with("camera:"))
        }) else {
            continue;
        };
        let duplicates = uses
            .iter()
            .filter(|entry| {
                entry.host_id != camera_use.host_id || entry.effect_id != camera_use.effect_id
            })
            .collect::<Vec<_>>();
        if duplicates.is_empty() {
            continue;
        }

        let mut pushed_specific = false;
        for duplicate in duplicates {
            let specific = match family {
                "film_scan" if duplicate.kind == "film_noise" => Some((
                    "camera_film_scan_duplicated",
                    format!(
                        "camera film/scan duplicates scene film grain host={} effect={}",
                        duplicate.host_id, duplicate.effect_id
                    ),
                )),
                "lens_surface" if matches!(duplicate.kind, "rain_glass" | "lens_droplets") => {
                    Some((
                        "camera_lens_surface_duplicated",
                        format!(
                            "camera lens surface duplicates scene lens surface host={} effect={}",
                            duplicate.host_id, duplicate.effect_id
                        ),
                    ))
                }
                "look" if matches!(duplicate.kind, "color_ramp" | "color_quantize") => Some((
                    "camera_look_duplicated",
                    format!(
                        "camera look duplicates presentation look host={} effect={}",
                        duplicate.host_id, duplicate.effect_id
                    ),
                )),
                "shutter" if duplicate.kind == "shutter_blur" => Some((
                    "camera_shutter_duplicated",
                    format!(
                        "camera shutter duplicates manual shutter blur host={} effect={}",
                        duplicate.host_id, duplicate.effect_id
                    ),
                )),
                "dof" if duplicate.kind == "blur" => Some((
                    "camera_dof_may_be_duplicated",
                    format!(
                        "camera dof may be duplicated by manual blur host={} effect={}",
                        duplicate.host_id, duplicate.effect_id
                    ),
                )),
                _ => None,
            };

            if let Some((code, message)) = specific {
                diagnostics.push(PostFxDiagnostic2d {
                    severity: PostFxDiagnosticSeverity::Warning,
                    code,
                    message,
                    host_id: Some(duplicate.host_id.clone()),
                    effect_id: Some(duplicate.effect_id.clone()),
                    family: Some(family),
                });
                pushed_specific = true;
            }
        }

        if !pushed_specific {
            diagnostics.push(PostFxDiagnostic2d {
                severity: PostFxDiagnosticSeverity::Warning,
                code: "duplicate_photographic_family",
                message: format!(
                    "camera capture stack and another post-fx role both affect `{family}` camera_host={} duplicate_host={}",
                    camera_use.host_id,
                    uses
                        .iter()
                        .find(|entry| entry.role != PostFxRole2d::CameraCapture)
                        .map(|entry| entry.host_id.as_str())
                        .unwrap_or("-")
                ),
                host_id: Some(camera_use.host_id.clone()),
                effect_id: Some(camera_use.effect_id.clone()),
                family: Some(family),
            });
        }
    }

    diagnostics
}

fn effect_kind_label(effect: &PostFx2d) -> &'static str {
    effect.kind()
}
