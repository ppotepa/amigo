    
    use std::path::PathBuf;

    use crate::{
        HydratedSceneSnapshot, HydratedSceneState,
    };
    
    

    #[test]
    fn hydrated_scene_state_replaces_snapshots() {
        let hydrated = HydratedSceneState::default();
        let previous = hydrated.replace(HydratedSceneSnapshot {
            source_mod: Some("playground-2d".to_owned()),
            scene_id: Some("sprite-lab".to_owned()),
            relative_document_path: Some(PathBuf::from("scenes/sprite-lab/scene.yml")),
            entity_names: vec![
                "playground-2d-camera".to_owned(),
                "playground-2d-sprite".to_owned(),
            ],
            component_kinds: vec!["Camera2D x1".to_owned(), "Sprite2D x1".to_owned()],
        });

        assert_eq!(previous, HydratedSceneSnapshot::default());
        assert_eq!(hydrated.snapshot().scene_id.as_deref(), Some("sprite-lab"));
        assert_eq!(hydrated.clear().entity_names.len(), 2);
        assert_eq!(hydrated.snapshot(), HydratedSceneSnapshot::default());
    }

