use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorConsoleLogEntryDto {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub args: Vec<String>,
    pub source: Option<String>,
}

pub struct EditorConsoleLogState {
    path: PathBuf,
    lock: Mutex<()>,
}

impl EditorConsoleLogState {
    pub fn initialize() -> Result<Self, String> {
        let log_dir = editor_log_dir();
        fs::create_dir_all(&log_dir).map_err(|error| {
            format!(
                "EDITOR_CONSOLE_LOG_DIR_CREATE_FAILED: {}: {error}",
                log_dir.display()
            )
        })?;

        let path = create_run_log_file(&log_dir)?;
        Ok(Self {
            path,
            lock: Mutex::new(()),
        })
    }
}

#[tauri::command]
pub fn append_editor_console_logs(
    state: State<'_, EditorConsoleLogState>,
    entries: Vec<EditorConsoleLogEntryDto>,
) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }

    let _guard = state
        .lock
        .lock()
        .map_err(|_| "EDITOR_CONSOLE_LOG_LOCK_FAILED".to_owned())?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(&state.path)
        .map_err(|error| {
            format!(
                "EDITOR_CONSOLE_LOG_OPEN_FAILED: {}: {error}",
                state.path.display()
            )
        })?;

    for entry in entries {
        let source = entry
            .source
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("webview");
        writeln!(
            file,
            "[{}] [{}] [{}] {}",
            entry.timestamp, source, entry.level, entry.message
        )
        .map_err(|error| format!("EDITOR_CONSOLE_LOG_WRITE_FAILED: {error}"))?;

        for arg in entry.args {
            if arg != entry.message {
                writeln!(file, "    {arg}")
                    .map_err(|error| format!("EDITOR_CONSOLE_LOG_WRITE_FAILED: {error}"))?;
            }
        }
    }

    Ok(())
}

fn editor_log_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(&manifest_dir).join("logs")
}

fn create_run_log_file(log_dir: &PathBuf) -> Result<PathBuf, String> {
    let date = current_utc_date();
    for run_no in 1..=9999 {
        let path = log_dir.join(format!("{date}-{run_no:03}.log"));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                writeln!(file, "# Amigo Editor console log")
                    .map_err(|error| format!("EDITOR_CONSOLE_LOG_WRITE_FAILED: {error}"))?;
                writeln!(file, "# path: {}", path.display())
                    .map_err(|error| format!("EDITOR_CONSOLE_LOG_WRITE_FAILED: {error}"))?;
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "EDITOR_CONSOLE_LOG_CREATE_FAILED: {}: {error}",
                    path.display()
                ));
            }
        }
    }

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let path = log_dir.join(format!("{date}-{millis}.log"));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            format!(
                "EDITOR_CONSOLE_LOG_CREATE_FAILED: {}: {error}",
                path.display()
            )
        })?;
    Ok(path)
}

fn current_utc_date() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or_default();
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}
