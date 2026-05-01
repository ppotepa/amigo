#[derive(Debug, Clone, PartialEq)]
pub struct UiDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub document: UiDocument,
}

#[derive(Debug, Default)]
pub struct UiSceneService {
    commands: Mutex<Vec<UiDrawCommand>>,
}

impl UiSceneService {
    pub fn queue(&self, command: UiDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("ui scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("ui scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<UiDrawCommand> {
        self.commands
            .lock()
            .expect("ui scene service mutex should not be poisoned")
            .clone()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}
