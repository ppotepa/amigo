mod blur;
mod camera_exposure;
mod camera_optics;
mod color_quantize;
mod crt;
pub(crate) mod dirty_bloom;
mod downscale;
mod emboss_edges;
mod film_emulsion;
mod film_noise;
pub(crate) mod focus_blur;
mod lens_droplets;
pub(crate) mod rain_glass;
mod registry;
pub(crate) mod runtime_key;
mod scan_output;
pub(crate) mod shutter_blur;
pub(crate) mod wet_reflections;

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
        PostFx2d::ColorQuantize(_)
        | PostFx2d::CameraExposure(_)
        | PostFx2d::CameraOptics(_)
        | PostFx2d::ColorRamp(_)
        | PostFx2d::Crt(_)
        | PostFx2d::Downscale(_)
        | PostFx2d::DirtyBloom(_)
        | PostFx2d::FilmEmulsion(_)
        | PostFx2d::FilmNoise(_)
        | PostFx2d::FocusBlur(_)
        | PostFx2d::ScanOutput(_)
        | PostFx2d::ShutterBlur(_) => source,
        PostFx2d::LensDroplets(_) => {
            // LensDroplets is a screen-space frame post-fx and is intentionally not applied to
            // cached layered images.
            source
        }
        PostFx2d::RainGlass(_) => {
            // RainGlass is a screen-space frame post-fx and is intentionally not applied to
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
