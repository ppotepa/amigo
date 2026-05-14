use super::*;

#[test]
fn scene_helpers_resolve_scene_ids_and_wrap_indices() {
    let mod_catalog = ModCatalog::from_discovered_mods(vec![amigo_modding::DiscoveredMod {
        manifest: amigo_modding::ModManifest {
            id: "playground-2d".to_owned(),
            name: "Playground 2D".to_owned(),
            version: "0.1.0".to_owned(),
            description: None,
            authors: Vec::new(),
            dependencies: vec!["core".to_owned()],
            capabilities: vec!["rendering_2d".to_owned()],
            scripting: None,
            scenes: vec![
                amigo_modding::ModSceneManifest {
                    id: "sprite-lab".to_owned(),
                    label: "Sprite Lab".to_owned(),
                    description: None,
                    path: "scenes/sprite-lab".to_owned(),
                    document: None,
                    script: None,
                    launcher_visible: true,
                },
                amigo_modding::ModSceneManifest {
                    id: "text-lab".to_owned(),
                    label: "Text Lab".to_owned(),
                    description: None,
                    path: "scenes/text-lab".to_owned(),
                    document: None,
                    script: None,
                    launcher_visible: true,
                },
            ],
        },
        root_path: mods_root().join("playground-2d"),
    }]);
    let launch_selection = LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("sprite-lab".to_owned()),
        vec!["core".to_owned(), "playground-2d".to_owned()],
        true,
    );

    let scene_ids = scene_ids_for_launch_selection(&mod_catalog, &launch_selection);

    assert_eq!(
        scene_ids,
        vec!["sprite-lab".to_owned(), "text-lab".to_owned()]
    );
    assert_eq!(
        next_scene_id(&scene_ids, Some("sprite-lab"), 1).as_deref(),
        Some("text-lab")
    );
    assert_eq!(
        next_scene_id(&scene_ids, Some("sprite-lab"), -1).as_deref(),
        Some("text-lab")
    );
}
