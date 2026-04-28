# Playground Sidescroller

2D vertical-slice playground for Amigo. This mod is the first gameplay-oriented integration scene that combines:

- tilemap rendering
- kinematic platformer movement
- trigger events
- screen-space HUD
- generated audio
- camera follow

# Asset provenance

## Visual assets

### Pack: New Platformer Pack
- Source: https://kenney.nl/assets/new-platformer-pack
- Author: Kenney
- License: Creative Commons CC0 1.0 Universal
- Download date: 2026-04-28
- Original archive/file name: `kenney_new-platformer-pack-1.1.zip`
- Used files:
  - `Sprites/Characters/Default/character_beige_idle.png` -> `textures/player.png`
  - `Sprites/Characters/Default/character_beige_walk_a.png` -> `textures/player.png`
  - `Sprites/Characters/Default/character_beige_walk_b.png` -> `textures/player.png`
  - `Sprites/Characters/Default/character_beige_jump.png` -> `textures/player.png`
  - `Sprites/Tiles/Default/coin_gold.png` -> `textures/coin.png`
  - `Sprites/Tiles/Default/coin_gold_side.png` -> `textures/coin.png`
  - `Sprites/Tiles/Default/flag_green_b.png` -> `textures/finish.png`
  - `Sprites/Tiles/Default/terrain_grass_block.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_horizontal_left.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_horizontal_middle.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_horizontal_right.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_left.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_right.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_top.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_vertical_middle.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_bottom.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_top_left.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_top_right.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_bottom_left.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_bottom_right.png` -> `tilesets/platformer.png`
  - `Sprites/Tiles/Default/terrain_grass_block_center.png` -> `tilesets/platformer.png`
  - terrain rules metadata -> `tilesets/platformer-rules.yml`
  - generator script -> `tools/generate-platformer-kit-assets.ps1`
- Modifications:
  - `textures/player.png`: repacked four character frames into a 4x1 spritesheet
  - `textures/coin.png`: repacked coin front/side sprites into a 4-frame loop
  - `textures/finish.png`: copied and renamed from the original flag sprite
  - `tilesets/platformer.png`: packed selected terrain tiles into an 18-tile atlas matching the current autotile resolver
  - metadata YAML added for `player`, `coin`, `finish`, `platformer`, and `platformer-rules`
- Notes:
  - only required files were copied into the repo
  - the downloaded archive itself is not committed
  - all imported visuals come from one primary pack to keep the style consistent
  - generated gameplay sprite and tileset outputs can be reproduced with `tools/generate-platformer-kit-assets.ps1`
  - `left_cap/right_cap/middle` are used for horizontal platform runs
  - `side_left/side_right/center` are used for wall sides and fully enclosed block interiors
  - the pack does not include explicit raster inner-corner tiles, so `inner_corner_*` currently fall back to `terrain_grass_block_center.png`

## Generated audio

Audio files in this mod are generated from metadata by Amigo generated audio systems.
No external SFX files are used.

Used generated audio definitions:
- `audio/jump.yml`
- `audio/coin.yml`
- `audio/hurt.yml`
- `audio/level-complete.yml`
- `audio/proximity-beep.yml`

## Internal placeholders

- `backgrounds/layer-01.png`
- `backgrounds/layer-02.png`
- `backgrounds/layer-03.png`
- `backgrounds/layer-04.png`
- Created for Amigo playground-sidescroller using image generation
- License/ownership: internal project asset
- Purpose: custom numbered parallax background layers tuned for the current vertical-slice scene; the numbering is scene-level naming only, not an engine-imposed near/far restriction
- Layer layout:
  - `layer-01`: moonlit sky base
  - `layer-02`: distant forest silhouette overlay with alpha
  - `layer-03`: rolling hill overlay with alpha
  - `layer-04`: foreground foliage overlay with alpha
- `fonts/debug-ui.yml`
- Created for Amigo playground-sidescroller
- License/ownership: internal project placeholder metadata
- Purpose: keep the current debug UI font path stable until the font pipeline is upgraded to load a real font asset in this mod
