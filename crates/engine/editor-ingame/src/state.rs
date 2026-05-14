use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use amigo_editor_authoring::AuthoringRuntimeBinding;

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
        min: f32,
        max: f32,
        current: f32,
    },
    Toggle {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
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
        value: String,
    },
    NumberCommit {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
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
    pub selection: Option<EditorSelection>,
    pub cursor: Option<(f32, f32)>,
    pub property_overrides: BTreeMap<String, EditorPropertyValue>,
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
    selection: Option<EditorSelection>,
    cursor: Option<(f32, f32)>,
    property_overrides: BTreeMap<String, EditorPropertyValue>,
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
                selection: None,
                cursor: None,
                property_overrides: BTreeMap::new(),
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
            selection: inner.selection.clone(),
            cursor: inner.cursor,
            property_overrides: inner.property_overrides.clone(),
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
            inner.status = if inner.open {
                "editor opened".to_owned()
            } else {
                "editor closed".to_owned()
            };
        }
    }

    pub fn set_open(&self, open: bool) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if inner.enabled {
            inner.open = open;
            inner.status = if open {
                "editor opened".to_owned()
            } else {
                "editor closed".to_owned()
            };
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
