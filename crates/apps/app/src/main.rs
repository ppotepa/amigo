use std::path::PathBuf;

use amigo_app::{BootstrapOptions, ScenePreviewHost, ScenePreviewOptions};
use amigo_core::{AmigoError, AmigoResult};
use amigo_math::Vec3;
use amigo_modding::requested_mods_for_root;

fn main() -> AmigoResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if has_flag(&args, "--preview-capture") {
        return run_preview_capture(&args);
    }

    let editor_requested = has_flag(&args, "--editor");
    let hosted = has_flag(&args, "--hosted") || editor_requested;
    let dev_mode = has_flag(&args, "--dev") || editor_requested;
    let mods_root = parse_option_value(&args, "--mods-root").unwrap_or_else(|| "mods".to_owned());
    let startup_mod = parse_option_value(&args, "--mod")
        .or_else(|| editor_requested.then(|| "playground-2d".to_owned()));
    let startup_scene = parse_option_value(&args, "--scene")
        .or_else(|| editor_requested.then(|| "screen-space-preview".to_owned()));
    let editor_mode = editor_requested;
    let active_mods = parse_option_value(&args, "--mods").map(|mods| {
        mods.split(',')
            .filter(|mod_id| !mod_id.trim().is_empty())
            .map(|mod_id| mod_id.trim().to_owned())
            .collect::<Vec<_>>()
    });

    let mut options = BootstrapOptions::new(mods_root)
        .with_dev_mode(dev_mode)
        .with_editor_mode(editor_mode);

    if let Some(active_mods) = active_mods {
        options = options.with_active_mods(active_mods);
    }

    if let Some(startup_mod) = startup_mod {
        if options.active_mods.is_none() {
            options = options.with_active_mods(requested_mods_for_root(&startup_mod));
        }
        options = options.with_startup_mod(startup_mod);
    }

    if let Some(startup_scene) = startup_scene {
        options = options.with_startup_scene(startup_scene);
    };

    if hosted {
        amigo_app::run_hosted_with_options(options)?;
    } else {
        let bootstrap = amigo_app::bootstrap_session_with_options(options)?;
        let summary = bootstrap.summary().clone();
        println!("{summary}");
    }

    Ok(())
}

fn run_preview_capture(args: &[String]) -> AmigoResult<()> {
    let mods_root = parse_option_value(args, "--mods-root").unwrap_or_else(|| "mods".to_owned());
    let startup_mod =
        parse_option_value(args, "--mod").unwrap_or_else(|| "playground-npr".to_owned());
    let startup_scene =
        parse_option_value(args, "--scene").unwrap_or_else(|| "comic-lines".to_owned());
    let active_mods = parse_option_value(args, "--mods")
        .map(parse_csv_values)
        .unwrap_or_else(|| requested_mods_for_root(&startup_mod));
    let width = parse_u32_option(args, "--width").unwrap_or(1280);
    let height = parse_u32_option(args, "--height").unwrap_or(720);
    let warmup_frames = parse_u32_option(args, "--warmup").unwrap_or(3);
    let settle_frames = parse_u32_option(args, "--settle").unwrap_or(1);
    let entity = parse_option_value(args, "--entity")
        .unwrap_or_else(|| "playground-npr-model-1-soldier".to_owned());
    let hud_entity =
        parse_option_value(args, "--hud-entity").unwrap_or_else(|| "playground-npr-hud".to_owned());
    let hud_path =
        parse_option_value(args, "--hud-path").unwrap_or_else(|| "playground-npr-hud.root".to_owned());
    let hide_hud = !has_flag(args, "--show-hud");
    let preset = parse_option_value(args, "--preset");
    let model_query =
        parse_option_value(args, "--mesh").or_else(|| parse_option_value(args, "--model"));
    let strategy = parse_option_value(args, "--strategy");
    let debug_mode = parse_option_value(args, "--debug");
    let scale = parse_f32_option(args, "--scale");
    let translation = parse_vec3_option(args, "--translate")?;
    let rotation_degrees = parse_vec3_option(args, "--rotation-deg")?;
    let output = PathBuf::from(
        parse_option_value(args, "--output").unwrap_or_else(|| "images/current.png".to_owned()),
    );

    let options = ScenePreviewOptions::new(mods_root, startup_mod, startup_scene, width, height)
        .with_active_mods(active_mods)
        .with_warmup_frames(warmup_frames);
    let mut host = ScenePreviewHost::new(options);
    host.prime()?;

    let resolved_mesh = if let Some(model_query) = model_query.as_deref() {
        let Some(mesh_key) = host.mesh3d_asset_key_for_query(model_query)? else {
            return Err(AmigoError::Message(format!(
                "preview capture model `{model_query}` did not match a prepared mesh-3d asset"
            )));
        };
        host.set_mesh3d_asset(entity.clone(), mesh_key.clone())?;
        Some(mesh_key)
    } else {
        None
    };

    if let Some(preset) = preset.as_deref() {
        host.apply_mesh3d_npr_preset(entity.clone(), preset.to_owned())?;
    }
    if let Some(strategy) = strategy.as_deref() {
        host.set_mesh3d_npr_render_strategy(entity.clone(), strategy.to_owned())?;
    }
    if let Some(debug_mode) = debug_mode.as_deref() {
        host.set_mesh3d_npr_gpu_debug_mode(entity.clone(), debug_mode.to_owned())?;
    }
    if scale.is_some() || translation.is_some() || rotation_degrees.is_some() {
        host.set_scene_entity_transform_overrides(
            entity.clone(),
            scale,
            translation,
            rotation_degrees,
        )?;
    }
    if hide_hud {
        host.set_scene_entity_visible(&hud_entity, false)?;
        host.submit_script_command("ui", "hide", vec![hud_path.clone()])?;
    }
    if settle_frames > 0 {
        host.warmup(settle_frames)?;
    }

    let frame = host.capture_rgba8()?;
    save_rgba8_png(&output, frame.width, frame.height, &frame.pixels_rgba8)?;

    println!(
        "preview capture saved: path={} size={}x{} entity={} mesh={} preset={} strategy={} debug={} hud={}",
        output.display(),
        frame.width,
        frame.height,
        entity,
        resolved_mesh.as_deref().unwrap_or("<scene default>"),
        preset.as_deref().unwrap_or("<scene default>"),
        strategy.as_deref().unwrap_or("<preset/default>"),
        debug_mode.as_deref().unwrap_or("final"),
        if hide_hud { "hidden" } else { "visible" },
    );

    Ok(())
}

fn save_rgba8_png(path: &PathBuf, width: u32, height: u32, pixels: &[u8]) -> AmigoResult<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let image = image::RgbaImage::from_raw(width, height, pixels.to_vec()).ok_or_else(|| {
        AmigoError::Message(format!(
            "preview capture produced invalid RGBA8 buffer: {} bytes for {}x{}",
            pixels.len(),
            width,
            height
        ))
    })?;
    image.save(path).map_err(|error| {
        AmigoError::Message(format!("failed to save `{}`: {error}", path.display()))
    })
}

fn parse_option_value(args: &[String], flag: &str) -> Option<String> {
    for argument in args {
        if let Some(value) = argument.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_owned());
        }
    }

    args.windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}

fn parse_u32_option(args: &[String], flag: &str) -> Option<u32> {
    parse_option_value(args, flag).and_then(|value| value.parse::<u32>().ok())
}

fn parse_f32_option(args: &[String], flag: &str) -> Option<f32> {
    parse_option_value(args, flag).and_then(|value| value.parse::<f32>().ok())
}

fn parse_vec3_option(args: &[String], flag: &str) -> AmigoResult<Option<Vec3>> {
    let Some(value) = parse_option_value(args, flag) else {
        return Ok(None);
    };
    let parts = value
        .split(',')
        .map(|part| part.trim())
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(AmigoError::Message(format!(
            "{flag} expects `x,y,z`, got `{value}`"
        )));
    }

    let x = parts[0].parse::<f32>().map_err(|error| {
        AmigoError::Message(format!("{flag} has invalid x component `{}`: {error}", parts[0]))
    })?;
    let y = parts[1].parse::<f32>().map_err(|error| {
        AmigoError::Message(format!("{flag} has invalid y component `{}`: {error}", parts[1]))
    })?;
    let z = parts[2].parse::<f32>().map_err(|error| {
        AmigoError::Message(format!("{flag} has invalid z component `{}`: {error}", parts[2]))
    })?;
    Ok(Some(Vec3::new(x, y, z)))
}

fn parse_csv_values(value: String) -> Vec<String> {
    value
        .split(',')
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| entry.trim().to_owned())
        .collect()
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|argument| argument == flag)
}
