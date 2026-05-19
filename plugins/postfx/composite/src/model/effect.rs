use super::*;
use crate::PostFxRole2d;

#[derive(Debug, Clone, PartialEq)]
pub enum PostFx2d {
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

impl PostFx2d {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Blur(_) => "blur",
            Self::CameraExposure(_) => "camera_exposure",
            Self::CameraOptics(_) => "camera_optics",
            Self::ColorQuantize(_) => "color_quantize",
            Self::ColorRamp(_) => "color_ramp",
            Self::Crt(_) => "crt",
            Self::Downscale(_) => "downscale",
            Self::DirtyBloom(_) => "dirty_bloom",
            Self::EmbossEdges(_) => "embossed_edges",
            Self::FilmEmulsion(_) => "film_emulsion",
            Self::FilmNoise(_) => "film_noise",
            Self::FocusBlur(_) => "focus_blur",
            Self::LensDroplets(_) => "lens_droplets",
            Self::RainGlass(_) => "rain_glass",
            Self::ScanOutput(_) => "scan_output",
            Self::ShutterBlur(_) => "shutter_blur",
            Self::WetReflections(_) => "wet_reflections",
        }
    }

    pub fn default_role(&self) -> PostFxRole2d {
        match self {
            Self::CameraExposure(_)
            | Self::CameraOptics(_)
            | Self::FocusBlur(_)
            | Self::RainGlass(_)
            | Self::ShutterBlur(_)
            | Self::FilmEmulsion(_)
            | Self::ScanOutput(_) => PostFxRole2d::CameraCapture,
            Self::ColorRamp(_) | Self::Crt(_) | Self::Downscale(_) | Self::ColorQuantize(_) => {
                PostFxRole2d::Presentation
            }
            Self::Blur(_)
            | Self::DirtyBloom(_)
            | Self::EmbossEdges(_)
            | Self::LensDroplets(_)
            | Self::WetReflections(_) => PostFxRole2d::SceneLocal,
            Self::FilmNoise(_) => PostFxRole2d::Legacy,
        }
    }

    pub fn photographic_family(&self) -> Option<&'static str> {
        match self {
            Self::CameraExposure(_) => Some("exposure"),
            Self::CameraOptics(_) => Some("lens"),
            Self::FocusBlur(_) => Some("dof"),
            Self::RainGlass(_) | Self::LensDroplets(_) => Some("lens_surface"),
            Self::ShutterBlur(_) => Some("shutter"),
            Self::FilmEmulsion(_) | Self::FilmNoise(_) | Self::ScanOutput(_) => Some("film_scan"),
            Self::ColorRamp(_) | Self::ColorQuantize(_) => Some("look"),
            Self::DirtyBloom(_) => Some("highlight_response"),
            _ => None,
        }
    }

    pub fn is_cached_image_compatible(&self) -> bool {
        matches!(self, Self::Blur(_) | Self::EmbossEdges(_))
    }

    pub fn is_frame_graph_compatible(&self) -> bool {
        !self.is_cached_image_compatible()
    }

    pub fn normalized(self) -> Self {
        match self {
            Self::Blur(blur) => Self::Blur(blur.normalized()),
            Self::CameraExposure(effect) => Self::CameraExposure(effect.normalized()),
            Self::CameraOptics(effect) => Self::CameraOptics(effect.normalized()),
            Self::ColorQuantize(effect) => Self::ColorQuantize(effect.normalized()),
            Self::ColorRamp(effect) => Self::ColorRamp(effect.normalized()),
            Self::Crt(crt) => Self::Crt(crt.normalized()),
            Self::Downscale(effect) => Self::Downscale(effect.normalized()),
            Self::DirtyBloom(bloom) => Self::DirtyBloom(bloom.normalized()),
            Self::EmbossEdges(emboss) => Self::EmbossEdges(emboss.normalized()),
            Self::FilmEmulsion(effect) => Self::FilmEmulsion(effect.normalized()),
            Self::FilmNoise(noise) => Self::FilmNoise(noise.normalized()),
            Self::FocusBlur(effect) => Self::FocusBlur(effect.normalized()),
            Self::LensDroplets(lens) => Self::LensDroplets(lens.normalized()),
            Self::RainGlass(rain) => Self::RainGlass(rain.normalized()),
            Self::ScanOutput(effect) => Self::ScanOutput(effect.normalized()),
            Self::ShutterBlur(effect) => Self::ShutterBlur(effect.normalized()),
            Self::WetReflections(effect) => Self::WetReflections(effect.normalized()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Blur(blur) => blur.is_active(),
            Self::CameraExposure(effect) => effect.is_active(),
            Self::CameraOptics(effect) => effect.is_active(),
            Self::ColorQuantize(effect) => effect.is_active(),
            Self::ColorRamp(effect) => effect.is_active(),
            Self::Crt(crt) => crt.is_active(),
            Self::Downscale(effect) => effect.is_active(),
            Self::DirtyBloom(bloom) => bloom.is_active(),
            Self::EmbossEdges(emboss) => emboss.is_active(),
            Self::FilmEmulsion(effect) => effect.is_active(),
            Self::FilmNoise(noise) => noise.is_active(),
            Self::FocusBlur(effect) => effect.is_active(),
            Self::LensDroplets(lens) => lens.is_active(),
            Self::RainGlass(rain) => rain.is_active(),
            Self::ScanOutput(effect) => effect.is_active(),
            Self::ShutterBlur(effect) => effect.is_active(),
            Self::WetReflections(effect) => effect.is_active(),
        }
    }
}
