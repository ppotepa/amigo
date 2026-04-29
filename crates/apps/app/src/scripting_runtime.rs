use super::*;

#[derive(Debug, Clone)]
struct PreparedScriptSource {
    executed: ExecutedScript,
    source: String,
}

fn mod_script_source_name(mod_id: &str) -> String {
    format!("mod:{mod_id}")
}

fn scene_script_source_name(mod_id: &str, scene_id: &str) -> String {
    format!("scene:{mod_id}:{scene_id}")
}

fn build_script_descriptor(
    root_path: &Path,
    absolute_script_path: &Path,
    source_name: String,
    mod_id: &str,
    scene_id: Option<&str>,
    role: ScriptExecutionRole,
) -> AmigoResult<ExecutedScript> {
    let relative_script_path =
        crate::app_helpers::relative_path_within_root(root_path, absolute_script_path)?;

    Ok(ExecutedScript {
        source_name,
        mod_id: mod_id.to_owned(),
        scene_id: scene_id.map(str::to_owned),
        relative_script_path,
        role,
    })
}

fn prepare_mod_script_source(
    script_runtime: &ScriptRuntimeService,
    discovered_mod: &amigo_modding::DiscoveredMod,
) -> AmigoResult<Option<PreparedScriptSource>> {
    let Some(scripting) = discovered_mod.manifest.scripting.as_ref() else {
        return Ok(None);
    };

    let role = match scripting.mod_script_mode {
        ModScriptMode::Disabled => return Ok(None),
        ModScriptMode::Bootstrap => ScriptExecutionRole::ModBootstrap,
        ModScriptMode::Persistent => ScriptExecutionRole::ModPersistent,
    };

    let script_path = discovered_mod.mod_script_path().ok_or_else(|| {
        AmigoError::Message(format!(
            "mod `{}` enables scripting but has no configured mod script path",
            discovered_mod.manifest.id
        ))
    })?;
    let descriptor = build_script_descriptor(
        &discovered_mod.root_path,
        &script_path,
        mod_script_source_name(&discovered_mod.manifest.id),
        &discovered_mod.manifest.id,
        None,
        role,
    )?;
    crate::app_helpers::validate_script_path(
        script_runtime,
        &descriptor.relative_script_path,
        &format!("mod script for mod `{}`", discovered_mod.manifest.id),
    )?;

    let source = fs::read_to_string(&script_path).map_err(|error| {
        AmigoError::Message(format!(
            "failed to read mod script for mod `{}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;
    script_runtime.validate_source(&source).map_err(|error| {
        AmigoError::Message(format!(
            "failed to validate mod script for mod `{}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;

    Ok(Some(PreparedScriptSource {
        executed: descriptor,
        source,
    }))
}

fn scene_script_descriptor_for_scene(
    discovered_mod: &amigo_modding::DiscoveredMod,
    scene_id: &str,
) -> AmigoResult<Option<ExecutedScript>> {
    let Some(scene_manifest) = discovered_mod.scene_by_id(scene_id) else {
        return Ok(None);
    };
    let Some(script_path) = discovered_mod.scene_script_path(scene_id) else {
        return Ok(None);
    };

    if !script_path.is_file() {
        return if scene_manifest.script.is_some() {
            Err(AmigoError::Message(format!(
                "scene `{}` for mod `{}` declares script `{}` but the file does not exist",
                scene_id,
                discovered_mod.manifest.id,
                script_path.display()
            )))
        } else {
            Ok(None)
        };
    }

    build_script_descriptor(
        &discovered_mod.root_path,
        &script_path,
        scene_script_source_name(&discovered_mod.manifest.id, scene_id),
        &discovered_mod.manifest.id,
        Some(scene_id),
        ScriptExecutionRole::Scene,
    )
    .map(Some)
}

fn prepare_scene_script_source(
    script_runtime: &ScriptRuntimeService,
    discovered_mod: &amigo_modding::DiscoveredMod,
    scene_id: &str,
) -> AmigoResult<Option<PreparedScriptSource>> {
    let Some(descriptor) = scene_script_descriptor_for_scene(discovered_mod, scene_id)? else {
        return Ok(None);
    };
    let script_path = discovered_mod.scene_script_path(scene_id).ok_or_else(|| {
        AmigoError::Message(format!(
            "scene `{scene_id}` for mod `{}` has no resolved script path",
            discovered_mod.manifest.id
        ))
    })?;
    crate::app_helpers::validate_script_path(
        script_runtime,
        &descriptor.relative_script_path,
        &format!(
            "scene script for `{}` scene `{scene_id}`",
            discovered_mod.manifest.id
        ),
    )?;

    let source = fs::read_to_string(&script_path).map_err(|error| {
        AmigoError::Message(format!(
            "failed to read scene script for `{}` scene `{scene_id}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;
    script_runtime.validate_source(&source).map_err(|error| {
        AmigoError::Message(format!(
            "failed to validate scene script for `{}` scene `{scene_id}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;

    Ok(Some(PreparedScriptSource {
        executed: descriptor,
        source,
    }))
}

pub(crate) fn execute_mod_scripts(runtime: &Runtime) -> AmigoResult<()> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let script_runtime = required::<ScriptRuntimeService>(runtime)?;

    for discovered_mod in mod_catalog.mods() {
        let Some(prepared) = prepare_mod_script_source(script_runtime.as_ref(), discovered_mod)?
        else {
            continue;
        };
        script_runtime.execute_source(&prepared.executed.source_name, &prepared.source)?;
        if prepared.executed.role == ScriptExecutionRole::ModBootstrap {
            script_runtime.unload_source(&prepared.executed.source_name)?;
        }
    }

    Ok(())
}

pub(crate) fn persistent_mod_script_descriptors(
    mod_catalog: &ModCatalog,
) -> AmigoResult<Vec<ExecutedScript>> {
    let mut scripts = Vec::new();

    for discovered_mod in mod_catalog.mods() {
        let Some(scripting) = discovered_mod.manifest.scripting.as_ref() else {
            continue;
        };
        if scripting.mod_script_mode != ModScriptMode::Persistent {
            continue;
        }
        let Some(script_path) = discovered_mod.mod_script_path() else {
            continue;
        };
        if !script_path.is_file() {
            return Err(AmigoError::Message(format!(
                "persistent mod script for `{}` does not exist at `{}`",
                discovered_mod.manifest.id,
                script_path.display()
            )));
        }

        scripts.push(build_script_descriptor(
            &discovered_mod.root_path,
            &script_path,
            mod_script_source_name(&discovered_mod.manifest.id),
            &discovered_mod.manifest.id,
            None,
            ScriptExecutionRole::ModPersistent,
        )?);
    }

    Ok(scripts)
}

pub(crate) fn active_scene_script_descriptor(
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
    active_scene: Option<&str>,
) -> AmigoResult<Option<ExecutedScript>> {
    let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
        return Ok(None);
    };
    let Some(active_scene) = active_scene else {
        return Ok(None);
    };

    mod_catalog
        .mod_by_id(startup_mod)
        .map(|discovered_mod| scene_script_descriptor_for_scene(discovered_mod, active_scene))
        .transpose()
        .map(|descriptor| descriptor.flatten())
}

pub(crate) fn current_executed_scripts(runtime: &Runtime) -> AmigoResult<Vec<ExecutedScript>> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let launch_selection = required::<LaunchSelection>(runtime)?;
    let scene_service = required::<SceneService>(runtime)?;

    let mut scripts = persistent_mod_script_descriptors(mod_catalog.as_ref())?;
    if let Some(scene_script) = active_scene_script_descriptor(
        mod_catalog.as_ref(),
        launch_selection.as_ref(),
        scene_service.selected_scene().as_ref().map(SceneKey::as_str),
    )? {
        scripts.push(scene_script);
    }

    Ok(scripts)
}

pub(crate) fn dispatch_script_event_to_active_scripts(
    script_runtime: &ScriptRuntimeService,
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
    scene_service: &SceneService,
    event: &ScriptEvent,
) -> AmigoResult<()> {
    for script in persistent_mod_script_descriptors(mod_catalog)? {
        script_runtime.call_on_event(&script.source_name, &event.topic, &event.payload)?;
    }
    if let Some(scene_script) = active_scene_script_descriptor(
        mod_catalog,
        launch_selection,
        scene_service.selected_scene().as_ref().map(SceneKey::as_str),
    )? {
        script_runtime.call_on_event(&scene_script.source_name, &event.topic, &event.payload)?;
    }

    Ok(())
}

pub(crate) fn sync_active_scene_script_lifecycle(
    scene_service: &SceneService,
    script_lifecycle_state: &ScriptLifecycleState,
    script_runtime: &ScriptRuntimeService,
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
) -> AmigoResult<bool> {
    let current_scene = scene_service
        .selected_scene()
        .map(|scene| scene.as_str().to_owned());
    let previous_scene = script_lifecycle_state.active_scene();

    if current_scene == previous_scene {
        return Ok(false);
    }

    let previous_scene_script =
        active_scene_script_descriptor(mod_catalog, launch_selection, previous_scene.as_deref())?;
    let current_scene_script = if let Some(current_scene_id) = current_scene.as_deref() {
        let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
            script_lifecycle_state.set_active_scene(current_scene.clone());
            return Ok(false);
        };
        let Some(discovered_mod) = mod_catalog.mod_by_id(startup_mod) else {
            script_lifecycle_state.set_active_scene(current_scene.clone());
            return Ok(false);
        };
        prepare_scene_script_source(script_runtime, discovered_mod, current_scene_id)?
    } else {
        None
    };

    if let Some(previous_script) = &previous_scene_script {
        script_runtime.call_on_exit(&previous_script.source_name)?;
        script_runtime.unload_source(&previous_script.source_name)?;
    }

    script_lifecycle_state.set_active_scene(current_scene.clone());

    if let Some(current_script) = current_scene_script {
        script_runtime.execute_source(&current_script.executed.source_name, &current_script.source)?;
        script_runtime.call_on_enter(&current_script.executed.source_name)?;
        return Ok(true);
    }

    Ok(previous_scene_script.is_some())
}
