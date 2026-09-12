//! Transport-independent scene panel contracts. No UI backend or domain dependencies.
use amigo_runtime_control::{ControlRange, ControlValue, ControlValueType};
use amigo_scene::SceneUiNodeComponentDocument;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read, Write},
};

pub const PROTOCOL_VERSION: u32 = 3;
pub const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelDocument {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub preset_name_bind: Option<String>,
    #[serde(default)]
    pub preset_domain_bind: Option<String>,
    #[serde(default)]
    pub confirm_actions: Vec<String>,
    /// Optional, authored capabilities for the generic asset browser.
    #[serde(default)]
    pub asset_browser: Option<PanelAssetBrowserDocument>,
    #[serde(default)]
    pub presentation: BTreeMap<String, PanelPresentation>,
    #[serde(default)]
    pub artwork: BTreeMap<String, Vec<PreviewTriangle>>,
    pub root: SceneUiNodeComponentDocument,
}

/// Actions that a panel author explicitly permits for the generic asset browser.
/// The usage strings are routed to a domain handler; panel-api never assigns
/// semantics such as "model" or "scene" to an asset entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelAssetBrowserDocument {
    /// The authored source to open first when it is present in the runtime catalog.
    #[serde(default)]
    pub preferred_source: Option<String>,
    pub add_usage: Option<String>,
    pub replace_usage: Option<String>,
}

/// Backend-neutral layout hints. Domain actions remain authored script events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelPresentation {
    pub tooltip: Option<String>,
    pub suffix: Option<String>,
    pub reset: Option<bool>,
    #[serde(default)]
    pub collapsed: bool,
    pub pin: Option<PanelPin>,
    #[serde(default)]
    pub choices: Vec<PanelChoice>,
    #[serde(default)]
    pub navigation: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PanelPin {
    Top,
    Bottom,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelChoice {
    pub value: String,
    pub label: String,
    pub artwork_bind: Option<String>,
    pub artwork: Option<String>,
    pub status_bind: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewTriangle {
    pub points: [[f32; 2]; 3],
    pub color: [u8; 3],
}

impl PanelDocument {
    pub fn nodes(&self) -> Vec<&SceneUiNodeComponentDocument> {
        fn walk<'a>(
            node: &'a SceneUiNodeComponentDocument,
            out: &mut Vec<&'a SceneUiNodeComponentDocument>,
        ) {
            out.push(node);
            for child in &node.children {
                walk(child, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.root, &mut out);
        out
    }
    pub fn binding_paths(&self) -> BTreeSet<String> {
        self.nodes()
            .into_iter()
            .flat_map(|n| {
                [
                    &n.value_bind,
                    &n.text_bind,
                    &n.visible_bind,
                    &n.enabled_bind,
                    &n.options_bind,
                ]
                .into_iter()
                .flatten()
                .cloned()
            })
            .chain(
                self.presentation
                    .values()
                    .flat_map(|p| p.choices.iter())
                    .flat_map(|c| {
                        [&c.artwork_bind, &c.status_bind]
                            .into_iter()
                            .flatten()
                            .cloned()
                    }),
            )
            .chain(self.preset_domain_bind.iter().cloned())
            .collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("panel id is empty".into());
        }
        let mut ids = BTreeSet::new();
        for node in self.nodes() {
            if node.kind == amigo_scene::SceneUiNodeTypeComponentDocument::TabView {
                let tabs = node.tabs.iter().map(|t| &t.id).collect::<BTreeSet<_>>();
                if tabs.is_empty()
                    || tabs.len() != node.tabs.len()
                    || node.children.len() != tabs.len()
                    || node
                        .children
                        .iter()
                        .any(|n| !n.id.as_ref().is_some_and(|id| tabs.contains(id)))
                {
                    return Err(
                        "tabs must have unique ids and exactly one matching page each".into(),
                    );
                }
            }
            if node.min.is_some_and(|v| !v.is_finite())
                || node.max.is_some_and(|v| !v.is_finite())
                || node.step.is_some_and(|v| !v.is_finite() || v <= 0.0)
                || matches!((node.min,node.max),(Some(a),Some(b)) if a>b)
            {
                return Err("invalid control range/step".into());
            }
            if node.kind == amigo_scene::SceneUiNodeTypeComponentDocument::CurveEditor {
                return Err("curve-editor is not supported by runtime panels".into());
            }
            if node.options_bind.is_some()
                && !matches!(
                    node.kind,
                    amigo_scene::SceneUiNodeTypeComponentDocument::Dropdown
                        | amigo_scene::SceneUiNodeTypeComponentDocument::OptionSet
                )
            {
                return Err("options_bind is only valid for dropdowns and option sets".into());
            }
            if let Some(id) = &node.id {
                if id.is_empty() || !ids.insert(id) {
                    return Err(format!("duplicate/empty control id: {id}"));
                }
            } else if node.value_bind.is_some()
                || node.on_click.is_some()
                || node.on_change.is_some()
            {
                return Err("interactive controls require stable ids".into());
            }
        }
        for (id, hint) in &self.presentation {
            let node = self
                .nodes()
                .into_iter()
                .find(|n| n.id.as_ref() == Some(id))
                .ok_or_else(|| format!("presentation references unknown node {id}"))?;
            if hint.pin.is_some() && !self.root.children.iter().any(|n| n.id.as_ref() == Some(id)) {
                return Err(format!("pinned node {id} must be a direct root child"));
            }
            let mut choices = BTreeSet::new();
            for choice in &hint.choices {
                if choice
                    .artwork
                    .as_ref()
                    .is_some_and(|key| !self.artwork.contains_key(key))
                {
                    return Err(format!("missing choice artwork in {id}"));
                }
                if !node.options.contains(&choice.value) || !choices.insert(&choice.value) {
                    return Err(format!("invalid/duplicate choice in {id}"));
                }
            }
        }
        if self.artwork.values().any(|triangles| {
            triangles.len() > 20000
                || triangles.iter().any(|t| {
                    t.points
                        .iter()
                        .flatten()
                        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                })
        }) {
            return Err("invalid normalized preview artwork".into());
        }
        Ok(())
    }
    pub fn validate_bindings(
        &self,
        registry: &amigo_runtime_control::RuntimeControlRegistry,
    ) -> Result<(), String> {
        self.validate()?;
        for path in self.binding_paths() {
            if !registry.property(&path).is_some_and(|p| p.readable) {
                return Err(format!("unknown/unreadable binding {path}"));
            }
        }
        for node in self.nodes() {
            for path in [
                &node.value_bind,
                &node.text_bind,
                &node.visible_bind,
                &node.enabled_bind,
                &node.options_bind,
            ]
            .into_iter()
            .flatten()
            {
                let property = registry
                    .property(path)
                    .ok_or_else(|| format!("unknown binding {path}"))?;
                if !property.readable {
                    return Err(format!("binding {path} is not readable"));
                }
            }
            for path in [&node.visible_bind, &node.enabled_bind]
                .into_iter()
                .flatten()
            {
                if registry.property(path).unwrap().value_type != ControlValueType::Bool {
                    return Err(format!("{path} must be boolean"));
                }
            }
            if let Some(path) = &node.options_bind {
                if registry.property(path).unwrap().value_type != ControlValueType::String {
                    return Err(format!("{path} must be a newline-separated string"));
                }
            }
            if let Some(path) = &node.value_bind {
                use amigo_scene::SceneUiNodeTypeComponentDocument as K;
                let ty = registry.property(path).unwrap().value_type;
                let valid = match node.kind {
                    K::Toggle => ty == ControlValueType::Bool,
                    K::Dropdown | K::OptionSet => {
                        matches!(ty, ControlValueType::String | ControlValueType::AssetRef)
                    }
                    K::ColorPickerRgb => ty == ControlValueType::Color,
                    _ => true,
                };
                if !valid {
                    return Err(format!("widget type does not match {path}"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySnapshot {
    pub path: String,
    pub value: ControlValue,
    pub value_type: ControlValueType,
    pub writable: bool,
    pub range: Option<ControlRange>,
    pub description: Option<String>,
}

/// A complete, transport-independent view of one live scene panel.
///
/// Native, embedded and remote hosts all consume this exact snapshot. The
/// generation/revision pair is carried with user input so a host cannot apply
/// an interaction to a scene or layout that has since changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSnapshot {
    pub generation: u64,
    pub revision: u64,
    pub document: PanelDocument,
    pub preset_names: Vec<String>,
    pub values: BTreeMap<String, PropertySnapshot>,
    #[serde(default)]
    pub asset_browser: Option<PanelAssetBrowserSnapshot>,
}

/// Transport-safe asset catalog data for a panel host. This mirrors a runtime
/// catalog snapshot without coupling the external protocol to engine services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PanelAssetBrowserSnapshot {
    pub selected_source: Option<String>,
    pub sources: Vec<PanelAssetSourceSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PanelAssetSourceSnapshot {
    pub id: String,
    pub label: String,
    pub refreshable: bool,
    pub revision: u64,
    pub state: PanelAssetSourceState,
    pub entries: Vec<PanelAssetEntrySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PanelAssetSourceState {
    Empty,
    Loading,
    Ready,
    Failed { failed_assets: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PanelAssetEntrySnapshot {
    pub id: String,
    pub label: String,
    pub tags: Vec<String>,
    pub kind: Option<String>,
    pub state: PanelAssetEntryState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PanelAssetEntryState {
    Unloaded,
    Loading,
    Ready,
    Failed { message: String },
}

/// A revision-checked request to use one catalog entry for an authored purpose.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PanelAssetIntent {
    pub panel_id: String,
    pub generation: u64,
    pub revision: u64,
    pub source_id: String,
    pub source_revision: u64,
    pub asset_id: String,
    pub usage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Hello {
        version: u32,
    },
    Edit {
        request: u64,
        generation: u64,
        revision: u64,
        control: String,
        value: ControlValue,
    },
    Reset {
        request: u64,
        generation: u64,
        revision: u64,
        control: String,
    },
    Click {
        request: u64,
        generation: u64,
        revision: u64,
        control: String,
    },
    AssetIntent {
        request: u64,
        intent: PanelAssetIntent,
    },
    RefreshAssetSource {
        request: u64,
        panel_id: String,
        generation: u64,
        revision: u64,
        source_id: String,
        source_revision: u64,
    },
    Close,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Document {
        version: u32,
        generation: u64,
        revision: u64,
        document: PanelDocument,
    },
    Snapshot {
        generation: u64,
        revision: u64,
        acknowledged: u64,
        preset_names: Vec<String>,
        values: BTreeMap<String, PropertySnapshot>,
        #[serde(default)]
        asset_browser: Option<PanelAssetBrowserSnapshot>,
    },
    Result {
        request: u64,
        error: Option<String>,
    },
    Diagnostic(String),
    Shutdown,
}

pub fn write_message(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "panel frame exceeds limit",
        ));
    }
    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}
pub fn read_message<T: DeserializeOwned>(reader: &mut impl Read) -> io::Result<T> {
    let mut length = [0; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "panel frame exceeds limit",
        ));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    serde_json::from_slice(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presentation_bindings_and_invalid_artwork_are_validated() {
        let mut doc: PanelDocument = serde_json::from_value(serde_json::json!({
            "id":"test", "title":"Test", "root":{"type":"column", "id":"root", "children":[
                {"type":"option-set","id":"objects","value_bind":"selected","options_bind":"object_ids","options":["a"]}
            ]}, "presentation":{"objects":{"pin":"top","choices":[{"value":"a","label":"A","artwork_bind":"model","status_bind":"status"}]}}
        })).unwrap();
        doc.validate().unwrap();
        assert_eq!(
            doc.binding_paths(),
            ["selected", "object_ids", "model", "status"]
                .into_iter()
                .map(String::from)
                .collect()
        );
        doc.artwork.insert(
            "a".into(),
            vec![PreviewTriangle {
                points: [[2.0, 0.0]; 3],
                color: [0; 3],
            }],
        );
        assert!(doc.validate().is_err());
        doc.artwork.clear();
        doc.presentation.get_mut("objects").unwrap().choices[0].value = "missing".into();
        assert!(doc.validate().is_err());
    }
    #[test]
    fn tab_ids_must_match_pages() {
        let doc: PanelDocument = serde_json::from_value(serde_json::json!({"id":"test","title":"Test","root":{
            "type":"tab-view","tabs":[{"id":"a","label":"A"}],"children":[{"type":"column","id":"b"}]
        }})).unwrap();
        assert!(doc.validate().is_err());
    }
    #[test]
    fn framing_handles_fragmented_transport() {
        struct Fragmented(std::io::Cursor<Vec<u8>>);
        impl Read for Fragmented {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                let n = b.len().min(1);
                self.0.read(&mut b[..n])
            }
        }
        let mut bytes = Vec::new();
        write_message(
            &mut bytes,
            &ClientMessage::Hello {
                version: PROTOCOL_VERSION,
            },
        )
        .unwrap();
        assert!(matches!(
            read_message::<ClientMessage>(&mut Fragmented(std::io::Cursor::new(bytes))).unwrap(),
            ClientMessage::Hello {
                version: PROTOCOL_VERSION
            }
        ));
    }
    #[test]
    fn oversized_frame_is_rejected_before_allocation() {
        let bytes = u32::MAX.to_le_bytes();
        assert!(read_message::<ClientMessage>(&mut &bytes[..]).is_err());
    }
}
