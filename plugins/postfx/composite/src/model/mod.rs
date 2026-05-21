pub const POST_FX_2D_CAPABILITY: &str = "post_fx_2d";
pub const POST_FX_2D_PLUGIN_LABEL: &str = "amigo-composite-plugin";

mod flat_metadata;
pub use flat_metadata::*;

pub use amigo_render_api::{
    CameraExposure2d, CameraExposureMode2d, CameraOptics2d, ColorQuantize2d, ColorRamp2d, Crt2d,
    DirtyBloom2d, Downscale2d, FilmEmulsion2d, FilmNoise2d, FocusBlur2d,
    FocusBlurDebugView2d, FocusTarget2d, LensDroplets2dCertificationIssue,
    LensDroplets2dCertificationReport, LensDroplets2dCertificationSeverity,
    LensDroplets2dStage, PostFx2d, PostFx2dCacheKey, PostFx2dStack, PostFxBlur2d,
    PostFxCachedImagePolicy, PostFxDebugPolicy, PostFxEmbossEdges2d, PostFxEmbossMode2d,
    PostFxLensDroplets2d, PostFxRenderDescriptor, PostFxRenderInput, PostFxRenderOutput,
    PostFxWetReflections2d, RainGlass2d, RainGlassDebugView, RainGlassPatch,
    RainGlassRaindropCompose, ScanOutput2d, ShutterBlur2d, WetReflectionsDebugView,
};
