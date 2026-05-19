use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use amigo_math::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub enum CameraFocusTarget2dKind {
    RenderLayer,
    SceneObject,
    Marker,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraFocusTargetDepth2d {
    Distance { meters: f32, z_depth: f32 },
    Depth { z_depth: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFocusTarget2d {
    pub id: String,
    pub aliases: BTreeSet<String>,
    pub kind: CameraFocusTarget2dKind,
    pub entity_name: Option<String>,
    pub render_layer: Option<String>,
    pub source_component: Option<String>,
    pub world_position: Option<Vec2>,
    pub depth: CameraFocusTargetDepth2d,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCameraFocusTarget2d {
    pub selector: String,
    pub target: CameraFocusTarget2d,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraFocusTransitionTarget2d {
    Distance { meters: f32 },
    Depth { value: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFocusTransition2d {
    pub selector: String,
    pub elapsed_seconds: f32,
    pub duration_seconds: f32,
    pub start: CameraFocusTransitionTarget2d,
    pub end: CameraFocusTransitionTarget2d,
}

#[derive(Debug, Default)]
pub struct CameraFocusTarget2dService {
    targets: Mutex<BTreeMap<String, CameraFocusTarget2d>>,
    aliases: Mutex<BTreeMap<String, String>>,
    last_error: Mutex<Option<String>>,
}

impl CameraFocusTarget2dService {
    pub fn replace_all<I>(&self, targets: I)
    where
        I: IntoIterator<Item = CameraFocusTarget2d>,
    {
        let mut target_map = BTreeMap::new();
        let mut alias_map = BTreeMap::<String, String>::new();
        let mut ambiguous = BTreeSet::new();

        for target in targets {
            let id = normalize_selector(&target.id);
            if id.is_empty() {
                continue;
            }
            for alias in target.aliases.iter().map(|alias| normalize_selector(alias)) {
                if alias.is_empty() {
                    continue;
                }
                if let Some(existing) = alias_map.insert(alias.clone(), id.clone()) {
                    if existing != id {
                        ambiguous.insert(alias);
                    }
                }
            }
            target_map.insert(id.clone(), target);
        }

        for alias in ambiguous {
            alias_map.remove(&alias);
        }

        *self
            .targets
            .lock()
            .expect("camera focus target mutex should not be poisoned") = target_map;
        *self
            .aliases
            .lock()
            .expect("camera focus target aliases mutex should not be poisoned") = alias_map;
        *self
            .last_error
            .lock()
            .expect("camera focus target error mutex should not be poisoned") = None;
    }

    pub fn clear(&self) {
        self.targets
            .lock()
            .expect("camera focus target mutex should not be poisoned")
            .clear();
        self.aliases
            .lock()
            .expect("camera focus target aliases mutex should not be poisoned")
            .clear();
        *self
            .last_error
            .lock()
            .expect("camera focus target error mutex should not be poisoned") = None;
    }

    pub fn resolve(&self, selector: &str) -> Option<ResolvedCameraFocusTarget2d> {
        let selector = normalize_selector(selector);
        if selector.is_empty() {
            self.set_error("empty focus selector");
            return None;
        }

        let targets = self
            .targets
            .lock()
            .expect("camera focus target mutex should not be poisoned");
        if let Some(target) = targets.get(&selector) {
            self.set_error("");
            return Some(ResolvedCameraFocusTarget2d {
                selector,
                target: target.clone(),
            });
        }

        let aliases = self
            .aliases
            .lock()
            .expect("camera focus target aliases mutex should not be poisoned");
        let Some(id) = aliases.get(&selector) else {
            self.set_error(format!("unknown focus target `{selector}`"));
            return None;
        };
        let Some(target) = targets.get(id) else {
            self.set_error(format!("stale focus target alias `{selector}`"));
            return None;
        };
        self.set_error("");
        Some(ResolvedCameraFocusTarget2d {
            selector,
            target: target.clone(),
        })
    }

    pub fn has(&self, selector: &str) -> bool {
        self.resolve(selector).is_some()
    }

    pub fn targets(&self) -> Vec<CameraFocusTarget2d> {
        self.targets
            .lock()
            .expect("camera focus target mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn summary(&self) -> String {
        let targets = self.targets();
        if targets.is_empty() {
            return "camera.focus.targets: none".to_owned();
        }

        let mut lines = Vec::with_capacity(targets.len() + 1);
        lines.push("camera.focus.targets:".to_owned());
        for target in targets {
            let component = target.source_component.as_deref().unwrap_or("-");
            let layer = target.render_layer.as_deref().unwrap_or("-");
            let visible = target.visible;
            let aliases = target.aliases.iter().cloned().collect::<Vec<_>>().join(",");
            let depth = match target.depth {
                CameraFocusTargetDepth2d::Distance { meters, z_depth } => {
                    format!("distance {meters:.2}m z_depth={z_depth:.3}")
                }
                CameraFocusTargetDepth2d::Depth { z_depth } => format!("depth z_depth={z_depth:.3}"),
            };
            lines.push(format!(
                "{} component={} layer={} {} visible={} aliases=[{}]",
                target.id, component, layer, depth, visible, aliases
            ));
        }
        lines.join("\n")
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error
            .lock()
            .expect("camera focus target error mutex should not be poisoned")
            .clone()
    }

    fn set_error(&self, error: impl Into<String>) {
        let error = error.into();
        *self
            .last_error
            .lock()
            .expect("camera focus target error mutex should not be poisoned") =
            (!error.is_empty()).then_some(error);
    }
}

fn normalize_selector(selector: &str) -> String {
    selector.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, aliases: &[&str]) -> CameraFocusTarget2d {
        CameraFocusTarget2d {
            id: id.to_owned(),
            aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
            kind: CameraFocusTarget2dKind::SceneObject,
            entity_name: Some(id.trim_start_matches("entity:").to_owned()),
            render_layer: Some("title.depth2d".to_owned()),
            source_component: Some("Text2D".to_owned()),
            world_position: None,
            depth: CameraFocusTargetDepth2d::Distance {
                meters: 1.0,
                z_depth: 0.8,
            },
            visible: true,
        }
    }

    #[test]
    fn camera_focus_target_service_resolves_exact_entity_alias() {
        let service = CameraFocusTarget2dService::default();
        service.replace_all([target("entity:title", &["title"])]);

        let resolved = service.resolve("title").expect("target should resolve");
        assert_eq!(resolved.target.id, "entity:title");
        assert!(service.has("entity:title"));
    }

    #[test]
    fn camera_focus_target_service_rejects_unknown_selector() {
        let service = CameraFocusTarget2dService::default();
        service.replace_all([target("entity:title", &["title"])]);

        assert!(service.resolve("missing").is_none());
        assert!(service.last_error().is_some());
    }

    #[test]
    fn camera_focus_target_service_rejects_ambiguous_alias() {
        let service = CameraFocusTarget2dService::default();
        service.replace_all([
            target("entity:title-a", &["title"]),
            target("entity:title-b", &["title"]),
        ]);

        assert!(service.resolve("title").is_none());
    }
}
