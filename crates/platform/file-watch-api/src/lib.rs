use std::path::PathBuf;
use std::sync::Arc;

use amigo_core::AmigoResult;

#[derive(Debug, Clone)]
pub struct FileWatchBackendInfo {
    pub backend_name: &'static str,
    pub automatic_notifications: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileWatchEventKind {
    Changed,
    Removed,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileWatchEvent {
    pub path: PathBuf,
    pub kind: FileWatchEventKind,
}

pub trait FileWatchBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn automatic_notifications(&self) -> bool;
    fn sync_paths(&self, paths: &[PathBuf]) -> AmigoResult<()>;
    fn drain_events(&self) -> Vec<FileWatchEvent>;
}

#[derive(Clone)]
pub struct FileWatchService {
    backend: Arc<dyn FileWatchBackend>,
}

impl FileWatchService {
    pub fn new<T>(backend: T) -> Self
    where
        T: FileWatchBackend + 'static,
    {
        Self {
            backend: Arc::new(backend),
        }
    }

    pub fn backend_name(&self) -> &'static str {
        self.backend.backend_name()
    }

    pub fn automatic_notifications(&self) -> bool {
        self.backend.automatic_notifications()
    }

    pub fn sync_paths(&self, paths: &[PathBuf]) -> AmigoResult<()> {
        self.backend.sync_paths(paths)
    }

    pub fn drain_events(&self) -> Vec<FileWatchEvent> {
        self.backend.drain_events()
    }
}

#[cfg(test)]
mod tests {
    use super::{FileWatchBackend, FileWatchEvent, FileWatchEventKind, FileWatchService};
    use amigo_core::AmigoResult;
    use std::path::PathBuf;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeWatchBackend {
        synced: Mutex<Vec<PathBuf>>,
        events: Mutex<Vec<FileWatchEvent>>,
    }

    impl FileWatchBackend for FakeWatchBackend {
        fn backend_name(&self) -> &'static str {
            "fake-watch"
        }

        fn automatic_notifications(&self) -> bool {
            true
        }

        fn sync_paths(&self, paths: &[PathBuf]) -> AmigoResult<()> {
            let mut synced = self
                .synced
                .lock()
                .expect("sync mutex should not be poisoned");
            synced.clear();
            synced.extend_from_slice(paths);
            Ok(())
        }

        fn drain_events(&self) -> Vec<FileWatchEvent> {
            let mut events = self
                .events
                .lock()
                .expect("events mutex should not be poisoned");
            events.drain(..).collect()
        }
    }

    #[test]
    fn service_wraps_backend_contract() {
        let backend = FakeWatchBackend::default();
        {
            let mut events = backend
                .events
                .lock()
                .expect("events mutex should not be poisoned");
            events.push(FileWatchEvent {
                path: PathBuf::from("mods/playground-2d/scenes/sprite-lab.yml"),
                kind: FileWatchEventKind::Changed,
            });
        }
        let service = FileWatchService::new(backend);

        service
            .sync_paths(&[PathBuf::from("mods/playground-2d/scenes/sprite-lab.yml")])
            .expect("sync should succeed");

        assert_eq!(service.backend_name(), "fake-watch");
        assert!(service.automatic_notifications());
        assert_eq!(service.drain_events().len(), 1);
    }
}
