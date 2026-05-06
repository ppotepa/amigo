#[derive(Debug, Clone)]
pub struct EditorTransaction {
    pub id: String,
    pub label: String,
    pub changed_entities: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EditorTransactionLog {
    pub done: Vec<EditorTransaction>,
    pub undone: Vec<EditorTransaction>,
}

impl EditorTransactionLog {
    pub fn push(&mut self, transaction: EditorTransaction) {
        self.done.push(transaction);
        self.undone.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    pub fn undo(&mut self) -> Option<EditorTransaction> {
        let transaction = self.done.pop()?;
        self.undone.push(transaction.clone());
        Some(transaction)
    }

    pub fn redo(&mut self) -> Option<EditorTransaction> {
        let transaction = self.undone.pop()?;
        self.done.push(transaction.clone());
        Some(transaction)
    }
}
