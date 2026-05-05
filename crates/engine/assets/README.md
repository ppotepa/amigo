# amigo-assets

Asset catalog and preparation layer for the engine runtime.

## Responsibility
- Asset manifests and catalog entries.
- Asset load/preparation state.
- Prepared asset payloads used by runtime systems.

## Not here
- Filesystem watching.
- Renderer GPU upload code.
- Domain-specific asset interpretation beyond catalog/preparation.

## Depends on
- amigo-core.
- amigo-runtime.
- serde_yaml.
- toml.
