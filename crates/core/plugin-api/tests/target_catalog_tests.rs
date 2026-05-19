use amigo_plugin_api::{
    camera_artifact_layer, final_composite, scene_color, scene_depth,
    scene_emissive, scene_highlight, StandardTarget, TargetRef,
};

#[test]
fn standard_targets_have_stable_names() {
    assert_eq!(StandardTarget::SceneColor.as_str(), "SceneColor");
    assert_eq!(StandardTarget::SceneDepth.as_str(), "SceneDepth");
    assert_eq!(StandardTarget::SceneHighlight.as_str(), "SceneHighlight");
    assert_eq!(StandardTarget::SceneEmissive.as_str(), "SceneEmissive");
    assert_eq!(
        StandardTarget::CameraArtifactLayer.as_str(),
        "CameraArtifactLayer"
    );
    assert_eq!(StandardTarget::FinalComposite.as_str(), "FinalComposite");
}

#[test]
fn helpers_return_target_ids() {
    assert_eq!(scene_color().0, "SceneColor");
    assert_eq!(scene_depth().0, "SceneDepth");
    assert_eq!(scene_highlight().0, "SceneHighlight");
    assert_eq!(scene_emissive().0, "SceneEmissive");
    assert_eq!(camera_artifact_layer().0, "CameraArtifactLayer");
    assert_eq!(final_composite().0, "FinalComposite");
}

#[test]
fn standard_target_refs_can_be_constructed() {
    let read = TargetRef::read_standard(StandardTarget::SceneDepth);
    let write =
        TargetRef::write_standard(StandardTarget::CameraArtifactLayer);
    let contribute =
        TargetRef::contribute_standard(StandardTarget::SceneHighlight);

    assert_eq!(read.id.0, "SceneDepth");
    assert_eq!(write.id.0, "CameraArtifactLayer");
    assert_eq!(contribute.id.0, "SceneHighlight");
}
