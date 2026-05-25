mod blur;
mod camera_exposure;
mod camera_optics;
mod color_quantize;
mod crt;
pub(crate) mod dirty_bloom;
mod downscale;
mod emboss_edges;
mod executor;
mod executor_registry;
mod film_emulsion;
mod film_noise;
pub(crate) mod focus_blur;
mod lens_droplets;
pub(crate) mod pipelines;
pub(crate) mod rain_glass;
mod registry;
pub(crate) mod runtime_key;
mod scan_output;
pub(crate) mod shaders;
pub(crate) mod shutter_blur;
pub(crate) mod wet_reflections;

use amigo_render_api::{PostFx2d, PostFxEmbossMode2d};
use image::RgbaImage;

pub(crate) use executor::{WgpuPostFxExecutionContext, WgpuPostFxExecutor};
pub(crate) use executor_registry::WgpuPostFxExecutorRegistry;
pub(crate) use registry::default_wgpu_screen_effect_executors;
pub(crate) use registry::execute_screen_space_post_fx;

pub(crate) fn apply_cached_raster_effect_rgba(source: RgbaImage, effect: PostFx2d) -> RgbaImage {
    match effect.kind() {
        "blur" => {
            let Some(blur) = effect.into_blur() else {
                return source;
            };
            blur::apply_blur(source, blur)
        }
        "embossed_edges" => {
            let Some(emboss) = effect.into_emboss_edges() else {
                return source;
            };
            match emboss.mode {
                PostFxEmbossMode2d::PrebakedImage | PostFxEmbossMode2d::LightAwareRuntime => {
                    // Runtime light-aware pass is not wired into the frame graph yet.
                    // For now this mode uses prebaked application.
                    emboss_edges::apply_emboss_edges(source, emboss)
                }
            }
        }
        "lens_droplets" => {
            // LensDroplets is a screen-space frame post-fx and is intentionally not applied to
            // cached layered images.
            source
        }
        "rain_glass" => {
            // RainGlass is a screen-space frame post-fx and is intentionally not applied to
            // cached layered images.
            source
        }
        "wet_reflections" => {
            // WetReflections is a screen-space frame post-fx and is intentionally not applied
            // to cached layered images.
            source
        }
        _ => source,
    }
}
