# Asteroids

Procedural, vector-first 2D playground mod for early Asteroids-style engine work.

Current scope:

- native `vector_2d` scene components
- WGPU stroke/fill rendering for simple vector shapes
- generated synthetic arcade SFX for shot, thrust, asteroid hits, wave progression, respawn protection, and game over cues
- lightweight runtime-owned `CircleCollider2D` overlap queries exposed through `world.physics`
- runtime vector mutation API exposed through `world.vector` for procedural shape updates
- no sprite assets
- no world wrap yet
- no full rigid-body or wrap-around gameplay physics yet

The first slice is intentionally small and exists to validate:

1. scene YAML -> vector hydration
2. runtime scene queueing
3. vector rendering in hosted mode
4. first playable ship / bullet / asteroid loop without bitmap assets
5. thin collision seam from scene components into Rhai through `world.physics.overlaps(...)`
6. first bounded two-wave arena flow before world-wrap work

Audio notes:

- all Asteroids SFX in this mod are procedural `generated-audio` metadata assets
- `thrust.yml` adds a soft low-frequency engine pulse intended for short boost/thrust accents
- `wave-start.yml` is a short upward cue for spawning/announcing the next wave
- `sector-clear.yml` is a brighter progression sting for clearing the current arena slice
- `respawn-shield.yml` is a soft protective cue for ship respawn / temporary invulnerability beats

Scene polish notes:

- `vector-preview` includes a slightly richer low-contrast vector backdrop so the arena reads less empty without competing with gameplay shapes
- the arena now uses a full black vector backdrop and a larger field/frame footprint tuned for full-window readability
- the HUD copy is tuned to feel a bit more arcade-like while keeping the same ids and script wiring
- the scene now runs a real second wave using additional off-screen-staged asteroid entities instead of stopping after the first clear
- asteroid silhouettes are generated procedurally in script and pushed into runtime via `world.vector.set_polygon(...)`
