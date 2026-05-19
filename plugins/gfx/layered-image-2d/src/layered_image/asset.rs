use amigo_assets::{AssetCatalog, AssetKey, PreparedAsset, PreparedAssetKind};
use amigo_math::{ColorRgba, Vec2};

use super::{
    LayeredImageAsset, LayeredImageBlendMode2d, LayeredImageLayer, LayeredImageLayerOverride,
};

pub trait LayeredImageAssetSource {
    fn layered_image_asset(&self, key: &AssetKey) -> Option<LayeredImageAsset>;
}

impl LayeredImageAssetSource for AssetCatalog {
    fn layered_image_asset(&self, key: &AssetKey) -> Option<LayeredImageAsset> {
        self.prepared_asset(key)
            .and_then(|prepared| infer_layered_image_asset_from_prepared(&prepared))
    }
}

pub fn infer_layered_image_asset_from_prepared(
    prepared: &PreparedAsset,
) -> Option<LayeredImageAsset> {
    if !matches!(prepared.kind, PreparedAssetKind::LayeredImage2d) {
        return None;
    }

    let canvas_width = metadata_f32(prepared, "canvas.width")?;
    let canvas_height = metadata_f32(prepared, "canvas.height")?;
    let base_image = metadata_string(prepared, "base.image")
        .or_else(|| metadata_string(prepared, "base.file"))
        .or_else(|| metadata_string(prepared, "base"))?;

    let mut layers = Vec::new();
    for index in 0..infer_indexed_count(prepared, "layers") {
        let prefix = format!("layers.{index}");
        let id = metadata_string(prepared, &format!("{prefix}.id"))
            .unwrap_or_else(|| format!("layer_{index:03}"));
        let Some(image) = metadata_string(prepared, &format!("{prefix}.image")) else {
            continue;
        };
        let label =
            metadata_string(prepared, &format!("{prefix}.label")).unwrap_or_else(|| id.clone());
        let blend_mode = metadata_string(prepared, &format!("{prefix}.blend"))
            .map(|value| LayeredImageBlendMode2d::parse(&value))
            .unwrap_or(LayeredImageBlendMode2d::Additive);
        let opacity = metadata_f32(prepared, &format!("{prefix}.default_opacity"))
            .unwrap_or(1.0)
            .clamp(0.0, 4.0);

        layers.push(LayeredImageLayer {
            id,
            label,
            image,
            blend_mode,
            opacity,
            color: metadata_string(prepared, &format!("{prefix}.color"))
                .and_then(|value| parse_hex_rgba(&value)),
            animation_hint: metadata_string(prepared, &format!("{prefix}.animation_hint")),
            post_fx: amigo_2d_post_fx::cached_image_post_fx_stack_from_flat_metadata(
                &prepared.metadata,
                &format!("{prefix}.post_fx"),
            ),
            enabled: metadata_bool(prepared, &format!("{prefix}.enabled")).unwrap_or(true),
        });
    }

    Some(LayeredImageAsset {
        key: prepared.key.clone(),
        label: prepared.label.clone(),
        canvas_size: Vec2::new(canvas_width, canvas_height),
        base_image,
        layers,
        preview_image: metadata_string(prepared, "preview.image"),
    })
}

pub fn apply_layer_overrides(
    asset: &mut LayeredImageAsset,
    overrides: &[LayeredImageLayerOverride],
) {
    for override_ in overrides {
        let Some(layer) = asset
            .layers
            .iter_mut()
            .find(|layer| layer.id == override_.id)
        else {
            continue;
        };
        if let Some(opacity) = override_.opacity {
            layer.opacity = opacity.clamp(0.0, 4.0);
        }
        if let Some(enabled) = override_.enabled {
            layer.enabled = enabled;
        }
        if let Some(blend_mode) = override_.blend_mode {
            layer.blend_mode = blend_mode;
        }
    }
}

fn infer_indexed_count(prepared: &PreparedAsset, prefix: &str) -> usize {
    let prefix = format!("{prefix}.");
    prepared
        .metadata
        .keys()
        .filter_map(|key| key.strip_prefix(&prefix))
        .filter_map(|rest| rest.split_once('.').map(|(index, _)| index))
        .filter_map(|index| index.parse::<usize>().ok())
        .max()
        .map_or(0, |index| index + 1)
}

fn metadata_string(prepared: &PreparedAsset, key: &str) -> Option<String> {
    prepared
        .metadata
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Option<f32> {
    prepared.metadata.get(key)?.parse::<f32>().ok()
}

fn metadata_bool(prepared: &PreparedAsset, key: &str) -> Option<bool> {
    prepared.metadata.get(key)?.parse::<bool>().ok()
}

fn parse_hex_rgba(value: &str) -> Option<ColorRgba> {
    let hex = value.trim().trim_start_matches('#');
    let parse = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&hex[range], 16)
            .ok()
            .map(|v| v as f32 / 255.0)
    };

    match hex.len() {
        6 => Some(ColorRgba::new(
            parse(0..2)?,
            parse(2..4)?,
            parse(4..6)?,
            1.0,
        )),
        8 => Some(ColorRgba::new(
            parse(0..2)?,
            parse(2..4)?,
            parse(4..6)?,
            parse(6..8)?,
        )),
        _ => None,
    }
}
