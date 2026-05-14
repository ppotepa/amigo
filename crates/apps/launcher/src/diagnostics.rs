use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use amigo_core::{AmigoError, AmigoResult};
use amigo_modding::{ModCatalog, requested_mods_for_root};

use crate::config::{CargoProfile, LauncherConfig, LauncherProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileDiagnostics {
    pub profile_id: String,
    pub root_mod: String,
    pub resolved_mod_ids: Vec<String>,
    pub diagnostics: Vec<LauncherDiagnostic>,
}

impl ProfileDiagnostics {
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    pub fn is_launchable(&self) -> bool {
        !self.has_errors()
    }
}

pub fn collect_profile_diagnostics(
    config: &LauncherConfig,
) -> BTreeMap<String, ProfileDiagnostics> {
    config
        .profiles
        .iter()
        .map(|profile| (profile.id.clone(), diagnose_profile(config, profile)))
        .collect()
}

pub fn diagnose_profile(config: &LauncherConfig, profile: &LauncherProfile) -> ProfileDiagnostics {
    let root_mod = profile.root_mod_or_core().to_owned();
    let mut report = ProfileDiagnostics {
        profile_id: profile.id.clone(),
        root_mod: root_mod.clone(),
        resolved_mod_ids: Vec::new(),
        diagnostics: Vec::new(),
    };
    let mods_root = Path::new(&config.mods_root);

    if !mods_root.exists() {
        report.diagnostics.push(LauncherDiagnostic {
            severity: DiagnosticSeverity::Error,
            message: format!("mods root `{}` does not exist", mods_root.display()),
        });
        return report;
    }

    match ModCatalog::discover_selected(mods_root, &requested_mods_for_root(&root_mod)) {
        Ok(catalog) => {
            report.resolved_mod_ids = catalog
                .mod_ids()
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>();

            if let Some(root_mod_manifest) = catalog.mod_by_id(&root_mod) {
                validate_scene_selection(
                    &mut report,
                    profile,
                    root_mod_manifest.manifest.scenes.len(),
                );

                if let Some(scene_id) = profile.startup_scene.as_deref() {
                    if root_mod_manifest.scene_by_id(scene_id).is_none() {
                        report.diagnostics.push(LauncherDiagnostic {
                            severity: DiagnosticSeverity::Error,
                            message: format!(
                                "startup scene `{scene_id}` is not declared by root mod `{root_mod}`"
                            ),
                        });
                    }
                }
            }
        }
        Err(error) => report.diagnostics.push(LauncherDiagnostic {
            severity: DiagnosticSeverity::Error,
            message: error.to_string(),
        }),
    }

    if profile.cargo_profile == CargoProfile::Release {
        let release_app_binary = release_app_binary_path();

        if !release_app_binary.exists() {
            report.diagnostics.push(LauncherDiagnostic {
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "release app binary is missing at `{}`; build it with `cargo build --release -p amigo-app` before using this preset",
                    release_app_binary.display()
                ),
            });
        }
    }

    report
}

pub fn ensure_profile_launchable(
    config: &LauncherConfig,
    profile: &LauncherProfile,
) -> AmigoResult<ProfileDiagnostics> {
    let report = diagnose_profile(config, profile);

    if !report.is_launchable() {
        let errors = report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .map(|diagnostic| diagnostic.message.clone())
            .collect::<Vec<_>>()
            .join("; ");

        return Err(AmigoError::Message(format!(
            "launcher profile `{}` is not launchable: {errors}",
            profile.id
        )));
    }

    Ok(report)
}

fn validate_scene_selection(
    report: &mut ProfileDiagnostics,
    profile: &LauncherProfile,
    declared_scene_count: usize,
) {
    if let Some(scene_id) = profile.startup_scene.as_deref() {
        if scene_id.trim().is_empty() {
            report.diagnostics.push(LauncherDiagnostic {
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "profile `{}` has an empty startup scene; the runtime will fall back to the root mod default",
                    profile.id
                ),
            });
        }
        return;
    }

    if declared_scene_count > 0 {
        report.diagnostics.push(LauncherDiagnostic {
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "profile `{}` does not select a startup scene for root mod `{}`",
                profile.id, report.root_mod
            ),
        });
    }
}

fn release_app_binary_path() -> PathBuf {
    launcher_workspace_root()
        .join("target")
        .join("release")
        .join(app_binary_name())
}

fn launcher_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}

fn app_binary_name() -> &'static str {
    if cfg!(windows) {
        "amigo-app.exe"
    } else {
        "amigo-app"
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{DiagnosticSeverity, collect_profile_diagnostics, diagnose_profile};
    use crate::config::LauncherConfig;

    fn config() -> LauncherConfig {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let mut config = LauncherConfig::default();
        config.mods_root = workspace_root.join("mods").display().to_string();
        config
    }

    #[test]
    fn valid_dev_profile_has_no_errors() {
        let config = config();
        let profile = config
            .profile_by_id("dev")
            .expect("dev profile should exist");
        let report = diagnose_profile(&config, profile);

        assert!(report.is_launchable());
        assert_eq!(
            report.resolved_mod_ids,
            vec!["core".to_owned(), "core-game".to_owned()]
        );
    }

    #[test]
    fn missing_scene_is_reported_as_error() {
        let config = config();
        let profile = config
            .profile_by_id("dev")
            .expect("dev profile should exist")
            .clone();
        let mut broken_profile = profile.clone();
        broken_profile.startup_scene = Some("missing-scene".to_owned());

        let report = diagnose_profile(&config, &broken_profile);

        assert!(report.has_errors());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic
                    .message
                    .contains("startup scene `missing-scene` is not declared")
        }));
    }

    #[test]
    fn diagnostics_are_collected_for_each_profile() {
        let config = config();
        let reports = collect_profile_diagnostics(&config);

        assert!(reports.contains_key("dev"));
        assert!(reports.contains_key("release"));
        assert!(reports.contains_key("wgpu-playground"));
    }
}
