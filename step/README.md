# Amigo patch pack: editor metadata catalog foundation

This pack applies the first architecture step for editor metadata.

It updates the existing metadata catalog in place. It does not create any parallel `v2` system and does not add new component editing behavior yet.

## What changes

- Expands `EditorMetadataCatalogDto` with:
  - target kind descriptors
  - asset kind descriptors
  - document kind descriptors
  - control descriptors
  - patch operation descriptors
- Converts component `editorControls` and `patchOps` from flat strings to structured refs.
- Extends the Item Context navigator so selected entity components can show capabilities, policies, controls, and patch operations.

## Apply

Run from the repository root:

```powershell
.\apply.ps1
```

or, if the pack is outside the repository:

```powershell
.\apply.ps1 -RepoRoot C:\path\to\amigo
```

## Notes

- The plan uses CodeMap `content_from`, `replace_file`, strict checks, backup, and stop-on-error.
- `expected_hash` guards are based on the uploaded snapshot.
- The script refreshes CodeMap after apply.
