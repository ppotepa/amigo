use amigo_core::AmigoResult;
use amigo_render_api::{
    post_fx_blur, post_fx_camera_optics, post_fx_color_quantize, post_fx_color_ramp,
    post_fx_film_emulsion, post_fx_film_noise, post_fx_focus_blur, post_fx_lens_droplets,
    post_fx_rain_glass,
};

use crate::{
    CameraOptics2d, FilmNoise2d, FocusBlur2d, PostFx2dInstance, PostFx2dService, PostFx2dStack,
    PostFxBlur2d, PostFxPipelineKind, PostFxRole2d, PostFxSceneCommandContext, PostFxScope2d,
    RainGlass2d, ScopedPostFx2dStack, diagnose_post_fx_stacks, handle_post_fx_scoped_stacks,
};

#[test]
fn post_fx_scoped_stacks_handler_updates_service() -> AmigoResult<()> {
    let service = PostFx2dService::default();
    let stack = PostFx2dStack::single(post_fx_blur(PostFxBlur2d::default()));

    let outcome = handle_post_fx_scoped_stacks(
        PostFxSceneCommandContext {
            post_fx2d_service: &service,
        },
        vec![ScopedPostFx2dStack::from_frame_stack(stack)],
        Vec::new(),
    )?;

    assert_eq!(outcome.effect_count, 1);
    assert_eq!(service.frame_stack().unwrap_or_default().effects.len(), 1);
    assert!(service.lens_certification_reports().is_empty());
    Ok(())
}

#[test]
fn frame_effect_toggle_disables_rendering_without_removing_slot() {
    let service = PostFx2dService::default();
    let stack = PostFx2dStack {
        effects: vec![
            post_fx_blur(PostFxBlur2d::default()),
            post_fx_blur(PostFxBlur2d::default()),
        ],
    };

    service.set_scoped_stacks(vec![ScopedPostFx2dStack::from_frame_stack(stack)]);

    assert_eq!(service.frame_effect_count(), 2);
    assert_eq!(service.frame_stack().unwrap_or_default().effects.len(), 2);
    assert!(service.frame_effect_enabled(1));

    assert!(service.set_frame_effect_enabled(1, false));

    assert_eq!(service.frame_effect_count(), 2);
    assert!(!service.frame_effect_enabled(1));
    assert_eq!(service.frame_stack().unwrap_or_default().effects.len(), 1);

    assert!(service.set_frame_effect_enabled(1, true));

    assert!(service.frame_effect_enabled(1));
    assert_eq!(service.frame_stack().unwrap_or_default().effects.len(), 2);
}

#[test]
fn camera_capture_effects_have_camera_capture_role() {
    assert_eq!(
        post_fx_camera_optics(CameraOptics2d::default()).default_role(),
        PostFxRole2d::CameraCapture
    );
    assert_eq!(
        post_fx_focus_blur(FocusBlur2d::default()).photographic_family(),
        Some("dof")
    );
}

#[test]
fn duplicate_film_scan_reports_warning() {
    let camera_stack = ScopedPostFx2dStack::new(
        "camera:main:frame",
        PostFxScope2d::Frame,
        vec![PostFx2dInstance::new(
            "film",
            post_fx_film_emulsion(Default::default()),
        )],
    );
    let scene_stack = ScopedPostFx2dStack::new(
        "scene:frame",
        PostFxScope2d::Frame,
        vec![PostFx2dInstance::new(
            "noise",
            post_fx_film_noise(FilmNoise2d::default()),
        )],
    );

    let diagnostics = diagnose_post_fx_stacks(&[camera_stack, scene_stack]);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "camera_film_scan_duplicated" && diagnostic.family == Some("film_scan")
    }));
}

#[test]
fn duplicate_lens_surface_reports_specific_warning() {
    let camera_stack = ScopedPostFx2dStack::new(
        "camera:main:frame",
        PostFxScope2d::Frame,
        vec![PostFx2dInstance::new(
            "rain",
            post_fx_rain_glass(RainGlass2d::default()),
        )],
    );
    let scene_stack = ScopedPostFx2dStack::new(
        "scene:frame",
        PostFxScope2d::Frame,
        vec![PostFx2dInstance::new(
            "droplets",
            post_fx_lens_droplets(Default::default()),
        )],
    );

    let diagnostics = diagnose_post_fx_stacks(&[camera_stack, scene_stack]);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "camera_lens_surface_duplicated"
            && diagnostic.family == Some("lens_surface")
    }));
}

#[test]
fn duplicate_look_reports_specific_warning() {
    let camera_stack = ScopedPostFx2dStack::new(
        "camera:main:frame",
        PostFxScope2d::Frame,
        vec![PostFx2dInstance::new(
            "look",
            post_fx_color_ramp(Default::default()),
        )],
    );
    let scene_stack = ScopedPostFx2dStack::new(
        "presentation:frame",
        PostFxScope2d::Frame,
        vec![PostFx2dInstance::new(
            "quantize",
            post_fx_color_quantize(Default::default()),
        )],
    );

    let diagnostics = diagnose_post_fx_stacks(&[camera_stack, scene_stack]);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "camera_look_duplicated" && diagnostic.family == Some("look")
    }));
}

#[test]
fn non_frame_scoped_post_fx_reports_unsupported_warning() {
    let mut object_stack = ScopedPostFx2dStack::new(
        "object:rain",
        PostFxScope2d::SceneObjectPixels {
            scene_object_id: "rain-mid-emitter".to_owned(),
        },
        vec![PostFx2dInstance::new(
            "blur",
            post_fx_blur(PostFxBlur2d::default()),
        )],
    );
    object_stack.pipeline = PostFxPipelineKind::Unsupported;

    let diagnostics = diagnose_post_fx_stacks(&[object_stack]);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "unsupported_scoped_post_fx" })
    );
}
