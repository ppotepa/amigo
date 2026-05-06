use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::dto::EditorDiagnosticDto;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorSceneCanvasKindDto {
    #[serde(rename = "2d")]
    TwoD,
    #[serde(rename = "2.5d")]
    TwoHalfD,
    #[serde(rename = "3d")]
    ThreeD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorSceneSnapshotLayoutSourceDto {
    Runtime,
    Document,
    Fallback,
    Unavailable,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneSnapshotQualityDto {
    pub indexed_entities: usize,
    pub objects: usize,
    pub editable_objects: usize,
    pub objects_without_transform: usize,
    pub objects_without_bounds: usize,
    pub unsupported_bounds_providers: usize,
    pub diagnostics_by_code: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorFrameTransportKindDto {
    ImageUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorRenderTransportPreferenceDto {
    Auto,
    ImageUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorModeDto {
    Edit,
    Preview,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorToolDto {
    Select,
    Move,
    Scale,
    Rotate,
    Rect,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorToolSpaceDto {
    World,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSnapSettingsDto {
    pub enabled: bool,
    pub grid_size: f32,
    pub angle_step_deg: f32,
    pub scale_step: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorUiNodeSelectionDto {
    pub entity_id: String,
    pub component_index: usize,
    pub node_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum EditorUiNodePropertyValueDto {
    String(String),
    Number(f32),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorUiNodeCreateKindDto {
    Column,
    Row,
    Panel,
    Stack,
    Spacer,
    Text,
    Button,
    Image,
    ProgressBar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorUiTemplateKindDto {
    EmptyDocument,
    VerticalMenu,
    ButtonRow,
    HealthBar,
    AmmoCounter,
    DialogueBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorUiNodeMoveDirectionDto {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorUiNodeCreateDto {
    pub kind: EditorUiNodeCreateKindDto,
    pub id: String,
    pub label: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSelectionDto {
    pub selected_entity_ids: Vec<String>,
    pub selected_ui_node: Option<EditorUiNodeSelectionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorToolStateDto {
    pub active_tool: EditorToolDto,
    pub space: EditorToolSpaceDto,
    pub snap: EditorSnapSettingsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorViewportDto {
    pub css_width: f32,
    pub css_height: f32,
    pub render_width: u32,
    pub render_height: u32,
    pub device_pixel_ratio: f32,
    #[serde(default)]
    pub camera_x: Option<f32>,
    #[serde(default)]
    pub camera_y: Option<f32>,
    #[serde(default)]
    pub zoom: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorFrameDto {
    pub session_id: String,
    pub revision: u64,
    pub transport: EditorFrameTransportKindDto,
    pub width: u32,
    pub height: u32,
    pub device_pixel_ratio: f32,
    pub image_url: Option<String>,
    pub render_time_ms: Option<f32>,
    pub encoded_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorModeSessionDto {
    pub editor_mode_session_id: String,
    pub editor_session_id: String,
    pub mod_id: String,
    pub scene_id: String,
    pub mode: EditorModeDto,
    pub tool: EditorToolDto,
    pub dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub revision: u64,
    pub transport: EditorFrameTransportKindDto,
    pub cursor: EditorCursorDto,
    pub hovered_control_id: Option<String>,
    pub hovered_handle_id: Option<String>,
    pub active_control_id: Option<String>,
    pub active_handle_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorCameraDto {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorTransform2Dto {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorTransform3Dto {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorBounds2Dto {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorObjectPlacementKindDto {
    Transform2,
    TilemapMarker,
    Attached,
    UiLayout,
    ComputedRuntime,
    NotEditable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorObjectEditCommandKindDto {
    SetTransform2,
    SetTilemapMarkerOffset,
    SetAttachedLocalOffset,
    SetUiRect,
    SetTilemapOrigin,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneObjectDto {
    pub entity_id: String,
    pub name: String,
    pub visible: bool,
    pub selectable: bool,
    pub locked: bool,
    pub movable: bool,
    pub locked_reason: Option<String>,
    pub category: String,
    pub component_types: Vec<String>,
    pub placement_kind: EditorObjectPlacementKindDto,
    pub edit_command_kind: EditorObjectEditCommandKindDto,
    #[serde(rename = "transform2")]
    pub transform_2: Option<EditorTransform2Dto>,
    #[serde(rename = "transform3")]
    pub transform_3: Option<EditorTransform3Dto>,
    #[serde(rename = "bounds2")]
    pub bounds_2: Option<EditorBounds2Dto>,
    #[serde(rename = "renderBounds2")]
    pub render_bounds_2: Option<EditorBounds2Dto>,
    #[serde(rename = "selectionBounds2")]
    pub selection_bounds_2: Option<EditorBounds2Dto>,
    pub prefab_instance: Option<EditorPrefabInstanceDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPrefabInstanceDto {
    pub prefab_id: String,
    pub root_entity_id: String,
    pub is_prefab_root: bool,
    pub source_entity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorUiNodeKindDto {
    Panel,
    GroupBox,
    Row,
    Column,
    Stack,
    Text,
    Button,
    ProgressBar,
    Slider,
    Toggle,
    OptionSet,
    Dropdown,
    TabView,
    ColorPickerRgb,
    CurveEditor,
    Spacer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorUiNodeObjectDto {
    pub entity_id: String,
    pub component_index: usize,
    pub node_path: String,
    pub node_id: String,
    pub node_kind: EditorUiNodeKindDto,
    pub label: String,
    pub visible: bool,
    pub selectable: bool,
    pub locked: bool,
    #[serde(rename = "bounds2")]
    pub bounds_2: EditorBounds2Dto,
    #[serde(rename = "renderBounds2")]
    pub render_bounds_2: EditorBounds2Dto,
    pub action_event: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorGizmoKindDto {
    SelectionBounds2D,
    Move2D,
    Rotate2D,
    Scale2D,
    Rect2D,
    UiNodeBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorGizmoHandleKindDto {
    Body,
    AxisX,
    AxisY,
    PlaneXY,
    RotationRing,
    ScaleCornerNW,
    ScaleCornerNE,
    ScaleCornerSW,
    ScaleCornerSE,
    ScaleEdgeN,
    ScaleEdgeE,
    ScaleEdgeS,
    ScaleEdgeW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorGizmoToneDto {
    Neutral,
    Selection,
    X,
    Y,
    Rotation,
    Scale,
    Warning,
    HoverX,
    HoverY,
    Center,
    CenterHover,
    CenterActive,
    RotationHover,
    RotationActive,
    ScaleHover,
    ScaleActive,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorCursorIconDto {
    Default,
    Select,
    Move,
    MoveX,
    MoveY,
    Rotate,
    Scale,
    ScaleX,
    ScaleY,
    Rect,
    Pan,
    Grab,
    Grabbing,
    NotAllowed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorCursorDto {
    pub icon: EditorCursorIconDto,
    pub visible: bool,
    pub label: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorControlStateDto {
    Idle,
    Hovered,
    Active,
    Disabled,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorControlKindDto {
    Transform2D,
    Text2D,
    VectorShape2D,
    Sprite2D,
    TileMap2D,
    Camera2D,
    UiRect2D,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorControlSpaceDto {
    Scene2D,
    Screen,
    World3D,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorControlHandleDto {
    pub id: String,
    pub kind: EditorGizmoHandleKindDto,
    pub state: EditorControlStateDto,
    pub cursor: Option<EditorCursorDto>,
    pub hit_shape: EditorGizmoHitShapeDto,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorControlDto {
    pub id: String,
    pub kind: EditorControlKindDto,
    pub entity_id: Option<String>,
    pub component_type: Option<String>,
    pub space: EditorControlSpaceDto,
    pub state: EditorControlStateDto,
    pub gizmos: Vec<EditorGizmoDto>,
    pub handles: Vec<EditorControlHandleDto>,
    pub cursor: Option<EditorCursorDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorGizmoPointDto {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorGizmoRectDto {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "PascalCase",
    rename_all_fields = "camelCase"
)]
pub enum EditorGizmoPrimitiveDto {
    Line2D {
        from: EditorGizmoPointDto,
        to: EditorGizmoPointDto,
        tone: EditorGizmoToneDto,
    },
    Arrow2D {
        from: EditorGizmoPointDto,
        to: EditorGizmoPointDto,
        tone: EditorGizmoToneDto,
    },
    Rect2D {
        rect: EditorGizmoRectDto,
        tone: EditorGizmoToneDto,
    },
    Circle2D {
        center: EditorGizmoPointDto,
        radius: f32,
        tone: EditorGizmoToneDto,
    },
    Ring2D {
        center: EditorGizmoPointDto,
        inner_radius: f32,
        outer_radius: f32,
        tone: EditorGizmoToneDto,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "PascalCase",
    rename_all_fields = "camelCase"
)]
pub enum EditorGizmoHitShapeDto {
    Rect2D {
        rect: EditorGizmoRectDto,
    },
    Circle2D {
        center: EditorGizmoPointDto,
        radius: f32,
    },
    Ring2D {
        center: EditorGizmoPointDto,
        inner_radius: f32,
        outer_radius: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorGizmoHandleDto {
    pub id: String,
    pub kind: EditorGizmoHandleKindDto,
    pub cursor: Option<String>,
    pub hit_shape: EditorGizmoHitShapeDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorGizmoDto {
    pub id: String,
    pub kind: EditorGizmoKindDto,
    pub entity_id: Option<String>,
    pub primitives: Vec<EditorGizmoPrimitiveDto>,
    pub handles: Vec<EditorGizmoHandleDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneSnapshotDto {
    pub mod_id: String,
    pub scene_id: String,
    pub canvas_kind: EditorSceneCanvasKindDto,
    pub layout_source: EditorSceneSnapshotLayoutSourceDto,
    pub width: u32,
    pub height: u32,
    pub camera: EditorCameraDto,
    pub quality: EditorSceneSnapshotQualityDto,
    pub objects: Vec<EditorSceneObjectDto>,
    pub ui_nodes: Vec<EditorUiNodeObjectDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
    pub gizmos: Vec<EditorGizmoDto>,
    pub selection: EditorSelectionDto,
    pub tool_state: EditorToolStateDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenEditorModeSessionResultDto {
    pub session: EditorModeSessionDto,
    pub snapshot: EditorSceneSnapshotDto,
    pub frame: EditorFrameDto,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorFrameResultDto {
    pub ok: bool,
    pub session: Option<EditorModeSessionDto>,
    pub snapshot: Option<EditorSceneSnapshotDto>,
    pub frame: Option<EditorFrameDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPointerModifiersDto {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPointerEventDto {
    #[serde(rename = "type")]
    pub r#type: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub scene_x: Option<f32>,
    #[serde(default)]
    pub scene_y: Option<f32>,
    #[serde(default)]
    pub frame_x: Option<f32>,
    #[serde(default)]
    pub frame_y: Option<f32>,
    pub button: Option<i32>,
    pub buttons: Option<i32>,
    pub pointer_id: i32,
    pub delta_x: Option<f32>,
    pub delta_y: Option<f32>,
    pub modifiers: EditorPointerModifiersDto,
    pub viewport: EditorViewportDto,
}

impl EditorPointerEventDto {
    pub fn scene_x(&self) -> f32 {
        self.scene_x.unwrap_or(self.x)
    }

    pub fn scene_y(&self) -> f32 {
        self.scene_y.unwrap_or(self.y)
    }

    pub fn frame_x(&self) -> Option<f32> {
        self.frame_x
    }

    pub fn frame_y(&self) -> Option<f32> {
        self.frame_y
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorViewportPointDto {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EditorCommandDto {
    #[serde(rename = "SelectEntity", rename_all = "camelCase")]
    SelectEntity { scene_id: String, entity_id: String },
    #[serde(rename = "SetEntityTransform2D", rename_all = "camelCase")]
    SetEntityTransform2D {
        scene_id: String,
        entity_id: String,
        transform: EditorTransform2Dto,
    },
    #[serde(rename = "MoveEntity2D", rename_all = "camelCase")]
    MoveEntity2D {
        scene_id: String,
        entity_id: String,
        dx: f32,
        dy: f32,
    },
    #[serde(rename = "SetTileMapMarker2D", rename_all = "camelCase")]
    SetTileMapMarker2D {
        scene_id: String,
        entity_id: String,
        offset: EditorViewportPointDto,
    },
    #[serde(rename = "SetAttachedLocalOffset2D", rename_all = "camelCase")]
    SetAttachedLocalOffset2D {
        scene_id: String,
        entity_id: String,
        local_offset: EditorViewportPointDto,
    },
    #[serde(rename = "SetUiNodeProperty", rename_all = "camelCase")]
    SetUiNodeProperty {
        scene_id: String,
        entity_id: String,
        component_index: usize,
        node_path: String,
        property_path: String,
        value: EditorUiNodePropertyValueDto,
    },
    #[serde(rename = "CreateUiDocument", rename_all = "camelCase")]
    CreateUiDocument {
        scene_id: String,
        entity_id: String,
        label: String,
        viewport_width: f32,
        viewport_height: f32,
        template: EditorUiTemplateKindDto,
    },
    #[serde(rename = "AddUiNode", rename_all = "camelCase")]
    AddUiNode {
        scene_id: String,
        entity_id: String,
        component_index: usize,
        parent_path: String,
        node: EditorUiNodeCreateDto,
        insert_index: Option<usize>,
    },
    #[serde(rename = "AddUiTemplate", rename_all = "camelCase")]
    AddUiTemplate {
        scene_id: String,
        entity_id: String,
        component_index: usize,
        parent_path: String,
        template: EditorUiTemplateKindDto,
        id_prefix: String,
        insert_index: Option<usize>,
    },
    #[serde(rename = "DuplicateUiNode", rename_all = "camelCase")]
    DuplicateUiNode {
        scene_id: String,
        entity_id: String,
        component_index: usize,
        node_path: String,
        new_id: Option<String>,
        copy_actions: bool,
    },
    #[serde(rename = "RemoveUiNode", rename_all = "camelCase")]
    RemoveUiNode {
        scene_id: String,
        entity_id: String,
        component_index: usize,
        node_path: String,
    },
    #[serde(rename = "MoveUiNode", rename_all = "camelCase")]
    MoveUiNode {
        scene_id: String,
        entity_id: String,
        component_index: usize,
        node_path: String,
        direction: EditorUiNodeMoveDirectionDto,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorCommandResultDto {
    pub ok: bool,
    pub scene_dirty: bool,
    pub changed_entities: Vec<String>,
    pub snapshot: Option<EditorSceneSnapshotDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
    pub message: Option<String>,
}
