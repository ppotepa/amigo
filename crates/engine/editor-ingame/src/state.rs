use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use amigo_editor_authoring::AuthoringRuntimeBinding;

#[derive(Debug, Clone, PartialEq)]
pub enum EditorPropertyValue {
    Number(f32),
    Bool(bool),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorViewportSelection {
    pub node_id: String,
    pub entity_name: Option<String>,
    pub component_type: Option<String>,
    pub logical_x: f32,
    pub logical_y: f32,
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
    },
    Toggle {
        property_id: String,
        target: Option<AuthoringRuntimeBinding>,
        current: bool,
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
    pub selected_node_id: Option<String>,
    pub selected_source_path: Option<String>,
    pub selected_yaml_pointer: Option<String>,
    pub viewport_selection: Option<EditorViewportSelection>,
    pub cursor: Option<(f32, f32)>,
    pub property_overrides: BTreeMap<String, EditorPropertyValue>,
    pub hit_targets: Vec<EditorHitTarget>,
    pub tree_scroll: f32,
    pub properties_scroll: f32,
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
    selected_node_id: Option<String>,
    selected_source_path: Option<String>,
    selected_yaml_pointer: Option<String>,
    viewport_selection: Option<EditorViewportSelection>,
    cursor: Option<(f32, f32)>,
    property_overrides: BTreeMap<String, EditorPropertyValue>,
    hit_targets: Vec<EditorHitTarget>,
    tree_scroll: f32,
    properties_scroll: f32,
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
                selected_node_id: None,
                selected_source_path: None,
                selected_yaml_pointer: None,
                viewport_selection: None,
                cursor: None,
                property_overrides: BTreeMap::new(),
                hit_targets: Vec::new(),
                tree_scroll: 0.0,
                properties_scroll: 0.0,
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
            selected_node_id: inner.selected_node_id.clone(),
            selected_source_path: inner.selected_source_path.clone(),
            selected_yaml_pointer: inner.selected_yaml_pointer.clone(),
            viewport_selection: inner.viewport_selection.clone(),
            cursor: inner.cursor,
            property_overrides: inner.property_overrides.clone(),
            hit_targets: inner.hit_targets.clone(),
            tree_scroll: inner.tree_scroll,
            properties_scroll: inner.properties_scroll,
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

    pub fn select_node(
        &self,
        node_id: impl Into<String>,
        source_path: Option<String>,
        yaml_pointer: Option<String>,
    ) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let node_id = node_id.into();
        inner.selected_node_id = Some(node_id.clone());
        inner.selected_source_path = source_path;
        inner.selected_yaml_pointer = yaml_pointer;
        inner.viewport_selection = None;
        inner.status = format!("selected {node_id}");
    }

    pub fn select_viewport_node(
        &self,
        node_id: impl Into<String>,
        source_path: Option<String>,
        yaml_pointer: Option<String>,
        entity_name: Option<String>,
        component_type: Option<String>,
        logical_x: f32,
        logical_y: f32,
        logical_bounds: Option<EditorRect>,
    ) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let node_id = node_id.into();
        inner.selected_node_id = Some(node_id.clone());
        inner.selected_source_path = source_path;
        inner.selected_yaml_pointer = yaml_pointer;
        inner.viewport_selection = Some(EditorViewportSelection {
            node_id: node_id.clone(),
            entity_name,
            component_type,
            logical_x,
            logical_y,
            logical_bounds,
        });
        inner.status = format!("viewport selected {node_id} @ {logical_x:.1},{logical_y:.1}");
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
