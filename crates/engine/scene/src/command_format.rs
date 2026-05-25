use crate::*;

pub fn format_scene_command(command: &SceneCommand) -> String {
    match command {
        SceneCommand::SpawnNamedEntity { name, .. } => format!("scene.spawn({name})"),
        SceneCommand::ConfigureEntity { entity_name, .. } => {
            format!("scene.configure({entity_name})")
        }
        SceneCommand::SelectScene { scene } => format!("scene.select({})", scene.as_str()),
        SceneCommand::Plugin { command } => format!("scene.plugin({})", command.command_type),
        SceneCommand::ReloadActiveScene => "scene.reload_active".to_owned(),
        SceneCommand::ClearEntities => "scene.clear".to_owned(),
        SceneCommand::SetPostFx2dStacks { stacks, .. } => {
            let effects: usize = stacks.iter().map(|stack| stack.effects.len()).sum();
            format!(
                "scene.2d.post_fx_stacks({} stacks, {} effects)",
                stacks.len(),
                effects
            )
        }
        SceneCommand::ConfigureActivationSet { command } => {
            format!(
                "scene.activation_set({}, {} entries)",
                command.id,
                command.entries.len()
            )
        }
        SceneCommand::ActivateSet { id } => format!("scene.activate_set({id})"),
    }
}
