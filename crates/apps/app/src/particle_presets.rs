use std::fs;
use std::path::Path;

use amigo_2d_particles::{ParticleEmitter2d, ParticlePreset2d, ParticlePreset2dService};
use amigo_core::{AmigoError, AmigoResult};
use amigo_modding::ModCatalog;
use amigo_runtime::Runtime;
use amigo_scene::{
    SceneCommand, SceneComponentDocument, SceneDocument, SceneEntityDocument,
    SceneMetadataDocument, build_scene_hydration_plan,
};

use crate::runtime_context::required;

pub(crate) fn load_particle_preset_catalog(runtime: &Runtime) -> AmigoResult<()> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let presets = required::<ParticlePreset2dService>(runtime)?;
    presets.clear();

    for discovered_mod in mod_catalog.mods() {
        let preset_dir = discovered_mod.root_path.join("presets");
        if !preset_dir.is_dir() {
            continue;
        }

        for entry in fs::read_dir(&preset_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("yml") {
                continue;
            }
            let preset = load_particle_preset_file(&discovered_mod.manifest.id, &path)?;
            presets.register(preset);
        }
    }

    Ok(())
}

fn load_particle_preset_file(source_mod: &str, path: &Path) -> AmigoResult<ParticlePreset2d> {
    let raw = fs::read_to_string(path)?;
    let document = serde_yaml::from_str::<serde_yaml::Value>(&raw).map_err(|error| {
        AmigoError::Message(format!(
            "failed to parse particle preset `{}`: {error}",
            path.display()
        ))
    })?;
    if string_field(&document, "kind") != Some("particle-preset-2d") {
        return Err(AmigoError::Message(format!(
            "particle preset `{}` must declare kind: particle-preset-2d",
            path.display()
        )));
    }

    let id = string_field(&document, "id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "particle preset `{}` must declare non-empty id",
                path.display()
            ))
        })?
        .to_owned();
    let label = string_field(&document, "label")
        .unwrap_or(id.as_str())
        .to_owned();
    let category = string_field(&document, "category")
        .unwrap_or_default()
        .to_owned();
    let tags = string_sequence_field(&document, "tags");
    let emitter_value = mapping_value(&document, "emitter").ok_or_else(|| {
        AmigoError::Message(format!(
            "particle preset `{}` must declare emitter",
            path.display()
        ))
    })?;
    let emitter_component = serde_yaml::from_value::<SceneComponentDocument>(emitter_value.clone())
        .map_err(|error| {
            AmigoError::Message(format!(
                "failed to parse emitter in particle preset `{}`: {error}",
                path.display()
            ))
        })?;
    if !matches!(
        emitter_component,
        SceneComponentDocument::ParticleEmitter2d { .. }
    ) {
        return Err(AmigoError::Message(format!(
            "particle preset `{}` emitter must be type: ParticleEmitter2D",
            path.display()
        )));
    }

    let scene_document = SceneDocument {
        version: 1,
        scene: SceneMetadataDocument {
            id: format!("particle-preset-{id}"),
            label: label.clone(),
            description: None,
        },
        transitions: Vec::new(),
        collision_events: Vec::new(),
        audio_cues: Vec::new(),
        activation_sets: Vec::new(),
        entities: vec![SceneEntityDocument {
            id: id.clone(),
            name: format!("particle-preset-{id}"),
            tags: Vec::new(),
            groups: Vec::new(),
            visible: false,
            simulation_enabled: false,
            collision_enabled: false,
            properties: Default::default(),
            transform2: None,
            transform3: None,
            components: vec![emitter_component],
        }],
    };
    let plan = build_scene_hydration_plan(source_mod, &scene_document).map_err(|error| {
        AmigoError::Message(format!(
            "failed to hydrate particle preset `{}`: {error}",
            path.display()
        ))
    })?;
    let emitter = plan
        .commands
        .iter()
        .find_map(|command| match command {
            SceneCommand::QueueParticleEmitter2d { command } => {
                Some(ParticleEmitter2d::from_scene_command(command))
            }
            _ => None,
        })
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "particle preset `{}` did not produce ParticleEmitter2D command",
                path.display()
            ))
        })?;

    Ok(ParticlePreset2d {
        source_mod: source_mod.to_owned(),
        id,
        label,
        category,
        tags,
        emitter,
    })
}

fn string_field<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a str> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
        .and_then(serde_yaml::Value::as_str)
}

fn mapping_value<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
}

fn string_sequence_field(value: &serde_yaml::Value, key: &str) -> Vec<String> {
    mapping_value(value, key)
        .and_then(serde_yaml::Value::as_sequence)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_yaml::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
