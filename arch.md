Poniżej masz **pełny plan refaktoru dla snapshotu `concat-output-v79jx.zip`** jako instrukcję dla agenta/kodera. To jest plan pod obecny stan repo, nie ogólna teoria.

Założenie refaktoru:

```text
Docelowo architektura 2D ma być semantyczna:

Scene2D
├─ post_fx[]                         // Frame scope
├─ DrawLayer2D
│  └─ post_fx[]                      // DrawLayer scope
└─ SceneObject2D
   ├─ Transform2D                    // transform jest własnością obiektu
   ├─ post_fx[]                      // object/group scope
   └─ Component2D
      └─ Sprite2D / LayeredImage2D / Text2D / ...
         ├─ Renderable2D
         ├─ UsesTransform2D
         ├─ RenderLayered2D
         ├─ HasBounds2D / HasAssetRefs gdzie dotyczy
         └─ post_fx[] opcjonalnie
```

Najważniejsze: **runtime może dalej używać draw commandów**, ale authoring/editor/metadata muszą mieć jeden czysty semantic scene graph.

---

# 0. Meta-prompt dla agenta

To możesz wkleić jako początek promptu do Codex/agenta:

```md
Pracujesz na snapshotcie `concat-output-v79jx.zip` projektu Amigo.

Nie przeszukuj całego repo losowo. Zakres refaktoru jest znany i opisany poniżej. Używaj tylko celowanego `rg` dla symboli wymienionych w planie, np.:

- `scoped_stacks`
- `post_fx_stacks`
- `effect_id`
- `SetPostFx2dStacks`
- `PostFxPassPlan`
- `FrameGraphNodeKind::PostFx`
- `PostFx2dRenderOutput`
- `SceneGraphNodeKind`
- `MetadataTraitKind`
- `SceneEntityDocument`
- `RenderLayer2dDocument`
- `SceneComponentDocument`
- `LayeredImageLayerOverrideDocument`

Nie zmieniaj architektury poza opisanym zakresem.

Cel refaktoru:
1. Uporządkować model 2D scene graphu:
   - Scene2D jako root i PostFxHost2D.
   - SceneObject2D jako właściciel Transform2D.
   - Komponenty renderowalne jako Renderable2D + UsesTransform2D.
   - DrawLayer2D jako osobny authored draw layer, nie backend CompositionLayer.
2. Dodać capabilities do semantic graph nodes.
3. Rozszerzyć schema o `post_fx[]` na draw layer, scene object, renderable component i image part/layer override.
4. Zastąpić legacy frame-only `PostFx2dStack` w runtime przez `Vec<ScopedPostFx2dStack>`.
5. Zastąpić liczbową tożsamość passów post-fx przez stabilne pola:
   - `PostFxHost2dId`
   - `PostFx2dId`
   - `PostFxScope2d`
   - `PostFxPipelineKind`
6. Frame-level post-fx musi dalej działać identycznie jak wcześniej.
7. Non-frame post-fx scopes mogą zostać zarejestrowane i widoczne w graph/editor/runtime packet, ale renderer ma obsługiwać tylko te pipeline, które są rzeczywiście zaimplementowane. Dla nieobsługiwanych scope’ów ma być diagnostyka, nie cicha awaria.
8. Nie dodawaj compatibility shimów poza krótkimi helperami `legacy_*` potrzebnymi do przejścia testów.
9. Po każdej fazie uruchamiaj minimalne testy/cargo check dla dotkniętych crate’ów.
```

---

# 1. Docelowe nazewnictwo

W całym refaktorze trzymaj takie nazwy:

```text
Scene2D             = semantyczny root sceny 2D
SceneObject2D       = obiekt sceny, właściciel transform2
Component2D         = komponent na obiekcie
Renderable2D        = komponent dający draw command
DrawLayer2D         = authored layer w scenie/modzie
CompositionLayer    = backendowa warstwa kompozycji renderera
PostFxHost2D        = dowolny node, który ma `post_fx[]`
PostFxScope2D       = zakres działania efektu
ImagePart           = część assetu/obrazu, np. warstwa layered image
```

Nie mieszać:

```text
DrawLayer2D != CompositionLayer
SceneObject2D != RenderCommand
Scene graph != Raw YAML tree
post_fx scope != frame pass index
```

---

# 2. Faza A — semantic capabilities w `engine/scene`

## A1. ADD `crates/engine/scene/src/graph/semantics.rs`

Dodać nowy plik:

```rust
use crate::metadata_traits::MetadataTraitKind;

/// Semantic capabilities attached to a scene graph node.
///
/// This is intentionally independent from YAML shape.
/// YAML is only one source format. Editor, hydration and diagnostics should
/// reason about capabilities instead of raw document keys.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SceneGraphSemantics {
    pub traits: Vec<MetadataTraitKind>,
    pub role: Option<SceneGraphSemanticRole>,
    pub post_fx_host: Option<SceneGraphPostFxHost>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneGraphSemanticRole {
    Scene2D,
    SceneSettings2D,
    DrawLayer2D,
    SceneObject2D,
    Component2D,
    Renderable2D,
    ImagePart2D,
    LightGroup2D,
    LightRoute2D,
    AssetProxy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneGraphPostFxHost {
    pub host_id: String,
    pub scope_label: String,
}

impl SceneGraphSemantics {
    pub fn new(role: SceneGraphSemanticRole) -> Self {
        Self {
            role: Some(role),
            ..Self::default()
        }
    }

    pub fn with_traits(mut self, traits: impl IntoIterator<Item = MetadataTraitKind>) -> Self {
        for trait_kind in traits {
            self.push_trait(trait_kind);
        }
        self
    }

    pub fn with_post_fx_host(
        mut self,
        host_id: impl Into<String>,
        scope_label: impl Into<String>,
    ) -> Self {
        self.post_fx_host = Some(SceneGraphPostFxHost {
            host_id: host_id.into(),
            scope_label: scope_label.into(),
        });
        self
    }

    pub fn push_trait(&mut self, trait_kind: MetadataTraitKind) {
        if !self.traits.contains(&trait_kind) {
            self.traits.push(trait_kind);
        }
    }

    pub fn has_trait(&self, trait_kind: MetadataTraitKind) -> bool {
        self.traits.contains(&trait_kind)
    }
}
```

Cel: każdy node scene graphu może dostać semantykę, a nie tylko `kind`.

---

## A2. ADD `crates/engine/scene/src/graph/component_capabilities.rs`

Dodać registry capabilities komponentów 2D.

```rust
use crate::document::components::SceneComponentDocument;
use crate::metadata_traits::MetadataTraitKind;

/// Static semantic capabilities for scene components.
///
/// Runtime can still hydrate components manually, but editor/diagnostics must
/// use this table as the shared semantic contract.
pub fn component_2d_traits(component: &SceneComponentDocument) -> Vec<MetadataTraitKind> {
    use MetadataTraitKind::*;

    match component {
        SceneComponentDocument::Sprite2d { .. } => vec![
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            HasAssetRefs,
            Selectable,
            RuntimeControllable,
            Patchable,
        ],

        SceneComponentDocument::LayeredImage2d { .. } => vec![
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            HasAssetRefs,
            Selectable,
            RuntimeControllable,
            Patchable,
        ],

        SceneComponentDocument::TileMap2d { .. } => vec![
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            HasAssetRefs,
            Selectable,
        ],

        SceneComponentDocument::Text2d { .. } => vec![
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            Selectable,
            RuntimeControllable,
            Patchable,
        ],

        SceneComponentDocument::VectorShape2d { .. } => vec![
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            Selectable,
        ],

        SceneComponentDocument::ParticleEmitter2d { .. } => vec![
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            Simulatable,
            RuntimeControllable,
            Patchable,
        ],

        SceneComponentDocument::BeaconLight2d { .. } => vec![
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            LightReceiver2D,
            Selectable,
        ],

        SceneComponentDocument::Camera2d { .. }
        | SceneComponentDocument::CameraFollow2d { .. } => {
            vec![UsesTransform2D, Camera, RuntimeControllable, Patchable]
        }

        SceneComponentDocument::Motion2d { .. } => {
            vec![UsesTransform2D, Motion2D, RuntimeControllable, Patchable]
        }

        SceneComponentDocument::PhysicsBody2d { .. }
        | SceneComponentDocument::Collider2d { .. }
        | SceneComponentDocument::Trigger2d { .. } => {
            vec![UsesTransform2D, Collidable2D, RuntimeControllable, Patchable]
        }

        SceneComponentDocument::Script { .. } => {
            vec![Scriptable, RuntimeControllable, Patchable]
        }

        _ => Vec::new(),
    }
}

pub fn component_is_renderable_2d(component: &SceneComponentDocument) -> bool {
    component_2d_traits(component).contains(&MetadataTraitKind::Renderable2D)
}

pub fn component_uses_transform_2d(component: &SceneComponentDocument) -> bool {
    component_2d_traits(component).contains(&MetadataTraitKind::UsesTransform2D)
}
```

Jeżeli nazwy wariantów w enumie różnią się minimalnie w lokalnym kodzie, agent ma dopasować match do istniejącego `SceneComponentDocument` z `crates/engine/scene/src/document/components.rs`.

---

## A3. MODIFY `crates/engine/scene/src/graph/mod.rs`

Dodać exporty:

```rust
pub mod component_capabilities;
pub mod semantics;
```

Jeżeli `mod.rs` ma już listę modułów, dopisać do istniejącej listy.

---

## A4. MODIFY `crates/engine/scene/src/graph/node.rs`

Obecnie `SceneGraphNode` ma pola mniej więcej w liniach 41–49:

```rust
pub struct SceneGraphNode {
    pub id: SceneGraphNodeId,
    pub label: String,
    pub kind: SceneGraphNodeKind,
    pub source: SceneGraphSource,
    pub parent: Option<SceneGraphNodeId>,
    pub children: Vec<SceneGraphNodeId>,
}
```

Dodać import:

```rust
use super::semantics::SceneGraphSemantics;
```

Dodać nowe pole:

```rust
pub semantics: SceneGraphSemantics,
```

Docelowo:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneGraphNode {
    pub id: SceneGraphNodeId,
    pub label: String,
    pub kind: SceneGraphNodeKind,
    pub source: SceneGraphSource,
    pub parent: Option<SceneGraphNodeId>,
    pub children: Vec<SceneGraphNodeId>,
    pub semantics: SceneGraphSemantics,
}
```

W konstruktorze `SceneGraphNode::new(...)` dodać:

```rust
semantics: SceneGraphSemantics::default(),
```

Dodać helper:

```rust
impl SceneGraphNode {
    pub fn with_semantics(mut self, semantics: SceneGraphSemantics) -> Self {
        self.semantics = semantics;
        self
    }
}
```

---

## A5. MODIFY `crates/engine/scene/src/graph/node.rs` — rozszerzyć `SceneGraphNodeKind`

Obecnie enum ma m.in.:

```rust
Root,
Settings,
Visual2d,
Objects,
SceneObject,
Components,
Component,
DrawLayers,
DrawLayer,
FramePostFxHost,
PostFxItem,
...
ImagePart,
```

Dodać/zmienić nazwy tak, żeby semantyka była czytelna, ale bez masowego rename’u w pierwszym commicie.

Minimalna zmiana:

```rust
Scene2d,
PostFxHost,
```

Docelowy enum:

```rust
pub enum SceneGraphNodeKind {
    Root,
    Scene2d,
    Settings,
    Visual2d,
    Objects,
    SceneObject,
    Components,
    Component,
    DrawLayers,
    DrawLayer,
    PostFxHost,
    FramePostFxHost,
    PostFxItem,
    LightGroups,
    LightGroup,
    LightRoutes,
    LightRoute,
    ImagePart,
    Resources,
    Curve2d,
    AssetProxy,
}
```

`FramePostFxHost` może zostać tymczasowo dla kompatybilności, ale nowe hosty powinny używać `PostFxHost`.

---

## A6. MODIFY `crates/engine/scene/src/graph/build.rs`

Ten plik jest kluczowy. Obecnie `build_semantic_scene_graph` buduje root, settings, visual2d, draw layers, frame post-fx host, objects i components.

Dodać importy:

```rust
use super::component_capabilities::component_2d_traits;
use super::semantics::{SceneGraphSemanticRole, SceneGraphSemantics};
use crate::metadata_traits::MetadataTraitKind;
```

### Root node

Przy tworzeniu root node dodać semantykę:

```rust
.with_semantics(
    SceneGraphSemantics::new(SceneGraphSemanticRole::Scene2D)
        .with_traits([
            MetadataTraitKind::SceneDocument,
            MetadataTraitKind::HasEntities,
            MetadataTraitKind::HasDiagnostics,
        ])
        .with_post_fx_host(
            format!("scene:{}:visual2d", document.scene.id),
            "Frame",
        ),
)
```

### Visual2D settings node

Dodać:

```rust
.with_semantics(
    SceneGraphSemantics::new(SceneGraphSemanticRole::SceneSettings2D)
)
```

### Draw layer node

W miejscu tworzenia draw layer node, okolice linii 75–106, dodać:

```rust
.with_semantics(
    SceneGraphSemantics::new(SceneGraphSemanticRole::DrawLayer2D)
        .with_traits([
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::Patchable,
            MetadataTraitKind::RuntimeControllable,
        ])
        .with_post_fx_host(
            format!("draw_layer:{}", layer.id),
            "DrawLayer",
        ),
)
```

Uwaga: `DrawLayer2D` nie jest renderable w tym samym sensie co sprite, ale jest render hostem i runtime-controllable. Jeżeli nie chcesz semantycznie oznaczać warstwy jako `Renderable2D`, usuń `Renderable2D` z tej listy. Ja bym dał raczej nowy trait `PostFxHost2D` w fazie A7 i nie dawał `Renderable2D`.

### Scene object node

W okolicach linii 181–203, przy `SceneObject`, dodać:

```rust
.with_semantics(
    SceneGraphSemantics::new(SceneGraphSemanticRole::SceneObject2D)
        .with_traits([
            MetadataTraitKind::HasIdentity,
            MetadataTraitKind::HasVisibility,
            MetadataTraitKind::HasComponents,
            MetadataTraitKind::Transformable2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::RuntimeControllable,
            MetadataTraitKind::Patchable,
        ])
        .with_post_fx_host(
            format!("scene_object:{}", entity.id),
            "SceneObjectPixels",
        ),
)
```

### Component node

W okolicach linii 218–233, przy tworzeniu komponentu:

```rust
let component_traits = component_2d_traits(component);
```

Potem node:

```rust
.with_semantics(
    SceneGraphSemantics::new(SceneGraphSemanticRole::Component2D)
        .with_traits(component_traits)
        .with_post_fx_host(
            format!("component:{}:{}:{}", entity.id, component_index, component.kind()),
            "SceneObjectPixels",
        ),
)
```

Trzeba zmienić pętlę z:

```rust
for component in &entity.components {
```

na:

```rust
for (component_index, component) in entity.components.iter().enumerate() {
```

---

## A7. MODIFY `crates/engine/scene/src/metadata_traits.rs`

Obecnie `MetadataTraitKind` ma dużo dobrych traitów, ale brakuje jawnego `PostFxHost2D`.

Dodać do enumu `MetadataTraitKind`, okolice linii 5–50:

```rust
PostFxHost2D,
DrawLayer2D,
SceneObject2D,
Component2D,
```

W `id()` dodać:

```rust
Self::PostFxHost2D => "post_fx_host_2d",
Self::DrawLayer2D => "draw_layer_2d",
Self::SceneObject2D => "scene_object_2d",
Self::Component2D => "component_2d",
```

W `default_metadata_trait_descriptors()` dodać deskryptory:

```rust
metadata_trait_descriptor(
    MetadataTraitKind::PostFxHost2D,
    "Post FX Host 2D",
    "Can own a scoped 2D post-processing stack.",
    vec![MetadataTargetScope::Scene, MetadataTargetScope::Entity, MetadataTargetScope::Component],
),
metadata_trait_descriptor(
    MetadataTraitKind::DrawLayer2D,
    "Draw Layer 2D",
    "Authored 2D draw layer used for sorting, visibility, opacity and optional scoped post FX.",
    vec![MetadataTargetScope::Scene],
),
metadata_trait_descriptor(
    MetadataTraitKind::SceneObject2D,
    "Scene Object 2D",
    "Semantic 2D scene object. Owns identity, transform, visibility and components.",
    vec![MetadataTargetScope::Entity],
),
metadata_trait_descriptor(
    MetadataTraitKind::Component2D,
    "Component 2D",
    "2D component attached to a scene object.",
    vec![MetadataTargetScope::Component],
),
```

Jeżeli helper nazywa się inaczej niż `metadata_trait_descriptor`, użyć istniejącego wzorca z tego pliku.

---

# 3. Faza B — schema: `post_fx[]` na scene object, draw layer, component, image part

## B1. MODIFY `crates/engine/scene/src/document/core.rs`

Na górze pliku obecnie jest import:

```rust
use super::visual2d::SceneVisual2dDocument;
```

Dodać:

```rust
use super::visual2d::PostFx2dDocument;
```

Jeżeli `PostFx2dDocument` nie jest re-exportowany z `visual2d`, użyć pełniejszego importu:

```rust
use super::visual2d::post_fx::PostFx2dDocument;
```

W `SceneEntityDocument`, okolice linii 80–107, dodać pole:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub post_fx: Vec<PostFx2dDocument>,
```

Docelowo:

```rust
pub struct SceneEntityDocument {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_true")]
    pub simulation_enabled: bool,
    #[serde(default = "default_true")]
    pub collision_enabled: bool,
    #[serde(default)]
    pub properties: BTreeMap<String, serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform2: Option<Transform2Document>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform3: Option<Transform3Document>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_fx: Vec<PostFx2dDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefab: Option<String>,
    #[serde(default)]
    pub prefab_overrides: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub components: Vec<SceneComponentDocument>,
}
```

---

## B2. MODIFY `crates/engine/scene/src/document/visual2d/draw_layer.rs`

Obecnie `RenderLayer2dDocument` ma pola:

```rust
pub id: String,
pub label: Option<String>,
pub order: i32,
pub visible: bool,
pub opacity: f32,
```

Dodać import:

```rust
use super::PostFx2dDocument;
```

albo:

```rust
use crate::document::visual2d::post_fx::PostFx2dDocument;
```

Dodać pole:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub post_fx: Vec<PostFx2dDocument>,
```

Docelowo:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderLayer2dDocument {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_fx: Vec<PostFx2dDocument>,
}
```

---

## B3. MODIFY `crates/engine/scene/src/document/components.rs`

Dodać import `PostFx2dDocument`.

Na górze pliku dopisać:

```rust
use crate::document::visual2d::PostFx2dDocument;
```

albo zgodnie z re-exportami:

```rust
use crate::document::visual2d::post_fx::PostFx2dDocument;
```

### Sprite2d

Obecnie okolice linii 36–48:

```rust
Sprite2d {
    render_layer: String,
    texture: String,
    size: [f32; 2],
    #[serde(default)]
    sheet: Option<SpriteSheetDocument>,
    #[serde(default)]
    animation: Option<SpriteAnimationDocument>,
    #[serde(default)]
    z_index: i32,
},
```

Dodać:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
post_fx: Vec<PostFx2dDocument>,
```

Docelowo:

```rust
Sprite2d {
    render_layer: String,
    texture: String,
    size: [f32; 2],
    #[serde(default)]
    sheet: Option<SpriteSheetDocument>,
    #[serde(default)]
    animation: Option<SpriteAnimationDocument>,
    #[serde(default)]
    z_index: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    post_fx: Vec<PostFx2dDocument>,
},
```

### LayeredImage2d

Obecnie okolice linii 49–63. Dodać pole:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
post_fx: Vec<PostFx2dDocument>,
```

### TileMap2d

Dodać:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
post_fx: Vec<PostFx2dDocument>,
```

### Text2d

Dodać:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
post_fx: Vec<PostFx2dDocument>,
```

### VectorShape2d

Dodać:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
post_fx: Vec<PostFx2dDocument>,
```

### ParticleEmitter2d

Dodać:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
post_fx: Vec<PostFx2dDocument>,
```

### BeaconLight2d

Dodać tylko jeżeli beacon jest faktycznie renderowany jako sprite/overlay w render stacku. Jeżeli jest wyłącznie światłem, nie dodawać component-level post-fx.

Rekomendacja: **na tym etapie dodać do BeaconLight2d też**, bo obecnie uczestniczy w render layer/z-index path.

---

## B4. MODIFY `LayeredImageLayerOverrideDocument`

W `crates/engine/scene/src/document/components.rs`, okolice linii 569–578:

Poprzednio:

```rust
pub struct LayeredImageLayerOverrideDocument {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<LayerBlendModeDocument>,
}
```

Dodać:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub post_fx: Vec<PostFx2dDocument>,
```

Docelowo:

```rust
pub struct LayeredImageLayerOverrideDocument {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<LayerBlendModeDocument>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_fx: Vec<PostFx2dDocument>,
}
```

---

# 4. Faza C — post-fx hydration jako wspólny helper

Obecnie największy problem jest w `crates/engine/scene/src/hydration/plan.rs`: `hydrate_visual2d` buduje `PostFx2dStack` tylko z `document.visual2d.post_fx`.

Trzeba wyciągnąć mapping `PostFx2dDocument -> PostFx2d` do helpera.

## C1. ADD `crates/engine/scene/src/hydration/post_fx.rs`

Dodać plik:

```rust
use amigo_2d_post_fx::{
    PostFx2d, PostFx2dId, PostFx2dInstance, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d,
    ScopedPostFx2dStack,
};

use crate::document::visual2d::PostFx2dDocument;
use crate::document::{SceneDocumentError, SceneDocumentResult};

pub fn frame_post_fx_host_id(scene_id: &str) -> PostFxHost2dId {
    PostFxHost2dId::new(format!("scene:{scene_id}:visual2d"))
}

pub fn draw_layer_post_fx_host_id(draw_layer_id: &str) -> PostFxHost2dId {
    PostFxHost2dId::new(format!("draw_layer:{draw_layer_id}"))
}

pub fn scene_object_post_fx_host_id(scene_object_id: &str) -> PostFxHost2dId {
    PostFxHost2dId::new(format!("scene_object:{scene_object_id}"))
}

pub fn component_post_fx_host_id(
    scene_object_id: &str,
    component_index: usize,
    component_kind: &str,
) -> PostFxHost2dId {
    PostFxHost2dId::new(format!(
        "component:{scene_object_id}:{component_index}:{component_kind}"
    ))
}

pub fn image_part_post_fx_host_id(
    scene_object_id: &str,
    component_index: usize,
    part_id: &str,
) -> PostFxHost2dId {
    PostFxHost2dId::new(format!(
        "image_part:{scene_object_id}:{component_index}:{part_id}"
    ))
}

pub fn build_scoped_post_fx_stack(
    host_id: PostFxHost2dId,
    scope: PostFxScope2d,
    docs: &[PostFx2dDocument],
) -> SceneDocumentResult<Option<ScopedPostFx2dStack>> {
    if docs.is_empty() {
        return Ok(None);
    }

    let pipeline = PostFxPipelineKind::for_scope(&scope);
    let mut effects = Vec::with_capacity(docs.len());

    for (index, document) in docs.iter().enumerate() {
        let effect = post_fx_from_document(document)?;
        let effect_type = document.type_name();

        effects.push(PostFx2dInstance {
            id: PostFx2dId::new(format!("{}:{index}:{effect_type}", host_id.as_str())),
            label: Some(effect_type.to_string()),
            enabled: true,
            effect,
        });
    }

    Ok(Some(ScopedPostFx2dStack::new(
        host_id,
        scope,
        pipeline,
        effects,
    )))
}

pub fn post_fx_from_document(document: &PostFx2dDocument) -> SceneDocumentResult<PostFx2d> {
    // Move the existing match from `hydrate_visual2d` here.
    //
    // Do not reimplement effect defaults differently.
    // The resulting PostFx2d must be byte-for-byte semantically equivalent
    // to the current frame-level hydration path.
    match document {
        PostFx2dDocument::ColorQuantize(config) => {
            Ok(PostFx2d::ColorQuantize(config.clone().into()))
        }
        PostFx2dDocument::Crt(config) => {
            Ok(PostFx2d::Crt(config.clone().into()))
        }
        PostFx2dDocument::Downscale(config) => {
            Ok(PostFx2d::Downscale(config.clone().into()))
        }
        PostFx2dDocument::DirtyBloom(config) => {
            Ok(PostFx2d::DirtyBloom(config.clone().into()))
        }
        PostFx2dDocument::FilmNoise(config) => {
            Ok(PostFx2d::FilmNoise(config.clone().into()))
        }
        PostFx2dDocument::LensDroplets(config) => {
            Ok(PostFx2d::LensDroplets(config.clone().into()))
        }
        PostFx2dDocument::RainGlass(config) => {
            Ok(PostFx2d::RainGlass(config.clone().into()))
        }
        PostFx2dDocument::ShutterBlur(config) => {
            Ok(PostFx2d::ShutterBlur(config.clone().into()))
        }
        PostFx2dDocument::WetReflections(config) => {
            Ok(PostFx2d::WetReflections(config.clone().into()))
        }
    }
}
```

Uwaga: powyższe `.into()` może wymagać dopasowania do istniejących helperów w `plan.rs`. W snapshotcie obecny match jest w `hydrate_visual2d`, okolice linii 219–367. Agent ma **przenieść ten istniejący match**, nie pisać nowego z pamięci.

---

## C2. MODIFY `crates/engine/scene/src/hydration/mod.rs`

Dodać:

```rust
pub mod post_fx;
```

Jeżeli moduły są prywatne, wystarczy:

```rust
mod post_fx;
```

Ale `plan.rs` ma z tego korzystać, więc może być `pub(crate) mod post_fx;`.

---

## C3. MODIFY `crates/engine/scene/src/hydration/plan.rs`

### Importy

Usunąć bezpośredni import legacy `PostFx2dStack`, jeżeli jest już niepotrzebny.

Dodać:

```rust
use amigo_2d_post_fx::PostFxScope2d;

use super::post_fx::{
    build_scoped_post_fx_stack,
    component_post_fx_host_id,
    draw_layer_post_fx_host_id,
    frame_post_fx_host_id,
    image_part_post_fx_host_id,
    scene_object_post_fx_host_id,
};
```

### `hydrate_visual2d`

Obecnie `hydrate_visual2d`:

1. Queuje render layers.
2. Queuje light groups/routes.
3. Buduje `effects`.
4. Pushuje `SceneCommand::SetPostFx2dStacks`.

Zastąpić część 3–4 nowym zbieraniem scoped stacks.

Dodać helper lokalny albo funkcję:

```rust
let mut scoped_post_fx_stacks = Vec::new();

if let Some(stack) = build_scoped_post_fx_stack(
    frame_post_fx_host_id(&document.scene.id),
    PostFxScope2d::Frame,
    &document.visual2d.post_fx,
)? {
    scoped_post_fx_stacks.push(stack);
}

for layer in &document.visual2d.render_layers {
    if let Some(stack) = build_scoped_post_fx_stack(
        draw_layer_post_fx_host_id(&layer.id),
        PostFxScope2d::DrawLayer {
            draw_layer_id: layer.id.clone(),
        },
        &layer.post_fx,
    )? {
        scoped_post_fx_stacks.push(stack);
    }
}

for entity in &document.entities {
    if let Some(stack) = build_scoped_post_fx_stack(
        scene_object_post_fx_host_id(&entity.id),
        PostFxScope2d::SceneObjectPixels {
            scene_object_id: entity.id.clone(),
        },
        &entity.post_fx,
    )? {
        scoped_post_fx_stacks.push(stack);
    }

    for (component_index, component) in entity.components.iter().enumerate() {
        let component_kind = component.kind();

        if let Some(component_docs) = component_post_fx_documents(component) {
            if let Some(stack) = build_scoped_post_fx_stack(
                component_post_fx_host_id(&entity.id, component_index, component_kind),
                PostFxScope2d::SceneObjectPixels {
                    scene_object_id: entity.id.clone(),
                },
                component_docs,
            )? {
                scoped_post_fx_stacks.push(stack);
            }
        }

        if let Some(layer_override_docs) = layered_image_part_post_fx_documents(component) {
            for (part_id, docs) in layer_override_docs {
                if let Some(stack) = build_scoped_post_fx_stack(
                    image_part_post_fx_host_id(&entity.id, component_index, part_id),
                    PostFxScope2d::ImagePart {
                        owner_scene_object_id: entity.id.clone(),
                        component_id: format!("{}:{}:{}", entity.id, component_index, component_kind),
                        part_id: part_id.to_string(),
                    },
                    docs,
                )? {
                    scoped_post_fx_stacks.push(stack);
                }
            }
        }
    }
}

commands.push(SceneCommand::SetPostFx2dStacks {
    stacks: scoped_post_fx_stacks,
    lens_droplets_reports,
});
```

Dodać helpery w tym samym pliku albo w `hydration/post_fx.rs`:

```rust
fn component_post_fx_documents(component: &SceneComponentDocument) -> Option<&[PostFx2dDocument]> {
    match component {
        SceneComponentDocument::Sprite2d { post_fx, .. }
        | SceneComponentDocument::LayeredImage2d { post_fx, .. }
        | SceneComponentDocument::TileMap2d { post_fx, .. }
        | SceneComponentDocument::Text2d { post_fx, .. }
        | SceneComponentDocument::VectorShape2d { post_fx, .. }
        | SceneComponentDocument::ParticleEmitter2d { post_fx, .. }
        | SceneComponentDocument::BeaconLight2d { post_fx, .. } => Some(post_fx.as_slice()),
        _ => None,
    }
}

fn layered_image_part_post_fx_documents(
    component: &SceneComponentDocument,
) -> Option<Vec<(&str, &[PostFx2dDocument])>> {
    match component {
        SceneComponentDocument::LayeredImage2d { layer_overrides, .. } => {
            Some(
                layer_overrides
                    .iter()
                    .filter(|override_doc| !override_doc.post_fx.is_empty())
                    .map(|override_doc| (override_doc.id.as_str(), override_doc.post_fx.as_slice()))
                    .collect(),
            )
        }
        _ => None,
    }
}
```

Uwaga: `lens_droplets_reports` muszą zostać zachowane z obecnej logiki. Jeżeli obecny match generuje raporty certyfikacji dla `LensDroplets`, przenieść to do `post_fx_from_document` albo zwracać parę:

```rust
SceneDocumentResult<(PostFx2d, Option<LensDroplets2dCertificationReport>)>
```

Nie wolno zgubić tej funkcji.

---

# 5. Faza D — runtime post-fx service: frame-only stack → scoped stacks

## D1. MODIFY `crates/engine/scene/src/commands.rs`

Poprzedni frame-only shape:

```rust
SetPostFx2dFrameStack {
    stack: amigo_2d_post_fx::PostFx2dStack,
    lens_droplets_reports: Vec<amigo_2d_post_fx::LensDroplets2dCertificationReport>,
},
```

Zastąpić:

```rust
SetPostFx2dStacks {
    stacks: Vec<amigo_2d_post_fx::ScopedPostFx2dStack>,
    lens_droplets_reports: Vec<amigo_2d_post_fx::LensDroplets2dCertificationReport>,
},
```

Dla kompatybilności testów można tymczasowo zostawić stary wariant, ale docelowo usunąć.

---

## D2. MODIFY `crates/2d/post-fx/src/service.rs`

Obecny serwis ma:

```rust
scoped_stacks: RwLock<PostFx2dStack>,
```

Zastąpić:

```rust
scoped_stacks: RwLock<Vec<ScopedPostFx2dStack>>,
```

Pełny docelowy szkielet:

```rust
use std::sync::RwLock;

use crate::{
    LensDroplets2dCertificationReport, PostFx2d, PostFx2dStack, PostFxScope2d,
    ScopedPostFx2dStack,
};

pub struct PostFx2dService {
    default_blur: RwLock<f32>,
    scoped_stacks: RwLock<Vec<ScopedPostFx2dStack>>,
    certification_reports: RwLock<Vec<LensDroplets2dCertificationReport>>,
    renderer_mode: RwLock<PostFx2dRendererMode>,
}

impl Default for PostFx2dService {
    fn default() -> Self {
        Self {
            default_blur: RwLock::new(0.0),
            scoped_stacks: RwLock::new(Vec::new()),
            certification_reports: RwLock::new(Vec::new()),
            renderer_mode: RwLock::new(PostFx2dRendererMode::default()),
        }
    }
}

impl PostFx2dService {
    pub fn set_scoped_stacks(&self, stacks: Vec<ScopedPostFx2dStack>) {
        let mut target = self.scoped_stacks.write().expect("postfx stacks lock poisoned");
        *target = stacks.into_iter().map(ScopedPostFx2dStack::normalized).collect();
    }

    pub fn scoped_stacks(&self) -> Vec<ScopedPostFx2dStack> {
        self.scoped_stacks
            .read()
            .expect("postfx stacks lock poisoned")
            .clone()
    }

    pub fn frame_stack(&self) -> Option<PostFx2dStack> {
        self.scoped_stacks
            .read()
            .expect("postfx stacks lock poisoned")
            .iter()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .map(ScopedPostFx2dStack::as_frame_stack)
    }

    pub fn frame_effect_count(&self) -> usize {
        self.scoped_stacks
            .read()
            .expect("postfx stacks lock poisoned")
            .iter()
            .filter(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .map(|stack| stack.effects.len())
            .sum()
    }

    pub fn clear_scoped_stacks(&self) {
        self.scoped_stacks
            .write()
            .expect("postfx stacks lock poisoned")
            .clear();
    }

    pub fn push_frame_effect(&self, effect: PostFx2d) {
        let mut stacks = self.scoped_stacks.write().expect("postfx stacks lock poisoned");

        if let Some(frame_stack) = stacks
            .iter_mut()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
        {
            frame_stack.push_frame_effect(effect);
            return;
        }

        stacks.push(ScopedPostFx2dStack::from_frame_stack(PostFx2dStack {
            effects: vec![effect],
        }));
    }
}
```

Jeżeli `ScopedPostFx2dStack` nie ma `push_frame_effect`, dodać metodę w `scope.rs`.

---

## D3. MODIFY `crates/2d/post-fx/src/scope.rs`

Dodać do `impl ScopedPostFx2dStack`:

```rust
pub fn push_frame_effect(&mut self, effect: PostFx2d) {
    let index = self.effects.len();
    self.effects.push(PostFx2dInstance {
        id: PostFx2dId::new(format!("{}:{index}:legacy", self.host_id.as_str())),
        label: None,
        enabled: true,
        effect,
    });
}
```

Jeżeli `PostFxHost2dId` nie ma `as_str()`, dodać do makra ID albo użyć istniejącego dostępu. Jeśli makro nie daje `as_str`, dodać:

```rust
impl PostFxHost2dId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PostFx2dId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

---

## D4. MODIFY `crates/2d/post-fx/src/scene_command.rs`

Obecnie:

```rust
pub fn handle_post_fx_scoped_stacks(
    service: &PostFx2dService,
    stack: PostFx2dStack,
    reports: Vec<LensDroplets2dCertificationReport>,
) {
    service.set_scoped_stacks(stack);
    service.set_certification_reports(reports);
}
```

Zastąpić:

```rust
use crate::{LensDroplets2dCertificationReport, PostFx2dService, ScopedPostFx2dStack};

pub fn handle_post_fx_scoped_stacks(
    service: &PostFx2dService,
    stacks: Vec<ScopedPostFx2dStack>,
    reports: Vec<LensDroplets2dCertificationReport>,
) {
    service.set_scoped_stacks(stacks);
    service.set_certification_reports(reports);
}
```

---

## D5. MODIFY `crates/engine/scene/src/scene_command.rs`

Poprzedni handler `ScenePostFx2dRuntimeSceneCommandHandler`, okolice linii 256-284, obsluguje frame-only command:

```rust
SceneCommand::SetPostFx2dFrameStack { stack, lens_droplets_reports } => ...
```

Zastąpić:

```rust
SceneCommand::SetPostFx2dStacks {
    stacks,
    lens_droplets_reports,
} => {
    amigo_2d_post_fx::handle_post_fx_scoped_stacks(
        &self.post_fx_service,
        stacks.clone(),
        lens_droplets_reports.clone(),
    );
}
```

Usunąć ścieżki starego frame-only command shape, jeżeli nie zostawiasz compatibility.

---

## D6. MODIFY `crates/2d/post-fx/src/render_extraction.rs`

Obecnie trait powinien mieć:

```rust
fn set_post_fx2d_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>);
```

Implementacje przekazują scoped stacks dalej bez konwersji do frame-only stack:

```rust
fn set_post_fx2d_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>);
```

Funkcja:

```rust
pub fn extract_post_fx2d_render_stack(...)
```

Zmienić na:

```rust
pub fn extract_post_fx2d_render_stacks(
    ctx: &PostFx2dRenderExtractionContext<'_>,
    output: &mut dyn PostFx2dRenderOutput,
) {
    output.set_post_fx2d_stacks(ctx.post_fx_service.scoped_stacks());
}
```

Jeżeli nazwa funkcji jest używana w wielu miejscach, albo zmienić call site’y, albo tymczasowo zostawić wrapper:

```rust
pub fn extract_post_fx2d_render_stack(
    ctx: &PostFx2dRenderExtractionContext<'_>,
    output: &mut dyn PostFx2dRenderOutput,
) {
    extract_post_fx2d_render_stacks(ctx, output);
}
```

---

# 6. Faza E — render packet i render-api: stabilne ID passów post-fx

## E1. MODIFY `crates/engine/render-api/Cargo.toml`

Dodać zależność:

```toml
amigo-2d-post-fx = { path = "../../2d/post-fx" }
```

Powód: `render-api` ma używać `PostFxHost2dId`, `PostFx2dId`, `PostFxScope2d`, `PostFxPipelineKind`.

Sprawdzić brak cyklu. W snapshotcie `amigo-2d-post-fx` nie powinien zależeć od `render-api`.

---

## E2. MODIFY `crates/engine/render-api/src/composition.rs`

Dodać import:

```rust
use amigo_2d_post_fx::{PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d};
```

Poprzedni `PostFxPassPlan`, okolice linii 203-218, używał liczbowej kolejności efektu:

```rust
pub struct PostFxPassPlan {
    pub feature_id: String,
    pub effect_order: usize,
    pub input: String,
    pub output: String,
}
```

Zastąpić:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PostFxPassPlan {
    pub host_id: PostFxHost2dId,
    pub effect_id: PostFx2dId,
    pub scope: PostFxScope2d,
    pub pipeline: PostFxPipelineKind,
    pub feature_id: String,
    pub input: String,
    pub output: String,
}
```

W `RenderPassPlan::label`, obecnie używa:

```rust
format!("post_fx:{}#{}", pass.feature_id, pass.effect_id)
```

Zastąpić:

```rust
format!(
    "post_fx:{}:{}:{}",
    pass.host_id.as_str(),
    pass.effect_id.as_str(),
    pass.feature_id
)
```

Jeżeli `as_str()` nie istnieje, dodać w ID typach.

---

## E3. MODIFY `crates/engine/render-api/src/frame_graph.rs`

Obecnie:

```rust
PostFx {
    feature_id: String,
    effect_order: usize,
},
```

Zastąpić:

```rust
PostFx {
    host_id: amigo_2d_post_fx::PostFxHost2dId,
    effect_id: amigo_2d_post_fx::PostFx2dId,
    scope: amigo_2d_post_fx::PostFxScope2d,
    pipeline: amigo_2d_post_fx::PostFxPipelineKind,
    feature_id: String,
},
```

W miejscach tworzenia frame graph node trzeba przekazać nowe pola z `PostFxPassPlan`.

---

## E4. MODIFY `crates/engine/render-wgpu/src/frame_packet.rs`

Obecnie:

```rust
use amigo_2d_post_fx::PostFx2dStack;

pub post_fx_stacks: Vec<ScopedPostFx2dStack>,
```

Finalny kształt:

```rust
use amigo_2d_post_fx::ScopedPostFx2dStack;

pub post_fx_stacks: Vec<ScopedPostFx2dStack>,
```

Metody:

```rust
pub fn set_post_fx_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>)
pub fn post_fx_stacks(&self) -> &[ScopedPostFx2dStack]
```

Implementacja:

```rust
pub fn set_post_fx_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>) {
    self.post_fx_stacks = stacks;
}

pub fn post_fx_stacks(&self) -> &[ScopedPostFx2dStack] {
    &self.post_fx_stacks
}
```

Implementacja `PostFx2dRenderOutput`:

```rust
impl PostFx2dRenderOutput for WgpuRenderFramePacket {
    fn set_post_fx2d_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>) {
        self.set_post_fx_stacks(stacks);
    }
}
```

W `Default`/constructorze ustawić:

```rust
post_fx_stacks: Vec::new(),
```

---

## E5. MODIFY `crates/engine/render-wgpu/src/renderer/service/render_request.rs`

Obecnie:

```rust
pub post_fx_stacks: Option<&'a PostFx2dStack>,
```

Zastąpić:

```rust
pub post_fx_stacks: &'a [ScopedPostFx2dStack],
```

Import:

```rust
use amigo_2d_post_fx::ScopedPostFx2dStack;
```

W konstruktorach requestu przekazać:

```rust
post_fx_stacks: packet.post_fx_stacks(),
```

---

## E6. MODIFY `crates/runtime/bundles/src/wgpu_render_extractors/composition.rs`

Obecnie:

```rust
active_post_fx(packet.post_fx_stacks())
append_post_fx_passes(... Vec<(usize, PostFx2d)> ...)
```

Zastąpić modelem scoped:

```rust
use amigo_2d_post_fx::{
    PostFx2d, PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d,
    ScopedPostFx2dStack,
};
```

Nowy typ pomocniczy:

```rust
#[derive(Debug, Clone)]
struct ActivePostFxPass {
    host_id: PostFxHost2dId,
    effect_id: PostFx2dId,
    scope: PostFxScope2d,
    pipeline: PostFxPipelineKind,
    feature_id: String,
    effect: PostFx2d,
}
```

Nowe `active_post_fx`:

```rust
fn active_post_fx(stacks: &[ScopedPostFx2dStack]) -> Vec<ActivePostFxPass> {
    stacks
        .iter()
        .filter(|stack| matches!(stack.pipeline, PostFxPipelineKind::FrameGraph))
        .flat_map(|stack| {
            stack.effects.iter().filter(|effect| effect.enabled).map(|effect| {
                ActivePostFxPass {
                    host_id: stack.host_id.clone(),
                    effect_id: effect.id.clone(),
                    scope: stack.scope.clone(),
                    pipeline: stack.pipeline,
                    feature_id: effect.effect.feature_id().to_string(),
                    effect: effect.effect.clone(),
                }
            })
        })
        .collect()
}
```

W `append_post_fx_passes` zamiast `effect_id`:

```rust
PostFxPassPlan {
    host_id: pass.host_id,
    effect_id: pass.effect_id,
    scope: pass.scope,
    pipeline: pass.pipeline,
    feature_id: pass.feature_id,
    input,
    output,
}
```

Ważne: na tym etapie `FrameGraph` obsługuje tylko `PostFxScope2d::Frame`. Dla innych scope’ów, jeżeli pipeline zwraca `FrameGraph`, należy to jawnie filtrować albo raportować. Bezpieczniej:

```rust
.filter(|stack| matches!(stack.scope, PostFxScope2d::Frame))
.filter(|stack| matches!(stack.pipeline, PostFxPipelineKind::FrameGraph))
```

Non-frame scopes mają wejść do packetu, ale nie do frame fullscreen chain, dopóki nie ma offscreen draw-layer/object implementation.

---

## E7. MODIFY `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs`

Obecnie funkcja:

```rust
execute_screen_space_post_fx(
    feature_id: &str,
    effect_order: usize,
    request: &WgpuFrameRenderRequest<'_>,
    ...
)
```

Zastąpić:

```rust
use amigo_2d_post_fx::{PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d};

pub fn execute_screen_space_post_fx(
    host_id: &PostFxHost2dId,
    effect_id: &PostFx2dId,
    scope: &PostFxScope2d,
    pipeline: PostFxPipelineKind,
    feature_id: &str,
    request: &WgpuFrameRenderRequest<'_>,
    ...
) -> Result<(), WgpuRenderError> {
    if !matches!(pipeline, PostFxPipelineKind::FrameGraph) {
        return Ok(());
    }

    if !matches!(scope, PostFxScope2d::Frame) {
        // Nie wykonywać scoped non-frame jako fullscreen frame effect.
        return Ok(());
    }

    let effect = request
        .post_fx_stacks
        .iter()
        .find(|stack| &stack.host_id == host_id)
        .and_then(|stack| stack.effects.iter().find(|effect| &effect.id == effect_id))
        .map(|instance| &instance.effect)
        .ok_or_else(|| WgpuRenderError::MissingPostFx {
            host_id: host_id.as_str().to_string(),
            effect_id: effect_id.as_str().to_string(),
        })?;

    if effect.feature_id() != feature_id {
        return Err(WgpuRenderError::PostFxFeatureMismatch {
            expected: feature_id.to_string(),
            actual: effect.feature_id().to_string(),
        });
    }

    // existing dispatch by PostFx2d variant remains here
}
```

Jeżeli `WgpuRenderError` nie ma takich wariantów, dodać albo użyć istniejącego wariantu tekstowego.

---

## E8. MODIFY `crates/engine/render-wgpu/src/renderer/service/render.rs`

`execute_post_fx_graph_node` powinien przyjmować pełną tożsamość passu.

Zastąpić parametry:

```rust
host_id: &PostFxHost2dId,
effect_id: &PostFx2dId,
scope: &PostFxScope2d,
pipeline: PostFxPipelineKind,
feature_id: &str,
```

I forward do `execute_screen_space_post_fx`.

---

## E9. MODIFY `crates/engine/render-wgpu/src/renderer/graph/executor.rs`

Obecnie match:

```rust
FrameGraphNodeKind::PostFx { feature_id, effect_order } => ...
```

Zastąpić:

```rust
FrameGraphNodeKind::PostFx {
    host_id,
    effect_id,
    scope,
    pipeline,
    feature_id,
} => {
    renderer.execute_post_fx_graph_node(
        host_id,
        effect_id,
        scope,
        *pipeline,
        feature_id,
        ...
    )
}
```

---

# 7. Faza F — przepięcie call site’ów legacy `post_fx_stacks`

## F1. MODIFY `crates/apps/app/src/render_runtime.rs`

W snapshocie są użycia `post_fx_stacks()` około linii 118, 213, 250.

Zamienić:

```rust
packet.post_fx_stacks()
```

na:

```rust
packet.post_fx_stacks()
```

Jeżeli kod sprawdza `Option`, zmienić na:

```rust
if !packet.post_fx_stacks().is_empty() {
    ...
}
```

---

## F2. MODIFY `crates/apps/app/src/scene_preview.rs`

Około linii 359 jest `post_fx_stacks`.

Zamienić na `post_fx_stacks`.

---

## F3. MODIFY `crates/apps/app/src/scene_runtime/mod.rs`

Około linii 808 resetuje stack.

Zamienić:

```rust
post_fx_service.clear_scoped_stacks();
```

na:

```rust
post_fx_service.clear_scoped_stacks();
```

---

## F4. MODIFY `crates/scripting/rhai/src/bindings/postfx.rs`

Występują legacy wywołania około linii 380, 398, 407, 427.

Zasada:

```rust
scoped_stacks()
frame_effects()
frame_effect_count()
push_scene_effect()
clear_scoped_stacks()
```

zamienić na:

```rust
frame_stack()
frame_effects()        // jeśli dodasz helper
frame_effect_count()
push_frame_effect()
clear_scoped_stacks()
```

Jeżeli API skryptowe ma zachować stare nazwy dla modów, można zostawić funkcje Rhai o starych nazwach, ale wewnątrz mają używać nowego serwisu.

Przykład:

```rust
fn postfx_clear_scoped_stacks(service: &PostFx2dService) {
    service.clear_scoped_stacks();
}
```

Nie utrzymywać starego pola `scoped_stacks`.

---

# 8. Faza G — LayeredImage image-part post-fx

## G1. MODIFY `crates/2d/layered-image/src/model.rs`

Obecnie `LayeredImageLayer` już ma:

```rust
pub post_fx: Option<PostFx2dStack>,
```

Zostawić, bo to dotyczy asset-level/cached image.

W `LayeredImageLayerOverride`, okolice linii 96–102, dodać:

```rust
pub post_fx: Option<PostFx2dStack>,
```

Docelowo:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageLayerOverride {
    pub id: String,
    pub opacity: Option<f32>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<LayerBlendMode>,
    pub post_fx: Option<PostFx2dStack>,
}
```

---

## G2. MODIFY `crates/2d/layered-image/src/asset.rs`

W `apply_layer_overrides`, okolice linii 76–98, dodać:

```rust
if let Some(post_fx) = override_layer.post_fx.clone() {
    layer.post_fx = Some(post_fx);
}
```

Czyli:

```rust
if let Some(opacity) = override_layer.opacity {
    layer.opacity = opacity;
}

if let Some(enabled) = override_layer.enabled {
    layer.enabled = enabled;
}

if let Some(blend_mode) = override_layer.blend_mode {
    layer.blend_mode = blend_mode;
}

if let Some(post_fx) = override_layer.post_fx.clone() {
    layer.post_fx = Some(post_fx);
}
```

---

## G3. MODIFY `crates/2d/layered-image/src/scene_bridge.rs`

W mapowaniu `LayeredImageLayerOverrideSceneCommand -> LayeredImageLayerOverride` dodać:

```rust
post_fx: override_command.post_fx.clone(),
```

---

## G4. MODIFY `crates/engine/scene/src/render_commands/render_2d.rs`

W `LayeredImageLayerOverrideSceneCommand`, okolice linii 35–41, dodać:

```rust
pub post_fx: Option<amigo_2d_post_fx::PostFx2dStack>,
```

W hydration layered image override trzeba zbudować legacy stack z `post_fx` docs dla image part. Jeżeli image-part scoped stacks są już osobno rejestrowane w `PostFx2dService`, to to pole jest potrzebne tylko dla asset-cached path. Dla prostoty:

```rust
post_fx: None
```

dopóki nie przeniesiesz dokumentów do cached layer model.

Docelowo jednak `LayeredImageLayerOverrideDocument.post_fx` powinno zasilać `LayeredImageLayerOverrideSceneCommand.post_fx`.

---

# 9. Faza H — editor-authoring: graph nie może być YAML viewerem

## H1. MODIFY `crates/engine/editor-authoring/src/graph.rs`

Obecnie `AuthoringNodeSemantic` ma pola:

```rust
pub parent_id: Option<String>,
pub owner_entity_name: Option<String>,
pub scene_object_id: Option<String>,
pub component_type: Option<String>,
pub render_layer_id: Option<String>,
pub postfx_id: Option<String>,
pub postfx_type: Option<String>,
pub postfx_scope: Option<String>,
...
```

Dodać:

```rust
pub capabilities: Vec<String>,
pub post_fx_host_id: Option<String>,
pub post_fx_pipeline: Option<String>,
```

Docelowo:

```rust
pub struct AuthoringNodeSemantic {
    pub parent_id: Option<String>,
    pub owner_entity_name: Option<String>,
    pub scene_object_id: Option<String>,
    pub component_type: Option<String>,
    pub render_layer_id: Option<String>,
    pub postfx_id: Option<String>,
    pub postfx_type: Option<String>,
    pub postfx_scope: Option<String>,
    pub post_fx_host_id: Option<String>,
    pub post_fx_pipeline: Option<String>,
    pub capabilities: Vec<String>,
    pub light_group_id: Option<String>,
    pub light_route_id: Option<String>,
}
```

Default:

```rust
capabilities: Vec::new(),
post_fx_host_id: None,
post_fx_pipeline: None,
```

---

## H2. MODIFY `crates/engine/editor-authoring/src/loader.rs`

Obecnie `build_sequence_children`, okolice linii 224–258, wszystko pod `/post_fx` dostaje:

```rust
post_fx_scope = Some("Frame")
```

Zastąpić funkcją rozpoznającą scope po ścieżce:

```rust
fn infer_post_fx_scope(path: &str) -> Option<&'static str> {
    if path.ends_with("/visual2d/post_fx") {
        return Some("Frame");
    }

    if path.contains("/visual2d/render_layers/") && path.ends_with("/post_fx") {
        return Some("DrawLayer");
    }

    if path.contains("/entities/") && path.ends_with("/post_fx") {
        return Some("SceneObjectPixels");
    }

    if path.contains("/components/") && path.ends_with("/post_fx") {
        return Some("SceneObjectPixels");
    }

    if path.contains("/layer_overrides/") && path.ends_with("/post_fx") {
        return Some("ImagePart");
    }

    None
}
```

W miejscu ustawiania `post_fx_scope` użyć:

```rust
semantic.postfx_scope = infer_post_fx_scope(parent_path).map(str::to_string);
```

Dodać capabilities dla głównych node’ów:

* entity mapping: `SceneObject2D`, `Transformable2D`, `HasComponents`
* component mapping: `Component2D`, plus `Renderable2D`, `UsesTransform2D` gdzie można rozpoznać typ
* render layer: `DrawLayer2D`, `PostFxHost2D`
* post_fx parent: `PostFxHost2D`

---

## H3. MODIFY `crates/engine/editor-authoring/src/projections.rs`

### Scene Objects projection

Utrzymać zasadę:

```text
Scene Objects pokazuje:
- SceneObject
- transform
- komponenty jako semantyczne dzieci
- post_fx hosty tylko jako zwięzły child, jeśli istnieją
```

Nie pokazywać raw YAML noise.

W `scene_objects_renders_node`, okolice linii 208–225, post-fx node ma mieć etykietę:

```text
Post FX [Frame]
Post FX [DrawLayer]
Post FX [SceneObject]
Post FX [ImagePart]
```

Nie używać starych tagów typu `[IMG]`, `[EDIT]`.

### Render Stack projection

W `render_stack_tree_projection`, okolice linii 99–138, dodać pod każdą draw layer:

```text
DrawLayer2D
├─ Post FX, jeśli layer.post_fx istnieje
└─ Renderables
```

Dla frame post-fx:

```text
Scene2D
└─ Frame Post FX
```

---

## H4. MODIFY `crates/engine/editor-ingame/src/theme.rs`

W snapshotcie label `Frame Post FX` około linii 48.

Zamienić na neutralniejsze:

```rust
"Post FX"
```

albo dodać zależne od scope:

```rust
"Frame Post FX"
"Layer Post FX"
"Object Post FX"
"Image Part Post FX"
```

---

# 10. Faza I — renderable component contract

Ta faza nie musi zmieniać renderingu, ale ma uporządkować kontrakt.

## I1. ADD `crates/engine/scene/src/renderable_2d.rs`

Nowy plik:

```rust
use crate::document::components::SceneComponentDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Renderable2dDescriptor {
    pub component_kind: &'static str,
    pub uses_owner_transform: bool,
    pub has_render_layer: bool,
    pub has_bounds: bool,
    pub has_asset_refs: bool,
    pub supports_component_post_fx: bool,
}

pub fn renderable_2d_descriptor(
    component: &SceneComponentDocument,
) -> Option<Renderable2dDescriptor> {
    match component {
        SceneComponentDocument::Sprite2d { .. } => Some(Renderable2dDescriptor {
            component_kind: "Sprite2d",
            uses_owner_transform: true,
            has_render_layer: true,
            has_bounds: true,
            has_asset_refs: true,
            supports_component_post_fx: true,
        }),
        SceneComponentDocument::LayeredImage2d { .. } => Some(Renderable2dDescriptor {
            component_kind: "LayeredImage2d",
            uses_owner_transform: true,
            has_render_layer: true,
            has_bounds: true,
            has_asset_refs: true,
            supports_component_post_fx: true,
        }),
        SceneComponentDocument::Text2d { .. } => Some(Renderable2dDescriptor {
            component_kind: "Text2d",
            uses_owner_transform: true,
            has_render_layer: true,
            has_bounds: true,
            has_asset_refs: false,
            supports_component_post_fx: true,
        }),
        SceneComponentDocument::VectorShape2d { .. } => Some(Renderable2dDescriptor {
            component_kind: "VectorShape2d",
            uses_owner_transform: true,
            has_render_layer: true,
            has_bounds: true,
            has_asset_refs: false,
            supports_component_post_fx: true,
        }),
        SceneComponentDocument::TileMap2d { .. } => Some(Renderable2dDescriptor {
            component_kind: "TileMap2d",
            uses_owner_transform: true,
            has_render_layer: true,
            has_bounds: true,
            has_asset_refs: true,
            supports_component_post_fx: true,
        }),
        SceneComponentDocument::ParticleEmitter2d { .. } => Some(Renderable2dDescriptor {
            component_kind: "ParticleEmitter2d",
            uses_owner_transform: true,
            has_render_layer: true,
            has_bounds: false,
            has_asset_refs: false,
            supports_component_post_fx: true,
        }),
        SceneComponentDocument::BeaconLight2d { .. } => Some(Renderable2dDescriptor {
            component_kind: "BeaconLight2d",
            uses_owner_transform: true,
            has_render_layer: true,
            has_bounds: false,
            has_asset_refs: false,
            supports_component_post_fx: true,
        }),
        _ => None,
    }
}
```

---

## I2. MODIFY `crates/engine/scene/src/lib.rs`

Dodać:

```rust
pub mod renderable_2d;
```

---

## I3. Użyć `renderable_2d_descriptor` w graph/build

W `graph/build.rs`, przy komponentach, zamiast rozpoznawać renderowalność ręcznie, użyć:

```rust
if let Some(descriptor) = renderable_2d_descriptor(component) {
    semantics.push_trait(MetadataTraitKind::Renderable2D);

    if descriptor.uses_owner_transform {
        semantics.push_trait(MetadataTraitKind::UsesTransform2D);
    }

    if descriptor.has_render_layer {
        semantics.push_trait(MetadataTraitKind::RenderLayered2D);
    }

    if descriptor.has_bounds {
        semantics.push_trait(MetadataTraitKind::HasBounds2D);
    }

    if descriptor.has_asset_refs {
        semantics.push_trait(MetadataTraitKind::HasAssetRefs);
    }

    if descriptor.supports_component_post_fx {
        semantics.push_trait(MetadataTraitKind::PostFxHost2D);
    }
}
```

---

# 11. Faza J — przykładowy YAML po refaktorze

Po refaktorze taki YAML ma być legalny:

```yaml
version: 1

scene:
  id: rotten-club.main-menu
  label: Rotten Club Main Menu

visual2d:
  render_layers:
    - id: background.city
      label: City Background
      order: 0
      visible: true
      opacity: 1.0
      post_fx:
        - type: dirty_bloom
          id: city_layer_bloom
          intensity: 0.25

  post_fx:
    - type: rain_glass
      id: rotten_lens_rain_glass
      reference_mode: true

entities:
  - id: club.sign
    name: Club Sign
    transform2:
      translation: [640, 260]
      scale: [1, 1]
      rotation: 0
    post_fx:
      - type: film_noise
        id: sign_noise
        intensity: 0.08
    components:
      - type: sprite2d
        render_layer: background.city
        texture: assets/sign.png
        size: [512, 256]
        z_index: 10
        post_fx:
          - type: color_quantize
            id: sign_quantize
            levels: 16

  - id: club.poster
    transform2:
      translation: [200, 300]
    components:
      - type: layered_image2d
        render_layer: background.city
        asset: assets/poster.layered.yml
        size: [512, 512]
        layer_overrides:
          - id: wet_streaks
            opacity: 0.7
            post_fx:
              - type: lens_droplets
                id: poster_wet_streaks
```

Ważne: finalne nazwy pól muszą pasować do obecnego serde taggingu `SceneComponentDocument`. Jeżeli obecny YAML używa `kind` zamiast `type`, nie zmieniać tego w tym refaktorze.

---

# 12. Faza K — testy i walidacja

## K1. ADD tests: `crates/engine/scene/src/graph/tests.rs`

Dodać testy:

```rust
#[test]
fn sprite_component_is_renderable_and_uses_owner_transform() {
    // build minimal SceneDocument with one entity + Sprite2d
    // build_semantic_scene_graph
    // find component node
    // assert Renderable2D
    // assert UsesTransform2D
    // assert RenderLayered2D
}

#[test]
fn scene_draw_layer_object_and_component_can_be_post_fx_hosts() {
    // scene visual2d.post_fx
    // render_layer.post_fx
    // entity.post_fx
    // sprite.post_fx
    // assert graph has PostFxHost semantics for all scopes
}
```

---

## K2. MODIFY existing tests

Uruchomić i poprawić:

```bash
cargo test -p amigo-scene
cargo test -p amigo-2d-post-fx
cargo test -p amigo-render-api
cargo test -p amigo-render-wgpu
cargo test -p amigo-runtime-bundles
```

Potem:

```bash
cargo check --workspace
```

Jeżeli workspace ma stare duże pliki i file-size checker nie przechodzi, nie naprawiać tego w tym refaktorze, chyba że dotyczy plików zmienianych powyżej.

---

# 13. Faza L — celowane wyszukiwania po refaktorze

Agent ma użyć tylko tych celowanych komend:

```bash
rg -n "scoped_stacks|set_scoped_stacks|clear_scoped_stacks|frame_effect|frame_effects|frame_effect_count" crates
rg -n "post_fx_stacks|set_post_fx_stacks|post_fx2d_stacks|set_post_fx2d_stacks" crates
rg -n "effect_id" crates
rg -n "SetPostFx2dStacks" crates
rg -n "FramePostFxHost" crates
rg -n "PostFxPassPlan" crates
rg -n "FrameGraphNodeKind::PostFx|PostFx \\{" crates
```

Po zakończeniu:

```bash
rg -n "effect_id" crates/engine/render-api crates/engine/render-wgpu crates/runtime/bundles
```

ma nie zwracać aktywnych użyć w post-fx path.

---

# 14. Kolejność commitów

Najbezpieczniejsza kolejność:

```text
commit 1:
  scene graph semantics:
  - ADD graph/semantics.rs
  - ADD graph/component_capabilities.rs
  - MODIFY graph/node.rs
  - MODIFY graph/build.rs
  - MODIFY metadata_traits.rs

commit 2:
  schema post_fx scopes:
  - MODIFY document/core.rs
  - MODIFY document/visual2d/draw_layer.rs
  - MODIFY document/components.rs

commit 3:
  hydration helper:
  - ADD hydration/post_fx.rs
  - MODIFY hydration/mod.rs
  - MODIFY hydration/plan.rs
  - MODIFY commands.rs
  - MODIFY scene_command.rs

commit 4:
  post_fx service scoped stacks:
  - MODIFY 2d/post-fx/src/scope.rs
  - MODIFY 2d/post-fx/src/service.rs
  - MODIFY 2d/post-fx/src/scene_command.rs
  - MODIFY 2d/post-fx/src/render_extraction.rs

commit 5:
  render-api identity migration:
  - MODIFY engine/render-api/Cargo.toml
  - MODIFY engine/render-api/src/composition.rs
  - MODIFY engine/render-api/src/frame_graph.rs
  - MODIFY runtime/bundles/src/wgpu_render_extractors/composition.rs

commit 6:
  wgpu packet/request/executor:
  - MODIFY render-wgpu/src/frame_packet.rs
  - MODIFY render-wgpu/src/renderer/service/render_request.rs
  - MODIFY render-wgpu/src/renderer/graph/executor.rs
  - MODIFY render-wgpu/src/renderer/service/render.rs
  - MODIFY render-wgpu/src/renderer/service/post_fx/registry.rs

commit 7:
  call site cleanup:
  - MODIFY apps/app/src/render_runtime.rs
  - MODIFY apps/app/src/scene_preview.rs
  - MODIFY apps/app/src/scene_runtime/mod.rs
  - MODIFY scripting/rhai/src/bindings/postfx.rs

commit 8:
  editor projections:
  - MODIFY editor-authoring/src/graph.rs
  - MODIFY editor-authoring/src/loader.rs
  - MODIFY editor-authoring/src/projections.rs
  - MODIFY editor-ingame/src/theme.rs

commit 9:
  layered image image-part postfx:
  - MODIFY 2d/layered-image/src/model.rs
  - MODIFY 2d/layered-image/src/asset.rs
  - MODIFY 2d/layered-image/src/scene_bridge.rs
  - MODIFY scene/src/render_commands/render_2d.rs

commit 10:
  tests/docs/examples:
  - ADD/MODIFY tests
  - update rotten-club example only after schema compiles
```

---

# 15. Acceptance criteria

Refaktor jest skończony dopiero gdy spełnione są te warunki:

```text
1. SceneObject2D w semantic graph ma:
   - Transformable2D
   - HasComponents
   - Selectable
   - RuntimeControllable/Patchable

2. Sprite2D w semantic graph ma:
   - Component2D
   - Renderable2D
   - UsesTransform2D
   - RenderLayered2D
   - HasBounds2D
   - HasAssetRefs
   - optional PostFxHost2D

3. Scene2D ma:
   - PostFxHost2D dla Frame scope

4. DrawLayer2D ma:
   - DrawLayer2D
   - PostFxHost2D dla DrawLayer scope

5. YAML może mieć:
   - visual2d.post_fx
   - visual2d.render_layers[].post_fx
   - entities[].post_fx
   - entities[].components[].post_fx dla renderable components
   - layered image layer_overrides[].post_fx

6. `PostFx2dService` nie ma już frame-only pola stacka.

7. `WgpuRenderFramePacket` nie ma już `post_fx_stacks: Option<PostFx2dStack>`.
   Ma `post_fx_stacks: Vec<ScopedPostFx2dStack>`.

8. `PostFxPassPlan` używa stabilnego `effect_id: PostFx2dId`.

9. `FrameGraphNodeKind::PostFx` używa stabilnego `effect_id: PostFx2dId`.

10. Frame-level post-fx działa jak wcześniej.

11. Non-frame scoped post-fx nie jest udawany jako fullscreen frame post-fx.
    Jeżeli scope nie ma jeszcze pipeline w WGPU, ma być pominięty z diagnostyką albo oznaczony jako Unsupported.

12. Edytor pokazuje semantyczne:
    - Scene Objects
    - Draw Layers / Render Stack
    - Post FX scopes
    a Raw YAML zostaje tylko debug/projection.
```

---

# 16. Najważniejsza decyzja techniczna

Nie robić tego tak:

```text
Sprite2D ma własny Transform2D.
LayeredImage2D ma własny Transform2D.
Text2D ma własny Transform2D.
```

Tylko tak:

```text
SceneObject2D ma Transform2D.
Renderable component ma UsesTransform2D.
Komponent może mieć local offset/local params, ale nie jest właścicielem world transform.
```

To jest główny kontrakt, który porządkuje projekt.

Docelowy mental model:

```text
Scene2D(PostFxHost2D)
└─ SceneObject2D(Transformable2D)
   └─ Sprite2D(Renderable2D, UsesTransform2D, RenderLayered2D, PostFxHost2D?)
```

To jest spójne z obecnym kodem, bo hydration już przekazuje `entity.transform2` do render commands. Refaktor ma tylko sprawić, żeby ta zasada była **formalnym kontraktem całego projektu**, a nie ukrytą implementacją w hydration.


