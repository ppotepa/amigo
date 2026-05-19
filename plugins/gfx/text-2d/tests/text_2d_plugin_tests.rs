use amigo_plugin_api::CandidateStatus;
use amigo_text_2d_plugin::participation::adapters::camera_optics::text_coverage_to_camera_optics;
use amigo_text_2d_plugin::runtime::collect_text_2d_candidates;
use amigo_text_2d_plugin::scene::{
    text_2d_scene_descriptor, Text2dDocument,
};

#[test]
fn text_document_collects_glyph_candidate() {
    let candidate = collect_text_2d_candidates(&[Text2dDocument {
        entity_name: "title".to_owned(),
        render_layer: "ui".to_owned(),
        text: "Amigo".to_owned(),
    }])
    .remove(0);

    assert_eq!(candidate.status, CandidateStatus::Active);
    assert!(candidate
        .target_ids
        .iter()
        .any(|target| target.0 == "SceneColor"));
    assert!(matches!(
        text_coverage_to_camera_optics(&candidate.coverage),
        amigo_camera_optics_plugin::api::CameraOpticalCoverage2d::Glyphs { .. }
    ));
}

#[test]
fn text_plugin_owns_scene_descriptor() {
    let descriptor = text_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.gfx.text-2d.Text2D");
}
