# Vector 2D Diagnostics

Channels:
- `vector-2d.render`
- `vector-2d.contributions`

## Candidate Trace
`format_vector_2d_candidates` reports the number of vector render candidates.

## Render Trace
- Hydration errors should include scene id, entity id, and component kind.
- Color parsing failures belong to the authored vector document.
- Visibility filtering runs before extraction.

Diagnostics should describe source command and role data before backend pipeline
state.
