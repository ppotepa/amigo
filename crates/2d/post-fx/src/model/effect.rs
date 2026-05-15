use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum PostFx2d {
    Blur(PostFxBlur2d),
    ColorQuantize(ColorQuantize2d),
    ColorRamp(ColorRamp2d),
    Crt(Crt2d),
    Downscale(Downscale2d),
    DirtyBloom(DirtyBloom2d),
    EmbossEdges(PostFxEmbossEdges2d),
    FilmNoise(FilmNoise2d),
    LensDroplets(PostFxLensDroplets2d),
    RainGlass(RainGlass2d),
    ShutterBlur(ShutterBlur2d),
    WetReflections(PostFxWetReflections2d),
}

impl PostFx2d {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Blur(_) => "blur",
            Self::ColorQuantize(_) => "color_quantize",
            Self::ColorRamp(_) => "color_ramp",
            Self::Crt(_) => "crt",
            Self::Downscale(_) => "downscale",
            Self::DirtyBloom(_) => "dirty_bloom",
            Self::EmbossEdges(_) => "embossed_edges",
            Self::FilmNoise(_) => "film_noise",
            Self::LensDroplets(_) => "lens_droplets",
            Self::RainGlass(_) => "rain_glass",
            Self::ShutterBlur(_) => "shutter_blur",
            Self::WetReflections(_) => "wet_reflections",
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
            Self::ColorQuantize(effect) => Self::ColorQuantize(effect.normalized()),
            Self::ColorRamp(effect) => Self::ColorRamp(effect.normalized()),
            Self::Crt(crt) => Self::Crt(crt.normalized()),
            Self::Downscale(effect) => Self::Downscale(effect.normalized()),
            Self::DirtyBloom(bloom) => Self::DirtyBloom(bloom.normalized()),
            Self::EmbossEdges(emboss) => Self::EmbossEdges(emboss.normalized()),
            Self::FilmNoise(noise) => Self::FilmNoise(noise.normalized()),
            Self::LensDroplets(lens) => Self::LensDroplets(lens.normalized()),
            Self::RainGlass(rain) => Self::RainGlass(rain.normalized()),
            Self::ShutterBlur(effect) => Self::ShutterBlur(effect.normalized()),
            Self::WetReflections(effect) => Self::WetReflections(effect.normalized()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Blur(blur) => blur.is_active(),
            Self::ColorQuantize(effect) => effect.is_active(),
            Self::ColorRamp(effect) => effect.is_active(),
            Self::Crt(crt) => crt.is_active(),
            Self::Downscale(effect) => effect.is_active(),
            Self::DirtyBloom(bloom) => bloom.is_active(),
            Self::EmbossEdges(emboss) => emboss.is_active(),
            Self::FilmNoise(noise) => noise.is_active(),
            Self::LensDroplets(lens) => lens.is_active(),
            Self::RainGlass(rain) => rain.is_active(),
            Self::ShutterBlur(effect) => effect.is_active(),
            Self::WetReflections(effect) => effect.is_active(),
        }
    }
}
