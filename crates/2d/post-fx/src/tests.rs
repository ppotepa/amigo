use amigo_core::AmigoResult;

use crate::{
    PostFx2d, PostFx2dService, PostFx2dStack, PostFxBlur2d, PostFxSceneCommandContext,
    ScopedPostFx2dStack, handle_post_fx_scoped_stacks,
};

#[test]
fn post_fx_scoped_stacks_handler_updates_service() -> AmigoResult<()> {
    let service = PostFx2dService::default();
    let stack = PostFx2dStack::single(PostFx2d::Blur(PostFxBlur2d::default()));

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
            PostFx2d::Blur(PostFxBlur2d::default()),
            PostFx2d::Blur(PostFxBlur2d::default()),
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
