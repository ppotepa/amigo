# Metadata Implementation Plan

## Cel

Budujemy jeden typowany przeplyw edytora:

```text
engine scene/component model
  -> metadata descriptors
  -> backend DTO catalog
  -> EditorTargetRef + resolved target instance
  -> Item Context
  -> GenericPropertiesPanel
  -> generic patch command
  -> YAML write
  -> engine hydration/validation
  -> refreshed hierarchy/snapshot
```

Frontend nie powinien znac `Sprite2D`, `Text2D`, `AabbCollider2D` jako osobnych formularzy. Frontend powinien znac typy kontrolne:

```text
String, Number, Bool, Vec2, Vec3, Color, AssetRef, Enum, ReadOnly
```

oraz:

```text
target, component instance, property path, patch op, YAML sink
```

## Zasady

- Nie tworzymy rownoleglego `v2` ani legacy shimow.
- Jeden aktywny system targetow to `src/editor-targets/*`.
- String literal jest detalem serializacji, nie glownym API komponentow UI.
- Kod aplikacyjny ma uzywac typow, descriptorow, helperow i type guardow.
- Metadata mowi, co istnieje i jakie pola/akcje sa dostepne.
- Instance DTO mowi, jakie konkretne wartosci sa w aktualnym YAML.
- Generic renderer renderuje po `valueKind`/`editor`, a nie po nazwie komponentu.
- Specjalne narzedzia zostaja specjalne: tile brush, gizmos, UI visual editor, vertex editor.

## Co juz zrobione

### EditorTarget

- Dodany aktywny przeplyw `EditorTargetRef + intent`.
- `WorkspaceRuntimeServices` ma `currentEditorTarget`, `activateEditorTarget`, `targetBridge`.
- Panele Project/Files/Assets/Scene/UI/Diagnostics zostaly przepiete na target activation.
- `PropertiesPanel`/right top zaczal dzialac jako host primary target context.
- `TargetContextPanel`/right bottom renderuje secondary target context.
- `ResolvedEditorTarget` zostal rozszerzony o:
  - `capabilities`
  - `metadataRefs`
  - `documentRefs`
  - `relatedTargets`
  - `diagnostics`
  - `breadcrumbs`
  - `actions`
- Dodany `editorTargetSemantics.ts` jako aktywna taksonomia semantyczna targetow.
- Usunieto duplikat `src/features/editor-targets/*`.

### Metadata catalog

- Backendowy `EditorMetadataCatalogDto` zostal rozszerzony o:
  - target kind descriptors
  - component capabilities
  - structured controls
  - patch operations
  - asset kinds
  - document kinds
- Frontendowe typy metadata zostaly rozszerzone w `features/metadata/editorMetadataTypes.ts`.
- `ItemContextNavigator` dostal scaffold do pokazywania structured component metadata.

### Codemap support

- `amigo-codemap` obsluguje `content_from`.
- Dodane realne file ops: copy/move/rename/create dir/delete dir.
- `ops-apply` jest verbose domyslnie.
- `--no-verbose` wycisza output.
- `create_file`/`replace_file` akceptuja `overwrite`.
- `expected_hash` akceptuje FNV/8 i SHA-256/8, zeby paczki PowerShell i codemap byly kompatybilne.

## Braki do celu

### 1. Metadata catalog w runtime services

Pierwszy nastepny krok.

Backend juz ma katalog, ale frontend musi go ladowac jako runtime state.

Pliki:

```text
crates/apps/amigo-editor/src/api/editorApi.ts
crates/apps/amigo-editor/src/features/metadata/editorMetadataTypes.ts
crates/apps/amigo-editor/src/main-window/workspaceRuntimeServices.ts
crates/apps/amigo-editor/src/main-window/MainEditorWindow.tsx
crates/apps/amigo-editor/src/main-window/hooks/useWorkspaceRuntimeServices.ts
```

Do dodania:

```ts
getEditorMetadataCatalog(): Promise<EditorMetadataCatalogDto>
```

Do `WorkspaceRuntimeServices`:

```ts
metadataCatalog?: EditorMetadataCatalogDto | null;
metadataCatalogError?: string | null;
metadataCatalogLoading?: boolean;
```

Efekt:

```text
Item Context moze pokazac component descriptors z backendu.
Frontend nie potrzebuje hardcoded listy komponentow.
```

### 2. Component instance DTO

Bez realnych wartosci z YAML generic panel bedzie mial descriptor, ale nie bedzie wiedzial, co edytuje.

Backend:

```text
crates/apps/amigo-editor/src-tauri/src/dto.rs
crates/apps/amigo-editor/src-tauri/src/commands/project_tree.rs
crates/apps/amigo-editor/src-tauri/src/editor_mode/dto.rs
crates/apps/amigo-editor/src-tauri/src/editor_mode/document_snapshot.rs
```

Frontend:

```text
crates/apps/amigo-editor/src/api/dto.ts
crates/apps/amigo-editor/src/editor-targets/editorTargetResolver.ts
crates/apps/amigo-editor/src/features/target-context/ItemContextNavigator.tsx
crates/apps/amigo-editor/src/features/metadata/GenericPropertiesPanel.tsx
```

Docelowy DTO:

```ts
export type EditorSceneComponentInstanceDto = {
  componentIndex: number;
  typeName: string;
  descriptorKind?: string | null;
  label: string;
  yamlPath: string;
  values: unknown;
  properties: EditorResolvedPropertyValueDto[];
  assetRefs: EditorResolvedAssetRefDto[];
  diagnostics: EditorDiagnosticDto[];
};
```

Do `EditorSceneEntityDto`:

```ts
components: EditorSceneComponentInstanceDto[];
```

Efekt:

```text
Klik encji daje liste realnych komponentow z values i yamlPath.
```

### 3. Realny component target

Dodac wariant:

```ts
{
  kind: "component";
  sceneId: string;
  entityId: string;
  componentIndex: number;
  componentType: string;
}
```

Pliki:

```text
crates/apps/amigo-editor/src/editor-targets/editorTargetTypes.ts
crates/apps/amigo-editor/src/editor-targets/editorTargetSemantics.ts
crates/apps/amigo-editor/src/editor-targets/editorTargetResolver.ts
crates/apps/amigo-editor/src/editor-targets/editorTargetContextProfiles.ts
crates/apps/amigo-editor/src/editor-targets/editorTargetActivation.ts
crates/apps/amigo-editor/src/editor-targets/adapters/sceneTargetAdapter.ts
```

Efekt:

```text
Klik Sprite2D w Item Context aktywuje currentEditorTarget = component.
Right bottom pokazuje generic properties dla tego komponentu.
```

### 4. Podzial right top / right bottom

Docelowo:

```text
rightTop:
  ItemContextNavigator

rightBottom:
  GenericPropertiesPanel
  TargetActionsPanel
  TargetDiagnosticsPanel
  TargetHistoryPanel
  TargetSourcePreviewPanel
```

`rightTop` ma byc mapa kontekstu i relacji. Nie powinien byc ciezkim formularzem.

### 5. GenericPropertiesPanel read-only

Najpierw bez zapisu.

Input:

```ts
metadata: EditorMetadataCatalogDto | null;
target: ResolvedEditorTarget;
component?: EditorSceneComponentInstanceDto | null;
```

Renderer:

```text
Text -> text display/input later
Number -> number display/input later
Bool -> checkbox later
Vec2 -> x/y
Vec3 -> x/y/z
Color -> color
AssetRef -> asset relation/picker later
Enum -> select later
ReadOnly -> static value
```

Milestone:

```text
klik encji -> komponenty
klik komponentu -> read-only generic properties z realnych values
```

### 6. Generic SetProperty

Dopiero po read-only.

Command:

```ts
{
  type: "SetProperty";
  target: EditorTargetRef;
  propertyPath: string;
  value: EditorPropertyValueDto;
  patchOp?: string | null;
  expectedValue?: EditorPropertyValueDto | null;
}
```

Backend flow:

```text
resolve target
load scene YAML as serde_yaml::Value
find descriptor
validate property exists
validate editable access
validate value kind
apply YAML mutation
hydrate through engine
if ok: write file
refresh hierarchy/snapshot
```

Efekt:

```text
Zmiana Sprite2D.size.x zapisuje YAML i po reloadzie zostaje.
```

### 7. Add Component

Wymaga default YAML/generatora.

Flow:

```text
select entity
-> Add Component
-> frontend pokazuje metadata.components
-> user wybiera component
-> backend generuje default YAML
-> insert into entity.components
-> hydrate/validate
-> write YAML
-> current target = new component
```

Efekt:

```text
Dodanie prostego komponentu z backend descriptor + default YAML nie wymaga React panelu.
```

## Co stanie sie mozliwe

Po etapach metadata runtime + component instance DTO + component target:

```text
klik encji
  -> Item Context pokazuje entity props i components

klik komponentu
  -> currentEditorTarget = component
  -> right bottom pokazuje generic read-only properties z YAML values
```

Po `SetProperty`:

```text
generic panel edytuje YAML przez backend
backend waliduje hydratacja engine
frontend nie zna specjalnych formularzy komponentow
```

Po `AddComponent`:

```text
nowy backend component + descriptor + default YAML
= widoczny i podstawowo edytowalny w editorze bez nowego React panelu
```

## Granice automatycznosci

Automatyczne:

```text
String
Number
Bool
Vec2
Vec3
Color
AssetRef
Enum
ReadOnly
```

Specjalizowane:

```text
TileMapBrush2D
Transform gizmos
Collider handles
UI visual layout editor
Vector vertex editor
```

Regula:

```text
normalne pola -> GenericPropertiesPanel + SetProperty
interaktywne narzedzia -> specialized control, ale wlaczane przez metadata
```

## Checklista dla nowego komponentu engine

Kazdy nowy komponent powinien miec:

```text
1. engine struct/YAML model
2. hydratacje z YAML
3. ComponentKind/type name
4. metadata descriptor
5. properties descriptors
6. asset refs descriptors, jesli dotyczy
7. default YAML/generator
8. patch op albo SetProperty
9. descriptor coverage test
```

Docelowo:

```text
backend dodaje komponent + metadata
frontend uzywa go automatycznie
```

