use amigo_math::{Curve1d, CurvePoint1d};

use crate::*;

pub(super) fn curve1d_from_optional_document(document: Option<&Curve1dSceneDocument>) -> Curve1d {
    document
        .map(curve1d_from_document)
        .unwrap_or(Curve1d::Linear)
}

pub(super) fn curve1d_from_document(document: &Curve1dSceneDocument) -> Curve1d {
    match document {
        Curve1dSceneDocument::Constant { value } => Curve1d::Constant(*value),
        Curve1dSceneDocument::Linear => Curve1d::Linear,
        Curve1dSceneDocument::EaseIn => Curve1d::EaseIn,
        Curve1dSceneDocument::EaseOut => Curve1d::EaseOut,
        Curve1dSceneDocument::EaseInOut => Curve1d::EaseInOut,
        Curve1dSceneDocument::SmoothStep => Curve1d::SmoothStep,
        Curve1dSceneDocument::Custom { points } => Curve1d::Custom {
            points: points
                .iter()
                .map(|point| CurvePoint1d {
                    t: point.t,
                    value: point.value,
                })
                .collect(),
        },
    }
}

pub(super) fn event_pipeline_step_from_document(
    step: &SceneEventPipelineStepDocument,
) -> EventPipelineStepSceneCommand {
    match step {
        SceneEventPipelineStepDocument::PlayAudio { clip } => {
            EventPipelineStepSceneCommand::PlayAudio { clip: clip.clone() }
        }
        SceneEventPipelineStepDocument::SetState { key, value } => {
            EventPipelineStepSceneCommand::SetState {
                key: key.clone(),
                value: value.clone(),
            }
        }
        SceneEventPipelineStepDocument::IncrementState { key, by } => {
            EventPipelineStepSceneCommand::IncrementState {
                key: key.clone(),
                by: *by,
            }
        }
        SceneEventPipelineStepDocument::ShowUi { path } => {
            EventPipelineStepSceneCommand::ShowUi { path: path.clone() }
        }
        SceneEventPipelineStepDocument::HideUi { path } => {
            EventPipelineStepSceneCommand::HideUi { path: path.clone() }
        }
        SceneEventPipelineStepDocument::BurstParticles { emitter, count } => {
            EventPipelineStepSceneCommand::BurstParticles {
                emitter: emitter.clone(),
                count: *count,
            }
        }
        SceneEventPipelineStepDocument::TransitionScene { scene } => {
            EventPipelineStepSceneCommand::TransitionScene {
                scene: scene.clone(),
            }
        }
        SceneEventPipelineStepDocument::EmitEvent { topic, payload } => {
            EventPipelineStepSceneCommand::EmitEvent {
                topic: topic.clone(),
                payload: payload.clone(),
            }
        }
        SceneEventPipelineStepDocument::Script { function } => {
            EventPipelineStepSceneCommand::Script {
                function: function.clone(),
            }
        }
    }
}

pub(super) fn ui_model_binding_from_document(
    binding: &SceneUiModelBindingDocument,
) -> UiModelBindingSceneCommand {
    UiModelBindingSceneCommand {
        path: binding.path.clone(),
        state_key: binding.state.clone(),
        kind: match binding.kind {
            SceneUiModelBindingKindDocument::Text => UiModelBindingKindSceneCommand::Text,
            SceneUiModelBindingKindDocument::Value => UiModelBindingKindSceneCommand::Value,
            SceneUiModelBindingKindDocument::Visible => UiModelBindingKindSceneCommand::Visible,
            SceneUiModelBindingKindDocument::Enabled => UiModelBindingKindSceneCommand::Enabled,
            SceneUiModelBindingKindDocument::Selected => UiModelBindingKindSceneCommand::Selected,
            SceneUiModelBindingKindDocument::Options => UiModelBindingKindSceneCommand::Options,
            SceneUiModelBindingKindDocument::Color => UiModelBindingKindSceneCommand::Color,
            SceneUiModelBindingKindDocument::Background => {
                UiModelBindingKindSceneCommand::Background
            }
            SceneUiModelBindingKindDocument::Theme => UiModelBindingKindSceneCommand::Theme,
        },
        format: binding.format.clone(),
    }
}

pub(super) fn script_component_param_from_document(
    value: &ScenePropertyValueDocument,
) -> ScriptComponentParamValueSceneCommand {
    match value {
        ScenePropertyValueDocument::Bool(value) => {
            ScriptComponentParamValueSceneCommand::Bool(*value)
        }
        ScenePropertyValueDocument::Int(value) => {
            ScriptComponentParamValueSceneCommand::Int(*value)
        }
        ScenePropertyValueDocument::Float(value) => {
            ScriptComponentParamValueSceneCommand::Float(*value)
        }
        ScenePropertyValueDocument::String(value) => {
            ScriptComponentParamValueSceneCommand::String(value.clone())
        }
    }
}

