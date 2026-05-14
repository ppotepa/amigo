use crate::{SceneSnapshotError, SceneSnapshotImage, SceneSnapshotRequest};

pub trait SceneSnapshotService {
    fn capture(
        &self,
        request: SceneSnapshotRequest,
    ) -> Result<SceneSnapshotImage, SceneSnapshotError>;
}
