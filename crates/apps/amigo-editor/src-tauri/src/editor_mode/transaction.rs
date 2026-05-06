use super::dto::{EditorBounds2Dto, EditorSceneSnapshotDto, EditorTransform2Dto};

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct EditorTransaction {
    pub label: String,
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
            EditorTransactionFragment::PrefabOverride { .. } => {}
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
            EditorTransactionFragment::PrefabOverride { .. } => {}
        }
    }
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
