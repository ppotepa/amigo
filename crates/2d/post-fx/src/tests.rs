use amigo_core::AmigoResult;

use crate::{
    PostFx2d, PostFx2dService, PostFx2dStack, PostFxBlur2d, PostFxSceneCommandContext,
    handle_post_fx_scene_stack,
};

#[test]
fn post_fx_scene_stack_handler_updates_service() -> AmigoResult<()> {
    let service = PostFx2dService::default();
    let stack = PostFx2dStack::single(PostFx2d::Blur(PostFxBlur2d::default()));

    let outcome = handle_post_fx_scene_stack(
        PostFxSceneCommandContext {
            post_fx2d_service: &service,
        },
        stack,
        Vec::new(),
    )?;

    assert_eq!(outcome.effect_count, 1);
    assert_eq!(service.scene_stack().effects.len(), 1);
    assert!(service.lens_certification_reports().is_empty());
    Ok(())
}

