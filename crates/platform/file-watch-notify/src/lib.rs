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
include!("tests.rs");
