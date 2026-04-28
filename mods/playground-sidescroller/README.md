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
  - `Sprites/Tiles/Default/terrain_grass_horizontal_middle.png` -> `tilesets/platformer.png`
  - derived tile rules -> `tilesets/platformer-rules.yml`
- Modifications:
  - `textures/player.png`: repacked four character frames into a 4x1 spritesheet
  - `textures/coin.png`: repacked front/side coin sprites into a 4-frame loop
  - `textures/finish.png`: copied and renamed from the original flag sprite
- `tilesets/platformer.png`: repacked one block tile and one horizontal middle tile into a 15-tile strip covering `single`, horizontal caps/middle, vertical caps/middle, and placeholder inner/outer corners
- `tilesets/platformer.png`: `right_cap` is a horizontal mirror of the imported left edge tile
- `tilesets/platformer.png`: `vertical_middle`, `bottom_cap`, and placeholder corner variants are locally derived from the imported terrain tiles to support forward-compatible autotiling variants
  - metadata YAML added for `player`, `coin`, `finish`, `platformer`, and `platformer-rules`
- Notes:
  - only required files were copied into the repo
  - the downloaded archive itself is not committed
  - all imported visuals come from one primary pack to keep the style consistent

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

- `fonts/debug-ui.yml`
- Created for Amigo playground-sidescroller
- License/ownership: internal project placeholder metadata
- Purpose: keep the current debug UI font path stable until the font pipeline is upgraded to load a real font asset in this mod
- `backgrounds/far.png`
- `backgrounds/far.yml`
- `backgrounds/near.png`
- `backgrounds/near.yml`
- Created for Amigo playground-sidescroller
- License/ownership: internal project asset
- Purpose: layered full-screen sidescroller background placeholders until a dedicated CC0 background pack is selected
