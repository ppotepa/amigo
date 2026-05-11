mod blur;
mod emboss_edges;

use amigo_2d_post_fx::{PostFx2d, PostFxEmbossMode2d};
use image::RgbaImage;

pub(crate) fn apply_post_fx_rgba(source: RgbaImage, effect: PostFx2d) -> RgbaImage {
    match effect {
        PostFx2d::Blur(blur) => blur::apply_blur(source, blur),
        PostFx2d::EmbossEdges(emboss) => match emboss.mode {
            PostFxEmbossMode2d::PrebakedImage | PostFxEmbossMode2d::LightAwareRuntime => {
                // Runtime light-aware pass is not wired into the frame graph yet.
                // For now this mode falls back to prebaked application.
                emboss_edges::apply_emboss_edges(source, emboss)
            }
        },
    }
}
