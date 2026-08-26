# Contributing to Amigo

## Change size and reviewability

Keep changes small enough that architectural ownership and behavioral effects can be reviewed independently.

Recommended target:

- fewer than 100 changed files;
- fewer than 20,000 added + deleted lines;
- one architectural seam or behavior change per commit/change set.

The automated review budget fails pull requests above either hard limit:

- more than 200 changed files; or
- more than 50,000 added + deleted lines.

Large generated-file refreshes, mechanical removals, or vendored data should be isolated from semantic code changes. If an exceptional change must cross the hard budget, split generated/mechanical work from runtime behavior first and document why the remaining semantic change cannot be decomposed safely.

## Validation

Start with the smallest owner-crate validation, then expand as needed. Mainline CI runs formatting, plugin-contract validation, workspace checks, targeted clippy, and contract tests.
