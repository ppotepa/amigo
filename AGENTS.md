# AGENTS.md

This file defines how Codex CLI agents must work in the Amigo repository.

The goal is not only to make changes, but to make changes with the smallest practical amount of token usage, file reading, command execution, and architectural disruption.

Amigo is a fresh project. Do not preserve legacy paths after migration. Evolve systems in place. Prefer clean final names over compatibility shims.

---

# Prime Directive

Use `amigo-codemap` first.

Do not scan the repository blindly. Do not read whole files when a symbol slice is enough. Do not run broad build/test commands when a targeted crate check is enough.

Every task should follow this shape:

```text
understand scope
  → use codemap to locate exact files/symbols
  → inspect only relevant slices
  → apply precise raw ops
  → verify only touched crates
  → report concise result
````

Prefer the shortest codemap command that answers the question. Use `--print` on `refresh` only when you are diagnosing refresh speed or scan behavior. In normal work, prefer `refresh --level 1`, `status`, `brief`, `trace`, `open-set --why`, and `change-plan` without extra diagnostic flags.

---

# Why `amigo-codemap` Exists

`amigo-codemap` is the main navigation, planning, and patching tool for this repo.

Its purpose is to avoid the most expensive agent behavior:

```text
bad:
  recursively search repo
  open large files
  read concat snapshots
  inspect unrelated modules
  run workspace checks
  patch broad areas without certainty

good:
  ask codemap what files/symbols matter
  open only those symbols
  patch exact ranges
  verify only impacted crates
```

The repo is large enough that careless file reading burns tokens quickly. Codemap gives targeted context, symbol slices, impact checks, and raw operation validation.

---

# Hard Prohibitions

Do not use these by default:

```text
Get-Content <large file>
cat <large file>
type <large file>
rg across the entire repo as first discovery
fd/find across the entire repo as first discovery
opening concat-output.txt
reading generated snapshots
cargo check --workspace
cargo test --workspace
full git diff
manual file-by-file browsing
large rewrite without codemap plan
YAML ops plans
creating .amigo/ops/*
```

Forbidden unless explicitly requested:

```text
workspace-wide test/check
broad formatting of unrelated files
mass renames
compatibility layers
v2 systems
parallel duplicate systems
leaving legacy paths after migration
```

---

# Required Start Sequence

Start every non-trivial task with:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"

& $cm brief
& $cm changes --compact --hide-generated --limit 20
```

Then use a task-specific `change-plan`:

```powershell
& $cm change-plan "<task summary>" --limit 20
& $cm open-set "<target architecture or feature area>" --why --limit 12
```

Example:

```powershell
& $cm change-plan "finalize render graph cleanup remove legacy render path" --limit 20
& $cm open-set "WgpuFrameGraphExecutor LegacyComposite render_frame_request_legacy render.mode" --why --limit 12
```

---

# Token-Saving Discovery Rules

## First use `trace`

Use `trace` for exact symbols, strings, commands, feature names, YAML IDs, and function names.

```powershell
& $cm trace "WgpuFrameGraphExecutor" --limit 20
& $cm trace "render_frame_request_legacy" --limit 20
& $cm trace "ConsoleCommandDescriptor" --limit 20
& $cm trace "ParticleMotionStretch2d" --limit 20
```

Use multiple focused traces instead of one broad search.

Bad:

```powershell
rg render
```

Good:

```powershell
& $cm trace "render_frame_request" --limit 20
& $cm trace "FrameGraphNodeKind" --limit 20
& $cm trace "render.plan" --limit 20
```

## Then use `open-set --why`

Use `open-set --why` to get the smallest useful workset.

```powershell
& $cm open-set "dev console completion registry overlay input handling" --why --limit 12
```

The `--why` output tells why each file matters. Do not open unrelated files.

## Then use `symbols`

Use `symbols` before reading a file.

```powershell
& $cm symbols --file crates/apps/app/src/dev_console/registry.rs --metadata --limit 80
```

## Then use `slice --symbol`

Read only the symbol you need.

```powershell
& $cm slice crates/apps/app/src/dev_console/registry.rs --symbol ConsoleCommandRegistry
& $cm slice crates/apps/app/src/host_runtime.rs --symbol on_redraw_requested
& $cm slice crates/engine/render-wgpu/src/renderer/graph/executor.rs --symbol WgpuFrameGraphExecutor
```

## Use `range-for-symbol` only when necessary

Use it for raw ops that need exact line/range boundaries.

```powershell
& $cm range-for-symbol crates/engine/render-wgpu/src/renderer/service/render.rs render_frame_request_legacy
```

Do not use line ranges manually unless codemap gives them.

---

# Cost Tiers

Prefer lower-cost operations.

## Tier 0: Always allowed

```text
codemap brief
codemap changes --compact
codemap trace
codemap open-set --why
codemap symbols
codemap slice --symbol
codemap range-for-symbol
codemap ops-preview/check/apply
codemap verify-plan --changed
codemap fallout
```

Use `refresh --print` sparingly. Default to `refresh --level 1` or `refresh` unless you specifically need timings/progress output.

## Tier 1: Allowed after scoped discovery

```text
cargo check -p touched_crate
cargo test -p touched_crate exact_test_name
codemap impact
codemap anchors --write
codemap anchor-check
```

## Tier 2: Use only with clear need

```text
small targeted rg in one file or one directory
reading a short file fully
cargo test -p crate without exact test name
```

## Tier 3: Avoid unless explicitly requested

```text
cargo check --workspace
cargo test --workspace
full repo search
full file dumps
global format
```

---

# Raw Ops Only

Use inline raw ops.

Do not create YAML operation files.

Required pattern:

```powershell
$ops = @'
ACTION: ...
FILE: ...
CONTENT:
...
END
'@

$ops | & $cm ops-preview --raw
$ops | & $cm ops-check --raw --strict
$ops | & $cm ops-apply --raw --write --backup --stop-on-error
$ops | & $cm ops-summary --raw --changed
```

Do not use:

```text
ops-skeleton --out plan.yml
ops-check --from plan.yml
ops-apply --from plan.yml
content_from
content_root
.amigo/ops/*
```

---

# Raw Ops Examples

## Create a file

```powershell
$ops = @'
ACTION: CREATE_FILE
FILE: crates/engine/render-api/src/composition.rs
CONTENT:
pub struct FrameCompositionPlan {
    pub views: Vec<RenderViewPlan>,
}

pub struct RenderViewPlan {
    pub passes: Vec<RenderPassPlan>,
}

pub enum RenderPassPlan {
    World2D,
    PostFx,
    GameUi,
    DebugOverlay,
    Present,
}
END
'@

$ops | & $cm ops-preview --raw
$ops | & $cm ops-check --raw --strict
$ops | & $cm ops-apply --raw --write --backup --stop-on-error
```

## Insert module export

```powershell
$ops = @'
ACTION: INSERT_AFTER_TEXT
FILE: crates/engine/render-api/src/lib.rs
FIND:
use std::marker::PhantomData;
CONTENT:

pub mod composition;
pub use composition::*;
END
'@

$ops | & $cm ops-check --raw --strict
$ops | & $cm ops-apply --raw --write --backup --stop-on-error
```

## Replace exact text

```powershell
$ops = @'
ACTION: REPLACE_TEXT
FILE: crates/apps/app/src/render_runtime/context.rs
FIND:
overlay: Vec<UiOverlayDocument>,
REPLACE:
game_ui_overlay: Vec<UiOverlayDocument>,
debug_overlay: Vec<UiOverlayDocument>,
END
'@

$ops | & $cm ops-check --raw --strict
$ops | & $cm ops-apply --raw --write --backup --stop-on-error
```

## Delete a legacy symbol

Prefer symbol deletion when supported.

```powershell
$ops = @'
ACTION: DELETE_SYMBOL
FILE: crates/engine/render-wgpu/src/renderer/service/render.rs
SYMBOL: render_frame_request_legacy
END
'@

$ops | & $cm ops-check --raw --strict
$ops | & $cm ops-apply --raw --write --backup --stop-on-error
```

If unavailable, use:

```powershell
& $cm range-for-symbol crates/engine/render-wgpu/src/renderer/service/render.rs render_frame_request_legacy
```

Then delete the returned range.

---

# Verification Rules

After any patch:

```powershell
& $cm verify-plan --changed
```

Then run only checks for touched crates.

Examples:

```powershell
cargo check -p amigo-render-api 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-app 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-render-wgpu 2>&1 | & $cm fallout --limit 80
```

Targeted test example:

```powershell
cargo test -p amigo-app completion_suggests_registered_debug_commands 2>&1 | & $cm fallout --limit 80
```

Do not paste full compiler output. Always pipe through `fallout`.

---

# When a Command Fails

Do not immediately broaden the search.

Use this sequence:

```text
1. Read the error.
2. Use fallout summary.
3. Trace the missing symbol/type.
4. Slice the exact failing symbol.
5. Patch the exact issue.
6. Re-run only the failed crate check/test.
```

Example:

```powershell
cargo check -p amigo-app 2>&1 | & $cm fallout --limit 80
& $cm trace "missing_symbol_name" --limit 20
& $cm open-set "missing_symbol_name usage" --why --limit 8
```

---

# Required Report Style

Final response must be concise and factual:

```text
Changed:
- file/symbol
- file/symbol

Verified:
- command
- command

Notes:
- any skipped verification and why
- any remaining follow-up
```

Do not claim success without verification.

Do not say workspace tests passed unless they were actually run.

---

# Architecture Rules

## No v2 Systems

Do not create:

```text
RenderPipelineV2
SceneCompilerV2
EditorTargetV2
ParticleSystemV2
NewRenderer
LegacyRenderer
```

Evolve existing systems in place.

## Cleanup-As-We-Go

Compatibility paths are allowed only during a migration step.

After migration, delete:

```text
legacy entrypoints
old wrappers
fallback branches
temporary compatibility helpers
dead_code allowances
migration comments
```

Fresh project rule: no permanent legacy.

## Engine-Level Contracts

Shared contracts must live in engine crates, not app crates.

Good:

```text
crates/engine/render-api:
  FrameCompositionPlan
  FrameGraph
  RenderCompositionDiagnostics

crates/engine/scene:
  SceneDocument
  SceneCompiler
  Scene validation

crates/2d/post-fx:
  PostFx2d
  certification model
```

Bad:

```text
crates/apps/app:
  defining engine render graph contracts
  owning reusable post-fx model
  implementing scene validation that engine should own
```

## App Is Glue

`crates/apps/app` owns:

```text
window loop
host runtime orchestration
dev console
app-specific extraction wiring
runtime service registration
```

It must not own reusable engine architecture.

## Future Editor Compatibility

Assume a future editor app will need to reuse:

```text
scene compiler
render composition plan
frame graph diagnostics
render target/offscreen preview
post-fx certification
metadata descriptors
asset references
```

Do not bury these in `crates/apps/app`.

---

# Engine / Editor / App / Mod Boundaries

## Engine owns

```text
runtime data contracts
scene compilation
scene validation
render graph contracts
render feature contracts
post-fx models
diagnostics/certification
asset reference semantics
metadata descriptors
```

## App owns

```text
host loop
input integration
dev console
runtime glue
winit/wgpu surface orchestration
```

## Editor owns

```text
UI for editing
target navigation
inspectors
preview panels
commands that call engine APIs
```

## Mods own

```text
authoring YAML
scene-local assets
mod-level reusable assets
scripts
content-specific values
```

Editor must not duplicate engine validation. It should consume engine metadata and diagnostics.

---

# Code Smells to Avoid

Avoid these patterns:

```text
app-level glue knowing about a specific engine effect
renderer core containing feature-specific if/else branches
new v2 systems next to old systems
compatibility paths left after migration
huge functions with unrelated responsibilities
DTO duplication across engine/app/editor
YAML shape that cannot map to editor targets
runtime feature with no diagnostics
per-frame allocations in hot paths
thread spawn/join inside frame loop
post-fx affecting debug overlay accidentally
engine contracts hidden inside app crate
manual parsing in editor that differs from engine parser
stringly typed target refs without validation
large public functions used as alternate pipelines
```

Specific render smells:

```text
host_runtime knows LensDroplets
render.mode switches legacy/split after migration
game UI and debug UI joined before post-fx
FrameGraphNodeKind::LegacyComposite remains
render_frame_request_legacy remains
renderer has public render_scene_with_ui_primitives_and_3d_commands
```

---

# Change Granularity Rules

A good patch changes one conceptual layer.

Good sequence:

```text
1. data model
2. parser/compiler
3. runtime service
4. diagnostics
5. renderer/runtime use
6. tests
7. cleanup
```

Bad patch:

```text
adds YAML schema
adds renderer shader
adds scheduler
adds console commands
renames files
removes legacy
changes mod content
all at once
```

Keep phases small enough that each can be checked independently.

---

# Feature Implementation Shape

A complete feature should normally include:

```text
1. Authoring model
2. Runtime model
3. Validation/certification
4. Diagnostics/console visibility
5. Tests
6. Editor-readable metadata
7. Cleanup of replaced paths
```

Example for a post-fx feature:

```text
PostFx2d model
SceneVisual2dDocument parse
Scene hydration command
PostFxService state
Certification report
postfx.cert command
FrameGraph pass
WGPU node/shader
Mod YAML sample
Cleanup old fallback
```

---

# Diagnostics-First Rule

Do not add hard-to-debug runtime behavior without diagnostics.

Examples:

```text
new scheduler behavior → scheduler.stats / scheduler.overrides
new post-fx → postfx.cert / postfx.stats
new render path → render.plan / render.graph
new particle optimization → particles.stats / debug.particles
new input feature → debug.input
```

Diagnostics may be simple, but must exist before the feature becomes opaque.

---

# Performance Rules

Avoid:

```text
per-frame allocation in hot paths
thread spawn per frame
immediate worker spawn + join pretending to be async
cloning large buffers per particle
string clones per draw command
offscreen render target when no post-fx needs it
rebuilding GPU pipelines per frame
reading world data from worker without snapshot/command model
```

Prefer:

```text
persistent worker pools
double buffers for async visual data
frame-local transient resource allocator
prepared render batches
cached pipelines
small immutable snapshots
command/result buffers
diagnostic counters
```

Render-specific:

```text
No post-fx:
  render world directly to surface.

With post-fx:
  render world to transient texture.
  post-fx samples texture.
  game UI after post-fx.
  debug overlay last.
```

---

# Render Pipeline Rules

Final render flow should be:

```text
host_runtime
  → AppFrameCompositionBuilder
  → build_frame_graph_from_plan
  → WgpuFrameRenderRequest
  → WgpuFrameGraphExecutor
  → graph nodes
```

Forbidden final-state render paths:

```text
render_frame_request_legacy
LegacyComposite
SplitPassExperimental
render.mode legacy/split
host_runtime lens_droplets overlay hack
AppRenderFramePacket::overlay()
public render_scene_with_ui_primitives_and_3d_commands
```

Renderer core should not know feature-specific app hacks.

Effects should become graph nodes/features.

---

# Scene / Mod Authoring Rules

Use scope-based authoring.

```text
mod-level folder = reusable for the whole mod
scene-level folder = local to that scene
```

Good:

```text
mods/they-are-rotten/ui/themes/rotten-noir.yml
mods/they-are-rotten/scenes/main-menu/ui/bindings.yml
mods/they-are-rotten/scenes/main-menu/visual/lens.yml
```

Avoid generic `parts/`.

Use domain folders:

```text
visual/
entities/
ui/
input/
events/
state/
audio/
timelines/
scripts/
```

Scene manifest should compose domains:

```yaml
use:
  visual:
    - ./visual/render.yml
    - ./visual/lens.yml
  ui:
    - ./ui/mount.yml
```

---

# Scheduler / Jobs Rules

Workers must not mutate world directly.

Good pattern:

```text
main thread:
  owns world mutation
  collects input
  applies command/results
  submits GPU work

workers:
  receive snapshots
  compute results
  return typed output
```

Avoid:

```text
spawn thread and immediately join every frame
worker directly locking live render state while renderer waits
jobs without stats
scheduler config that silently fails to match targets
```

Required diagnostics:

```text
scheduler.stats
scheduler.overrides
worker_jobs_submitted
worker_jobs_completed
worker_waited_this_frame
job_in_flight
reused_previous_frame
```

---

# Dev Console / Debug Overlay Rules

Console commands are engine-level unless explicitly mod-specific.

Each public command should usually live in its own file or focused module.

Command descriptors must be accurate because completion uses them.

Use categories:

```text
debug
render
postfx
scheduler
particles
audio
input
scene
```

Completion/hinting must use registry descriptors, not hardcoded command lists.

Debug overlay must render after all game effects.

---

# Module and File Granularity

Prefer cohesive files.

Good:

```text
render_runtime/composition.rs
render_runtime/graph.rs
render_runtime/diagnostics.rs
dev_console/completion.rs
renderer/graph/executor.rs
renderer/graph/resources.rs
```

Bad:

```text
one 2000-line render.rs owning graph, resources, post-fx, UI, debug, and diagnostics
misc.rs
utils.rs with domain logic
manager.rs without clear responsibility
```

Create a new file when:

```text
the concept has a stable name
it will be reused
it has tests or diagnostics
it reduces a monolithic file
```

Do not create a new file for one tiny helper unless it clarifies a boundary.

---

# Naming Rules

Avoid:

```text
new
old
v2
legacy
manager
stuff
helper
misc
temp
experimental
```

Use domain language:

```text
FrameCompositionPlan
FrameGraph
RenderViewPlan
RenderPassPlan
WgpuFrameRenderRequest
PostFxCertificationReport
DebugOverlayService
ConsoleCompletionState
```

Temporary migration names must be removed during cleanup.

---

# Refactor Completion Definition

A refactor is complete only when:

```text
new path is used by all call-sites
old path is deleted
compatibility helper is deleted
temporary flags are deleted
diagnostics exist
tests/checks pass
names match final architecture
no v2/legacy/experimental remains
```

Migration fallback is allowed during a pass, not as final state.

---

# Preferred Task Templates

## Template: Add Engine Contract

```text
1. change-plan/open-set for target area
2. add model file in engine crate
3. export from lib.rs
4. add tiny tests
5. cargo check target crate
6. no app wiring until contract compiles
```

## Template: Add Runtime Wiring

```text
1. trace runtime service/handler
2. slice exact registration/apply symbol
3. add service/command handling
4. verify touched app crate
5. add diagnostics
```

## Template: Refactor Render Path

```text
1. add engine-level contract
2. add app-level builder/packet changes
3. add diagnostics command
4. route host through new request object
5. add executor skeleton
6. switch to graph nodes
7. delete legacy path
```

## Template: Remove Legacy

```text
1. trace legacy symbol
2. verify no required call-sites remain
3. delete symbol/range
4. remove enum variants/config/commands
5. trace again
6. check touched crates
```

## Template: Fix Compile Error

```text
1. run targeted cargo check through fallout
2. trace exact missing symbol/type
3. slice owner symbol
4. patch exact issue
5. rerun same check
```

---

# Examples: Common Codemap Recipes

## Find where a command is registered

```powershell
& $cm trace "ConsoleCommandDescriptor" --limit 20
& $cm trace "register_builtin_console_commands" --limit 20
& $cm open-set "dev console command registry descriptors" --why --limit 8
& $cm slice crates/apps/app/src/dev_console/commands/mod.rs --symbol register_builtin_console_commands
```

## Work on render graph cleanup

```powershell
& $cm trace "LegacyComposite" --limit 20
& $cm trace "render_frame_request_legacy" --limit 20
& $cm trace "WgpuFrameGraphExecutionMode" --limit 20
& $cm open-set "remove legacy render path FrameGraph executor" --why --limit 12
```

## Work on particles performance

```powershell
& $cm trace "Particle2dSceneService" --limit 20
& $cm trace "draw_commands" --limit 20
& $cm trace "Particle2dDrawCommand" --limit 20
& $cm open-set "particles draw commands light sampling scheduler quality scale" --why --limit 12
```

## Work on scene YAML compiler

```powershell
& $cm trace "compile_scene_document_from_path" --limit 20
& $cm trace "SceneDocumentDependencyKind" --limit 20
& $cm trace "UiDocumentRef" --limit 20
& $cm open-set "scene compiler use refs modular yaml" --why --limit 12
```

## Work on post-fx

```powershell
& $cm trace "PostFx2d" --limit 20
& $cm trace "PostFx2dStack" --limit 20
& $cm trace "postfx.cert" --limit 20
& $cm open-set "PostFx2d LensDroplets certification render graph node" --why --limit 12
```

---

# Anchor Policy

Use codemap anchors for important architecture points.

Add/update anchors when touching:

```text
render graph contracts
frame composition builder
host runtime render flow
scene compiler entry
metadata catalog
target registry
scheduler/task system
dev console registry
post-fx certification
```

After anchor changes:

```powershell
& $cm anchors --write
& $cm anchor-check
```

Do not tag every file. Anchor important navigation points only.

---

# Final Response Checklist

Before reporting done:

```text
Did I use codemap-first?
Did I avoid broad scans?
Did I edit only targeted files?
Did I run ops-check --strict?
Did I run verify-plan --changed?
Did I run only touched crate checks/tests?
Did I remove legacy paths introduced by the task?
Did I avoid claiming unrun verification?
```

Final response format:

```text
Changed:
- file/symbol
- file/symbol

Verified:
- command
- command

Notes:
- skipped checks and why
- remaining follow-up
```

---

# Stop Conditions

Stop and report when:

```text
requested phase is complete
a required symbol cannot be found after trace/open-set
a change would require broad unrelated refactor
checks reveal unrelated pre-existing errors
task scope would expand beyond pasted plan
```

Do not broaden the task without instruction.

---

# Summary

Use codemap to spend precision instead of tokens.

For Amigo refactors, do not solve domain-owned runtime logic by labeling it `app.host`.
Move the behavior to the owning domain or leave an explicit blocker.
`app.host` is reserved for true host responsibilities only.

The correct Amigo workflow is:

```text
targeted discovery
small patch
strict ops check
minimal verify
cleanup legacy
concise report
```

The correct Amigo architecture direction is:

```text
engine-level contracts
app as glue
editor-ready APIs
no v2 systems
no permanent legacy
diagnostics-first
clean final names
```
