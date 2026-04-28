mod config;
mod diagnostics;
mod tui;

use std::env;
use std::process::Command;

use amigo_app::BootstrapOptions;
use amigo_core::{AmigoError, AmigoResult};
use amigo_modding::requested_mods_for_root;
use config::{CargoProfile, LauncherConfig, LauncherProfile};
use diagnostics::ensure_profile_launchable;
use tui::{LaunchMode, TuiOutcome};

fn main() -> AmigoResult<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let run_hosted_direct = args.iter().any(|arg| arg == "--hosted");
    let run_headless_direct = args.iter().any(|arg| arg == "--headless");
    let selected_profile = parse_value_arg(&args, "--profile");
    let startup_mod_override = parse_value_arg(&args, "--mod");
    let startup_scene_override = parse_value_arg(&args, "--scene");
    let config_path = "config/launcher.toml";
    let mut config = LauncherConfig::load(config_path)?;
    config.validate_phase1()?;

    if let Some(profile_id) = selected_profile {
        config.set_active_profile(&profile_id)?;
    }

    if startup_mod_override.is_some() || startup_scene_override.is_some() {
        let profile = config.active_profile_mut()?;

        if let Some(startup_mod) = startup_mod_override {
            profile.root_mod = Some(startup_mod);
        }

        if let Some(startup_scene) = startup_scene_override {
            profile.startup_scene = Some(startup_scene);
        }
    }

    if run_hosted_direct {
        return launch_with_config(&config, LaunchMode::Hosted);
    }

    if run_headless_direct {
        return launch_with_config(&config, LaunchMode::Headless);
    }

    match tui::run_launcher_tui(config_path, config)? {
        TuiOutcome::Launch { config, mode } => launch_with_config(&config, mode),
        TuiOutcome::Quit => Ok(()),
    }
}

fn launch_with_config(config: &LauncherConfig, mode: LaunchMode) -> AmigoResult<()> {
    let profile = config.active_profile()?;

    match profile.cargo_profile {
        CargoProfile::Dev => launch_in_process(config, profile, mode),
        CargoProfile::Release => launch_external_binary(config, profile, mode),
    }
}

fn launch_in_process(
    config: &LauncherConfig,
    profile: &LauncherProfile,
    mode: LaunchMode,
) -> AmigoResult<()> {
    let diagnostics = ensure_profile_launchable(config, profile)?;
    emit_profile_warnings(&diagnostics);

    let mut options = BootstrapOptions::new(&config.mods_root)
        .with_dev_mode(profile.cargo_profile == CargoProfile::Dev);

    let root_mod = profile.root_mod_or_core().to_owned();
    options = options
        .with_active_mods(requested_mods_for_root(&root_mod))
        .with_startup_mod(root_mod.clone());

    if let Some(startup_scene) = profile.startup_scene.as_deref() {
        options = options.with_startup_scene(startup_scene.to_owned());
    }

    if mode == LaunchMode::Hosted {
        return amigo_app::run_hosted_with_options(options);
    }

    let summary = amigo_app::run_with_options(options)?;

    println!("Amigo Launcher");
    println!("profile id: {}", profile.id);
    println!("profile label: {}", profile.display_label());
    println!("cargo profile: {}", profile.cargo_profile.as_str());
    println!("window backend: {}", profile.window_backend);
    println!("input backend: {}", profile.input_backend);
    println!("render backend: {}", profile.render_backend);
    println!("script backend: {}", profile.script_backend);
    println!("file watch backend: {}", summary.file_watch_backend);
    println!("root mod: {}", profile.root_mod_or_core());
    println!(
        "startup scene: {}",
        profile.startup_scene.as_deref().unwrap_or("none")
    );
    println!(
        "active scene: {}",
        summary.active_scene.as_deref().unwrap_or("none")
    );
    println!(
        "scene document: {}",
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| format!(
                "{}:{}",
                document.source_mod,
                document.relative_path.display()
            ))
            .unwrap_or_else(|| "none".to_owned())
    );
    println!(
        "scene document entities: {}",
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| display_list(&document.entity_names))
            .unwrap_or_else(|| "none".to_owned())
    );
    println!(
        "scene document components: {}",
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| display_list(&document.component_kinds))
            .unwrap_or_else(|| "none".to_owned())
    );
    println!(
        "scene document transitions: {}",
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| display_list(&document.transition_ids))
            .unwrap_or_else(|| "none".to_owned())
    );
    println!("mods: {}", display_list(&summary.loaded_mods));
    println!("scene entities: {}", display_list(&summary.scene_entities));
    println!(
        "registered assets: {}",
        display_list(&summary.registered_assets)
    );
    println!("loaded assets: {}", display_list(&summary.loaded_assets));
    println!(
        "prepared assets: {}",
        display_list(&summary.prepared_assets)
    );
    println!("failed assets: {}", display_list(&summary.failed_assets));
    println!(
        "pending asset loads: {}",
        display_list(&summary.pending_asset_loads)
    );
    println!(
        "watched reload targets: {}",
        display_list(&summary.watched_reload_targets)
    );
    println!(
        "2d sprite entities: {}",
        display_list(&summary.sprite_entities_2d)
    );
    println!(
        "2d text entities: {}",
        display_list(&summary.text_entities_2d)
    );
    println!(
        "3d mesh entities: {}",
        display_list(&summary.mesh_entities_3d)
    );
    println!(
        "3d material entities: {}",
        display_list(&summary.material_entities_3d)
    );
    println!(
        "3d text entities: {}",
        display_list(&summary.text_entities_3d)
    );
    println!(
        "script commands: {}",
        display_list(&summary.processed_script_commands)
    );
    println!(
        "scene commands: {}",
        display_list(&summary.processed_scene_commands)
    );
    println!(
        "script events: {}",
        display_list(&summary.processed_script_events)
    );
    println!(
        "console commands: {}",
        display_list(&summary.console_commands)
    );
    println!("console output: {}", display_list(&summary.console_output));
    let executed_scripts = summary
        .executed_scripts
        .iter()
        .map(|script| {
            format!(
                "{}:{}",
                script.mod_id,
                script.relative_script_path.display()
            )
        })
        .collect::<Vec<_>>();
    println!("scripts: {}", display_list(&executed_scripts));

    Ok(())
}

fn launch_external_binary(
    config: &LauncherConfig,
    profile: &LauncherProfile,
    mode: LaunchMode,
) -> AmigoResult<()> {
    let diagnostics = ensure_profile_launchable(config, profile)?;
    emit_profile_warnings(&diagnostics);
    let app_binary = release_app_binary_path()?;

    if !app_binary.exists() {
        return Err(AmigoError::Message(format!(
            "release profile `{}` requires `{}`; build it first with `cargo build --release -p amigo-app`",
            profile.id,
            app_binary.display()
        )));
    }

    println!(
        "Launching external {} binary for launcher profile `{}`",
        profile.cargo_profile.as_str(),
        profile.id
    );

    let mut command = Command::new(&app_binary);

    if mode == LaunchMode::Hosted {
        command.arg("--hosted");
    }

    command.args(["--mods-root", &config.mods_root]);
    command.arg(format!("--mod={}", profile.root_mod_or_core()));

    if let Some(startup_scene) = profile.startup_scene.as_deref() {
        command.arg(format!("--scene={startup_scene}"));
    }

    if profile.cargo_profile == CargoProfile::Dev {
        command.arg("--dev");
    }

    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(AmigoError::Message(format!(
            "external app launch for profile `{}` failed with status `{status}`",
            profile.id
        )))
    }
}

fn release_app_binary_path() -> AmigoResult<std::path::PathBuf> {
    let current_exe = env::current_exe()?;
    let profile_dir = current_exe.parent().ok_or_else(|| {
        AmigoError::Message(format!(
            "could not resolve executable directory from `{}`",
            current_exe.display()
        ))
    })?;
    let target_dir = profile_dir.parent().ok_or_else(|| {
        AmigoError::Message(format!(
            "could not resolve target directory from `{}`",
            profile_dir.display()
        ))
    })?;

    Ok(target_dir.join("release").join(app_binary_name()))
}

fn app_binary_name() -> &'static str {
    if cfg!(windows) {
        "amigo-app.exe"
    } else {
        "amigo-app"
    }
}

fn parse_value_arg(args: &[String], flag: &str) -> Option<String> {
    for argument in args {
        if let Some(value) = argument.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_owned());
        }
    }

    args.windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
}

fn emit_profile_warnings(diagnostics: &diagnostics::ProfileDiagnostics) {
    for warning in diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == diagnostics::DiagnosticSeverity::Warning)
    {
        eprintln!("launcher warning: {}", warning.message);
    }
}
