use super::*;
use crate::PostFxRole2d;

#[derive(Debug, Clone, PartialEq)]
pub struct PostFx2d {
    payload: EffectPayload,
}

#[derive(Debug, Clone, PartialEq)]
enum EffectPayload {
    Blur(PostFxBlur2d),
    CameraExposure(CameraExposure2d),
    CameraOptics(CameraOptics2d),
    ColorQuantize(ColorQuantize2d),
    ColorRamp(ColorRamp2d),
    Crt(Crt2d),
    Downscale(Downscale2d),
    DirtyBloom(DirtyBloom2d),
    EmbossEdges(PostFxEmbossEdges2d),
    FilmEmulsion(FilmEmulsion2d),
    FilmNoise(FilmNoise2d),
    FocusBlur(FocusBlur2d),
    LensDroplets(PostFxLensDroplets2d),
    RainGlass(RainGlass2d),
    ScanOutput(ScanOutput2d),
    ShutterBlur(ShutterBlur2d),
    WetReflections(PostFxWetReflections2d),
}

pub fn post_fx_blur(effect: PostFxBlur2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::Blur(effect),
    }
}

pub fn post_fx_camera_exposure(effect: CameraExposure2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::CameraExposure(effect),
    }
}

pub fn post_fx_camera_optics(effect: CameraOptics2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::CameraOptics(effect),
    }
}

pub fn post_fx_color_quantize(effect: ColorQuantize2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::ColorQuantize(effect),
    }
}

pub fn post_fx_color_ramp(effect: ColorRamp2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::ColorRamp(effect),
    }
}

pub fn post_fx_crt(effect: Crt2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::Crt(effect),
    }
}

pub fn post_fx_downscale(effect: Downscale2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::Downscale(effect),
    }
}

pub fn post_fx_dirty_bloom(effect: DirtyBloom2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::DirtyBloom(effect),
    }
}

pub fn post_fx_emboss_edges(effect: PostFxEmbossEdges2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::EmbossEdges(effect),
    }
}

pub fn post_fx_film_emulsion(effect: FilmEmulsion2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::FilmEmulsion(effect),
    }
}

pub fn post_fx_film_noise(effect: FilmNoise2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::FilmNoise(effect),
    }
}

pub fn post_fx_focus_blur(effect: FocusBlur2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::FocusBlur(effect),
    }
}

pub fn post_fx_lens_droplets(effect: PostFxLensDroplets2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::LensDroplets(effect),
    }
}

pub fn post_fx_rain_glass(effect: RainGlass2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::RainGlass(effect),
    }
}

pub fn post_fx_scan_output(effect: ScanOutput2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::ScanOutput(effect),
    }
}

pub fn post_fx_shutter_blur(effect: ShutterBlur2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::ShutterBlur(effect),
    }
}

pub fn post_fx_wet_reflections(effect: PostFxWetReflections2d) -> PostFx2d {
    PostFx2d {
        payload: EffectPayload::WetReflections(effect),
    }
}

impl PostFx2d {
    pub fn into_blur(self) -> Option<PostFxBlur2d> {
        match self.payload {
            EffectPayload::Blur(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_emboss_edges(self) -> Option<PostFxEmbossEdges2d> {
        match self.payload {
            EffectPayload::EmbossEdges(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_camera_exposure(self) -> Option<CameraExposure2d> {
        match self.payload {
            EffectPayload::CameraExposure(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_camera_optics(self) -> Option<CameraOptics2d> {
        match self.payload {
            EffectPayload::CameraOptics(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_color_quantize(self) -> Option<ColorQuantize2d> {
        match self.payload {
            EffectPayload::ColorQuantize(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_color_ramp(self) -> Option<ColorRamp2d> {
        match self.payload {
            EffectPayload::ColorRamp(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_crt(self) -> Option<Crt2d> {
        match self.payload {
            EffectPayload::Crt(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_downscale(self) -> Option<Downscale2d> {
        match self.payload {
            EffectPayload::Downscale(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_dirty_bloom(self) -> Option<DirtyBloom2d> {
        match self.payload {
            EffectPayload::DirtyBloom(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_film_emulsion(self) -> Option<FilmEmulsion2d> {
        match self.payload {
            EffectPayload::FilmEmulsion(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_film_noise(self) -> Option<FilmNoise2d> {
        match self.payload {
            EffectPayload::FilmNoise(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_focus_blur(self) -> Option<FocusBlur2d> {
        match self.payload {
            EffectPayload::FocusBlur(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_lens_droplets(self) -> Option<PostFxLensDroplets2d> {
        match self.payload {
            EffectPayload::LensDroplets(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_rain_glass(self) -> Option<RainGlass2d> {
        match self.payload {
            EffectPayload::RainGlass(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_scan_output(self) -> Option<ScanOutput2d> {
        match self.payload {
            EffectPayload::ScanOutput(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_shutter_blur(self) -> Option<ShutterBlur2d> {
        match self.payload {
            EffectPayload::ShutterBlur(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn into_wet_reflections(self) -> Option<PostFxWetReflections2d> {
        match self.payload {
            EffectPayload::WetReflections(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn as_wet_reflections(&self) -> Option<&PostFxWetReflections2d> {
        match &self.payload {
            EffectPayload::WetReflections(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn as_rain_glass(&self) -> Option<&RainGlass2d> {
        match &self.payload {
            EffectPayload::RainGlass(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn as_color_ramp(&self) -> Option<&ColorRamp2d> {
        match &self.payload {
            EffectPayload::ColorRamp(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn as_color_quantize(&self) -> Option<&ColorQuantize2d> {
        match &self.payload {
            EffectPayload::ColorQuantize(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn as_focus_blur(&self) -> Option<&FocusBlur2d> {
        match &self.payload {
            EffectPayload::FocusBlur(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn as_shutter_blur(&self) -> Option<&ShutterBlur2d> {
        match &self.payload {
            EffectPayload::ShutterBlur(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn kind(&self) -> &'static str {
        match &self.payload {
            EffectPayload::Blur(_) => "blur",
            EffectPayload::CameraExposure(_) => "camera_exposure",
            EffectPayload::CameraOptics(_) => "camera_optics",
            EffectPayload::ColorQuantize(_) => "color_quantize",
            EffectPayload::ColorRamp(_) => "color_ramp",
            EffectPayload::Crt(_) => "crt",
            EffectPayload::Downscale(_) => "downscale",
            EffectPayload::DirtyBloom(_) => "dirty_bloom",
            EffectPayload::EmbossEdges(_) => "embossed_edges",
            EffectPayload::FilmEmulsion(_) => "film_emulsion",
            EffectPayload::FilmNoise(_) => "film_noise",
            EffectPayload::FocusBlur(_) => "focus_blur",
            EffectPayload::LensDroplets(_) => "lens_droplets",
            EffectPayload::RainGlass(_) => "rain_glass",
            EffectPayload::ScanOutput(_) => "scan_output",
            EffectPayload::ShutterBlur(_) => "shutter_blur",
            EffectPayload::WetReflections(_) => "wet_reflections",
        }
    }

    pub fn default_role(&self) -> PostFxRole2d {
        match &self.payload {
            EffectPayload::CameraExposure(_)
            | EffectPayload::CameraOptics(_)
            | EffectPayload::FocusBlur(_)
            | EffectPayload::RainGlass(_)
            | EffectPayload::ShutterBlur(_)
            | EffectPayload::FilmEmulsion(_)
            | EffectPayload::ScanOutput(_) => PostFxRole2d::CameraCapture,
            EffectPayload::ColorRamp(_)
            | EffectPayload::Crt(_)
            | EffectPayload::Downscale(_)
            | EffectPayload::ColorQuantize(_) => PostFxRole2d::Presentation,
            EffectPayload::Blur(_)
            | EffectPayload::DirtyBloom(_)
            | EffectPayload::EmbossEdges(_)
            | EffectPayload::LensDroplets(_)
            | EffectPayload::WetReflections(_) => PostFxRole2d::SceneLocal,
            EffectPayload::FilmNoise(_) => PostFxRole2d::Legacy,
        }
    }

    pub fn photographic_family(&self) -> Option<&'static str> {
        match &self.payload {
            EffectPayload::CameraExposure(_) => Some("exposure"),
            EffectPayload::CameraOptics(_) => Some("lens"),
            EffectPayload::FocusBlur(_) => Some("dof"),
            EffectPayload::RainGlass(_) | EffectPayload::LensDroplets(_) => Some("lens_surface"),
            EffectPayload::ShutterBlur(_) => Some("shutter"),
            EffectPayload::FilmEmulsion(_)
            | EffectPayload::FilmNoise(_)
            | EffectPayload::ScanOutput(_) => Some("film_scan"),
            EffectPayload::ColorRamp(_) | EffectPayload::ColorQuantize(_) => Some("look"),
            EffectPayload::DirtyBloom(_) => Some("highlight_response"),
            _ => None,
        }
    }

    pub fn uses_cached_image_pipeline(&self) -> bool {
        matches!(
            self.render_descriptor().cached_image_policy,
            PostFxCachedImagePolicy::RasterEffect
                | PostFxCachedImagePolicy::RasterEffectWithBoundsExpansion
        )
    }

    pub fn uses_frame_graph_pipeline(&self) -> bool {
        self.render_descriptor().frame_graph_enabled
    }

    pub fn normalized(self) -> Self {
        match self.payload {
            EffectPayload::Blur(blur) => post_fx_blur(blur.normalized()),
            EffectPayload::CameraExposure(effect) => post_fx_camera_exposure(effect.normalized()),
            EffectPayload::CameraOptics(effect) => post_fx_camera_optics(effect.normalized()),
            EffectPayload::ColorQuantize(effect) => post_fx_color_quantize(effect.normalized()),
            EffectPayload::ColorRamp(effect) => post_fx_color_ramp(effect.normalized()),
            EffectPayload::Crt(crt) => post_fx_crt(crt.normalized()),
            EffectPayload::Downscale(effect) => post_fx_downscale(effect.normalized()),
            EffectPayload::DirtyBloom(bloom) => post_fx_dirty_bloom(bloom.normalized()),
            EffectPayload::EmbossEdges(emboss) => post_fx_emboss_edges(emboss.normalized()),
            EffectPayload::FilmEmulsion(effect) => post_fx_film_emulsion(effect.normalized()),
            EffectPayload::FilmNoise(noise) => post_fx_film_noise(noise.normalized()),
            EffectPayload::FocusBlur(effect) => post_fx_focus_blur(effect.normalized()),
            EffectPayload::LensDroplets(lens) => post_fx_lens_droplets(lens.normalized()),
            EffectPayload::RainGlass(rain) => post_fx_rain_glass(rain.normalized()),
            EffectPayload::ScanOutput(effect) => post_fx_scan_output(effect.normalized()),
            EffectPayload::ShutterBlur(effect) => post_fx_shutter_blur(effect.normalized()),
            EffectPayload::WetReflections(effect) => post_fx_wet_reflections(effect.normalized()),
        }
    }

    pub fn is_active(&self) -> bool {
        match &self.payload {
            EffectPayload::Blur(blur) => blur.is_active(),
            EffectPayload::CameraExposure(effect) => effect.is_active(),
            EffectPayload::CameraOptics(effect) => effect.is_active(),
            EffectPayload::ColorQuantize(effect) => effect.is_active(),
            EffectPayload::ColorRamp(effect) => effect.is_active(),
            EffectPayload::Crt(crt) => crt.is_active(),
            EffectPayload::Downscale(effect) => effect.is_active(),
            EffectPayload::DirtyBloom(bloom) => bloom.is_active(),
            EffectPayload::EmbossEdges(emboss) => emboss.is_active(),
            EffectPayload::FilmEmulsion(effect) => effect.is_active(),
            EffectPayload::FilmNoise(noise) => noise.is_active(),
            EffectPayload::FocusBlur(effect) => effect.is_active(),
            EffectPayload::LensDroplets(lens) => lens.is_active(),
            EffectPayload::RainGlass(rain) => rain.is_active(),
            EffectPayload::ScanOutput(effect) => effect.is_active(),
            EffectPayload::ShutterBlur(effect) => effect.is_active(),
            EffectPayload::WetReflections(effect) => effect.is_active(),
        }
    }
}
