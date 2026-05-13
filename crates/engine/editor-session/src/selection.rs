#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected_entities: Vec<String>,
}

impl SelectionState {
    pub fn clear(&mut self) {
        self.selected_entities.clear();
    }
}

