use std::path::PathBuf;

#[derive(Debug)]
pub enum PluginLoadError {
    Io { path: PathBuf, source: std::io::Error },
    Parse { path: PathBuf, message: String },
}
