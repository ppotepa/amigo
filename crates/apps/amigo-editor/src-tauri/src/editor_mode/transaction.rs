use super::dto::{
    EditorBounds2Dto, EditorHistoryDto, EditorHistoryEntryDto, EditorSceneSnapshotDto,
    EditorTransform2Dto, EditorUiNodeSelectionDto,
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EditorTransactionFragment {
    Transform2 {
        entity_id: String,
        before: EditorTransform2Dto,
        after: EditorTransform2Dto,
    },
    PrefabOverride {
        entity_id: String,
        prefab_id: String,
        target: String,
        before: serde_yaml::Value,
        after: serde_yaml::Value,
    },
    SetUiNodeProperty {
        entity_id: String,
        component_index: usize,
        node_path: String,
        property_path: String,
        value: super::dto::EditorUiNodePropertyValueDto,
    },
    UiDocumentValue {
        entity_id: String,
        before: serde_yaml::Value,
        after: serde_yaml::Value,
        selected_before: Option<EditorUiNodeSelectionDto>,
        selected_after: Option<EditorUiNodeSelectionDto>,
    },
}

#[derive(Debug, Clone)]
pub struct EditorTransaction {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub target: String,
    pub revision: u64,
    pub timestamp_ms: u64,
    pub changed_entities: Vec<String>,
    pub fragments: Vec<EditorTransactionFragment>,
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

    pub fn is_dirty(&self) -> bool {
        !self.done.is_empty()
    }

    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
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

    pub fn history_dto(&self, dirty: bool, revision: u64, saved_revision: u64) -> EditorHistoryDto {
        EditorHistoryDto {
            dirty,
            revision,
            saved_revision,
            undo_count: self.done.len(),
            redo_count: self.undone.len(),
            entries: self.done.iter().rev().map(history_entry_dto).collect(),
            redo_entries: self.undone.iter().rev().map(history_entry_dto).collect(),
        }
    }
}

pub fn new_transaction_id(revision: u64, kind: &str) -> String {
    format!("tx-{revision}-{kind}")
}

pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn history_entry_dto(transaction: &EditorTransaction) -> EditorHistoryEntryDto {
    EditorHistoryEntryDto {
        id: transaction.id.clone(),
        label: transaction.label.clone(),
        kind: transaction.kind.clone(),
        target: transaction.target.clone(),
        revision: transaction.revision,
        timestamp_ms: transaction.timestamp_ms,
    }
}

pub fn apply_transaction_before(
    snapshot: &mut EditorSceneSnapshotDto,
    transaction: &EditorTransaction,
) {
    for fragment in transaction.fragments.iter().rev() {
        match fragment {
            EditorTransactionFragment::Transform2 {
                entity_id, before, ..
            } => apply_snapshot_transform_2(snapshot, entity_id, before.clone()),
            EditorTransactionFragment::PrefabOverride { .. }
            | EditorTransactionFragment::SetUiNodeProperty { .. }
            | EditorTransactionFragment::UiDocumentValue { .. } => {}
        }
    }
}

pub fn apply_transaction_after(
    snapshot: &mut EditorSceneSnapshotDto,
    transaction: &EditorTransaction,
) {
    for fragment in &transaction.fragments {
        match fragment {
            EditorTransactionFragment::Transform2 {
                entity_id, after, ..
            } => apply_snapshot_transform_2(snapshot, entity_id, after.clone()),
            EditorTransactionFragment::PrefabOverride { .. }
            | EditorTransactionFragment::SetUiNodeProperty { .. }
            | EditorTransactionFragment::UiDocumentValue { .. } => {}
        }
    }
}

pub fn apply_transaction_before_to_document(
    document: &mut serde_yaml::Value,
    transaction: &EditorTransaction,
) -> Option<Option<EditorUiNodeSelectionDto>> {
    let mut selected = None;
    for fragment in transaction.fragments.iter().rev() {
        if let EditorTransactionFragment::UiDocumentValue {
            before,
            selected_before,
            ..
        } = fragment
        {
            *document = before.clone();
            selected = Some(selected_before.clone());
        }
    }
    selected
}

pub fn apply_transaction_after_to_document(
    document: &mut serde_yaml::Value,
    transaction: &EditorTransaction,
) -> Option<Option<EditorUiNodeSelectionDto>> {
    let mut selected = None;
    for fragment in &transaction.fragments {
        if let EditorTransactionFragment::UiDocumentValue {
            after,
            selected_after,
            ..
        } = fragment
        {
            *document = after.clone();
            selected = Some(selected_after.clone());
        }
    }
    selected
}

pub fn apply_snapshot_transform_2(
    snapshot: &mut EditorSceneSnapshotDto,
    entity_id: &str,
    next_transform: EditorTransform2Dto,
) {
    let Some(object) = snapshot
        .objects
        .iter_mut()
        .find(|object| object.entity_id == entity_id)
    else {
        return;
    };

    let (dx, dy) = object
        .transform_2
        .as_ref()
        .map(|current| (next_transform.x - current.x, next_transform.y - current.y))
        .unwrap_or((0.0, 0.0));

    object.transform_2 = Some(next_transform);
    translate_bounds(&mut object.bounds_2, dx, dy);
    translate_bounds(&mut object.render_bounds_2, dx, dy);
    translate_bounds(&mut object.selection_bounds_2, dx, dy);
}

fn translate_bounds(bounds: &mut Option<EditorBounds2Dto>, dx: f32, dy: f32) {
    if let Some(bounds) = bounds {
        bounds.x += dx;
        bounds.y += dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_dto_lists_done_and_redo_entries() {
        let mut log = EditorTransactionLog::default();

        log.push(EditorTransaction {
            id: "tx-1".to_owned(),
            label: "Set text".to_owned(),
            kind: "set-ui-node-property".to_owned(),
            target: "main-menu-ui:0:root.start".to_owned(),
            revision: 2,
            timestamp_ms: 100,
            changed_entities: vec!["main-menu-ui".to_owned()],
            fragments: Vec::new(),
        });

        let undone = log.undo().unwrap();
        assert_eq!(undone.label, "Set text");

        let dto = log.history_dto(true, 3, 1);
        assert!(dto.dirty);
        assert_eq!(dto.revision, 3);
        assert_eq!(dto.saved_revision, 1);
        assert_eq!(dto.undo_count, 0);
        assert_eq!(dto.redo_count, 1);
        assert_eq!(dto.redo_entries[0].label, "Set text");
    }
}
