# Composite Diagnostics

Primary channel: `postfx.composite`.

## Stack Diagnostics
`diagnose_post_fx_stacks` reports warnings for unsupported scope and pipeline
pairs and for duplicated photographic families.

## Duplicate Families
Specific warnings cover duplicated camera film scan, lens surface, look, shutter,
and depth-of-field style effects. A generic `duplicate_photographic_family`
warning is used when no specific code matches.

Diagnostics should include host id, effect id, family, severity, code, and a
message that names the conflicting stack.
