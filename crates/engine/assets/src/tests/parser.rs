use std::path::PathBuf;

use crate::{
    AssetKey, AssetSourceKind, LoadedAsset, PreparedAssetKind, prepare_asset_from_contents,
    prepare_debug_placeholder_asset,
};

#[test]
fn parses_debug_placeholder_asset_metadata() {
    let loaded = LoadedAsset {
        key: AssetKey::new("playground-3d/materials/debug-surface"),
        source: AssetSourceKind::Mod("playground-3d".to_owned()),
        resolved_path: PathBuf::from("mods/playground-3d/materials/debug-surface"),
        byte_len: 96,
    };

    let prepared = prepare_debug_placeholder_asset(
        &loaded,
        r#"
            kind = "material-3d"
            label = "Debug Surface Placeholder"
            format = "debug-placeholder"
        "#,
    )
    .expect("placeholder asset should parse");

    assert_eq!(prepared.kind, PreparedAssetKind::Material3d);
    assert_eq!(prepared.label.as_deref(), Some("Debug Surface Placeholder"));
    assert_eq!(prepared.format.as_deref(), Some("debug-placeholder"));
    assert_eq!(
        prepared.metadata.get("kind").map(String::as_str),
        Some("material-3d")
    );
}

#[test]
fn parses_yaml_sprite_sheet_asset_metadata() {
    let loaded = LoadedAsset {
        key: AssetKey::new("playground-sidescroller/sprites/player"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from("mods/playground-sidescroller/assets/sprites/player.sprite.yml"),
        byte_len: 128,
    };

    let prepared = prepare_asset_from_contents(
        &loaded,
        r#"
kind: sprite-sheet-2d
image: player.png
label: Sidescroller Player
frame_size:
  x: 32
  y: 32
columns: 8
rows: 4
animations:
  idle:
    frames: [0, 1, 2, 3]
    fps: 6
    looping: true
"#,
    )
    .expect("yaml sprite sheet metadata should parse");

    assert_eq!(prepared.kind, PreparedAssetKind::SpriteSheet2d);
    assert_eq!(prepared.label.as_deref(), Some("Sidescroller Player"));
    assert_eq!(
        prepared.metadata.get("image").map(String::as_str),
        Some("player.png")
    );
    assert_eq!(
        prepared.metadata.get("frame_size.x").map(String::as_str),
        Some("32")
    );
    assert_eq!(
        prepared
            .metadata
            .get("animations.idle.frames")
            .map(String::as_str),
        Some("0,1,2,3")
    );
}

#[test]
fn parses_descriptor_first_sheet_aliases() {
    let loaded = LoadedAsset {
        key: AssetKey::new("ink-wars/tilesets/dirt"),
        source: AssetSourceKind::Mod("ink-wars".to_owned()),
        resolved_path: PathBuf::from("mods/ink-wars/assets/tilesets/dirt.tileset.yml"),
        byte_len: 128,
    };

    let prepared = prepare_asset_from_contents(
        &loaded,
        r#"
kind: tileset-2d
schema_version: 1
id: dirt
source:
  file: ../raw/images/dirt.png
atlas:
  image_size: { width: 2048, height: 2048 }
  tile_size: { width: 256, height: 256 }
  columns: 8
  rows: 8
  tile_count: 64
"#,
    )
    .expect("descriptor-first sheet metadata should parse");

    assert_eq!(prepared.kind, PreparedAssetKind::TileSet2d);
    assert_eq!(
        prepared.metadata.get("image").map(String::as_str),
        Some("../raw/images/dirt.png")
    );
    assert_eq!(
        prepared.metadata.get("tile_size.x").map(String::as_str),
        Some("256")
    );
    assert_eq!(
        prepared.metadata.get("image_size.x").map(String::as_str),
        Some("2048")
    );
}

#[test]
fn parses_yaml_generated_audio_metadata() {
    let loaded = LoadedAsset {
        key: AssetKey::new("playground-sidescroller/audio/jump"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from("mods/playground-sidescroller/audio/jump.yml"),
        byte_len: 96,
    };

    let prepared = prepare_asset_from_contents(
        &loaded,
        r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 44100
sequence:
  - wave: square
    frequency: 330
    duration_ms: 40
    volume: 0.35
envelope:
  attack_ms: 2
  release_ms: 30
"#,
    )
    .expect("yaml generated audio metadata should parse");

    assert_eq!(prepared.kind, PreparedAssetKind::GeneratedAudio);
    assert_eq!(
        prepared.metadata.get("generator").map(String::as_str),
        Some("pc-speaker")
    );
    assert_eq!(
        prepared.metadata.get("sample_rate").map(String::as_str),
        Some("44100")
    );
    assert_eq!(
        prepared.metadata.get("sequence").map(String::as_str),
        Some("<mapping>")
    );
    assert_eq!(
        prepared.metadata.get("sequence.0.wave").map(String::as_str),
        Some("square")
    );
    assert_eq!(
        prepared
            .metadata
            .get("sequence.0.frequency")
            .map(String::as_str),
        Some("330")
    );
    assert_eq!(
        prepared
            .metadata
            .get("envelope.attack_ms")
            .map(String::as_str),
        Some("2")
    );
}

#[test]
fn parses_yaml_tile_ruleset_asset_metadata() {
    let loaded = LoadedAsset {
        key: AssetKey::new("playground-sidescroller/tilesets/platformer-rules"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from("mods/playground-sidescroller/assets/tilesets/platformer-rules.tile-ruleset.yml"),
        byte_len: 256,
    };

    let prepared = prepare_asset_from_contents(
        &loaded,
        r##"
kind: tile-ruleset-2d
label: Platformer Ground Rules
format: amigo-rules-v1
tile_size:
  x: 16
  y: 16
terrains:
  ground:
    symbol: "#"
    collision: solid
    variants:
      single: 1
      left_cap: 2
      middle: 3
      right_cap: 4
      top_cap: 5
      bottom_cap: 6
"##,
    )
    .expect("yaml tile ruleset metadata should parse");

    assert_eq!(prepared.kind, PreparedAssetKind::TileRuleSet2d);
    assert_eq!(prepared.label.as_deref(), Some("Platformer Ground Rules"));
    assert_eq!(prepared.format.as_deref(), Some("amigo-rules-v1"));
    assert_eq!(
        prepared.metadata.get("tile_size.x").map(String::as_str),
        Some("16")
    );
    assert_eq!(
        prepared
            .metadata
            .get("terrains.ground.symbol")
            .map(String::as_str),
        Some("#")
    );
    assert_eq!(
        prepared
            .metadata
            .get("terrains.ground.variants.left_cap")
            .map(String::as_str),
        Some("2")
    );
    assert_eq!(
        prepared
            .metadata
            .get("terrains.ground.variants.top_cap")
            .map(String::as_str),
        Some("5")
    );
}
