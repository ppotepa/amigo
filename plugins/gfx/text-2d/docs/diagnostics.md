# Text 2D Diagnostics

Channels:
- `text-2d.render`
- `text-2d.contributions`

## Candidate Trace
`format_text_2d_candidates` reports the number of text render candidates.

## Render Trace
- Hydration errors should include scene id, entity id, and component kind.
- Color parsing failures are source document errors.
- Extraction should be checked after scene visibility filtering.

Diagnostics should keep text layout and camera contribution state separate from
font atlas or backend implementation details.
