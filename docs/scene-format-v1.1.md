# Scene Format v1.1

## Scope
This version adds optional prefab instance fields to scene entities and introduces standalone prefab documents.

## New Entity Fields
- `prefab` (optional)
- `prefab_overrides` (optional, defaults to empty)

Example:

```yaml
entities:
  - id: start-button
    prefab:
      id: ink-wars/ui/menu-button
    prefab_overrides:
      - target: text
        value: START
```

## New Document Type
- `*.prefab.yml` / `*.prefab.yaml`

Prefab documents contain reusable entity trees and exposed override points.

## Compatibility
- Existing scenes without `prefab` fields remain valid.
- Existing component schemas are unchanged.
- Prefab support is opt-in at document level.

