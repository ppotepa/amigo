mod blur;
mod emboss_edges;
mod lens_droplets;
mod wet_reflections;
mod registry;

use amigo_2d_post_fx::{PostFx2d, PostFxEmbossMode2d};
use image::RgbaImage;

pub(crate) use registry::execute_screen_space_post_fx;

pub(crate) fn apply_cached_image_post_fx_rgba(source: RgbaImage, effect: PostFx2d) -> RgbaImage {
    match effect {
        PostFx2d::Blur(blur) => blur::apply_blur(source, blur),
        PostFx2d::EmbossEdges(emboss) => match emboss.mode {
            PostFxEmbossMode2d::PrebakedImage | PostFxEmbossMode2d::LightAwareRuntime => {
                // Runtime light-aware pass is not wired into the frame graph yet.
                // For now this mode falls back to prebaked application.
                emboss_edges::apply_emboss_edges(source, emboss)
            }
        },
        PostFx2d::LensDroplets(_) => {
            // LensDroplets is a screen-space frame post-fx and is intentionally not applied to
            // cached layered images.
            source
        }
        PostFx2d::WetReflections(_) => {
            // WetReflections is a screen-space frame post-fx and is intentionally not applied
            // to cached layered images.
            source
        }
    }
}

