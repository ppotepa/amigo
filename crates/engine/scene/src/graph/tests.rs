use amigo_runtime::{RuntimeBuilder, RuntimePlugin, ServiceRegistry};

use crate::{
    build_semantic_scene_graph, build_semantic_scene_graph_for_runtime,
    ComponentGraphProviderRegistry, ComponentSchemaRegistry, PluginComponentGraphContext,
    PluginComponentGraphProvider, SceneComponentPayload, SceneComponentSchemaProvider,
    SceneDocument, SceneDocumentError, SceneGraphDiagnosticSeverity, ScenePlugin,
    ScenePluginComponentDescriptor, SceneReferenceKind, SceneReferenceTargetKind,
};
use serde_yaml::{Mapping, Value};

#[derive(Debug)]
struct TestSpritePayload {
    render_layer: String,
    texture: String,
}

impl SceneComponentPayload for TestSpritePayload {
    fn component_type(&self) -> &'static str {
        "amigo.test.Sprite2D"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct TestTextPayload {
    render_layer: String,
    font: String,
}

impl SceneComponentPayload for TestTextPayload {
    fn component_type(&self) -> &'static str {
        "amigo.test.Text2D"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Copy)]
struct TestSpriteSchemaProvider;

impl SceneComponentSchemaProvider for TestSpriteSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.test.Sprite2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["Sprite2D"]
    }

    fn parse_yaml(&self, payload: Mapping) -> Result<Value, serde_yaml::Error> {
        Ok(Value::Mapping(payload))
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> crate::SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        let mapping = payload
            .as_mapping()
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "expected sprite payload mapping".to_owned(),
            })?;
        let render_layer = mapping
            .get(Value::String("render_layer".to_owned()))
            .and_then(Value::as_str)
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "missing sprite render_layer".to_owned(),
            })?;
        let texture = mapping
            .get(Value::String("texture".to_owned()))
            .and_then(Value::as_str)
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "missing sprite texture".to_owned(),
            })?;
        Ok(Box::new(TestSpritePayload {
            render_layer: render_layer.to_owned(),
            texture: texture.to_owned(),
        }))
    }
}

#[derive(Clone, Copy)]
struct TestTextSchemaProvider;

impl SceneComponentSchemaProvider for TestTextSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.test.Text2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["Text2D"]
    }

    fn parse_yaml(&self, payload: Mapping) -> Result<Value, serde_yaml::Error> {
        Ok(Value::Mapping(payload))
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> crate::SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        let mapping = payload
            .as_mapping()
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "expected text payload mapping".to_owned(),
            })?;
        let render_layer = mapping
            .get(Value::String("render_layer".to_owned()))
            .and_then(Value::as_str)
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "missing text render_layer".to_owned(),
            })?;
        let font = mapping
            .get(Value::String("font".to_owned()))
            .and_then(Value::as_str)
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "missing text font".to_owned(),
            })?;
        Ok(Box::new(TestTextPayload {
            render_layer: render_layer.to_owned(),
            font: font.to_owned(),
        }))
    }
}

struct TestSpriteGraphProvider;

impl PluginComponentGraphProvider for TestSpriteGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.test.sprite"
    }

    fn component_type(&self) -> &'static str {
        "amigo.test.Sprite2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        Some(
            payload
                .as_any()
                .downcast_ref::<TestSpritePayload>()?
                .render_layer
                .clone(),
        )
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<TestSpritePayload>() else {
            return;
        };
        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
        ctx.add_external_ref(
            "texture",
            SceneReferenceKind::UsesAsset,
            SceneReferenceTargetKind::Asset,
            &payload.texture,
            true,
        );
    }
}

struct TestTextGraphProvider;

impl PluginComponentGraphProvider for TestTextGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.test.text"
    }

    fn component_type(&self) -> &'static str {
        "amigo.test.Text2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        Some(
            payload
                .as_any()
                .downcast_ref::<TestTextPayload>()?
                .render_layer
                .clone(),
        )
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<TestTextPayload>() else {
            return;
        };
        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
        ctx.add_external_ref(
            "font",
            SceneReferenceKind::UsesFont,
            SceneReferenceTargetKind::Font,
            &payload.font,
            true,
        );
    }
}

struct TestSpriteRuntimePlugin;

impl RuntimePlugin for TestSpriteRuntimePlugin {
    fn name(&self) -> &'static str {
        "test-sprite-runtime-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        if let Some(schemas) = registry.resolve::<ComponentSchemaRegistry>() {
            schemas.register_descriptor(ScenePluginComponentDescriptor::new(
                "amigo.test.Sprite2D",
                "test",
                "Sprite2D",
            ));
            schemas.register_schema_provider(TestSpriteSchemaProvider);
        }
        if let Some(providers) = registry.resolve::<ComponentGraphProviderRegistry>() {
            providers.register(TestSpriteGraphProvider);
        }
        Ok(())
    }
}

struct TestTextRuntimePlugin;

impl RuntimePlugin for TestTextRuntimePlugin {
    fn name(&self) -> &'static str {
        "test-text-runtime-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        if let Some(schemas) = registry.resolve::<ComponentSchemaRegistry>() {
            schemas.register_descriptor(ScenePluginComponentDescriptor::new(
                "amigo.test.Text2D",
                "test",
                "Text2D",
            ));
            schemas.register_schema_provider(TestTextSchemaProvider);
        }
        if let Some(providers) = registry.resolve::<ComponentGraphProviderRegistry>() {
            providers.register(TestTextGraphProvider);
        }
        Ok(())
    }
}

#[test]
fn semantic_graph_builds_draw_layer_and_component_references() {
    let document: SceneDocument = serde_yaml::from_str(
        r##"
version: 1
scene:
  id: test
visual2d:
  render_layers:
    - id: world
      order: 0
  post_fx:
    - id: fx_quantize
      type: color_quantize
entities:
  - id: player
    name: Player
    components:
      - type: Sprite2D
        render_layer: world
        texture: player.png
        size: [32, 32]
"##,
    )
    .expect("scene should parse");

    let graph = build_semantic_scene_graph(&document, "test.yml");

    assert!(!graph.has_errors());
    assert!(graph.references.iter().any(|edge| {
        edge.kind == SceneReferenceKind::RendersIntoDrawLayer && edge.raw_target == "world"
    }));
    assert!(graph.references.iter().any(|edge| {
        edge.kind == SceneReferenceKind::UsesAsset && edge.raw_target == "player.png"
    }));
}

#[test]
fn semantic_graph_reports_missing_draw_layer() {
    let document: SceneDocument = serde_yaml::from_str(
        r##"
version: 1
scene:
  id: test
visual2d:
  render_layers:
    - id: world
entities:
  - id: player
    components:
      - type: Sprite2D
        render_layer: missing
        texture: player.png
        size: [32, 32]
"##,
    )
    .expect("scene should parse");

    let graph = build_semantic_scene_graph(&document, "test.yml");

    assert!(graph.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == SceneGraphDiagnosticSeverity::Error
            && diagnostic.code == "missing_draw_layer_ref"
    }));
}

#[test]
fn semantic_graph_uses_runtime_sprite_plugin_provider() {
    let runtime = RuntimeBuilder::default()
        .with_plugin(ScenePlugin)
        .expect("scene plugin should register")
        .with_plugin(TestSpriteRuntimePlugin)
        .expect("test sprite plugin should register")
        .build();
    let document = crate::load_scene_document_from_str_with_component_schemas(
        r##"
version: 1
scene:
  id: test
visual2d:
  render_layers:
    - id: world
      order: 0
entities:
  - id: player
    components:
      - type: Sprite2D
        render_layer: world
        texture: player.png
        size: [32, 32]
"##,
        runtime.resolve::<crate::ComponentSchemaRegistry>().as_deref(),
    )
    .expect("scene should parse");

    let graph = build_semantic_scene_graph_for_runtime(&runtime, &document, "test.yml");

    assert!(graph.references.iter().any(|edge| {
        edge.kind == SceneReferenceKind::RendersIntoDrawLayer && edge.raw_target == "world"
    }));
    assert!(graph.references.iter().any(|edge| {
        edge.kind == SceneReferenceKind::UsesAsset && edge.raw_target == "player.png"
    }));
}

#[test]
fn semantic_graph_uses_runtime_text_plugin_provider() {
    let runtime = RuntimeBuilder::default()
        .with_plugin(ScenePlugin)
        .expect("scene plugin should register")
        .with_plugin(TestTextRuntimePlugin)
        .expect("test text plugin should register")
        .build();
    let document = crate::load_scene_document_from_str_with_component_schemas(
        r##"
version: 1
scene:
  id: test
visual2d:
  render_layers:
    - id: ui
      order: 0
entities:
  - id: title
    components:
      - type: Text2D
        render_layer: ui
        content: Amigo
        font: ui/font
        bounds: [128, 32]
"##,
        runtime.resolve::<crate::ComponentSchemaRegistry>().as_deref(),
    )
    .expect("scene should parse");

    let graph = build_semantic_scene_graph_for_runtime(&runtime, &document, "test.yml");

    assert!(graph.references.iter().any(|edge| {
        edge.kind == SceneReferenceKind::RendersIntoDrawLayer && edge.raw_target == "ui"
    }));
    assert!(graph.references.iter().any(|edge| {
        edge.kind == SceneReferenceKind::UsesFont && edge.raw_target == "ui/font"
    }));
}
