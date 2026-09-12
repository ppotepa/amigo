use amigo_npr_playground_plugin::{documents::*, state::Settings};
use amigo_render_npr::NprStyleLayers;
use serde_json::json;
use std::{collections::BTreeMap, fs};

fn width(value: f32) -> NprLookPatch {
    NprLookPatch {
        style: BTreeMap::from([("outline_width".into(), json!(value))]),
        ..Default::default()
    }
}

#[test]
fn unified_stack_preserves_interleaved_paint_strokes_and_repeated_generators() {
    let mut layers = NprStyleLayers::default();
    let mut duplicate = layers.layer("contours").unwrap().clone();
    duplicate.id = "fine-contours".into();
    duplicate.opacity = 0.3;
    layers.layers.insert(2, duplicate);
    layers.layers.swap(1, 5);
    let resolved = NprResolvedLook {
        style: Default::default(),
        layers,
    };
    let patch = NprLookPatch::from_resolved(&resolved).unwrap();
    let encoded = serde_yaml::to_string(&patch).unwrap();
    let decoded: NprLookPatch = serde_yaml::from_str(&encoded).unwrap();
    let empty = NprLookPatch::default();
    assert_eq!(
        resolve_look(&BTreeMap::new(), &empty, None, &decoded, &empty, &empty).unwrap(),
        resolved
    );
    assert!(encoded.starts_with("style:") || encoded.starts_with("layers:"));
    assert!(!encoded.contains("\nstroke:"));
    assert!(!encoded.contains("\npaint:"));
}

#[test]
fn shipped_drawing_presets_resolve_to_pinned_brushes_for_all_five_media() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../mods/npr-playground/npr/looks");
    let mut looks = BTreeMap::new();
    for id in ["comic-ink", "pencil-study", "watercolour-wash"] {
        let path = root.join(format!("{id}.npr-look.yml"));
        let document: NprLookDocument =
            serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        looks.insert(id.to_owned(), document);
    }
    let library = builtin_brush_library();
    for id in ["comic-ink", "pencil-study", "watercolour-wash"] {
        let resolved = resolve_look(
            &looks,
            &NprLookPatch::default(),
            Some(id),
            &NprLookPatch::default(),
            &NprLookPatch::default(),
            &NprLookPatch::default(),
        )
        .unwrap();
        let media = resolved
            .layers
            .layers
            .iter()
            .filter_map(|layer| layer.brush.as_ref())
            .map(|instance| library.resolve(&instance.brush).unwrap().medium)
            .collect::<Vec<_>>();
        assert!(media.contains(&amigo_render_npr::BrushMedium::Ink));
        assert!(media.contains(&amigo_render_npr::BrushMedium::Graphite));
        assert!(media.contains(&amigo_render_npr::BrushMedium::Hatching));
        assert!(media.contains(&amigo_render_npr::BrushMedium::FlatFill));
        assert!(media.contains(&amigo_render_npr::BrushMedium::WatercolourWash));
    }
}

#[test]
fn rejected_duplicate_layer_patch_leaves_inherited_state_unchanged() {
    let mut target = width(2.0);
    let before = target.clone();
    let layer = NprLayerDocument {
        layer_id: "contours".into(),
        order: 0,
        parameters: NprStyleLayers::default().layer("contours").unwrap().clone(),
    };
    let mut patch = width(9.0);
    patch.layers = vec![layer.clone(), layer];
    assert!(target.merge(&patch).is_err());
    assert_eq!(target, before);
}

#[test]
fn builtin_brush_library_pins_versions_and_rejects_missing_references() {
    let library = builtin_brush_library();
    let ink = library
        .resolve(&amigo_render_npr::BrushReference {
            id: "ink-liner".into(),
            version: 1,
        })
        .unwrap();
    assert_eq!(ink.medium, amigo_render_npr::BrushMedium::Ink);
    assert!(
        library
            .resolve(&amigo_render_npr::BrushReference {
                id: "ink-liner".into(),
                version: 2
            })
            .is_err()
    );
}

#[test]
fn flat_layer_stack_round_trips_with_independent_mask() {
    let mut layers = NprStyleLayers::default();
    layers.layer_mut("contours").unwrap().mask = amigo_render_npr::CoverageMask::Noise {
        amount: 0.7,
        seed: 19,
        invert: false,
    };
    let resolved = NprResolvedLook {
        style: Default::default(),
        layers,
    };
    let patch = NprLookPatch::from_resolved(&resolved).unwrap();
    let roundtrip = resolve_look(
        &BTreeMap::new(),
        &Default::default(),
        None,
        &patch,
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matches!(
        roundtrip.layers.layer("contours").unwrap().mask,
        amigo_render_npr::CoverageMask::Noise { amount, seed: 19, .. } if (amount - 0.7).abs() < f32::EPSILON
    ));
}

#[test]
fn scene_profile_round_trip_persists_variants_and_pinned_brush_library() {
    let mut profile =
        NprSceneProfileDocument::from_settings(&Settings::empty_scene(), None).unwrap();
    profile.variants.insert(
        "ink-study".into(),
        NprLookPatch::from_resolved(&NprResolvedLook {
            style: Default::default(),
            layers: NprStyleLayers::default(),
        })
        .unwrap(),
    );
    let custom = amigo_render_npr::BrushDefinition {
        id: "ink-liner".into(),
        version: 2,
        name: "Ink custom".into(),
        ..Default::default()
    };
    profile.brushes.add_version(custom).unwrap();
    let encoded = serde_yaml::to_string(&profile).unwrap();
    let decoded: NprSceneProfileDocument = serde_yaml::from_str(&encoded).unwrap();
    assert!(decoded.variants.contains_key("ink-study"));
    assert!(
        decoded
            .brushes
            .resolve(&amigo_render_npr::BrushReference {
                id: "ink-liner".into(),
                version: 2
            })
            .is_ok()
    );
}

fn split_document() -> serde_json::Value {
    let layers = NprLookPatch::from_resolved(&NprResolvedLook {
        style: Default::default(),
        layers: Default::default(),
    })
    .unwrap()
    .layers;
    json!({"id": "example", "look": {
        "style": {"gesture_jitter": 0.2},
        "paint": &layers[..3], "stroke": &layers[3..]
    }})
}

#[test]
fn explicit_migration_preserves_parameters_backs_up_and_is_idempotent() {
    use amigo_npr_playground_plugin::documents::migration::LayerStackMigration;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("example.yml");
    let backup = dir.path().join("original.yml");
    let original = serde_yaml::to_string(&split_document()).unwrap();
    fs::write(&path, &original).unwrap();
    // Runtime refuses the former shape; conversion is an explicit operation.
    assert!(serde_yaml::from_str::<NprLookDocument>(&original).is_err());
    let migration = LayerStackMigration::preview(path.clone()).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    assert_eq!(migration.changes.len(), 1);
    let converted: NprLookDocument = serde_yaml::from_slice(migration.output()).unwrap();
    assert_eq!(converted.look.layers.len(), 8);
    assert_eq!(converted.look.style["gesture_jitter"], json!(0.2));
    for (actual, expected) in converted
        .look
        .layers
        .iter()
        .zip(NprStyleLayers::default().layers)
    {
        assert_eq!(actual.parameters, expected);
    }
    migration.apply(&backup).unwrap();
    assert_eq!(fs::read_to_string(backup).unwrap(), original);
    let saved = fs::read(&path).unwrap();
    let again = LayerStackMigration::preview(path).unwrap();
    assert!(again.changes.is_empty());
    assert_eq!(again.output(), saved);
}

#[test]
fn explicit_migration_replaces_legacy_kind_with_geometry_source() {
    use amigo_npr_playground_plugin::documents::migration::LayerStackMigration;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy-kind.yml");
    let mut value = serde_json::to_value(NprLookDocument {
        id: "legacy-kind".into(),
        includes: vec![],
        look: NprLookPatch::from_resolved(&NprResolvedLook {
            style: Default::default(),
            layers: NprStyleLayers::default(),
        })
        .unwrap(),
    })
    .unwrap();
    let parameters = value["look"]["layers"][0]["parameters"]
        .as_object_mut()
        .unwrap();
    let source = parameters.remove("source").unwrap();
    parameters.insert("kind".into(), source);
    let original = serde_yaml::to_string(&value).unwrap();
    assert!(serde_yaml::from_str::<NprLookDocument>(&original).is_err());
    fs::write(&path, &original).unwrap();

    let migration = LayerStackMigration::preview(path).unwrap();
    assert!(
        migration
            .changes
            .iter()
            .any(|change| change.contains("kind -> source"))
    );
    let migrated: NprLookDocument = serde_yaml::from_slice(migration.output()).unwrap();
    assert_eq!(
        migrated.look.layers[0].parameters.source,
        amigo_render_npr::NprGeometrySource::Paper
    );
}

#[test]
fn migration_rejects_conflicts_mixed_stacks_and_duplicate_ids() {
    use amigo_npr_playground_plugin::documents::migration::LayerStackMigration;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("example.yml");
    let backup = dir.path().join("original.yml");
    let original = serde_yaml::to_string(&split_document()).unwrap();
    fs::write(&path, &original).unwrap();
    let migration = LayerStackMigration::preview(path.clone()).unwrap();
    fs::write(&path, "external edit").unwrap();
    assert!(
        migration
            .apply(&backup)
            .unwrap_err()
            .contains("external change")
    );
    assert!(!backup.exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), "external edit");
    for mixed in [true, false] {
        let mut value = split_document();
        if mixed {
            value["look"]["layers"] = json!([]);
        } else {
            value["look"]["stroke"][0] = value["look"]["paint"][0].clone();
        }
        fs::write(&path, serde_yaml::to_string(&value).unwrap()).unwrap();
        assert!(LayerStackMigration::preview(path.clone()).is_err());
    }
    fs::write(&path, &original).unwrap();
    fs::write(&backup, "existing backup").unwrap();
    assert!(
        LayerStackMigration::preview(path.clone())
            .unwrap()
            .apply(&backup)
            .is_err()
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    assert_eq!(fs::read_to_string(&backup).unwrap(), "existing backup");
}

#[test]
fn migration_visits_object_patches_without_rewriting_paint_medium() {
    use amigo_npr_playground_plugin::documents::migration::LayerStackMigration;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("scene.yml");
    let mut value = serde_json::to_value(
        NprSceneProfileDocument::from_settings(&Settings::empty_scene(), None).unwrap(),
    )
    .unwrap();
    value["object_overrides"] = json!({"cube": {"look": split_document()["look"].clone()}});
    fs::write(&path, serde_yaml::to_string(&value).unwrap()).unwrap();
    let migrated = LayerStackMigration::preview(path).unwrap();
    let scene: NprSceneProfileDocument = serde_yaml::from_slice(migrated.output()).unwrap();
    assert_eq!(
        scene.object_overrides["cube"].look.layers[1]
            .parameters
            .paint,
        NprStyleLayers::default()
            .layer("underpainting")
            .unwrap()
            .paint
    );
}

#[test]
fn depth_first_includes_and_each_precedence_level() {
    let empty = NprLookPatch::default();
    let mut looks = BTreeMap::from([
        (
            "base".into(),
            NprLookDocument {
                id: "base".into(),
                includes: vec![],
                look: width(2.0),
            },
        ),
        (
            "active".into(),
            NprLookDocument {
                id: "active".into(),
                includes: vec!["base".into()],
                look: width(3.0),
            },
        ),
    ]);
    for (scene, object, live, expected) in [
        (&empty, &empty, &empty, 3.0),
        (&width(4.0), &empty, &empty, 4.0),
        (&width(4.0), &width(5.0), &empty, 5.0),
        (&width(4.0), &width(5.0), &width(6.0), 6.0),
    ] {
        assert_eq!(
            resolve_look(&looks, &width(1.0), Some("active"), scene, object, live)
                .unwrap()
                .style
                .outline_width,
            expected
        );
    }
    looks.get_mut("active").unwrap().look = empty.clone();
    assert_eq!(
        resolve_look(&looks, &width(1.0), Some("active"), &empty, &empty, &empty)
            .unwrap()
            .style
            .outline_width,
        2.0
    );
    looks
        .get_mut("base")
        .unwrap()
        .includes
        .push("active".into());
    assert!(
        resolve_look(&looks, &empty, Some("active"), &empty, &empty, &empty)
            .unwrap_err()
            .contains("cycle")
    );
    assert!(
        resolve_look(&looks, &empty, Some("missing"), &empty, &empty, &empty)
            .unwrap_err()
            .contains("missing")
    );
}

#[test]
fn layers_replace_parameters_by_identity_and_use_explicit_order() {
    let empty = NprLookPatch::default();
    let mut layer = NprStyleLayers::default().layer("contours").unwrap().clone();
    layer.opacity = 0.25;
    let patch = NprLookPatch {
        layers: vec![NprLayerDocument {
            layer_id: layer.id.clone(),
            order: -1,
            parameters: layer,
        }],
        ..Default::default()
    };
    let resolved = resolve_look(&BTreeMap::new(), &empty, None, &patch, &empty, &empty).unwrap();
    assert_eq!(resolved.layers.layers[0].id, "contours");
    assert_eq!(resolved.layers.layers[0].opacity, 0.25);
    assert_eq!(
        resolved.layers.layers.len(),
        NprStyleLayers::default().layers.len()
    );
    let standalone = NprLookPatch::from_resolved(&resolved).unwrap();
    assert_eq!(
        resolve_look(&BTreeMap::new(), &empty, None, &standalone, &empty, &empty).unwrap(),
        resolved
    );
}

#[test]
fn sidecar_round_trip_includes_empty_scene_and_camera() {
    let mut profile =
        NprSceneProfileDocument::from_settings(&Settings::empty_scene(), None).unwrap();
    profile.look = width(3.0);
    let encoded = serde_yaml::to_string(&profile).unwrap();
    assert_eq!(
        serde_yaml::from_str::<NprSceneProfileDocument>(&encoded).unwrap(),
        profile
    );
    let resolved = profile
        .resolve(&BTreeMap::new(), &BTreeMap::new(), &NprLookPatch::default())
        .unwrap();
    assert!(resolved.objects.is_empty());
    assert_eq!(resolved.camera_distance, 5.0);
    assert_eq!(resolved.global.outline_width, 3.0);
}

#[test]
fn legacy_multi_model_profile_requires_an_explicit_drawing_studio_migration() {
    let mut profile =
        NprSceneProfileDocument::from_settings(&Settings::empty_scene(), None).unwrap();
    profile.render_all_objects = true;
    assert!(
        profile
            .resolve(&BTreeMap::new(), &BTreeMap::new(), &NprLookPatch::default())
            .unwrap_err()
            .contains("migrate")
    );
}

#[test]
fn atomic_save_conflict_reload_and_save_as_do_not_clobber_external_changes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("look.npr-look.yml");
    let mut doc = NprTrackedDocument::new(path.clone(), b"initial".to_vec());
    doc.save().unwrap();
    doc.stage(&width(4.0)).unwrap();
    fs::write(&path, b"external").unwrap();
    assert!(doc.save().unwrap_err().contains("Reload or Save As"));
    assert_eq!(fs::read(&path).unwrap(), b"external");
    let mut copy =
        NprTrackedDocument::new(temp.path().join("copy.npr-look.yml"), doc.pending.clone());
    copy.save().unwrap();
    doc.reload().unwrap();
    assert!(!doc.dirty);
    assert_eq!(doc.pending, b"external");
    assert!(authored_path(temp.path(), std::path::Path::new("../escape")).is_err());
    assert!(validate_document_id("CON").is_err());
}

#[test]
fn save_all_preflights_conflicts_before_writing_any_document() {
    let temp = tempfile::tempdir().unwrap();
    let mut docs = [
        NprTrackedDocument::new(temp.path().join("one"), b"one".to_vec()),
        NprTrackedDocument::new(temp.path().join("two"), b"two".to_vec()),
    ];
    fs::write(&docs[1].path, b"external").unwrap();
    assert!(save_all(&mut docs).is_err());
    assert!(!docs[0].path.exists());
}

#[test]
fn drawing_draft_requires_its_selected_source_model() {
    let mut draft = NprDrawingDraft {
        version: 1,
        source_model: "sphere".into(),
        settings: Settings::for_scene(),
    };
    assert!(draft
        .validate()
        .unwrap_err()
        .contains("source model does not match"));

    draft.source_model = "cube".into();
    draft.validate().unwrap();
    draft.version = 2;
    assert!(draft.validate().unwrap_err().contains("unsupported"));
}

#[test]
fn explicit_profile_migration_requires_a_source_model_and_keeps_a_durable_backup() {
    use amigo_npr_playground_plugin::documents::migration::DrawingStudioProfileMigration;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.npr-scene.yml");
    let backup = directory.path().join("legacy.backup.yml");
    let mut settings = Settings::for_scene();
    let mut sphere = settings.objects["cube"].clone();
    sphere.model = "sphere".into();
    let mut profile = NprSceneProfileDocument::from_settings(&settings, None).unwrap();
    profile.objects.insert("sphere".into(), sphere);
    profile.render_all_objects = true;
    let original = serde_yaml::to_string(&profile).unwrap();
    fs::write(&path, &original).unwrap();

    assert!(DrawingStudioProfileMigration::preview(path.clone(), "missing").is_err());
    let migration = DrawingStudioProfileMigration::preview(path.clone(), "sphere").unwrap();
    assert!(migration.removed_models.contains(&"cube".into()));
    let converted: NprSceneProfileDocument = serde_yaml::from_slice(migration.output()).unwrap();
    assert!(!converted.render_all_objects);
    assert_eq!(converted.objects.len(), 1);
    assert!(converted.objects.contains_key("sphere"));
    migration.apply(&backup).unwrap();
    assert_eq!(fs::read_to_string(&backup).unwrap(), original);
    let saved: NprSceneProfileDocument = serde_yaml::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert!(saved
        .resolve(&BTreeMap::new(), &BTreeMap::new(), &NprLookPatch::default())
        .is_ok());
}

#[test]
fn save_all_prepares_every_payload_before_replacing_any_document() {
    let temp = tempfile::tempdir().unwrap();
    let blocked_parent = temp.path().join("not-a-directory");
    fs::write(&blocked_parent, b"file").unwrap();
    let mut docs = [
        NprTrackedDocument::new(temp.path().join("first"), b"first".to_vec()),
        NprTrackedDocument::new(blocked_parent.join("second"), b"second".to_vec()),
    ];
    assert!(save_all(&mut docs).is_err());
    assert!(
        !docs[0].path.exists(),
        "a later prepare failure must not commit the first document"
    );
}

#[test]
fn drawing_studio_profile_rejects_multiple_sources_even_without_legacy_flag() {
    let settings = Settings::for_scene();
    let mut profile = NprSceneProfileDocument::from_settings(&settings, None).unwrap();
    let mut sphere = profile.objects["cube"].clone();
    sphere.model = "sphere".into();
    profile.objects.insert("sphere".into(), sphere);

    assert!(profile
        .resolve(&BTreeMap::new(), &BTreeMap::new(), &NprLookPatch::default())
        .unwrap_err()
        .contains("only one source model"));
}
