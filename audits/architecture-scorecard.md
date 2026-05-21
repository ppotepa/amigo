# Architecture scorecard

## Current score

Overall: 7/10

| Category | Score | Reason |
|---|---:|---|
| Separation of responsibilities | 8 | App is thinner; runtime/bundles/plugins split is clear. |
| Pipeline consistency | 7 | Waterfalls exist, but bridges and renderer have manual paths. |
| Simplicity | 6.5 | Macrostructure is clean; renderer internals are still complex. |
| Resistance to ifology | 6 | Candidate/target contracts help, but central PostFX/debug branches remain. |
| Plugin orientation | 7 | Plugin tree is strong; docs/tests are uneven. |
| Lack of special cases | 6 | FocusBlur/debug/cached image/optics still have exceptions. |
| Naming/boundaries | 7.5 | Mostly clear, but engine/plugin/runtime lines still need enforcement. |
| Architecture tests/docs | 7.5 | Good direction; plugin waterfall docs need detail. |
| Scalability | 6.5 | New domains are possible; new PostFX still touches central renderer. |

## Verdict

The refactor is successful but incomplete. The largest remaining risk is no longer app-centric architecture; it is central renderer ownership of effect dispatch, pipeline bootstrap, debug ordering, and special paths.
