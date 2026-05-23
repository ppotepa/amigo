use amigo_plugin_api::CandidateStatus;
use amigo_scene::{RenderContributionsDocument, SceneVectorShapeKindComponentDocument};
use amigo_vector_2d_plugin::participation::adapters::camera_optics::vector_coverage_to_camera_optics;
use amigo_vector_2d_plugin::runtime::collect_vector_2d_candidates;
use amigo_vector_2d_plugin::scene::{
    vector_2d_scene_descriptor, Vector2dDocument,
};

#[test]
fn vector_document_collects_vector_candidate() {
    let candidate = collect_vector_2d_candidates(&[Vector2dDocument {
        entity_name: "shape".to_owned(),
        render_layer: "world".to_owned(),
        kind: SceneVectorShapeKindComponentDocument::Polygon,
        points: Vec::new(),
        closed: false,
        radius: 0.0,
        segments: 32,
        stroke_color: None,
        stroke_width: 1.0,
        fill_color: None,
        render_contributions: RenderContributionsDocument::default(),
        material: None,
        z_index: 0.0,
    }])
    .remove(0);

    assert_eq!(candidate.status, CandidateStatus::Active);
    assert!(candidate
        .target_ids
        .iter()
        .any(|target| target.0 == "SceneColor"));
    assert!(matches!(
        vector_coverage_to_camera_optics(&candidate.coverage),
        amigo_camera_optics_plugin::api::CameraOpticalCoverage2d::VectorCoverage { .. }
    ));
}

#[test]
fn vector_plugin_owns_scene_descriptor() {
    let descriptor = vector_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.gfx.vector-2d.VectorShape2D"
    );
}
