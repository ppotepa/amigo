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
