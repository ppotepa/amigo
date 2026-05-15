use crate::{
    SceneDocument, SceneGraphDiagnosticSeverity, SceneReferenceKind, build_semantic_scene_graph,
};

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
