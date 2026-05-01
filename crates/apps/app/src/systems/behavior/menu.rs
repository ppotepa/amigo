fn tick_menu_navigation_controller(
    actions: &InputActionService,
    input: &InputState,
    scene_state: &SceneStateService,
    audio: Option<&AudioCommandQueue>,
    script_events: Option<&ScriptEventQueue>,
    config: &amigo_behavior::MenuNavigationControllerBehavior,
) {
    let item_count = config
        .item_count_state
        .as_deref()
        .and_then(|key| scene_state.get_int(key))
        .unwrap_or(config.item_count)
        .max(0);
    if item_count == 0 {
        return;
    }

    let current = scene_state
        .get_int(&config.index_state)
        .unwrap_or(0)
        .clamp(0, item_count - 1);
    let mut next = current;

    if actions.pressed(input, &config.up_action) {
        next -= 1;
        if next < 0 {
            next = if config.wrap { item_count - 1 } else { 0 };
        }
    }

    if actions.pressed(input, &config.down_action) {
        next += 1;
        if next >= item_count {
            next = if config.wrap { 0 } else { item_count - 1 };
        }
    }

    if next != current {
        scene_state.set_int(&config.index_state, next);
        if let (Some(audio), Some(clip)) = (audio, config.move_audio.as_ref()) {
            audio.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(clip.clone()),
            });
        }
    }

    write_menu_selection_state(scene_state, config, next, item_count);

    let Some(confirm_action) = config.confirm_action.as_deref() else {
        return;
    };
    if !actions.pressed(input, confirm_action) {
        return;
    }

    if let (Some(audio), Some(clip)) = (audio, config.confirm_audio.as_ref()) {
        audio.push(AudioCommand::PlayOnce {
            clip: AudioClipKey::new(clip.clone()),
        });
    }

    let Some(topic) = usize::try_from(next)
        .ok()
        .and_then(|index| config.confirm_events.get(index))
    else {
        return;
    };

    if let Some(script_events) = script_events {
        script_events.publish(ScriptEvent::new(topic.clone(), vec![next.to_string()]));
    }
}

