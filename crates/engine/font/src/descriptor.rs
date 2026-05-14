use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use amigo_assets::{AssetCatalog, AssetKey, PreparedAsset, PreparedAssetKind};

use crate::{
    Font2dAsset, Font2dFormat, Font2dMetrics, Font2dSource, FontFallbackPolicy, FontGlyphPreset,
    FontGlyphSet,
};

pub fn font2d_asset_from_prepared(prepared: &PreparedAsset) -> Option<Font2dAsset> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Font2d | PreparedAssetKind::Font3d
    ) {
        return None;
    }

    let format = prepared
        .format
        .as_deref()
        .or_else(|| prepared.metadata.get("format").map(String::as_str))
        .map(Font2dFormat::parse)
        .unwrap_or(Font2dFormat::DebugPlaceholder);

    let source = font_source_from_prepared(prepared);

    let glyphs = FontGlyphSet {
        preset: prepared
            .metadata
            .get("glyphs.preset")
            .or_else(|| prepared.metadata.get("glyph_preset"))
            .map(|value| FontGlyphPreset::parse(value))
            .unwrap_or(FontGlyphPreset::ConsoleLatinExt),
        extra: prepared
            .metadata
            .get("glyphs.extra")
            .or_else(|| prepared.metadata.get("extra_glyphs"))
            .cloned()
            .unwrap_or_default(),
    };

    let metrics = Font2dMetrics {
        default_size: metadata_f32(prepared, "metrics.default_size")
            .or_else(|| metadata_f32(prepared, "default_size"))
            .unwrap_or(12.0)
            .max(1.0),
        line_height: metadata_f32(prepared, "metrics.line_height")
            .or_else(|| metadata_f32(prepared, "line_height")),
        letter_spacing: metadata_f32(prepared, "metrics.letter_spacing")
            .or_else(|| metadata_f32(prepared, "letter_spacing"))
            .unwrap_or(0.0),
        tab_width: metadata_usize(prepared, "metrics.tab_width")
            .or_else(|| metadata_usize(prepared, "tab_width"))
            .unwrap_or(4)
            .max(1),
    };

    let fallback = FontFallbackPolicy {
        missing_glyph: prepared
            .metadata
            .get("fallback.missing_glyph")
            .or_else(|| prepared.metadata.get("missing_glyph"))
            .and_then(|value| value.chars().next())
            .unwrap_or('?'),
    };

    Some(Font2dAsset {
        key: prepared.key.clone(),
        label: prepared.label.clone(),
        format,
        source,
        glyphs,
        metrics,
        fallback,
    })
}

pub fn font2d_asset_from_catalog(catalog: &AssetCatalog, key: &AssetKey) -> Option<Font2dAsset> {
    let mut visited = BTreeSet::new();
    font2d_asset_from_catalog_inner(catalog, key, &mut visited, 0)
}

fn font2d_asset_from_catalog_inner(
    catalog: &AssetCatalog,
    key: &AssetKey,
    visited: &mut BTreeSet<AssetKey>,
    depth: usize,
) -> Option<Font2dAsset> {
    if depth > 4 || !visited.insert(key.clone()) {
        return None;
    }

    let prepared = catalog.prepared_asset(key)?;
    let mut asset = font2d_asset_from_prepared(&prepared)?;
    let Font2dSource::AssetRef { key: source_key } = asset.source.clone() else {
        return Some(asset);
    };

    let mut resolved = font2d_asset_from_catalog_inner(catalog, &source_key, visited, depth + 1)?;
    resolved.key = asset.key.clone();
    resolved.label = asset.label.take().or(resolved.label);
    Some(resolved)
}

fn font_source_from_prepared(prepared: &PreparedAsset) -> Font2dSource {
    if let Some(id) = prepared.metadata.get("source.embedded") {
        if !id.trim().is_empty() {
            return Font2dSource::Embedded { id: id.clone() };
        }
    }

    if let Some(key) = prepared
        .metadata
        .get("source.asset")
        .or_else(|| prepared.metadata.get("source.font"))
        .cloned()
    {
        if !key.trim().is_empty() {
            return Font2dSource::AssetRef {
                key: AssetKey::new(key),
            };
        }
    }

    let Some(relative) = prepared
        .metadata
        .get("source.file")
        .or_else(|| prepared.metadata.get("source"))
        .or_else(|| prepared.metadata.get("file"))
        .or_else(|| prepared.metadata.get("font"))
        .cloned()
    else {
        return Font2dSource::Missing;
    };

    if relative.trim().is_empty() {
        return Font2dSource::Missing;
    }

    let Some(parent) = prepared.resolved_path.parent() else {
        return Font2dSource::Missing;
    };

    Font2dSource::File {
        relative_path: relative.clone(),
        resolved_path: normalize_path(parent.join(relative)),
    }
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Option<f32> {
    prepared.metadata.get(key)?.parse::<f32>().ok()
}

fn metadata_usize(prepared: &PreparedAsset, key: &str) -> Option<usize> {
    prepared.metadata.get(key)?.parse::<usize>().ok()
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

#[allow(dead_code)]
fn is_font_source_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "ttf" | "otf" | "ttc"
            )
        })
        .unwrap_or(false)
}
