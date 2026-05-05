    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};

    use super::{
        glyph_rows, infer_sprite_sheet_from_asset, infer_tileset_from_asset, resolve_image_path,
        tile_uv_rect,
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
        for ch in
            "BASIC SCRIPTING DEMO LEFT / RIGHT rotate square via EntityRef.rotate_2d()".chars()
        {
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

        let tileset = infer_tileset_from_asset(&prepared, Vec2::new(16.0, 16.0))
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
