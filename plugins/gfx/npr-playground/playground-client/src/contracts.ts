/** Wire contracts shared by the Drawing Studio client and the NPR plugin. */
export type Vec3 = [number, number, number];
export type Vec4 = [number, number, number, number];

export type PresentationMode = 'native_gpu' | 'local_rgba' | 'jpeg';
export type GeometrySource =
  | 'paper' | 'wash' | 'flat-fill' | 'shadow-hatch' | 'silhouette'
  | 'creases' | 'form-lines' | 'construction';
export type BlendMode = 'normal' | 'multiply' | 'screen';
export type StrokeTool = 'pencil' | 'fineliner' | 'nib' | 'brush';
export type BrushMedium = 'ink' | 'graphite' | 'hatching' | 'flat-fill' | 'watercolour-wash';
export type BrushApplication = 'stroke' | 'surface';

export type GeometryTarget =
  | { kind: 'all' }
  | { kind: 'objects'; objects: string[] }
  | { kind: 'surface-features'; features: string[] };

export type CoverageMask =
  | { kind: 'none' }
  | { kind: 'tone-range' | 'height'; min: number; max: number; invert: boolean }
  | { kind: 'normal-direction'; direction: Vec3; threshold: number; invert: boolean }
  | { kind: 'noise'; amount: number; seed: number; invert: boolean }
  | { kind: 'multiply'; masks: CoverageMask[] };

export interface BrushReference { id: string; version: number }
export interface BrushInstance {
  brush: BrushReference;
  width?: number;
  taper?: number;
  softness?: number;
  irregularity?: number;
  dryness?: number;
  seed?: number;
  pressure_profile?: number;
  correction?: number;
  spacing?: number;
}
export interface BrushDefinition {
  id: string; name: string; version: number;
  medium: BrushMedium;
  applications: BrushApplication[];
  tool?: StrokeTool;
  paint?: PaintMedium;
  width: number;
  taper: number;
  softness: number;
  spacing: number;
  pressure_profile: number;
  irregularity: number;
  correction: number;
  dryness: number;
  seed: number;
}

export interface PaintMedium { wash: number; granulation: number }
export interface HatchSettings { density: number; spacing: number; angle?: number; cross?: number }
export interface LineSettings { crease_angle?: number; min_length_pixels?: number; simplification?: number }
export type LayerColorSource =
  | { kind: 'style-palette' }
  | { kind: 'model-base-color' }
  | { kind: 'constant'; color: Vec4 };

export interface NprStyleLayer {
  id: string;
  label: string;
  source: GeometrySource;
  enabled: boolean;
  opacity: number;
  blend: BlendMode;
  color_source: LayerColorSource;
  tool?: StrokeTool;
  paint?: PaintMedium;
  hatch?: HatchSettings;
  line?: LineSettings;
  brush?: BrushInstance;
  target: GeometryTarget;
  mask: CoverageMask;
}
export interface NprStyleLayers { layers: NprStyleLayer[] }

export interface ComicInk {
  tool: StrokeTool;
  tone_mode: 'three-band' | 'hatching';
  surface_mode: 'polygonal' | 'smooth';
  light_direction: Vec3;
  ink: Vec4; crease_angle: number; smooth_crease_angle: number; smooth_draw_creases: boolean;
  paper: Vec4; shadow: Vec4; mid: Vec4; light: Vec4;
  outline_width: number; crease_width: number; boundary_width: number;
  min_crease_length_pixels: number; min_smooth_contour_length_pixels: number;
  smooth_contour_simplification_pixels: number; taper: number; wobble: number;
  gesture_confidence: number; gesture_simplification: number; gesture_correction: number;
  gesture_overstroke: number; tool_pressure: number; tool_hardness: number;
  paper_tooth: number; paper_grain: number; nib_angle: number; nib_aspect: number;
  ink_dryness: number; tone_density: number; min_form_line_confidence: number;
  suggestive_contours: boolean; suggestive_contour_confidence: number;
  suggestive_contour_width_scale: number; suggestive_contour_opacity: number;
  form_line_width_scale: number; form_line_opacity: number;
  hatching_angle: number; hatching_spacing: number; hatching_cross: number;
}

export interface ObjectSettings {
  model: string; material_base_color: Vec4; surface_intent: 'hard-surface' | 'organic' | 'authored';
  surface_mode: 'polygonal' | 'smooth'; surface_subdivision_level: number;
  smooth_weld_relative_tolerance: number; visible: boolean; rotating: boolean;
  position: Vec3; rotation: Vec3; scale: number; angular_speed: Vec3; gesture_variant: number;
  style_overrides: Record<string, unknown>; style_layer_overrides: Record<string, unknown>;
  construction_marks: unknown[];
}
export interface NprSettings {
  global: ComicInk; style_layers: NprStyleLayers; brushes: BrushLibrary;
  objects: Record<string, ObjectSettings>; selected: string; paused: boolean;
  sketch_paused: boolean; speed: number; step: boolean; motion: MotionPolicy;
  seed: number; debug: 'Final' | 'FeatureClasses' | 'StrokeIds'; camera_target: Vec3;
  camera_yaw: number; camera_pitch: number; camera_distance: number; camera_fov: number;
  preset_name: string; style_scope: 'scene' | 'object'; preset_kind: 'scene' | 'look';
}
export interface MotionPolicy {
  mode: 'stable' | 'redraw-on-motion' | 'redraw-continuously';
  appearance_fade_seconds: number; redraw_hz: number; redraw_strength: number;
}
export interface BrushLibrary { brushes: Record<string, BrushDefinition[]> }

export interface LayerDiagnostics {
  source_geometry: number; generated_marks: number; generated_triangles: number;
  mask_coverage: number; extraction_micros: number; tessellation_micros: number;
  no_effect_reason?: 'disabled' | 'no-source-geometry' | 'mask-excludes-all' | 'missing-coverage-inputs' | 'zero-opacity' | 'hatch-selection-excludes-all' | 'target-excludes-geometry';
}
export interface ModelDescriptor {
  id: string; label: string; source: string; state: 'unloaded' | 'loading' | 'ready' | 'failed';
  kind: string; error?: string; thumbnail?: string;
}
export interface NprDrawingDraft { version: number; source_model: string; settings: NprSettings }
export interface NprSnapshot {
  revision: number; settings: NprSettings; dirty: boolean; can_undo: boolean; can_redo: boolean;
  look_dirty: boolean; preview_layer: string | null;
  scene: string | null; active_look: string | null; available_looks: string[]; included_looks: string[];
  locked_layers: string[]; metadata: Record<string, unknown>; brushes: BrushLibrary;
  variants: Record<string, NprSettings>; drafts: Record<string, NprDrawingDraft>;
  layer_diagnostics: Record<string, LayerDiagnostics>; build_up: number; solo_layer: string | null;
}
export interface LookCatalogEntry {
  id: string;
  includes: string[];
  preview: string | null;
  preview_error: string | null;
  resolved_layers: NprStyleLayers;
  brush_references: BrushReference[];
}
export interface PlaygroundValues {
  playback?: ModelPlayback;
  animation_clips?: AnimationClip[];
  npr?: NprSnapshot;
  models?: ModelDescriptor[];
  look_catalog?: Record<string, LookCatalogEntry>;
  brush_previews?: Record<string, string | null>;
  brush_preview_errors?: Record<string, string>;
  layer_previews?: Record<string, string>;
  layer_preview_errors?: Record<string, string>;
  catalogs?: unknown[];
  transfer?: boolean;
  diagnostics?: Record<string, number>;
  [key: string]: unknown;
}

export type PlaybackSource = { kind: 'turntable' } | { kind: 'clip'; index: number };
export type PlaybackCommand =
  | { kind: 'source'; source: PlaybackSource }
  | { kind: 'playing'; playing: boolean }
  | { kind: 'seek'; seconds: number }
  | { kind: 'options'; looping: boolean; speed: number };
export interface ModelPlayback {
  source: PlaybackSource; playing: boolean; time_seconds: number;
  duration_seconds: number | null; looping: boolean; speed: number;
}
export interface AnimationTrack {
  node: number; node_path: string;
  property: 'translation' | 'rotation' | 'scale' | 'weights';
  interpolation: 'step' | 'linear' | 'cubic-spline'; keyframes: number;
}
export interface AnimationClip {
  index: number; name: string; duration_seconds: number; tracks: AnimationTrack[];
}

export type NavigationMode = 'orbit' | 'pan' | 'zoom' | 'select' | 'fit' | 'focus' | 'reset';
/** Hand-written domain intents; authored state remains backend-owned. */
export type NprIntent =
  | { kind: 'begin_camera_gesture' | 'end_camera_gesture'; gesture_id: number }
  | { kind: 'navigate'; mode: NavigationMode; dx: number; dy: number; wheel: number; x: number; y: number; width: number; height: number; focused: boolean }
  | { kind: 'select'; object: string }
  | { kind: 'set_camera'; target: Vec3; yaw: number; pitch: number; distance: number; fov: number }
  | { kind: 'set_seed'; seed: number }
  | { kind: 'set_motion'; paused: boolean; speed: number; sketch_paused: boolean }
  | { kind: 'playback'; command: PlaybackCommand }
  | { kind: 'set_temporal_policy'; policy: MotionPolicy }
  | { kind: 'set_debug_view'; view: NprSettings['debug'] }
  | { kind: 'set_layer_lock'; layer: string; locked: boolean }
  | { kind: 'use_look' | 'save_as_look' | 'save_variant' | 'apply_variant'; id: string }
  | { kind: 'set_object'; object: string; settings: ObjectSettings }
  | { kind: 'set_object_pose'; object: string; position: Vec3; rotation: Vec3; scale: number }
  | { kind: 'set_look'; style: ComicInk; layers: NprStyleLayers }
  | { kind: 'add_layer'; source: GeometrySource; label?: string; brush?: BrushInstance; target?: GeometryTarget; mask?: CoverageMask }
  | { kind: 'replace_layer' | 'preview_layer'; layer: string; value: NprStyleLayer }
  | { kind: 'save_appearance'; layer: string; value: NprStyleLayer; name: string; update_matching?: boolean }
  | { kind: 'set_layer_enabled'; layer: string; enabled: boolean }
  | { kind: 'set_layer_brush'; layer: string; brush?: BrushInstance }
  | { kind: 'duplicate_layer' | 'delete_layer'; layer: string }
  | { kind: 'move_layer'; layer: string; direction: number }
  | { kind: 'select_model' | 'open_draft'; model: string }
  | { kind: 'update_brush_version'; id: string; from: number; to: number }
  | { kind: 'save_brush_version'; brush: BrushDefinition }
  | { kind: 'save_brush_version_and_pin'; brush: BrushDefinition; layer: string; update_matching?: boolean }
  | { kind: 'set_build_up'; value: number }
  | { kind: 'set_solo_layer'; layer: string | null }
  | { kind: 'refresh_catalog'; catalog: string }
  | { kind: 'import_remote'; catalog: string; model: string }
  | { kind: 'import_model'; path: string }
  | { kind: 'cancel_layer_preview' | 'undo' | 'redo' | 'save_all' | 'save_look' | 'reload' };

export interface CompanionBootstrap {
  endpoint: string; version: number; playground: string; token: string; native_error?: string | null;
}
