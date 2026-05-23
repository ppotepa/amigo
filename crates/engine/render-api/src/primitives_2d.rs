use amigo_assets::AssetKey;
use amigo_material_api::{Material2d, MaterialCoverageKind2d};
use amigo_math::{ColorRgba, Transform2, Vec2};

use crate::RenderContributionSet;

#[derive(Debug, Clone, PartialEq)]
pub struct RenderMaterialBinding2d {
    pub material: Option<Material2d>,
    pub contributions: RenderContributionSet,
    pub coverage_kind: MaterialCoverageKind2d,
}

impl RenderMaterialBinding2d {
    pub fn none(coverage_kind: MaterialCoverageKind2d) -> Self {
        Self {
            material: None,
            contributions: RenderContributionSet::default(),
            coverage_kind,
        }
    }

    pub fn new(
        material: Option<Material2d>,
        contributions: RenderContributionSet,
        coverage_kind: MaterialCoverageKind2d,
    ) -> Self {
        Self {
            material,
            contributions,
            coverage_kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TexturedQuad2dSheet {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualMaps2dPrimitive {
    pub normal: Option<AssetKey>,
    pub wetness: Option<AssetKey>,
    pub emissive: Option<AssetKey>,
    pub highlight: Option<AssetKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TexturedQuad2dPrimitive {
    pub texture: AssetKey,
    pub size: Vec2,
    pub transform: Transform2,
    pub sheet: Option<TexturedQuad2dSheet>,
    pub frame_index: u32,
    pub visual_maps: Option<VisualMaps2dPrimitive>,
    pub material: RenderMaterialBinding2d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderPrimitive2dKind {
    TexturedQuad,
    GlyphRun,
    VectorMesh,
    TileBatch,
    LayeredTexturedQuads,
    RadialLightVisual,
    ParticleBatch,
}

impl RenderPrimitive2dKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TexturedQuad => "textured_quad_2d",
            Self::GlyphRun => "glyph_run_2d",
            Self::VectorMesh => "vector_mesh_2d",
            Self::TileBatch => "tile_batch_2d",
            Self::LayeredTexturedQuads => "layered_textured_quads_2d",
            Self::RadialLightVisual => "radial_light_visual_2d",
            Self::ParticleBatch => "particle_batch_2d",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphRun2dBlendMode {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphRun2dShadow {
    pub color: ColorRgba,
    pub offset: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphRun2dOutline {
    pub color: ColorRgba,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphRun2dGlow {
    pub color: ColorRgba,
    pub radius: f32,
    pub intensity: f32,
    pub passes: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun2dPrimitive {
    pub font: AssetKey,
    pub text: String,
    pub bounds: Vec2,
    pub transform: Transform2,
    pub color: ColorRgba,
    pub font_size: Option<f32>,
    pub blend: GlyphRun2dBlendMode,
    pub shadow: Option<GlyphRun2dShadow>,
    pub outline: Option<GlyphRun2dOutline>,
    pub glow: Option<GlyphRun2dGlow>,
    pub material: RenderMaterialBinding2d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorShape2dViewportFit {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorShape2dStylePrimitive {
    pub stroke_color: ColorRgba,
    pub stroke_width: f32,
    pub fill_color: Option<ColorRgba>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VectorShape2dKindPrimitive {
    Polyline { points: Vec<Vec2>, closed: bool },
    Polygon { points: Vec<Vec2> },
    Circle { radius: f32, segments: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape2dPrimitive {
    pub shape: VectorShape2dKindPrimitive,
    pub style: VectorShape2dStylePrimitive,
    pub transform: Transform2,
    pub viewport_fit: VectorShape2dViewportFit,
    pub viewport_canvas_size: Option<Vec2>,
    pub material: RenderMaterialBinding2d,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileMapResolvedTile2dPrimitive {
    pub symbol: char,
    pub tile_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileMapResolved2dPrimitive {
    pub rows: Vec<Vec<TileMapResolvedTile2dPrimitive>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMap2dPrimitive {
    pub tileset: AssetKey,
    pub tile_size: Vec2,
    pub grid: Vec<String>,
    pub origin_offset: Vec2,
    pub resolved: Option<TileMapResolved2dPrimitive>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayeredImageBlendMode2dPrimitive {
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayeredImageViewportFit2dPrimitive {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageLayerOverride2dPrimitive {
    pub id: String,
    pub opacity: Option<f32>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<LayeredImageBlendMode2dPrimitive>,
    pub visual_maps: Option<VisualMaps2dPrimitive>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImage2dPrimitive {
    pub asset: AssetKey,
    pub size: Vec2,
    pub base_opacity: f32,
    pub viewport_fit: LayeredImageViewportFit2dPrimitive,
    pub transform: Transform2,
    pub visual_maps: Option<VisualMaps2dPrimitive>,
    pub layer_overrides: Vec<LayeredImageLayerOverride2dPrimitive>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeaconLight2dPrimitive {
    pub center: Vec2,
    pub color: ColorRgba,
    pub intensity: f32,
    pub pulse: f32,
    pub core_radius_px: f32,
    pub halo_radius_px: f32,
    pub glow_strength: f32,
    pub rotation_radians: f32,
    pub beam_enabled: bool,
    pub beam_length_px: f32,
    pub beam_width_degrees: f32,
    pub beam_strength: f32,
    pub aberration_px: f32,
    pub bloom: f32,
    pub camera_intensity: f32,
    pub camera_glare: f32,
    pub overlay_visible: bool,
    pub distance_m: Option<f32>,
    pub z_depth: Option<f32>,
    pub viewport_fit: LayeredImageViewportFit2dPrimitive,
    pub viewport_canvas_size: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleShape2dPrimitive {
    Circle { segments: u32 },
    Quad,
    Line { length: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLineAnchor2dPrimitive {
    Center,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleBlendMode2dPrimitive {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleMotionStretch2dPrimitive {
    pub enabled: bool,
    pub velocity_scale: f32,
    pub max_length: f32,
    pub shutter_seconds: f32,
    pub tail_alpha: f32,
    pub head_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLightMode2dPrimitive {
    Source,
    Particle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleLight2dPrimitive {
    pub radius: f32,
    pub intensity: f32,
    pub mode: ParticleLightMode2dPrimitive,
    pub glow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSampleStrategy2dPrimitive {
    Point,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightReceiverDarkPolicy2dPrimitive {
    Transparent,
    BaseColor,
    ShadowTint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightReceiverGlobalLight2dPrimitive {
    pub id: String,
    pub response: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightReceiver2dBindingPrimitive {
    pub groups: Vec<String>,
    pub source: String,
    pub channel: String,
    pub sample_strategy: LightSampleStrategy2dPrimitive,
    pub sample_points: u32,
    pub radius_px: f32,
    pub exposure: f32,
    pub dark_policy: LightReceiverDarkPolicy2dPrimitive,
    pub global_lights: Vec<LightReceiverGlobalLight2dPrimitive>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleMaterialLightingMode2dPrimitive {
    Unlit,
    DynamicLights,
    LightMapSampled,
    LightGroupSampled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleMaterial2dPrimitive {
    pub lighting_mode: ParticleMaterialLightingMode2dPrimitive,
    pub receives_light: bool,
    pub light_response: f32,
    pub light_receiver: Option<LightReceiver2dBindingPrimitive>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Particle2dPrimitive {
    pub emitter_entity_name: String,
    pub render_layer: String,
    pub previous_position: Vec2,
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: f32,
    pub color: ColorRgba,
    pub shape: ParticleShape2dPrimitive,
    pub line_anchor: ParticleLineAnchor2dPrimitive,
    pub blend_mode: ParticleBlendMode2dPrimitive,
    pub motion_stretch: Option<ParticleMotionStretch2dPrimitive>,
    pub material: ParticleMaterial2dPrimitive,
    pub light: Option<ParticleLight2dPrimitive>,
    pub light_position: Option<Vec2>,
    pub transform: Transform2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderPrimitive2d {
    TexturedQuad(TexturedQuad2dPrimitive),
    GlyphRun(GlyphRun2dPrimitive),
    VectorMesh(VectorShape2dPrimitive),
    TileBatch(TileMap2dPrimitive),
    LayeredTexturedQuads(LayeredImage2dPrimitive),
    RadialLightVisual(BeaconLight2dPrimitive),
    ParticleBatch(Particle2dPrimitive),
}

impl RenderPrimitive2d {
    pub fn kind(&self) -> RenderPrimitive2dKind {
        match self {
            Self::TexturedQuad(_) => RenderPrimitive2dKind::TexturedQuad,
            Self::GlyphRun(_) => RenderPrimitive2dKind::GlyphRun,
            Self::VectorMesh(_) => RenderPrimitive2dKind::VectorMesh,
            Self::TileBatch(_) => RenderPrimitive2dKind::TileBatch,
            Self::LayeredTexturedQuads(_) => RenderPrimitive2dKind::LayeredTexturedQuads,
            Self::RadialLightVisual(_) => RenderPrimitive2dKind::RadialLightVisual,
            Self::ParticleBatch(_) => RenderPrimitive2dKind::ParticleBatch,
        }
    }

    pub fn transform(&self) -> Transform2 {
        match self {
            Self::TexturedQuad(primitive) => primitive.transform,
            Self::GlyphRun(primitive) => primitive.transform,
            Self::VectorMesh(primitive) => primitive.transform,
            Self::TileBatch(primitive) => Transform2 {
                translation: primitive.origin_offset,
                ..Transform2::default()
            },
            Self::LayeredTexturedQuads(primitive) => primitive.transform,
            Self::RadialLightVisual(_) => Transform2::default(),
            Self::ParticleBatch(primitive) => primitive.transform,
        }
    }

    pub fn material_binding(&self) -> Option<&RenderMaterialBinding2d> {
        match self {
            Self::TexturedQuad(primitive) => Some(&primitive.material),
            Self::GlyphRun(primitive) => Some(&primitive.material),
            Self::VectorMesh(primitive) => Some(&primitive.material),
            _ => None,
        }
    }

    pub fn layered_textured_quads(&self) -> Option<&LayeredImage2dPrimitive> {
        match self {
            Self::LayeredTexturedQuads(primitive) => Some(primitive),
            _ => None,
        }
    }

    pub fn textured_quad(&self) -> Option<&TexturedQuad2dPrimitive> {
        match self {
            Self::TexturedQuad(primitive) => Some(primitive),
            _ => None,
        }
    }

    pub fn glyph_run(&self) -> Option<&GlyphRun2dPrimitive> {
        match self {
            Self::GlyphRun(primitive) => Some(primitive),
            _ => None,
        }
    }

    pub fn vector_mesh(&self) -> Option<&VectorShape2dPrimitive> {
        match self {
            Self::VectorMesh(primitive) => Some(primitive),
            _ => None,
        }
    }

    pub fn particle_batch(&self) -> Option<&Particle2dPrimitive> {
        match self {
            Self::ParticleBatch(primitive) => Some(primitive),
            _ => None,
        }
    }

    pub fn radial_light_visual(&self) -> Option<&BeaconLight2dPrimitive> {
        match self {
            Self::RadialLightVisual(primitive) => Some(primitive),
            _ => None,
        }
    }

    pub fn proxy_quad_transform(&self) -> Option<Transform2> {
        match self {
            Self::TexturedQuad(primitive) => Some(primitive.transform),
            Self::GlyphRun(primitive) => Some(primitive.transform),
            Self::TileBatch(primitive) => Some(Transform2 {
                translation: primitive.origin_offset,
                ..Transform2::default()
            }),
            Self::LayeredTexturedQuads(primitive) => Some(primitive.transform),
            Self::RadialLightVisual(primitive) => Some(Transform2 {
                translation: primitive.center,
                rotation_radians: primitive.rotation_radians,
                scale: Vec2::new(1.0, 1.0),
            }),
            Self::ParticleBatch(primitive) => Some(Transform2 {
                translation: primitive.position,
                rotation_radians: primitive.transform.rotation_radians,
                scale: primitive.transform.scale,
            }),
            Self::VectorMesh(_) => None,
        }
    }

    pub fn proxy_quad_size(&self) -> Option<Vec2> {
        match self {
            Self::TexturedQuad(primitive) => Some(primitive.size),
            Self::GlyphRun(primitive) => Some(primitive.bounds),
            Self::TileBatch(primitive) => Some(Vec2::new(
                primitive
                    .grid
                    .iter()
                    .map(|row| row.chars().count())
                    .max()
                    .unwrap_or(1) as f32
                    * primitive.tile_size.x.max(1.0),
                primitive.grid.len().max(1) as f32 * primitive.tile_size.y.max(1.0),
            )),
            Self::LayeredTexturedQuads(primitive) => Some(primitive.size),
            Self::RadialLightVisual(primitive) => {
                let radius = primitive.halo_radius_px.max(primitive.core_radius_px) * 2.0;
                Some(Vec2::new(radius, radius))
            }
            Self::ParticleBatch(primitive) => {
                let size = primitive.size.max(1.0);
                Some(Vec2::new(size, size))
            }
            Self::VectorMesh(_) => None,
        }
    }

    pub fn proxy_quad_color(&self) -> Option<ColorRgba> {
        match self {
            Self::GlyphRun(primitive) => Some(primitive.color),
            Self::RadialLightVisual(primitive) => Some(primitive.color),
            Self::ParticleBatch(primitive) => Some(primitive.color),
            _ => None,
        }
    }
}
