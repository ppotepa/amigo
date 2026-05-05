# amigo-modding

Mod discovery, manifest loading, and path resolution.

## Responsibility
- Load mod manifests.
- Resolve mod paths.
- Build requested mod chains.
- Expose catalog data for runtime and tools.

## Not here
- Scene hydration.
- Asset preparation.
- Script execution.

## Depends on
- amigo-core.
- amigo-runtime.
- serde.
- toml.
