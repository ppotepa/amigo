mod tests {
    use super::*;

    fn discovered_mod(id: &str, dependencies: &[&str]) -> DiscoveredMod {
        DiscoveredMod {
            manifest: ModManifest {
                id: id.to_owned(),
                name: id.to_owned(),
                version: "0.1.0".to_owned(),
                description: None,
                authors: Vec::new(),
                dependencies: dependencies
                    .iter()
                    .map(|dependency| (*dependency).to_owned())
                    .collect(),
                capabilities: Vec::new(),
                scripting: None,
                scenes: Vec::new(),
            },
            root_path: PathBuf::from(format!("mods/{id}")),
        }
    }

    #[test]
    fn deserializes_scripting_section_and_scene_paths() {
        let manifest = toml::from_str::<ModManifest>(
            r#"
                id = "playground-2d"
                name = "Playground 2D"
                version = "0.1.0"

                [scripting]
                mod_script = "scripts/mod.rhai"
                mod_script_mode = "persistent"

                [[scenes]]
                id = "basic-scripting-demo"
                label = "Basic Scripting Demo"
                path = "scenes/basic-scripting-demo"
                launcher_visible = true
            "#,
        )
        .expect("manifest should deserialize");

        assert_eq!(
            manifest.scripting,
            Some(ModScriptingManifest {
                mod_script: Some("scripts/mod.rhai".to_owned()),
                mod_script_mode: ModScriptMode::Persistent,
            })
        );
        assert_eq!(manifest.scenes[0].path, "scenes/basic-scripting-demo");
        assert_eq!(manifest.scenes[0].document, None);
        assert_eq!(manifest.scenes[0].script, None);
    }

    #[test]
    fn resolves_scene_folder_paths_from_canonical_scene_root() {
        let discovered_mod = DiscoveredMod {
            manifest: ModManifest {
                id: "playground-2d".to_owned(),
                name: "Playground 2D".to_owned(),
                version: "0.1.0".to_owned(),
                description: None,
                authors: Vec::new(),
                dependencies: vec!["core".to_owned()],
                capabilities: vec!["rendering_2d".to_owned()],
                scripting: Some(ModScriptingManifest {
                    mod_script: Some("scripts/mod.rhai".to_owned()),
                    mod_script_mode: ModScriptMode::Persistent,
                }),
                scenes: vec![ModSceneManifest {
                    id: "basic-scripting-demo".to_owned(),
                    label: "Basic Scripting Demo".to_owned(),
                    description: None,
                    path: "scenes/basic-scripting-demo".to_owned(),
                    document: None,
                    script: None,
                    launcher_visible: true,
                }],
            },
            root_path: PathBuf::from("mods/playground-2d"),
        };

        assert_eq!(
            discovered_mod.mod_script_path(),
            Some(PathBuf::from("mods/playground-2d/scripts/mod.rhai"))
        );
        assert_eq!(
            discovered_mod.scene_root_path("basic-scripting-demo"),
            Some(PathBuf::from(
                "mods/playground-2d/scenes/basic-scripting-demo"
            ))
        );
        assert_eq!(
            discovered_mod.scene_document_path("basic-scripting-demo"),
            Some(PathBuf::from(
                "mods/playground-2d/scenes/basic-scripting-demo/scene.yml"
            ))
        );
        assert_eq!(
            discovered_mod.scene_script_path("basic-scripting-demo"),
            Some(PathBuf::from(
                "mods/playground-2d/scenes/basic-scripting-demo/scene.rhai"
            ))
        );
    }

    #[test]
    fn resolves_scene_level_document_and_script_overrides_relative_to_scene_root() {
        let scene = ModSceneManifest {
            id: "hello-world-square".to_owned(),
            label: "Hello World Square".to_owned(),
            description: None,
            path: "scenes/hello-world-square".to_owned(),
            document: Some("custom-scene.yml".to_owned()),
            script: Some("logic/scene.rhai".to_owned()),
            launcher_visible: true,
        };
        let mod_root = Path::new("mods/playground-2d");

        assert_eq!(
            scene.root_path(mod_root),
            PathBuf::from("mods/playground-2d/scenes/hello-world-square")
        );
        assert_eq!(
            scene.document_path(mod_root),
            PathBuf::from("mods/playground-2d/scenes/hello-world-square/custom-scene.yml")
        );
        assert_eq!(
            scene.script_path(mod_root),
            PathBuf::from("mods/playground-2d/scenes/hello-world-square/logic/scene.rhai")
        );
    }

    #[test]
    fn resolves_selected_mods_with_dependencies_first() {
        let discovered = BTreeMap::from([
            ("core".to_owned(), discovered_mod("core", &[])),
            (
                "playground-2d".to_owned(),
                discovered_mod("playground-2d", &["core"]),
            ),
            (
                "playground-3d".to_owned(),
                discovered_mod("playground-3d", &["core"]),
            ),
        ]);

        let resolved = resolve_discovered_mods(&discovered, &[String::from("playground-2d")])
            .expect("dependency resolution should succeed");
        let resolved_ids = resolved
            .iter()
            .map(|discovered_mod| discovered_mod.manifest.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(resolved_ids, vec!["core", "playground-2d"]);
    }

    #[test]
    fn rejects_missing_dependencies() {
        let discovered = BTreeMap::from([(
            "playground-2d".to_owned(),
            discovered_mod("playground-2d", &["core"]),
        )]);

        let error = resolve_discovered_mods(&discovered, &[String::from("playground-2d")])
            .expect_err("missing dependency should fail");

        assert_eq!(
            error.to_string(),
            "mod `playground-2d` depends on missing mod `core`"
        );
    }

    #[test]
    fn rejects_dependency_cycles() {
        let discovered = BTreeMap::from([
            (
                "core".to_owned(),
                discovered_mod("core", &["playground-2d"]),
            ),
            (
                "playground-2d".to_owned(),
                discovered_mod("playground-2d", &["core"]),
            ),
        ]);

        let error = resolve_discovered_mods(&discovered, &[String::from("core")])
            .expect_err("cyclic dependencies should fail");

        assert_eq!(
            error.to_string(),
            "dependency cycle detected while resolving mod `core`"
        );
    }

    #[test]
    fn explicit_empty_selection_loads_no_mods() {
        let discovered = BTreeMap::from([
            ("core".to_owned(), discovered_mod("core", &[])),
            (
                "playground-2d".to_owned(),
                discovered_mod("playground-2d", &["core"]),
            ),
        ]);

        let resolved = resolve_discovered_mods(&discovered, &[])
            .expect("empty explicit selection should succeed");

        assert!(resolved.is_empty());
    }

    #[test]
    fn root_mod_selection_always_includes_core() {
        assert_eq!(requested_mods_for_root("core"), vec!["core".to_owned()]);
        assert_eq!(
            requested_mods_for_root("playground-2d"),
            vec!["core".to_owned(), "playground-2d".to_owned()]
        );
    }
}
