use super::*;

#[test]
fn script_can_import_standard_action_package() {
    let input = Arc::new(InputState::default());
    input.set_key(KeyCode::W, true);

    let actions = Arc::new(InputActionService::default());
    actions.register_map(
        InputActionMap {
            id: "gameplay".to_owned(),
            actions: BTreeMap::from([(
                InputActionId::new("actor.thrust"),
                InputActionBinding::Axis {
                    positive: vec![KeyCode::W],
                    negative: vec![KeyCode::S],
                },
            )]),
        },
        true,
    );

    let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(input),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(actions),
        None,
    );

    runtime
        .execute(
            "package-import-test",
            r#"
                    import "pkg:amigo.std/input" as input;

                    if input::axis(world, "actor.thrust") != 1.0 {
                        throw("standard input package should read action axis");
                    }
                "#,
        )
        .expect("script should import standard package");
}

#[test]
fn imported_package_alias_is_available_inside_lifecycle_callback() {
    let input = Arc::new(InputState::default());
    input.set_key(KeyCode::W, true);

    let actions = Arc::new(InputActionService::default());
    actions.register_map(
        InputActionMap {
            id: "gameplay".to_owned(),
            actions: BTreeMap::from([(
                InputActionId::new("actor.thrust"),
                InputActionBinding::Axis {
                    positive: vec![KeyCode::W],
                    negative: vec![KeyCode::S],
                },
            )]),
        },
        true,
    );

    let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(input),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(actions),
        None,
    );

    runtime
        .execute(
            "package-lifecycle-scope-test",
            r#"
                    import "pkg:amigo.std/input" as input;

                    fn update(dt) {
                        input::axis(world, "actor.thrust");
                    }
                "#,
        )
        .expect("script should compile and execute top-level import");

    runtime
        .call_update("package-lifecycle-scope-test", 1.0 / 60.0)
        .expect("imported package alias should be available in lifecycle callbacks");
}
