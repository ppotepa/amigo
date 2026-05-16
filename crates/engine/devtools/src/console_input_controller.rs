use amigo_core::AmigoResult;
use amigo_input_api::{InputEvent, InputModifiers, KeyCode};
use amigo_runtime::Runtime;
use amigo_runtime_control::RuntimeControlService;
use amigo_scene::SceneService;
use amigo_scripting_api::{DevConsoleCommand, DevConsoleQueue, DevConsoleState};

use crate::{
    ConsoleCompletionContext, ConsoleCompletionState, ConsoleRhaiSymbol,
    RuntimeConsoleCommandRegistry, collect_console_rhai_symbols_from_source,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevConsoleInputOutcome {
    Ignored,
    Consumed,
}

impl DevConsoleInputOutcome {
    pub fn is_consumed(self) -> bool {
        matches!(self, Self::Consumed)
    }
}

pub struct DevConsoleInputController;

impl DevConsoleInputController {
    pub fn handle_event(
        runtime: &Runtime,
        event: &InputEvent,
        modifiers: &mut InputModifiers,
    ) -> AmigoResult<DevConsoleInputOutcome> {
        let console = runtime.required::<DevConsoleState>()?;
        let completion = runtime.required::<ConsoleCompletionState>()?;
        let registry = runtime.required::<RuntimeConsoleCommandRegistry>()?;

        if let InputEvent::ModifiersChanged(next_modifiers) = event {
            *modifiers = *next_modifiers;
            return Ok(if console.is_open() {
                DevConsoleInputOutcome::Consumed
            } else {
                DevConsoleInputOutcome::Ignored
            });
        }

        if key_pressed(event, KeyCode::Backquote) {
            console.toggle_open();
            if console.is_open() {
                refresh_console_completion(
                    runtime,
                    completion.as_ref(),
                    registry.as_ref(),
                    console.as_ref(),
                );
            } else {
                completion.clear();
            }
            return Ok(DevConsoleInputOutcome::Consumed);
        }

        if key_pressed(event, KeyCode::F1) {
            console.set_open(true);
            refresh_console_completion(
                runtime,
                completion.as_ref(),
                registry.as_ref(),
                console.as_ref(),
            );
            return Ok(DevConsoleInputOutcome::Consumed);
        }

        if key_pressed(event, KeyCode::F2) {
            submit_console_command(runtime, "reload")?;
            return Ok(DevConsoleInputOutcome::Consumed);
        }

        if key_pressed(event, KeyCode::R) && (modifiers.control || modifiers.super_key) {
            submit_console_command(runtime, "reload")?;
            return Ok(DevConsoleInputOutcome::Consumed);
        }

        if key_pressed(event, KeyCode::D) && (modifiers.control || modifiers.super_key) {
            submit_console_command(runtime, "diagnostics")?;
            return Ok(DevConsoleInputOutcome::Consumed);
        }

        if !console.is_open() {
            return Ok(DevConsoleInputOutcome::Ignored);
        }

        match event {
            InputEvent::TextInput { text } => {
                if !modifiers.control && !modifiers.super_key {
                    console.insert_input_text(text);
                    refresh_after_console_edit(
                        runtime,
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
                }
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::MouseWheel { delta_y } => {
                let rows = if *delta_y > 0.0 {
                    3
                } else if *delta_y < 0.0 {
                    -3
                } else {
                    0
                };
                console.scroll_output(rows);
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Backspace,
                pressed: true,
            } => {
                console.backspace_input();
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Delete,
                pressed: true,
            } => {
                console.delete_input();
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Left,
                pressed: true,
            } => {
                console.move_input_left(modifiers.shift, modifiers.control);
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Right,
                pressed: true,
            } => {
                console.move_input_right(modifiers.shift, modifiers.control);
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Home,
                pressed: true,
            } => {
                console.move_input_home(modifiers.shift);
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::End,
                pressed: true,
            } => {
                console.move_input_end(modifiers.shift);
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::A,
                pressed: true,
            } if modifiers.control => {
                console.select_all_input();
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::C,
                pressed: true,
            } if modifiers.control => {
                console.copy_input_selection();
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::X,
                pressed: true,
            } if modifiers.control => {
                console.cut_input_selection();
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::V,
                pressed: true,
            } if modifiers.control => {
                console.paste_input_clipboard();
                refresh_after_console_edit(
                    runtime,
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Tab,
                pressed: true,
            } => {
                refresh_console_completion(
                    runtime,
                    completion.as_ref(),
                    registry.as_ref(),
                    console.as_ref(),
                );
                let snapshot = console.input_snapshot();
                if let Some(edit) = completion.accept_tab(&snapshot.text, snapshot.cursor) {
                    console.set_input_with_cursor(edit.input, edit.cursor_index);
                    refresh_after_console_edit(
                        runtime,
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
                }
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Enter,
                pressed: true,
            } => {
                let snapshot = console.input_snapshot();
                if completion.snapshot().is_some() {
                    if let Some(edit) = completion.accept_tab(&snapshot.text, snapshot.cursor) {
                        console.set_input_with_cursor(edit.input, edit.cursor_index);
                        refresh_after_console_edit(
                            runtime,
                            console.as_ref(),
                            completion.as_ref(),
                            registry.as_ref(),
                        );
                        return Ok(DevConsoleInputOutcome::Consumed);
                    }
                }

                let line = console.input();
                completion.clear();
                console.clear_input();
                if !line.trim().is_empty() {
                    console.reset_output_scroll();
                    submit_console_command(runtime, line)?;
                }
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Escape,
                pressed: true,
            } => {
                if completion.snapshot().is_some() {
                    completion.clear();
                } else {
                    console.set_open(false);
                }
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Up,
                pressed: true,
            } => {
                if completion.select_previous() {
                    return Ok(DevConsoleInputOutcome::Consumed);
                }
                if let Some(previous) = console.history_previous() {
                    console.set_input(previous);
                    refresh_after_console_edit(
                        runtime,
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
                }
                Ok(DevConsoleInputOutcome::Consumed)
            }
            InputEvent::Key {
                key: KeyCode::Down,
                pressed: true,
            } => {
                if completion.select_next() {
                    return Ok(DevConsoleInputOutcome::Consumed);
                }
                if let Some(next) = console.history_next() {
                    console.set_input(next);
                    refresh_after_console_edit(
                        runtime,
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
                }
                Ok(DevConsoleInputOutcome::Consumed)
            }
            _ => Ok(DevConsoleInputOutcome::Consumed),
        }
    }
}

fn key_pressed(event: &InputEvent, expected: KeyCode) -> bool {
    matches!(event, InputEvent::Key { key, pressed: true } if *key == expected)
}

fn submit_console_command(runtime: &Runtime, line: impl Into<String>) -> AmigoResult<()> {
    runtime
        .required::<DevConsoleQueue>()?
        .submit(DevConsoleCommand::new(line.into()));
    Ok(())
}

fn refresh_console_completion(
    runtime: &Runtime,
    completion: &ConsoleCompletionState,
    registry: &RuntimeConsoleCommandRegistry,
    console: &DevConsoleState,
) {
    let snapshot = console.input_snapshot();
    let descriptors = registry.descriptors();
    let schemas = registry.schemas();
    let context = console_completion_context(runtime, console);
    completion.refresh(
        &snapshot.text,
        snapshot.cursor,
        &descriptors,
        &schemas,
        &context,
    );
}

fn console_completion_context(
    runtime: &Runtime,
    console: &DevConsoleState,
) -> ConsoleCompletionContext {
    let entity_names = runtime
        .resolve::<SceneService>()
        .map(|scene| scene.entity_names())
        .unwrap_or_default();

    let postfx_indices = runtime
        .resolve::<amigo_2d_post_fx::PostFx2dService>()
        .map(|postfx| {
            (0..postfx.frame_effect_count())
                .map(|index| index.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let rhai_symbols = console
        .command_history()
        .into_iter()
        .flat_map(|line| collect_console_rhai_symbols_from_source(&line))
        .chain(runtime_rhai_symbols(runtime))
        .collect();

    let render_layer_ids = runtime
        .resolve::<amigo_2d_composition::RenderLayer2dSceneService>()
        .map(|layers| {
            layers
                .commands()
                .into_iter()
                .map(|layer| layer.id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let runtime_control = runtime.resolve::<RuntimeControlService>();

    ConsoleCompletionContext {
        entity_names,
        postfx_kinds: vec![
            "blur".to_owned(),
            "crt".to_owned(),
            "dirty_bloom".to_owned(),
            "rain_glass".to_owned(),
            "lens_droplets".to_owned(),
            "color_quantize".to_owned(),
            "film_noise".to_owned(),
            "shutter_blur".to_owned(),
        ],
        postfx_indices,
        render_layer_ids,
        rhai_symbols,
        runtime_control,
    }
}

fn runtime_rhai_symbols(_runtime: &Runtime) -> Vec<ConsoleRhaiSymbol> {
    Vec::new()
}

fn refresh_after_console_edit(
    runtime: &Runtime,
    console: &DevConsoleState,
    completion: &ConsoleCompletionState,
    registry: &RuntimeConsoleCommandRegistry,
) {
    refresh_console_completion(runtime, completion, registry, console);
}
