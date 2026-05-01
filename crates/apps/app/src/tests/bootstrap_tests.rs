use super::*;

#[test]
fn bootstrap_reports_task_003_scaffold_plugins_and_capabilities() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned()])
            .with_startup_mod("core")
            .with_startup_scene("bootstrap")
            .with_dev_mode(true),
    )
    .expect("core bootstrap should succeed");

    for capability in [
        "vector_2d",
        "physics_2d",
        "tilemap_2d",
        "motion_2d",
        "audio_api",
        "generated_audio",
        "audio_mix",
        "audio_output",
    ] {
        assert!(
            summary.capabilities.iter().any(|entry| entry == capability),
            "missing capability `{capability}` in bootstrap summary"
        );
    }

    for plugin in [
        "amigo-2d-vector",
        "amigo-2d-physics",
        "amigo-2d-tilemap",
        amigo_2d_motion::CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL,
        "amigo-audio-api",
        "amigo-audio-generated",
        "amigo-audio-mixer",
        "amigo-audio-output",
    ] {
        assert!(
            summary.plugins.iter().any(|entry| entry == plugin),
            "missing plugin `{plugin}` in bootstrap summary"
        );
    }
}

#[test]
fn hosted_playground_mods_use_interactive_handler_even_without_dev_flag() {
    let core_options = BootstrapOptions::new(mods_root())
        .with_startup_mod("core")
        .with_startup_scene("bootstrap");
    let playground_options = BootstrapOptions::new(mods_root())
        .with_startup_mod("playground-3d")
        .with_startup_scene("hello-world-cube");

    assert!(!crate::bootstrap::should_use_interactive_host(
        &core_options
    ));
    assert!(crate::bootstrap::should_use_interactive_host(
        &playground_options
    ));
}

#[test]
fn particle_preset_catalog_files_are_valid() {
    fn string_field<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a str> {
        value
            .as_mapping()
            .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
            .and_then(serde_yaml::Value::as_str)
    }

    fn mapping_field<'a>(
        value: &'a serde_yaml::Value,
        key: &str,
    ) -> Option<&'a serde_yaml::Mapping> {
        value
            .as_mapping()
            .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
            .and_then(serde_yaml::Value::as_mapping)
    }

    let preset_dir = mods_root().join("playground-2d-particles").join("presets");
    let expected_ids = [
        "fire",
        "smoke",
        "sparks",
        "magic",
        "snow",
        "dust",
        "engine_plume",
        "plasma",
        "portal",
        "rain",
        "explosion",
        "embers",
        "steam",
        "lightning",
        "healing",
        "poison",
        "starfield",
        "fountain",
        "shockwave",
        "aurora",
        "bubbles",
        "fireflies",
        "frost",
        "lava_sparks",
        "muzzle_flash",
        "pollen",
        "sandstorm",
        "spiral",
        "waterfall",
        "welding",
    ];
    let mut seen_ids = Vec::new();

    for entry in fs::read_dir(&preset_dir).expect("preset dir should exist") {
        let path = entry.expect("preset dir entry should be readable").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("preset file should be readable");
        let document: serde_yaml::Value =
            serde_yaml::from_str(&contents).expect("preset file should parse as YAML");
        assert_eq!(
            string_field(&document, "kind"),
            Some("particle-preset-2d"),
            "preset {:?} should declare kind",
            path
        );
        let id = string_field(&document, "id").expect("preset should declare id");
        assert!(
            string_field(&document, "label").is_some(),
            "preset `{id}` should declare label"
        );
        let emitter = mapping_field(&document, "emitter")
            .unwrap_or_else(|| panic!("preset `{id}` should declare emitter mapping"));
        assert_eq!(
            emitter
                .get(serde_yaml::Value::String("type".to_owned()))
                .and_then(serde_yaml::Value::as_str),
            Some("ParticleEmitter2D"),
            "preset `{id}` should declare emitter.type"
        );
        seen_ids.push(id.to_owned());
    }

    for expected in expected_ids {
        assert!(
            seen_ids.iter().any(|id| id == expected),
            "preset catalog should include `{expected}`"
        );
    }
}
