# Current docs audit

## Root AGENTS.md

The existing repository `AGENTS.md` in the inspected snapshot is not suitable as a canonical root file.

Observed problems:

```text
very large file
starts with pasted chat response text
wrapped in outer markdown fence
broken encoding characters
mixes operations, architecture, historical plans, and final reports
contains stale snapshot references
```

Recommendation:

```text
MODIFY AGENTS.md
  Replace entire file with the curated root AGENTS.md from this bundle.
```

## PROJECT.md

No root `PROJECT.md` was found in the inspected snapshot. This bundle provides one.

Recommendation:

```text
ADD PROJECT.md
  Use this as canonical short project-state overview.
```

## arch.md

`arch.md` appears to be historical pasted planning material, not a canonical current architecture document.

Recommendation:

```text
MOVE arch.md -> docs/archive/semantic-scene-graph-refactor-plan-v79jx.md
```

Do not do this move in the same commit as code refactor.

## docs/architecture

Existing `docs/architecture/**` appears partially useful and should be preserved. Promote new docs there gradually.

## plugins/**/docs

Many plugin docs are placeholders. Do not mass-update unrelated plugin docs. Improve plugin docs when the plugin is touched.
