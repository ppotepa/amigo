use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use amigo_editor_authoring::{AuthoringRuntimeBinding, AuthoringSourceScalarPatch};

#[derive(Debug, Clone, PartialEq)]
pub enum EditorPropertyValue {
    Number(f32),
    Bool(bool),
    Text(String),
    Enum(String),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Color(String),
    AssetRef(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTreeMode {
    Scene,
    Stack,
    RawYaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSource {
    Tree,
    Viewport,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorOpenMode {
    Full,
    InspectorDock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectTargetKind {
    Selected,
    AuthoringNode,
    Entity,
    PostFxFrameItem,
    RenderLayer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectTarget {
    pub kind: InspectTargetKind,
    pub label: String,
    pub subject: String,
    pub node_id: String,
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorSelection {
    pub node_id: String,
    pub source: SelectionSource,
    pub source_path: Option<String>,
    pub yaml_pointer: Option<String>,
    pub label: Option<String>,
    pub logical_x: Option<f32>,
    pub logical_y: Option<f32>,
    pub logical_bounds: Option<EditorRect>,
}

impl EditorRect {
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.width && y <= self.y + self.height
    }
}

/// Source identity attached to an editable scalar property. It is deliberately
/// separate from a runtime binding: a live edit need not be safe to persist.
#[derive(Debug, Clone, PartialEq)]
pub struct EditorSourceScalar {
    pub source_file: String,
    pub yaml_pointer: String,
    pub expected: serde_yaml::Value,
}

impl EditorSourceScalar {
    pub fn patch_for(&self, next: &EditorPropertyValue) -> Option<AuthoringSourceScalarPatch> {
        let replacement = match (&self.expected, next) {
            (serde_yaml::Value::Bool(_), EditorPropertyValue::Bool(value)) => {
                serde_yaml::Value::Bool(*value)
            }
            (serde_yaml::Value::String(_), EditorPropertyValue::Text(value))
            | (serde_yaml::Value::String(_), EditorPropertyValue::Enum(value)) => {
                serde_yaml::Value::String(value.clone())
            }
            // Numeric editor controls are f32. Persist only YAML floating
            // scalars, never integer identifiers such as a high-precision seed.
            (serde_yaml::Value::Number(number), EditorPropertyValue::Number(value))
                if number.as_i64().is_none()
                    && number.as_u64().is_none()
                    && number.as_f64().is_some()
                    && value.is_finite() =>
            {
                serde_yaml::to_value(*value).ok()?
            }
            _ => return None,
        };
        Some(AuthoringSourceScalarPatch {
            source_file: self.source_file.clone().into(),
            yaml_pointer: self.yaml_pointer.clone(),
            expected: self.expected.clone(),
            replacement,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorHitAction {
    SelectNode {
        node_id: String,
        source_path: Option<String>,
        yaml_pointer: Option<String>,
    },
    Slider {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        source: Option<EditorSourceScalar>,
        min: f32,
        max: f32,
        current: f32,
    },
    Toggle {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        source: Option<EditorSourceScalar>,
        current: bool,
    },
    TextCommit {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        value: String,
    },
    EnumSelect {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        source: Option<EditorSourceScalar>,
        value: String,
    },
    NumberCommit {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        source: Option<EditorSourceScalar>,
        value: f32,
    },
    Vec2Commit {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        x: f32,
        y: f32,
    },
    ColorCommit {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        value: String,
    },
    AssetPick {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        asset: String,
    },
    Command {
        command: String,
    },
    ConsumeOnly,
    ToggleTreeNode {
        node_id: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorHitTarget {
    pub id: String,
    pub rect: EditorRect,
    pub action: EditorHitAction,
}

#[derive(Debug, Clone)]
pub struct IngameEditorSnapshot {
    pub enabled: bool,
    pub open: bool,
    pub open_mode: EditorOpenMode,
    pub selection: Option<EditorSelection>,
    pub inspect_target: Option<InspectTarget>,
    pub cursor: Option<(f32, f32)>,
    pub property_overrides: BTreeMap<String, EditorPropertyValue>,
    pub has_pending_source_patch: bool,
    pub hit_targets: Vec<EditorHitTarget>,
    pub tree_scroll: f32,
    pub properties_scroll: f32,
    pub tree_mode: EditorTreeMode,
    pub tree_scroll_max: f32,
    pub properties_scroll_max: f32,
    pub collapsed_node_ids: BTreeSet<String>,
    pub tree_filter: String,
    pub viewport_pan_x: f32,
    pub viewport_pan_y: f32,
    pub viewport_zoom: f32,
    pub is_panning_viewport: bool,
    pub last_pan_cursor: Option<(f32, f32)>,
    pub status: String,
}

#[derive(Debug, Clone)]
struct IngameEditorInner {
    enabled: bool,
    open: bool,
    open_mode: EditorOpenMode,
    selection: Option<EditorSelection>,
    inspect_target: Option<InspectTarget>,
    cursor: Option<(f32, f32)>,
    property_overrides: BTreeMap<String, EditorPropertyValue>,
    pending_source_patch: Option<AuthoringSourceScalarPatch>,
    hit_targets: Vec<EditorHitTarget>,
    tree_scroll: f32,
    properties_scroll: f32,
    tree_mode: EditorTreeMode,
    tree_scroll_max: f32,
    properties_scroll_max: f32,
    collapsed_node_ids: BTreeSet<String>,
    tree_filter: String,
    viewport_pan_x: f32,
    viewport_pan_y: f32,
    viewport_zoom: f32,
    is_panning_viewport: bool,
    last_pan_cursor: Option<(f32, f32)>,
    status: String,
}

#[derive(Debug, Clone)]
pub struct IngameEditorState {
    inner: Arc<Mutex<IngameEditorInner>>,
}

impl IngameEditorState {
    pub fn new(enabled: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(IngameEditorInner {
                enabled,
                open: enabled,
                open_mode: EditorOpenMode::Full,
                selection: None,
                inspect_target: None,
                cursor: None,
                property_overrides: BTreeMap::new(),
                pending_source_patch: None,
                hit_targets: Vec::new(),
                tree_scroll: 0.0,
                properties_scroll: 0.0,
                tree_mode: EditorTreeMode::Scene,
                tree_scroll_max: 0.0,
                properties_scroll_max: 0.0,
                collapsed_node_ids: BTreeSet::new(),
                tree_filter: String::new(),
                viewport_pan_x: 0.0,
                viewport_pan_y: 0.0,
                viewport_zoom: 1.0,
                is_panning_viewport: false,
                last_pan_cursor: None,
                status: if enabled {
                    "editor mockup ready".to_owned()
                } else {
                    "editor disabled".to_owned()
                },
            })),
        }
    }

    pub fn snapshot(&self) -> IngameEditorSnapshot {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        IngameEditorSnapshot {
            enabled: inner.enabled,
            open: inner.open,
            open_mode: inner.open_mode,
            selection: inner.selection.clone(),
            inspect_target: inner.inspect_target.clone(),
            cursor: inner.cursor,
            property_overrides: inner.property_overrides.clone(),
            has_pending_source_patch: inner.pending_source_patch.is_some(),
            hit_targets: inner.hit_targets.clone(),
            tree_scroll: inner.tree_scroll,
            properties_scroll: inner.properties_scroll,
            tree_mode: inner.tree_mode,
            tree_scroll_max: inner.tree_scroll_max,
            properties_scroll_max: inner.properties_scroll_max,
            collapsed_node_ids: inner.collapsed_node_ids.clone(),
            tree_filter: inner.tree_filter.clone(),
            viewport_pan_x: inner.viewport_pan_x,
            viewport_pan_y: inner.viewport_pan_y,
            viewport_zoom: inner.viewport_zoom,
            is_panning_viewport: inner.is_panning_viewport,
            last_pan_cursor: inner.last_pan_cursor,
            status: inner.status.clone(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.snapshot().enabled
    }

    pub fn is_open(&self) -> bool {
        let snapshot = self.snapshot();
        snapshot.enabled && snapshot.open
    }

    pub fn toggle(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if inner.enabled {
            inner.open = !inner.open;
            if inner.open {
                inner.open_mode = EditorOpenMode::Full;
                inner.inspect_target = None;
                inner.status = "editor opened".to_owned();
            } else {
                inner.inspect_target = None;
                inner.status = "editor closed".to_owned();
            }
        }
    }

    pub fn set_open(&self, open: bool) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if inner.enabled {
            inner.open = open;
            if open {
                inner.open_mode = EditorOpenMode::Full;
                inner.inspect_target = None;
                inner.status = "editor opened".to_owned();
            } else {
                inner.inspect_target = None;
                inner.status = "editor closed".to_owned();
            }
        }
    }

    pub fn open_full_editor(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.enabled = true;
        inner.open = true;
        inner.open_mode = EditorOpenMode::Full;
        inner.inspect_target = None;
        inner.status = "editor opened".to_owned();
    }

    pub fn open_inspector_dock(&self, target: InspectTarget, selection: EditorSelection) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.enabled = true;
        inner.open = true;
        inner.open_mode = EditorOpenMode::InspectorDock;
        inner.selection = Some(selection);
        inner.inspect_target = Some(target);
        inner.properties_scroll = 0.0;
        inner.status = "inspector opened".to_owned();
    }

    pub fn close_inspector_dock(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if inner.open_mode == EditorOpenMode::InspectorDock {
            inner.open = false;
            inner.inspect_target = None;
            inner.selection = None;
            inner.status = "inspector closed".to_owned();
        }
    }

    pub fn select_scene_node(&self, selection: EditorSelection) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let label = selection
            .label
            .as_deref()
            .unwrap_or(selection.node_id.as_str())
            .to_owned();
        let source = selection.source;
        inner.selection = Some(selection);
        inner.status = match source {
            SelectionSource::Tree => format!("tree selected {label}"),
            SelectionSource::Viewport => format!("viewport selected {label}"),
            SelectionSource::Command => format!("selected {label}"),
        };
    }

    pub fn clear_selection(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.selection = None;
        inner.status = "No selection".to_owned();
    }

    pub fn selected_node_id(&self) -> Option<String> {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .selection
            .as_ref()
            .map(|selection| selection.node_id.clone())
    }

    pub fn set_cursor(&self, x: f32, y: f32) {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .cursor = Some((x, y));
    }

    pub fn set_hit_targets(&self, hit_targets: Vec<EditorHitTarget>) {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .hit_targets = hit_targets;
    }

    pub fn hit_target_at(&self, x: f32, y: f32) -> Option<EditorHitTarget> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner
            .hit_targets
            .iter()
            .rev()
            .find(|target| target.rect.contains(x, y))
            .cloned()
    }

    pub fn set_override(&self, property_id: impl Into<String>, value: EditorPropertyValue) {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .property_overrides
            .insert(property_id.into(), value);
    }

    pub fn override_value(&self, property_id: &str) -> Option<EditorPropertyValue> {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .property_overrides
            .get(property_id)
            .cloned()
    }

    pub fn stage_source_scalar_patch(&self, patch: AuthoringSourceScalarPatch) {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .pending_source_patch = Some(patch);
    }

    pub fn pending_source_scalar_patch(&self) -> Option<AuthoringSourceScalarPatch> {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .pending_source_patch
            .clone()
    }

    pub fn clear_pending_source_scalar_patch(&self) {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .pending_source_patch = None;
    }

    pub fn set_status(&self, status: impl Into<String>) {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .status = status.into();
    }

    pub fn scroll_tree(&self, delta: f32) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.tree_scroll = (inner.tree_scroll + delta).clamp(0.0, inner.tree_scroll_max);
    }

    pub fn scroll_properties(&self, delta: f32) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.properties_scroll =
            (inner.properties_scroll + delta).clamp(0.0, inner.properties_scroll_max);
    }

    pub fn set_scroll_bounds(&self, tree_max: f32, properties_max: f32) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.tree_scroll_max = tree_max.max(0.0);
        inner.properties_scroll_max = properties_max.max(0.0);
        inner.tree_scroll = inner.tree_scroll.clamp(0.0, inner.tree_scroll_max);
        inner.properties_scroll = inner
            .properties_scroll
            .clamp(0.0, inner.properties_scroll_max);
    }

    pub fn toggle_node_collapsed(&self, node_id: &str) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if !inner.collapsed_node_ids.remove(node_id) {
            inner.collapsed_node_ids.insert(node_id.to_owned());
        }
    }

    pub fn is_node_collapsed(&self, node_id: &str) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .collapsed_node_ids
            .contains(node_id)
    }

    pub fn expand_all(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.collapsed_node_ids.clear();
        inner.status = "tree expanded".to_owned();
    }

    pub fn collapse_all(&self, node_ids: impl IntoIterator<Item = String>) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.collapsed_node_ids.clear();
        inner.collapsed_node_ids.extend(node_ids);
        inner.status = format!("tree collapsed: {} nodes", inner.collapsed_node_ids.len());
    }

    pub fn collapsed_node_count(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .collapsed_node_ids
            .len()
    }

    pub fn set_tree_filter(&self, filter: impl Into<String>) {
        let filter = filter.into();
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.tree_filter = filter.trim().to_owned();
        inner.tree_scroll = 0.0;
        inner.status = if inner.tree_filter.is_empty() {
            "tree filter cleared".to_owned()
        } else {
            format!("tree filter: {}", inner.tree_filter)
        };
    }

    pub fn clear_tree_filter(&self) {
        self.set_tree_filter("");
    }

    pub fn set_tree_mode(&self, mode: EditorTreeMode) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.tree_mode = mode;
        inner.tree_scroll = 0.0;
        inner.status = match mode {
            EditorTreeMode::Scene => "tree mode: scene graph".to_owned(),
            EditorTreeMode::Stack => "tree mode: render stack".to_owned(),
            EditorTreeMode::RawYaml => "tree mode: raw yaml debug".to_owned(),
        };
    }

    pub fn begin_viewport_pan(&self, x: f32, y: f32) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.is_panning_viewport = true;
        inner.last_pan_cursor = Some((x, y));
        inner.status = "viewport pan: started".to_owned();
    }

    pub fn update_viewport_pan(&self, x: f32, y: f32) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if !inner.is_panning_viewport {
            return;
        }
        if let Some((last_x, last_y)) = inner.last_pan_cursor {
            inner.viewport_pan_x += x - last_x;
            inner.viewport_pan_y += y - last_y;
        }
        inner.last_pan_cursor = Some((x, y));
    }

    pub fn end_viewport_pan(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.is_panning_viewport = false;
        inner.last_pan_cursor = None;
        inner.status = format!(
            "viewport pan: x={:.1} y={:.1} zoom={:.2}",
            inner.viewport_pan_x, inner.viewport_pan_y, inner.viewport_zoom
        );
    }

    pub fn viewport_pan(&self) -> (f32, f32) {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        (inner.viewport_pan_x, inner.viewport_pan_y)
    }

    pub fn viewport_zoom(&self) -> f32 {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .viewport_zoom
    }

    pub fn reset_viewport_view(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.viewport_pan_x = 0.0;
        inner.viewport_pan_y = 0.0;
        inner.viewport_zoom = 1.0;
        inner.is_panning_viewport = false;
        inner.last_pan_cursor = None;
        inner.status = "viewport reset".to_owned();
    }
}
