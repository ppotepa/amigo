# AGENTS.md — Amigo Codex CLI Operating Contract

This file defines how AI coding agents must work in the Amigo repository.

The purpose of this file is not to explain every subsystem. Its purpose is to control agent behavior: how to inspect code, how to edit code, how to verify changes, and how to avoid wasting tokens, time, and compute.

If a user pastes a detailed implementation plan, that plan is the task scope. This file controls the execution discipline.

---

## 0. Prime Directive

Use `amigo-codemap` first.

Do not blindly scan the repository. Do not read large files in full. Do not run expensive workspace-wide commands unless explicitly requested. Do not broaden the scope.

Work like a precise codebase surgeon:

```text
Find the right symbols with codemap.
Read only the necessary slices.
Apply narrow raw ops.
Verify only what changed.
Report exactly what was done.
```

The Amigo repository is intentionally supported by `amigo-codemap`. It exists to prevent wasteful file-by-file exploration and to make LLM work deterministic, auditable, and cheap.

---

## 1. Hard Rules

### 1.1 Required by default

Always prefer this workflow:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"

& $cm brief
& $cm changes --compact --hide-generated --limit 20
& $cm change-plan "<task query>" --limit 20
& $cm open-set "<focused architecture/query>" --why --limit 12
& $cm trace "<symbol-or-string>" --limit 20
& $cm symbols --file <file> --metadata --limit 80
& $cm slice <file> --symbol <symbol>
```

Only use broader tools after codemap has narrowed the workset and only when required.

### 1.2 Forbidden by default

Do not use these unless the user explicitly asks or codemap cannot answer a necessary question:

```text
Get-Content <large file>
cat <large file>
type <large file>
rg over the whole repository as discovery
fd/find over the whole repository as discovery
reading concat-output.txt directly
cargo check --workspace
cargo test --workspace
cargo run without need
npm run build unless frontend/TS changed
full git diff as first inspection
opening files one by one manually
recursive scans of directories
broad rewrites without symbol/range control
```

### 1.3 Forbidden architectural behavior

Do not introduce:

```text
*v2 systems
parallel duplicate systems
temporary compatibility layers that are not removed
legacy fallback paths after migration is complete
app-only engine contracts
render hacks in host runtime
feature-specific if/else chains in core renderer
custom React/editor panels for every engine component by default
```

Prefer evolving existing systems in place with final names.

---

## 2. How to Use User-Pasted Plans

When the user pastes an implementation plan:

1. Treat the plan as authoritative scope.
2. Do not reinterpret it into a larger refactor.
3. Do not start by scanning the repo broadly.
4. Map each planned item to files/symbols using codemap.
5. If a referenced file or symbol changed, use `trace`, `open-set --why`, and `symbols` to find the equivalent.
6. Execute the requested phase only.
7. Stop when the phase is complete unless explicitly asked to continue.
8. Report deviations honestly.

Do not “helpfully” implement future phases early.

---

## 3. Standard Codemap Workflow

### 3.1 Initial discovery

Use this shape for almost every task:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"

& $cm brief
& $cm changes --compact --hide-generated --limit 20
& $cm change-plan "<task>" --limit 20
& $cm open-set "<focused query>" --why --limit 12
```

Examples:

```powershell
& $cm change-plan "render pipeline FrameCompositionPlan FrameGraph UI debug split" --limit 20
& $cm open-set "render-api AppRenderFramePacket render-wgpu post_fx_stack overlay" --why --limit 12
```

### 3.2 Focused symbol discovery

Use targeted traces:

```powershell
& $cm trace "AppRenderFramePacket" --limit 20
& $cm trace "render_scene_with_ui_primitives_and_3d_commands" --limit 20
& $cm trace "ConsoleCommandDescriptor" --limit 20
& $cm trace "Particle2dSceneService" --limit 20
```

Do not use `rg` across the whole repo for these.

### 3.3 Reading code

Prefer:

```powershell
& $cm symbols --file <file> --metadata --limit 80
& $cm slice <file> --symbol <symbol>
```

Use `range-for-symbol` only when a raw operation needs line anchors:

```powershell
& $cm range-for-symbol <file> <symbol>
```

Use `range-for-lines` only after codemap has already identified the range.

### 3.4 If codemap output is insufficient

Allowed escalation order:

```text
1. trace another exact symbol/string
2. open-set with a better focused query
3. symbols --file
4. slice --symbol
5. range-for-symbol
6. small range read
7. only then consider normal file read for a small file
```

Never jump directly to broad repository scanning.

---

## 4. Editing Rules

### 4.1 Use raw ops inline

All patches should be applied as raw ops through stdin:

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

### 4.2 Do not use YAML ops plans

Do not use:

```text
ops-skeleton --out plan.yml
ops-check --from plan.yml
ops-apply --from plan.yml
content_from
content_root
.amigo/ops/*
```

### 4.3 Preferred operation types

Prefer, in order:

```text
CREATE_FILE for new small modules
INSERT_AFTER_TEXT for module declarations/exports
INSERT_BEFORE_TEXT for registration points
REPLACE_TEXT for exact narrow replacements
REPLACE_RANGE only when codemap provides safe range/hash
REPLACE_FILE only for small files or intentionally total rewrites
APPEND_TO_FILE only for tests or clearly append-only sections
```

Do not use broad `REPLACE_FILE` for large files unless explicitly planned.

### 4.4 One concern per patch

Keep patches focused:

```text
Patch A: add model types
Patch B: export module
Patch C: add builder
Patch D: update call site
Patch E: add tests
```

Avoid combining unrelated architecture changes.

### 4.5 Preserve working fallback during migration

During migration, fallback is allowed.

After migration, fallback must be deleted.

Example:

```text
Allowed temporarily:
render_frame_request_legacy()

Required cleanup later:
remove render_frame_request_legacy()
remove old call sites
remove compatibility overlay()
remove app-level post-fx hacks
```

---

## 5. Verification Rules

### 5.1 Always verify changed plan first

```powershell
& $cm verify-plan --changed
```

### 5.2 Use minimal crate checks

Run only checks for touched crates.

Examples:

```powershell
cargo check -p amigo-render-api 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-app 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-render-wgpu 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-scene 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-2d-particles 2>&1 | & $cm fallout --limit 80
cargo check -p amigo-2d-post-fx 2>&1 | & $cm fallout --limit 80
```

### 5.3 Targeted tests only

Run exact tests where possible:

```powershell
cargo test -p amigo-render-api composition_plan_detects_post_fx 2>&1 | & $cm fallout --limit 80
cargo test -p amigo-app completion_suggests_registered_debug_commands 2>&1 | & $cm fallout --limit 80
cargo test -p amigo-2d-particles draw_command_preserves_particle_velocity 2>&1 | & $cm fallout --limit 80
```

### 5.4 Avoid workspace commands

Do not run:

```text
cargo check --workspace
cargo test --workspace
```

unless explicitly instructed.

### 5.5 Use fallout for compiler output

Always pipe long build/test output through `fallout`:

```powershell
cargo check -p amigo-app 2>&1 | & $cm fallout --limit 80
```

Do not paste entire compiler logs into the response.

---

## 6. Reporting Rules

Final report should include:

```text
Changed files
Major symbols touched
Commands run
Verification status
Known issues / skipped items
Next recommended step
```

Be concise but precise.

Do not claim tests passed if they were not run.

If verification failed, include only the relevant fallout summary.

---

## 7. Amigo Architecture Rules

### 7.1 No v2 systems

Never create:

```text
SceneDocumentV2
RendererV2
SchedulerV2
UiSystemV2
PostFxV2
```

The project is fresh. Evolve and rename existing systems in place.

### 7.2 Cleanup as we go

Do not leave stale paths, duplicate systems, compatibility wrappers, or unused modules unless there is a clearly stated short-term migration reason.

When migration is complete, delete old paths in the same task or in a clearly defined cleanup phase.

### 7.3 Engine contracts belong in engine crates

If a concept must be reused by runtime app, editor app, or tests, it belongs in an engine crate.

Examples:

```text
FrameCompositionPlan      -> crates/engine/render-api
FrameGraph                -> crates/engine/render-api
Render diagnostics        -> crates/engine/render-api
Scene document contracts  -> crates/engine/scene
Post-fx model             -> crates/2d/post-fx
```

App-specific glue belongs in:

```text
crates/apps/app
```

The app crate must not become the source of truth for engine architecture.

### 7.4 Future editor compatibility

Always design engine APIs so a future editor app can use them.

Editor must be able to render:

```text
scene preview
camera preview
isolated entity preview
UI document preview
post-fx preview
render graph diagnostics
```

without copying runtime app logic.

### 7.5 App crate is glue

`crates/apps/app` may:

```text
collect host input
own the window loop
register dev console commands
connect services
build app-specific render packets
```

It must not own reusable engine contracts.

---

## 8. Render Architecture Rules

### 8.1 Renderer core must not know game features directly

Avoid patterns like:

```rust
if lens_droplets_enabled {
    render_lens_droplets();
}
```

Prefer:

```text
Scene/Runtime data
  -> FrameCompositionPlan
  -> FrameGraph
  -> graph nodes
  -> WGPU executor
```

### 8.2 Target architecture

Desired flow:

```text
Scene/YAML
  -> SceneCompiler / Hydration
  -> Runtime services
  -> Extract render data
  -> FrameCompositionPlan
  -> FrameGraph
  -> WGPU executor
  -> Present
```

### 8.3 Explicit pass order

Default composition order:

```text
world_2d / world_3d
post_fx_after_world
game_ui
debug_overlay / console
present
```

Debug UI must be after all post-fx.

### 8.4 UI split is mandatory

Do not treat all UI overlays as one category.

Use:

```text
game_ui_overlay
debug_overlay
```

Game UI can be affected by scene composition policy.
Debug UI must not be affected by game post-fx.

### 8.5 Post-fx must be graph nodes

Post-fx like:

```text
LensDroplets
Blur
Bloom
EmbossEdges
```

must be render graph features/nodes, not `host_runtime` hacks and not UI overlays.

### 8.6 Legacy render path cleanup

During migration, legacy path is acceptable.

Final state must be:

```text
renderer.render_frame(request)
  -> FrameGraphExecutor::execute(...)
```

No permanent:

```text
render_frame_request_legacy
render_scene_with_ui_primitives_and_3d_commands public path
manual overlay concatenation
app-level lens droplets fallback
```

---

## 9. Scene / Mod Authoring Rules

### 9.1 Scope-based structure

Mod-level folders are reusable across the mod.

Scene-level folders are local to that scene.

```text
mods/<mod>/
  ui/
  audio/
  scripts/
  prefabs/
  scenes/<scene>/
    visual/
    entities/
    ui/
    input/
    events/
    state/
    timelines/
    scripts/
```

If an asset/component/descriptor is nested under a scene, it belongs only to that scene.
Moving it to mod-level makes it reusable by other scenes.

### 9.2 No generic `parts/` convention

Do not introduce `parts/` as the canonical scene structure.

Use domain folders:

```text
visual
entities
ui
input
events
state
timelines
audio
scripts
```

### 9.3 Scene manifest is composition

`scene.yml` should be a manifest, not a runtime dump.

Preferred pattern:

```yaml
version: 1
scene:
  id: main-menu
  label: Main Menu

use:
  visual:
    - ./visual/render.yml
    - ./visual/lighting.yml
  entities:
    - ./entities/background.yml
    - ./entities/rain.yml
  ui:
    - ./ui/mount.yml
  input:
    - ./input/actions.yml
  events:
    - ./events/pipelines.yml
  state:
    - ./state/defaults.yml

script: ./scene.rhai
```

### 9.4 SceneCompiler owns assembly

Authoring files are modular.
Runtime may still receive a compiled `SceneDocument`.

```text
authoring YAML -> SceneCompiler -> runtime SceneDocument
```

### 9.5 Optional script capability

Everything can be script-backed, but nothing must be script-backed.

Use optional script bindings:

```yaml
script:
  source: ./local.rhai
  hooks:
    - on_load
    - update
  params:
    intensity: 0.7
```

Do not create separate script systems per asset type unless required.

---

## 10. Scheduler / Job System Rules

### 10.1 Do not confuse threading with performance

A worker that is spawned and immediately joined does not hide work from the frame.

Bad:

```text
main -> spawn worker -> join immediately -> render
```

Good:

```text
main renders previous prepared result
worker computes next result
main swaps when ready
```

### 10.2 Workers must not mutate world directly

Workers get snapshots and return typed results.

Main thread applies results deterministically.

### 10.3 Logical lanes, not physical cores

Use lanes:

```text
main
simulation
render_prepare
background
```

Do not pin hard responsibilities to physical CPU cores unless explicitly requested.

### 10.4 Scheduling YAML is policy, not thread control

YAML may declare:

```yaml
scheduling:
  mode: hybrid
  overrides:
    - target: entity:rain/component:ParticleEmitter2D
      lane: simulation
      allow_frame_latency: true
      quality_scale: 0.5
      budget_ms: 0.8
```

YAML must not declare:

```yaml
core: 3
thread: 2
```

### 10.5 Validate scheduling overrides

Always report unmatched overrides.

A scheduling override that silently matches nothing is a bug.

---

## 11. Particle System Rules

### 11.1 Preserve visual behavior before optimization

When optimizing particles:

1. Add diagnostics first.
2. Verify target matching.
3. Verify particle counts.
4. Add new path with fallback.
5. Compare old/new.
6. Only then tune quality.

### 11.2 Useful diagnostics

Provide commands/stats for:

```text
live particles
spawned particles
emitter names
quality scale
matched scheduling overrides
worker waited or not
job in flight or not
previous frame reused or not
```

### 11.3 Render-prep often matters more than simulation

For many particles, heavy cost may be:

```text
draw command generation
sorting
light sampling
vertex building
GPU buffer creation
```

Do not assume moving simulation to a worker is enough.

---

## 12. Dev Console Rules

### 12.1 Console commands are engine-level unless stated otherwise

Debug commands must not be mod-specific.

Examples:

```text
debug.fps
debug.fps_graph
debug.stats
render.plan
render.graph
postfx.cert
scheduler.stats
particles.stats
```

### 12.2 One command file per command group or command

For many debug commands, prefer separate files:

```text
dev_console/commands/debug/fps.rs
dev_console/commands/debug/fps_graph.rs
dev_console/commands/debug/stats.rs
```

For tightly related commands, one group file is acceptable.

### 12.3 Completion / hinting

Console completion should use the command registry as source of truth.

First version should support:

```text
command name prefix completion
alias completion
simple enum args from usage strings: on|off|toggle
Tab accept/common prefix
Up/Down select suggestions when popup active
Up/Down history when popup inactive
Escape closes suggestions before closing console
```

Do not add custom completers per command in the first pass.

---

## 13. Debug Overlay Rules

Debug overlay must be engine-level.

It should show runtime/render data independent of mod.

Useful panels:

```text
FPS
frame time graph
render stats
particles
scheduler
input
audio
lights
layers
timings
memory placeholder
```

Debug overlay must render after game post-fx.

Post-fx must not affect debug overlay or dev console.

---

## 14. Post-FX Rules

### 14.1 Certified effects

Expensive effects must have certification/validation.

For effects like LensDroplets:

```text
max droplets
blur samples
blur radius
distortion
downsample
affects_debug_ui forbidden
stage validation
cost score
strict mode
warnings/errors
```

### 14.2 LensDroplets architecture

Lens droplets are not world particles.
They are screen-space/lens post-fx.

Correct:

```text
world render -> lens droplets post-fx -> game UI -> debug UI
```

Incorrect:

```text
LensDroplets as host_runtime-created UI overlay
LensDroplets as ParticleEmitter2D
LensDroplets hardcoded inside renderer core
```

---

## 15. File/Crate Placement Guidelines

### 15.1 Render contracts

```text
crates/engine/render-api
```

Put here:

```text
FrameCompositionPlan
RenderViewPlan
RenderPassPlan
FrameGraph
FrameGraphResource
RenderCompositionDiagnostics
RenderFeature traits if engine-level
```

### 15.2 WGPU implementation

```text
crates/engine/render-wgpu
```

Put here:

```text
WgpuFrameRenderRequest
WgpuFrameGraphExecutor
Wgpu transient resources
WGPU graph node executors
WGPU pipelines/shaders
```

### 15.3 Runtime app glue

```text
crates/apps/app
```

Put here:

```text
AppRenderFramePacket
AppFrameCompositionBuilder
app-specific extractors
host runtime window loop
dev console commands
runtime debug overlay
```

### 15.4 Scene contracts

```text
crates/engine/scene
```

Put here:

```text
SceneDocument structs
SceneCompiler
hydration commands
scene YAML validation
```

### 15.5 2D effect models

```text
crates/2d/post-fx
```

Put here:

```text
PostFx2d
PostFx2dStack
LensDroplets model/certification
future post-fx data models
```

---

## 16. Naming Rules

Use final names, not temporary versioned names.

Preferred:

```text
FrameCompositionPlan
FrameGraph
WgpuFrameRenderRequest
RenderCompositionDiagnosticsService
DebugOverlayService
ConsoleCompletionState
```

Avoid:

```text
FrameCompositionPlanV2
NewRenderer
BetterRenderer
ExperimentalRenderer
TempRenderer
```

Temporary private helpers may be named `legacy` during migration, but they must be deleted.

---

## 17. Anchors and Codemap Maintenance

When making significant architecture changes, add or update meaningful `@codemap` anchors where useful.

Good anchor locations:

```text
render composition builder
frame graph builder
WGPU graph executor
host runtime render handoff
scene compiler entry
post-fx certification
scheduler task system
console command registry
```

Do not spam anchors everywhere.

After adding anchors:

```powershell
& $cm anchors --write
& $cm anchor-check
```

Only do this when anchors are part of the task or architecture changed meaningfully.

---

## 18. Cleanup Policy

### 18.1 During migration

Allowed temporarily:

```text
legacy wrapper
compatibility method
old function still called by one path
```

### 18.2 After migration

Must remove:

```text
legacy wrapper
old call site
duplicated method
unused compatibility helper
unused imports
stale tests
```

### 18.3 No permanent legacy

Final code should have one obvious path.

Example final render flow:

```text
host_runtime
  -> build AppRenderFramePacket
  -> build FrameCompositionPlan
  -> build FrameGraph
  -> WgpuFrameRenderRequest
  -> renderer.render_frame(request)
  -> WgpuFrameGraphExecutor::execute
```

---

## 19. Common Task Templates

### 19.1 Adding a new engine-level model

```powershell
& $cm open-set "<model name> engine contract related files" --why --limit 12
& $cm slice <crate>/src/lib.rs --symbol <exports or main module>
```

Then:

```text
CREATE_FILE model module
INSERT module export
ADD tests
cargo check target crate
```

### 19.2 Modifying a function

```powershell
& $cm trace "<function_name>" --limit 20
& $cm slice <file> --symbol <function_name>
```

Then use `REPLACE_TEXT` or `REPLACE_RANGE` based on the slice.

### 19.3 Moving logic out of host_runtime

1. Locate exact block with `slice on_redraw_requested`.
2. Add new service/extractor/helper first.
3. Switch host to call new abstraction.
4. Verify.
5. Delete old helper/block.

Do not delete before replacement compiles.

### 19.4 Adding a console command

1. Inspect command registry with codemap.
2. Add command handler file or group.
3. Register in `commands/mod.rs`.
4. Add descriptor.
5. Add targeted registry test if useful.
6. `cargo check -p amigo-app`.

### 19.5 Adding YAML support

1. Add document struct in `crates/engine/scene/src/document/...`.
2. Add serde defaults.
3. Add validation/certification if cost-sensitive.
4. Add hydration command if runtime service needs it.
5. Add scene compiler merge/duplicate logic if needed.
6. Add focused parsing test.
7. Check `amigo-scene` and dependent app crate.

---

## 20. Response Checklist for Agents

Before final response, answer:

```text
Did I use codemap first?
Did I avoid broad scans?
Did I edit only scoped files?
Did I avoid workspace checks?
Did I run verify-plan --changed?
Did I run minimal relevant checks?
Did I leave any legacy path?
Did I report unverified areas honestly?
```

Final response format:

```text
Implemented:
- ...

Changed files:
- ...

Verification:
- ...

Notes:
- ...

Next step:
- ...
```

---

## 21. If Things Go Wrong

If a raw op fails:

1. Do not switch to broad file reading.
2. Use `slice` or `range-for-symbol` on the affected symbol.
3. Adjust the raw op with exact context.
4. Retry once.
5. If still failing, report the mismatch and stop.

If a check fails:

1. Pipe through `fallout`.
2. Fix the smallest compile error first.
3. Do not chase unrelated warnings.
4. Do not run broader commands to compensate.

If architecture differs from the plan:

1. Use `trace` and `open-set --why` to find the current equivalent.
2. Keep the original goal.
3. Do not invent a separate parallel system.

---

## 22. Project-Specific Current Priorities

The current long-term direction of Amigo is:

```text
scope-based mod/scene authoring
SceneCompiler-driven modular YAML
engine-level render composition and frame graph
post-fx as graph features
runtime debug overlay and console completion
scheduler/job-compatible systems
future editor reuse of engine contracts
```

When implementing, prefer changes that move the project toward these goals.

---

## 23. Final Principle

Do not optimize for appearing busy.

Optimize for:

```text
smallest necessary read set
smallest necessary patch
highest architectural clarity
best future reuse
cleanest cleanup
cheapest verification
```

Use `amigo-codemap` as the primary interface to the codebase.
