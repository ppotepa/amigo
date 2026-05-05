use std::path::Path;
use std::process::Command;

pub fn reveal_path(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = if path.is_file() {
            Command::new("explorer")
                .arg(format!("/select,{}", path.display()))
                .status()
        } else {
            Command::new("explorer").arg(path).status()
        }
        .map_err(|error| error.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("explorer failed for `{}`", path.display()))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let target = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        let status = Command::new(opener)
            .arg(target)
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("{opener} failed for `{}`", target.display()))
        }
    }
}
