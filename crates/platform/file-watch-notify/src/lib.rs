use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};

use amigo_core::{AmigoError, AmigoResult};
use amigo_file_watch_api::{
    FileWatchBackend, FileWatchBackendInfo, FileWatchEvent, FileWatchEventKind, FileWatchService,
};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Debug)]
struct NotifyFileWatchState {
    watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    watched_paths: BTreeMap<PathBuf, PathBuf>,
}

#[derive(Debug)]
pub struct NotifyFileWatchBackend {
    state: Mutex<NotifyFileWatchState>,
}

impl NotifyFileWatchBackend {
    pub fn new() -> AmigoResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let watcher = RecommendedWatcher::new(
            move |result| {
                let _ = sender.send(result);
            },
            Config::default(),
        )
        .map_err(|error| {
            AmigoError::Message(format!("failed to create notify watcher: {error}"))
        })?;

        Ok(Self {
            state: Mutex::new(NotifyFileWatchState {
                watcher,
                receiver,
                watched_paths: BTreeMap::new(),
            }),
        })
    }
}

impl FileWatchBackend for NotifyFileWatchBackend {
    fn backend_name(&self) -> &'static str {
        "notify"
    }

    fn automatic_notifications(&self) -> bool {
        true
    }

    fn sync_paths(&self, paths: &[PathBuf]) -> AmigoResult<()> {
        let mut state = self
            .state
            .lock()
            .expect("notify file watch mutex should not be poisoned");
        let desired = paths
            .iter()
            .map(|path| normalize_watch_path(path).map(|normalized| (normalized, path.clone())))
            .collect::<AmigoResult<BTreeMap<_, _>>>()?;

        let existing = state.watched_paths.keys().cloned().collect::<Vec<_>>();
        for existing_path in existing {
            if !desired.contains_key(&existing_path) {
                state.watcher.unwatch(&existing_path).map_err(|error| {
                    AmigoError::Message(format!(
                        "failed to stop watching `{}`: {error}",
                        existing_path.display()
                    ))
                })?;
                state.watched_paths.remove(&existing_path);
            }
        }

        for (normalized, requested) in desired {
            if !state.watched_paths.contains_key(&normalized) {
                state
                    .watcher
                    .watch(&normalized, RecursiveMode::NonRecursive)
                    .map_err(|error| {
                        AmigoError::Message(format!(
                            "failed to watch `{}`: {error}",
                            normalized.display()
                        ))
                    })?;
            }
            state.watched_paths.insert(normalized, requested);
        }

        Ok(())
    }

    fn drain_events(&self) -> Vec<FileWatchEvent> {
        let state = self
            .state
            .lock()
            .expect("notify file watch mutex should not be poisoned");
        let mut seen = BTreeSet::new();
        let mut events = Vec::new();

        while let Ok(result) = state.receiver.try_recv() {
            let Ok(event) = result else {
                continue;
            };
            let kind = map_event_kind(&event.kind);
            for path in event.paths {
                let Ok(normalized) = normalize_watch_path(&path) else {
                    continue;
                };
                let Some(requested_path) = state.watched_paths.get(&normalized) else {
                    continue;
                };
                if seen.insert((requested_path.clone(), kind)) {
                    events.push(FileWatchEvent {
                        path: requested_path.clone(),
                        kind,
                    });
                }
            }
        }

        events
    }
}

fn normalize_watch_path(path: &Path) -> AmigoResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| {
                AmigoError::Message(format!(
                    "failed to resolve current directory for watch path `{}`: {error}",
                    path.display()
                ))
            })?
            .join(path)
    };

    absolute.canonicalize().map_err(|error| {
        AmigoError::Message(format!(
            "failed to canonicalize watch path `{}`: {error}",
            absolute.display()
        ))
    })
}

fn map_event_kind(kind: &EventKind) -> FileWatchEventKind {
    match kind {
        EventKind::Create(_) | EventKind::Modify(_) => FileWatchEventKind::Changed,
        EventKind::Remove(_) => FileWatchEventKind::Removed,
        _ => FileWatchEventKind::Other,
    }
}

pub struct NotifyFileWatchPlugin;

impl RuntimePlugin for NotifyFileWatchPlugin {
    fn name(&self) -> &'static str {
        "amigo-file-watch-notify"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        match NotifyFileWatchBackend::new() {
            Ok(backend) => {
                registry.register(FileWatchBackendInfo {
                    backend_name: backend.backend_name(),
                    automatic_notifications: backend.automatic_notifications(),
                })?;
                registry.register(FileWatchService::new(backend))
            }
            Err(_) => registry.register(FileWatchBackendInfo {
                backend_name: "notify-unavailable",
                automatic_notifications: false,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NotifyFileWatchBackend, NotifyFileWatchPlugin};
    use amigo_file_watch_api::{
        FileWatchBackend, FileWatchBackendInfo, FileWatchEventKind, FileWatchService,
    };
    use amigo_runtime::RuntimeBuilder;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn plugin_registers_file_watch_services() {
        let runtime = RuntimeBuilder::default()
            .with_plugin(NotifyFileWatchPlugin)
            .expect("plugin should register")
            .build();

        assert!(runtime.resolve::<FileWatchService>().is_some());
        assert_eq!(
            runtime
                .resolve::<FileWatchBackendInfo>()
                .expect("backend info should exist")
                .backend_name,
            "notify"
        );
    }

    #[test]
    fn backend_reports_file_changes() {
        let root = temp_test_dir("notify-watch");
        let path = root.join("sprite-lab");
        fs::write(&path, "kind = \"sprite-2d\"\n").expect("file should be created");
        let backend = NotifyFileWatchBackend::new().expect("backend should be created");
        backend
            .sync_paths(std::slice::from_ref(&path))
            .expect("sync should succeed");

        fs::write(
            &path,
            "kind = \"sprite-2d\"\nlabel = \"Notify Reload\"\nformat = \"debug-placeholder\"\n",
        )
        .expect("file should be updated");

        let mut observed = Vec::new();
        for _ in 0..40 {
            let batch = backend.drain_events();
            if !batch.is_empty() {
                observed.extend(batch);
                break;
            }
            std::thread::sleep(Duration::from_millis(25));
        }

        assert!(observed.iter().any(|event| event.path == path));
        assert!(
            observed
                .iter()
                .any(|event| event.kind == FileWatchEventKind::Changed)
        );
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("amigo-file-watch-notify-{label}-{unique}"));
        fs::create_dir_all(&path).expect("temp test dir should be created");
        path
    }
}
