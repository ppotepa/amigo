# amigo-session

`amigo-session` is the host-independent runtime session layer for Amigo.

It sits above `amigo-runtime` and below concrete hosts such as `amigo-app`,
the future editor, headless validation tools and scene preview tools.

Responsibilities:

- own the high-level runtime session contract
- expose frame input/output DTOs
- expose runtime profiles for game/editor/headless/test usage
- become the future home for scene session, render session, script session,
  scheduler session and diagnostics orchestration

Non-goals:

- no window or event-loop ownership
- no WGPU surface ownership
- no app-specific startup UX
- no duplicate runtime v2

## Bootstrap boundary

`RuntimeSession` is the ownership boundary after host-specific bootstrap.

During migration, `amigo-app` may still assemble the low-level `Runtime`,
because some app-local plugins and handlers are not moved yet. New host/editor
facing code should prefer adapters returning
`RuntimeSessionBootstrap<TSummary>` instead of raw `(Runtime, Summary)` tuples.
