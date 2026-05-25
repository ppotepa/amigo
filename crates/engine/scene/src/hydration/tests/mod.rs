mod behavior;
mod plans;
mod runtime;
mod ui;

fn plugin_payload<T: 'static>(command: &crate::SceneCommand) -> Option<&T> {
    match command {
        crate::SceneCommand::Plugin { command } => command.payload_as::<T>(),
        _ => None,
    }
}
