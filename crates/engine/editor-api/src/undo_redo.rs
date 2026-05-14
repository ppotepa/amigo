#[derive(Debug, Clone)]
pub struct UndoRedoEntry {
    pub label: String,
}

#[derive(Debug, Default)]
pub struct UndoRedoStack {
    undo: Vec<UndoRedoEntry>,
    redo: Vec<UndoRedoEntry>,
}

impl UndoRedoStack {
    pub fn push(&mut self, entry: UndoRedoEntry) {
        self.undo.push(entry);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
