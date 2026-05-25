# Sprite 2D Diagnostics

Channels:
- `sprite-2d.render`
- `sprite-2d.contributions`

## Candidate Trace
`format_sprite_2d_candidates` reports entity, status, reason, and target ids.

## Render Trace
- Confirm the scene service received a hydrated sprite command.
- Confirm visibility filtering did not remove the entity before extraction.
- Confirm role flags before expecting `SceneDepth`, `SceneHighlight`, or
  `SceneVelocity` contributions.

Diagnostics should point to the sprite entity and role set, not to backend draw
state.
