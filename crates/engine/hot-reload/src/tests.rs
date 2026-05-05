mod tests {
    use super::{AssetWatch, HotReloadPlugin, HotReloadService, SceneDocumentWatch};
    use amigo_runtime::RuntimeBuilder;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn tracks_asset_file_changes_without_resetting_existing_stamps() {
        let root = temp_test_dir("asset-change");
        let path = root.join("sprite-lab");
        fs::write(&path, "kind = \"sprite-2d\"\n").expect("asset file should be written");

        let service = HotReloadService::default();
        service.sync_assets(vec![AssetWatch {
            asset_key: "playground-2d/spritesheets/sprite-lab".to_owned(),
            path: path.clone(),
        }]);

        assert!(service.poll_changes().is_empty());

        fs::write(
            &path,
            "kind = \"sprite-2d\"\nlabel = \"Reloaded\"\nformat = \"debug-placeholder\"\n",
        )
        .expect("asset file should be updated");

        let changes = service.poll_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes[0].watch.id,
            "asset:playground-2d/spritesheets/sprite-lab"
        );
    }

    #[test]
    fn sync_scene_document_replaces_previous_active_scene_watch() {
        let root = temp_test_dir("scene-watch");
        let first = root.join("sprite-lab.yml");
        let second = root.join("text-lab.yml");
        fs::write(&first, "version: 1\n").expect("scene file should exist");
        fs::write(&second, "version: 1\n").expect("scene file should exist");

        let service = HotReloadService::default();
        service.sync_scene_document(Some(SceneDocumentWatch {
            source_mod: "playground-2d".to_owned(),
            scene_id: "sprite-lab".to_owned(),
            path: first,
        }));
        service.sync_scene_document(Some(SceneDocumentWatch {
            source_mod: "playground-2d".to_owned(),
            scene_id: "text-lab".to_owned(),
            path: second.clone(),
        }));

        let watched = service.watched_targets();
        assert_eq!(watched.len(), 1);
        assert_eq!(watched[0].id, "scene:playground-2d:text-lab");
        assert_eq!(watched[0].path, second);
    }

    #[test]
    fn maps_explicit_paths_back_to_watch_descriptors() {
        let root = temp_test_dir("path-map");
        let path = root.join("sprite-lab");
        fs::write(&path, "kind = \"sprite-2d\"\n").expect("asset file should be written");

        let service = HotReloadService::default();
        service.sync_assets(vec![AssetWatch {
            asset_key: "playground-2d/spritesheets/sprite-lab".to_owned(),
            path: path.clone(),
        }]);

        let changes = service.changes_for_paths(std::slice::from_ref(&path));

        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes[0].watch.id,
            "asset:playground-2d/spritesheets/sprite-lab"
        );
    }

    #[test]
    fn plugin_registers_hot_reload_service() {
        let runtime = RuntimeBuilder::default()
            .with_plugin(HotReloadPlugin)
            .expect("plugin should register")
            .build();

        assert!(runtime.resolve::<HotReloadService>().is_some());
        assert_eq!(
            runtime.report().plugin_names,
            vec!["amigo-hot-reload".to_owned()]
        );
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("amigo-hot-reload-{label}-{unique}"));
        fs::create_dir_all(&path).expect("temp test dir should be created");
        path
    }
}
