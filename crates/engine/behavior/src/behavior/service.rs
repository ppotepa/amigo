
impl BehaviorSceneService {
    pub fn queue(&self, command: BehaviorCommand) {
        let mut behaviors = self
            .behaviors
            .lock()
            .expect("behavior scene service mutex should not be poisoned");
        let base_key = command.entity_name.clone();
        let mut key = base_key.clone();
        let mut suffix = 1;
        while behaviors.contains_key(&key) {
            suffix += 1;
            key = format!("{base_key}#{suffix}");
        }
        behaviors.insert(key, command);
    }

    pub fn behaviors(&self) -> Vec<BehaviorCommand> {
        self.behaviors
            .lock()
            .expect("behavior scene service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.behaviors
            .lock()
            .expect("behavior scene service mutex should not be poisoned")
            .clear();
        self.hold_seconds
            .lock()
            .expect("behavior hold state mutex should not be poisoned")
            .clear();
    }

    pub fn tick_hold_seconds(
        &self,
        key: &str,
        active: bool,
        delta_seconds: f32,
        max_seconds: f32,
    ) -> (f32, f32) {
        let mut holds = self
            .hold_seconds
            .lock()
            .expect("behavior hold state mutex should not be poisoned");
        let previous = holds.get(key).copied().unwrap_or(0.0);
        let next = if active {
            (previous + delta_seconds.max(0.0)).min(max_seconds.max(0.0))
        } else {
            0.0
        };
        holds.insert(key.to_owned(), next);
        (previous, next)
    }
}

