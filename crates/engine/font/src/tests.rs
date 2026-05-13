use std::collections::BTreeMap;
use std::path::PathBuf;

use amigo_assets::{AssetCatalog, AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};

use crate::{Font2dFormat, Font2dSource, font2d_asset_from_catalog, font2d_asset_from_prepared};

#[test]
fn parses_truetype_font_asset_from_prepared_metadata() {
    let prepared = PreparedAsset {
        key: AssetKey::new("test/fonts/console-mono"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/fonts/console-mono/font.yml"),
        byte_len: 123,
        kind: PreparedAssetKind::Font2d,
        label: Some("Console Mono".to_owned()),
        format: Some("truetype".to_owned()),
        metadata: BTreeMap::from([
            ("kind".to_owned(), "font-2d".to_owned()),
            ("format".to_owned(), "truetype".to_owned()),
            ("source.file".to_owned(), "console-mono.ttf".to_owned()),
            ("glyphs.preset".to_owned(), "console-latin-ext".to_owned()),
            ("glyphs.extra".to_owned(), "✓✗".to_owned()),
            ("metrics.default_size".to_owned(), "12".to_owned()),
            ("metrics.line_height".to_owned(), "15".to_owned()),
        ]),
    };

    let asset = font2d_asset_from_prepared(&prepared).expect("font should parse");
    assert_eq!(asset.format, Font2dFormat::TrueType);
    assert_eq!(asset.metrics.default_size, 12.0);
    assert_eq!(asset.metrics.line_height, Some(15.0));
    assert!(asset.glyphs.characters('?').contains(&'ą'));
    assert!(asset.glyphs.characters('?').contains(&'✓'));

    match asset.source {
        Font2dSource::File { resolved_path, .. } => {
            assert_eq!(
                resolved_path,
                PathBuf::from("mods/test/fonts/console-mono/console-mono.ttf")
            );
        }
        other => panic!("expected file source, got {other:?}"),
    }
}

#[test]
fn ignores_non_font_assets() {
    let prepared = PreparedAsset {
        key: AssetKey::new("test/images/foo"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/images/foo.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::Image2d,
        label: None,
        format: None,
        metadata: BTreeMap::new(),
    };

    assert!(font2d_asset_from_prepared(&prepared).is_none());
}

#[test]
fn parses_font_asset_reference_source() {
    let prepared = PreparedAsset {
        key: AssetKey::new("test/fonts/debug-ui"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/fonts/debug-ui/font.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::Font2d,
        label: Some("Debug UI".to_owned()),
        format: Some("truetype".to_owned()),
        metadata: BTreeMap::from([
            ("kind".to_owned(), "font-2d".to_owned()),
            ("format".to_owned(), "truetype".to_owned()),
            (
                "source.asset".to_owned(),
                "core/fonts/console-mono".to_owned(),
            ),
        ]),
    };

    let asset = font2d_asset_from_prepared(&prepared).expect("font should parse");
    assert_eq!(asset.format, Font2dFormat::TrueType);
    match asset.source {
        Font2dSource::AssetRef { key } => {
            assert_eq!(key, AssetKey::new("core/fonts/console-mono"));
        }
        other => panic!("expected asset ref source, got {other:?}"),
    }
}

#[test]
fn resolves_font_asset_reference_from_catalog() {
    let catalog = AssetCatalog::default();
    catalog.mark_prepared(PreparedAsset {
        key: AssetKey::new("core/fonts/console-mono"),
        source: AssetSourceKind::Mod("core".to_owned()),
        resolved_path: PathBuf::from("mods/core/fonts/console-mono/font.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::Font2d,
        label: Some("Console Mono".to_owned()),
        format: Some("truetype".to_owned()),
        metadata: BTreeMap::from([
            ("kind".to_owned(), "font-2d".to_owned()),
            ("format".to_owned(), "truetype".to_owned()),
            ("source.file".to_owned(), "CascadiaMono.ttf".to_owned()),
        ]),
    });
    catalog.mark_prepared(PreparedAsset {
        key: AssetKey::new("test/fonts/debug-ui"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/fonts/debug-ui/font.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::Font2d,
        label: Some("Debug UI".to_owned()),
        format: Some("truetype".to_owned()),
        metadata: BTreeMap::from([
            ("kind".to_owned(), "font-2d".to_owned()),
            ("format".to_owned(), "truetype".to_owned()),
            (
                "source.asset".to_owned(),
                "core/fonts/console-mono".to_owned(),
            ),
        ]),
    });

    let font = font2d_asset_from_catalog(&catalog, &AssetKey::new("test/fonts/debug-ui"))
        .expect("alias should resolve");
    assert_eq!(font.key, AssetKey::new("test/fonts/debug-ui"));
    assert_eq!(font.label, Some("Debug UI".to_owned()));
    match font.source {
        Font2dSource::File { resolved_path, .. } => {
            assert_eq!(
                resolved_path,
                PathBuf::from("mods/core/fonts/console-mono/CascadiaMono.ttf")
            );
        }
        other => panic!("expected resolved file source, got {other:?}"),
    }
}

#[test]
fn rejects_font_asset_reference_cycles() {
    let catalog = AssetCatalog::default();
    for (key, target) in [
        ("test/fonts/a", "test/fonts/b"),
        ("test/fonts/b", "test/fonts/a"),
    ] {
        catalog.mark_prepared(PreparedAsset {
            key: AssetKey::new(key),
            source: AssetSourceKind::Mod("test".to_owned()),
            resolved_path: PathBuf::from(format!("mods/{key}/font.yml")),
            byte_len: 0,
            kind: PreparedAssetKind::Font2d,
            label: None,
            format: Some("truetype".to_owned()),
            metadata: BTreeMap::from([
                ("kind".to_owned(), "font-2d".to_owned()),
                ("format".to_owned(), "truetype".to_owned()),
                ("source.asset".to_owned(), target.to_owned()),
            ]),
        });
    }

    assert!(font2d_asset_from_catalog(&catalog, &AssetKey::new("test/fonts/a")).is_none());
}

