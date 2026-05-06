# Editor / Engine Contract

## Engine Owns
- Scene and prefab document schema.
- Component kind taxonomy.
- Component descriptors and capability metadata.
- Runtime interpretation of documents.

## Editor Owns
- UI, docking, and interaction flow.
- Asset registry scanning and graphing.
- Editor mode sessions and transactions.
- YAML patch planning and apply orchestration.
- Inspector and gizmo presentation.

## Hard Rules
- Editor must not infer component capabilities from ad-hoc UI string checks when descriptor metadata exists.
- Editor must treat engine document models as source of truth.
- Editor viewport interaction should be derived from snapshot/capabilities, not duplicated front-end geometry logic.

## Integration Pattern
1. Engine adds/updates component variant and descriptor.
2. Editor consumes descriptor capabilities.
3. Editor extends controls/inspector only where needed.

