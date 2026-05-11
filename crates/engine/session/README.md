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

## SceneSession boundary

`SceneSession` is the host-independent scene ownership seam.

In Etap 3 it stores only authored scene metadata copied from the existing
app bootstrap summary. This keeps migration safe: app-owned scene loading,
hydration, command dispatch and handlers remain where they are until the next
passes.

Future passes should move these responsibilities behind `SceneSession`:

- scene document loading
- scene hydration queueing
- scene command dispatch
- runtime scene cleanup
- scene diagnostics/source-map metadata

## Scene lifecycle boundary

Etap 4 adds explicit scene lifecycle state to `SceneSession`.

The session now records:

- whether no scene is active,
- whether an authored scene document was loaded,
- whether hydration was queued,
- whether scene commands were applied,
- whether a transition or clear operation is in progress,
- whether the lifecycle entered an error state.

This is still a migration boundary. The concrete app-owned scene runtime paths
remain active until later passes move them into `SceneSession` or domain-owned
scene contributions.
