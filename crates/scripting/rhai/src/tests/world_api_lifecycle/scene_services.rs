use super::*;

#[test]
fn script_component_lifecycle_receives_entity_params_and_delta() {
    let console_queue = Arc::new(DevConsoleQueue::default());
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(console_queue.clone()),
    );
    let params = BTreeMap::from([
        ("amplitude".to_owned(), ScriptValue::Float(12.0)),
        ("speed".to_owned(), ScriptValue::Int(2)),
        ("label".to_owned(), ScriptValue::String("bob".to_owned())),
        ("enabled".to_owned(), ScriptValue::Bool(true)),
    ]);

    runtime
        .execute(
            "component-test",
            r#"
                fn on_attach(entity, params) {
                    if entity != "actor" { throw("wrong attach entity"); }
                    if params.amplitude != 12.0 { throw("wrong amplitude"); }
                    if params.speed != 2 { throw("wrong speed"); }
                    if params.label != "bob" { throw("wrong label"); }
                    if !params.enabled { throw("wrong enabled"); }
                    world.dev.command("attach");
                }

                fn update(entity, params, dt) {
                    if entity != "actor" { throw("wrong update entity"); }
                    if dt != 0.25 { throw("wrong dt"); }
                    if world.time.delta() != 0.25 { throw("wrong time delta"); }
                    world.dev.command("update");
                }

                fn on_detach(entity, params) {
                    if entity != "actor" { throw("wrong detach entity"); }
                    world.dev.command("detach");
                }
            "#,
        )
        .expect("script execution should succeed");

    runtime
        .call_component_on_attach("component-test", "actor", &params)
        .expect("on_attach should succeed");
    runtime
        .call_component_update("component-test", "actor", &params, 0.25)
        .expect("component update should succeed");
    runtime
        .call_component_on_detach("component-test", "actor", &params)
        .expect("on_detach should succeed");

    let commands = console_queue.pending();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].line, "attach");
    assert_eq!(commands[1].line, "update");
    assert_eq!(commands[2].line, "detach");
}

#[test]
fn script_can_write_trace_entries() {
    let trace = Arc::new(ScriptTraceService::default());
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
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(trace.clone()),
    );

    runtime
        .execute(
            "trace-test",
            r#"
                if !world.trace.begin("drive_actor") { throw("trace begin failed"); }
                world.trace.value("thrust", 1.0);
                world.trace.value("turn", -1);
                world.trace.value("armed", true);
                if !world.trace.end() { throw("trace end failed"); }
            "#,
        )
        .expect("script execution should succeed");

    let entries = trace.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].label, "drive_actor");
    assert_eq!(
        entries[0].values,
        vec![
            ("thrust".to_owned(), "1".to_owned()),
            ("turn".to_owned(), "-1".to_owned()),
            ("armed".to_owned(), "true".to_owned()),
        ]
    );
}
