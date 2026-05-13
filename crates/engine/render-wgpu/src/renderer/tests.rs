use std::collections::BTreeMap;
use std::path::PathBuf;

use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};

use super::{
    glyph_rows, infer_sprite_sheet_from_asset, infer_tileset_from_asset, resolve_image_path,
    resolve_tileset_sheet_key, tile_uv_rect,
};
use amigo_math::Vec2;

#[test]
fn glyph_rows_cover_hello_world_letters() {
    for ch in ['H', 'E', 'L', 'O', 'W', 'R', 'D', ' '] {
        assert!(glyph_rows(ch).iter().any(|row| *row != 0) || ch == ' ');
    }
}

#[test]
fn glyph_rows_cover_basic_scripting_demo_characters() {
    for ch in "BASIC SCRIPTING DEMO LEFT / RIGHT rotate square via EntityRef.rotate_2d()".chars() {
        assert!(glyph_rows(ch).iter().any(|row| *row != 0) || ch == ' ');
    }
}

#[test]
fn glyph_rows_cover_ui_showcase_punctuation() {
    for ch in
        "Theme: space_dark volume=65% F1 dark | F2 clean | T cycle [-] [+] UI; <START>".chars()
    {
        assert!(glyph_rows(ch).iter().any(|row| *row != 0) || ch == ' ');
    }
}

#[test]
fn font2d_descriptor_resolves_ttf_source_path() {
    use amigo_font::{Font2dFormat, Font2dSource, font2d_asset_from_prepared};

    let prepared = PreparedAsset {
        key: AssetKey::new("test/fonts/console-mono"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/fonts/console-mono/font.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::Font2d,
        label: Some("Console Mono".to_owned()),
        format: Some("truetype".to_owned()),
        metadata: BTreeMap::from([
            ("kind".to_owned(), "font-2d".to_owned()),
            ("format".to_owned(), "truetype".to_owned()),
            ("source.file".to_owned(), "console-mono.ttf".to_owned()),
            ("glyphs.preset".to_owned(), "console-latin-ext".to_owned()),
            ("metrics.default_size".to_owned(), "12".to_owned()),
        ]),
    };

    let font = font2d_asset_from_prepared(&prepared).expect("font descriptor should parse");
    assert_eq!(font.format, Font2dFormat::TrueType);
    match font.source {
        Font2dSource::File { resolved_path, .. } => {
            assert_eq!(
                resolved_path,
                PathBuf::from("mods/test/fonts/console-mono/console-mono.ttf")
            );
        }
        other => panic!("expected file font source, got {other:?}"),
    }
}

#[test]
fn resolves_image_path_relative_to_metadata_file() {
    let prepared = PreparedAsset {
        key: AssetKey::new("test/spritesheets/player"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/spritesheets/player/spritesheet.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::SpriteSheet2d,
        label: None,
        format: None,
        metadata: BTreeMap::from([("image".to_owned(), "../../raw/images/player.png".to_owned())]),
    };

    assert_eq!(
        resolve_image_path(&prepared),
        Some(PathBuf::from("mods/test/raw/images/player.png"))
    );
}

#[test]
fn infers_sprite_sheet_from_prepared_metadata() {
    let prepared = PreparedAsset {
        key: AssetKey::new("test/spritesheets/player"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/spritesheets/player/spritesheet.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::SpriteSheet2d,
        label: None,
        format: None,
        metadata: BTreeMap::from([
            ("columns".to_owned(), "8".to_owned()),
            ("rows".to_owned(), "4".to_owned()),
            ("frame_size.x".to_owned(), "32".to_owned()),
            ("frame_size.y".to_owned(), "32".to_owned()),
            ("fps".to_owned(), "10".to_owned()),
            ("looping".to_owned(), "true".to_owned()),
        ]),
    };

    let sheet = infer_sprite_sheet_from_asset(&prepared).expect("sheet metadata should parse");
    assert_eq!(sheet.columns, 8);
    assert_eq!(sheet.rows, 4);
    assert_eq!(sheet.frame_count, 32);
    assert_eq!(sheet.frame_size.x, 32.0);
    assert_eq!(sheet.frame_size.y, 32.0);
    assert_eq!(sheet.fps, 10.0);
    assert!(sheet.looping);
}

#[test]
fn infers_tileset_with_derived_variants_from_prepared_metadata() {
    let prepared = PreparedAsset {
        key: AssetKey::new("test/tilesets/platformer"),
        source: AssetSourceKind::Mod("test".to_owned()),
        resolved_path: PathBuf::from("mods/test/tilesets/platformer.yml"),
        byte_len: 0,
        kind: PreparedAssetKind::TileSet2d,
        label: None,
        format: None,
        metadata: BTreeMap::from([
            ("columns".to_owned(), "1".to_owned()),
            ("rows".to_owned(), "1".to_owned()),
            ("tile_size.x".to_owned(), "16".to_owned()),
            ("tile_size.y".to_owned(), "16".to_owned()),
            ("tiles.ground_single.id".to_owned(), "0".to_owned()),
            ("tiles.ground_left_cap.id".to_owned(), "1".to_owned()),
            ("tiles.ground_right_cap.id".to_owned(), "2".to_owned()),
            ("tiles.ground_top_cap.id".to_owned(), "3".to_owned()),
            ("tiles.ground_bottom_cap.id".to_owned(), "4".to_owned()),
            (
                "derived_variants.ground_left_cap.from_tile".to_owned(),
                "ground_single".to_owned(),
            ),
            (
                "derived_variants.ground_left_cap.mode".to_owned(),
                "split_x".to_owned(),
            ),
            (
                "derived_variants.ground_left_cap.segment".to_owned(),
                "left".to_owned(),
            ),
            (
                "derived_variants.ground_right_cap.from_tile".to_owned(),
                "ground_single".to_owned(),
            ),
            (
                "derived_variants.ground_right_cap.mode".to_owned(),
                "split_x".to_owned(),
            ),
            (
                "derived_variants.ground_right_cap.segment".to_owned(),
                "right".to_owned(),
            ),
            (
                "derived_variants.ground_top_cap.from_tile".to_owned(),
                "ground_single".to_owned(),
            ),
            (
                "derived_variants.ground_top_cap.mode".to_owned(),
                "split_y".to_owned(),
            ),
            (
                "derived_variants.ground_top_cap.segment".to_owned(),
                "top".to_owned(),
            ),
            (
                "derived_variants.ground_bottom_cap.from_tile".to_owned(),
                "ground_single".to_owned(),
            ),
            (
                "derived_variants.ground_bottom_cap.mode".to_owned(),
                "split_y".to_owned(),
            ),
            (
                "derived_variants.ground_bottom_cap.segment".to_owned(),
                "bottom".to_owned(),
            ),
        ]),
    };

    let tileset = infer_tileset_from_asset(&prepared, None, Vec2::new(16.0, 16.0))
        .expect("tileset should parse");

    let left = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 1);
    assert!(left.u0 > 0.0 && left.u0 < 0.1);
    assert!(left.u1 > 0.4 && left.u1 < 0.5);
    assert!(left.v0 > 0.0 && left.v0 < 0.1);
    assert!(left.v1 > 0.9 && left.v1 < 1.0);

    let right = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 2);
    assert!(right.u0 > 0.5 && right.u0 < 0.6);
    assert!(right.u1 > 0.9 && right.u1 < 1.0);
    assert!(right.v0 > 0.0 && right.v0 < 0.1);
    assert!(right.v1 > 0.9 && right.v1 < 1.0);

    let top = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 3);
    assert!(top.u0 > 0.0 && top.u0 < 0.1);
    assert!(top.u1 > 0.9 && top.u1 < 1.0);
    assert!(top.v0 > 0.0 && top.v0 < 0.1);
    assert!(top.v1 > 0.4 && top.v1 < 0.5);

    let bottom = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 4);
    assert!(bottom.u0 > 0.0 && bottom.u0 < 0.1);
    assert!(bottom.u1 > 0.9 && bottom.u1 < 1.0);
    assert!(bottom.v0 > 0.5 && bottom.v0 < 0.6);
    assert!(bottom.v1 > 0.9 && bottom.v1 < 1.0);
}

#[test]
fn resolves_descriptor_first_tileset_sheet_key() {
    let prepared = PreparedAsset {
        key: AssetKey::new(
            "playground-sidescroller/spritesheets/platformer/tilesets/platform/base",
        ),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from(
            "mods/playground-sidescroller/spritesheets/platformer/tilesets/platform/base.yml",
        ),
        byte_len: 0,
        kind: PreparedAssetKind::TileSet2d,
        label: None,
        format: None,
        metadata: BTreeMap::from([("spritesheet".to_owned(), "platformer".to_owned())]),
    };

    assert_eq!(
        resolve_tileset_sheet_key(&prepared).map(|key| key.as_str().to_owned()),
        Some("playground-sidescroller/spritesheets/platformer".to_owned())
    );
}

#[test]
fn infers_tileset_columns_from_referenced_sheet_metadata() {
    let tileset_prepared = PreparedAsset {
        key: AssetKey::new(
            "playground-sidescroller/spritesheets/platformer/tilesets/platform/base",
        ),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from(
            "mods/playground-sidescroller/spritesheets/platformer/tilesets/platform/base.yml",
        ),
        byte_len: 0,
        kind: PreparedAssetKind::TileSet2d,
        label: None,
        format: None,
        metadata: BTreeMap::from([
            ("spritesheet".to_owned(), "platformer".to_owned()),
            ("tile_size.x".to_owned(), "64".to_owned()),
            ("tile_size.y".to_owned(), "64".to_owned()),
            ("tiles.ground_single.id".to_owned(), "0".to_owned()),
        ]),
    };
    let sheet_prepared = PreparedAsset {
        key: AssetKey::new("playground-sidescroller/spritesheets/platformer"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from(
            "mods/playground-sidescroller/spritesheets/platformer/spritesheet.yml",
        ),
        byte_len: 0,
        kind: PreparedAssetKind::SpriteSheet2d,
        label: None,
        format: None,
        metadata: BTreeMap::from([
            (
                "image".to_owned(),
                "../../raw/images/platformer.png".to_owned(),
            ),
            ("columns".to_owned(), "18".to_owned()),
            ("rows".to_owned(), "1".to_owned()),
            ("frame_size.x".to_owned(), "64".to_owned()),
            ("frame_size.y".to_owned(), "64".to_owned()),
            ("frame_count".to_owned(), "18".to_owned()),
        ]),
    };

    let tileset = infer_tileset_from_asset(
        &tileset_prepared,
        Some(&sheet_prepared),
        Vec2::new(16.0, 16.0),
    )
    .expect("tileset should parse from referenced sheet metadata");

    assert_eq!(tileset.columns, 18);
    assert_eq!(tileset.tile_size.x, 64.0);
    assert_eq!(tileset.tile_size.y, 64.0);
}

