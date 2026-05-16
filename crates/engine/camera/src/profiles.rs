use amigo_2d_post_fx::{ColorRamp2d, RainGlass2d, RainGlassDebugView, RainGlassRaindropCompose};
use amigo_assets::{AssetCatalog, AssetKey, PreparedAsset, PreparedAssetKind};

pub use crate::film_grain::FilmGrainProfile2d;

#[derive(Debug, Clone, PartialEq)]
pub struct LensProfile2d {
    pub id: &'static str,
    pub label: &'static str,
    pub focal_length_mm: f32,
    pub aberration_px: f32,
    pub distortion: f32,
    pub vignette: f32,
    pub edge_softness_px: f32,
    pub flare_strength: f32,
    pub dirt: f32,
    pub halation_bias: f32,
    pub lens_bloom: f32,
    pub flare_ghosts: f32,
    pub anamorphic_squeeze: f32,
    pub coma: f32,
    pub cat_eye_bokeh: f32,
    pub focus_breathing: f32,
}

impl LensProfile2d {
    pub fn scaled(mut self, intensity: f32) -> Self {
        let intensity = intensity.clamp(0.0, 1.0);
        self.aberration_px *= intensity;
        self.distortion *= intensity;
        self.vignette *= intensity;
        self.edge_softness_px *= intensity;
        self.flare_strength *= intensity;
        self.dirt *= intensity;
        self.halation_bias *= intensity;
        self.lens_bloom *= intensity;
        self.flare_ghosts *= intensity;
        self.coma *= intensity;
        self.cat_eye_bokeh *= intensity;
        self.focus_breathing *= intensity;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilmStockProfile2d {
    pub id: &'static str,
    pub label: &'static str,
    pub base_iso: f32,
    pub color_shift: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub flicker: f32,
    pub vignette: f32,
    pub opacity: f32,
    pub toe: f32,
    pub shoulder: f32,
    pub black_lift: f32,
    pub print_fade: f32,
    pub dust: f32,
    pub scratches: f32,
    pub push_pull: f32,
    pub gate_weave: f32,
    pub scan_softness: f32,
    pub grain: FilmGrainProfile2d,
}

pub const BUILTIN_LENS_PROFILES_2D: &[LensProfile2d] = &[
    LensProfile2d {
        id: "clean_modern_35mm",
        label: "Clean Modern 35mm",
        focal_length_mm: 35.0,
        aberration_px: 0.04,
        distortion: 0.00,
        vignette: 0.04,
        edge_softness_px: 0.05,
        flare_strength: 0.02,
        dirt: 0.00,
        halation_bias: 0.02,
        lens_bloom: 0.02,
        flare_ghosts: 0.02,
        anamorphic_squeeze: 1.0,
        coma: 0.02,
        cat_eye_bokeh: 0.02,
        focus_breathing: 0.02,
    },
    LensProfile2d {
        id: "clean_modern_50mm",
        label: "Clean Modern 50mm",
        focal_length_mm: 50.0,
        aberration_px: 0.03,
        distortion: 0.00,
        vignette: 0.03,
        edge_softness_px: 0.04,
        flare_strength: 0.02,
        dirt: 0.00,
        halation_bias: 0.02,
        lens_bloom: 0.02,
        flare_ghosts: 0.02,
        anamorphic_squeeze: 1.0,
        coma: 0.02,
        cat_eye_bokeh: 0.02,
        focus_breathing: 0.02,
    },
    LensProfile2d {
        id: "vintage_soviet_35mm_dirty",
        label: "Vintage Soviet 35mm Dirty",
        focal_length_mm: 35.0,
        aberration_px: 0.55,
        distortion: 0.07,
        vignette: 0.32,
        edge_softness_px: 0.65,
        flare_strength: 0.36,
        dirt: 0.48,
        halation_bias: 0.22,
        lens_bloom: 0.22,
        flare_ghosts: 0.22,
        anamorphic_squeeze: 1.0,
        coma: 0.22,
        cat_eye_bokeh: 0.22,
        focus_breathing: 0.22,
    },
    LensProfile2d {
        id: "vintage_soviet_58mm_soft",
        label: "Vintage Soviet 58mm Soft",
        focal_length_mm: 58.0,
        aberration_px: 0.36,
        distortion: 0.03,
        vignette: 0.28,
        edge_softness_px: 0.92,
        flare_strength: 0.42,
        dirt: 0.24,
        halation_bias: 0.28,
        lens_bloom: 0.28,
        flare_ghosts: 0.28,
        anamorphic_squeeze: 1.0,
        coma: 0.28,
        cat_eye_bokeh: 0.28,
        focus_breathing: 0.28,
    },
    LensProfile2d {
        id: "cheap_cctv_1996",
        label: "Cheap CCTV 1996",
        focal_length_mm: 24.0,
        aberration_px: 0.86,
        distortion: 0.18,
        vignette: 0.48,
        edge_softness_px: 1.10,
        flare_strength: 0.14,
        dirt: 0.28,
        halation_bias: 0.08,
        lens_bloom: 0.08,
        flare_ghosts: 0.08,
        anamorphic_squeeze: 1.0,
        coma: 0.08,
        cat_eye_bokeh: 0.08,
        focus_breathing: 0.08,
    },
    LensProfile2d {
        id: "disposable_plastic_28mm",
        label: "Disposable Plastic 28mm",
        focal_length_mm: 28.0,
        aberration_px: 0.72,
        distortion: 0.12,
        vignette: 0.44,
        edge_softness_px: 1.25,
        flare_strength: 0.20,
        dirt: 0.18,
        halation_bias: 0.12,
        lens_bloom: 0.12,
        flare_ghosts: 0.12,
        anamorphic_squeeze: 1.0,
        coma: 0.12,
        cat_eye_bokeh: 0.12,
        focus_breathing: 0.12,
    },
    LensProfile2d {
        id: "anamorphic_rain_streak",
        label: "Anamorphic Rain Streak",
        focal_length_mm: 40.0,
        aberration_px: 0.42,
        distortion: 0.05,
        vignette: 0.24,
        edge_softness_px: 0.42,
        flare_strength: 0.74,
        dirt: 0.34,
        halation_bias: 0.35,
        lens_bloom: 0.35,
        flare_ghosts: 0.35,
        anamorphic_squeeze: 1.0,
        coma: 0.35,
        cat_eye_bokeh: 0.35,
        focus_breathing: 0.35,
    },
    LensProfile2d {
        id: "noir_prime_low_contrast",
        label: "Noir Prime Low Contrast",
        focal_length_mm: 45.0,
        aberration_px: 0.24,
        distortion: 0.02,
        vignette: 0.38,
        edge_softness_px: 0.48,
        flare_strength: 0.26,
        dirt: 0.16,
        halation_bias: 0.18,
        lens_bloom: 0.18,
        flare_ghosts: 0.18,
        anamorphic_squeeze: 1.0,
        coma: 0.18,
        cat_eye_bokeh: 0.18,
        focus_breathing: 0.18,
    },
    LensProfile2d {
        id: "neon_barrel_wide",
        label: "Neon Barrel Wide",
        focal_length_mm: 24.0,
        aberration_px: 0.68,
        distortion: 0.20,
        vignette: 0.34,
        edge_softness_px: 0.70,
        flare_strength: 0.58,
        dirt: 0.18,
        halation_bias: 0.30,
        lens_bloom: 0.30,
        flare_ghosts: 0.30,
        anamorphic_squeeze: 1.0,
        coma: 0.30,
        cat_eye_bokeh: 0.30,
        focus_breathing: 0.30,
    },
    LensProfile2d {
        id: "telephoto_soft_85mm",
        label: "Telephoto Soft 85mm",
        focal_length_mm: 85.0,
        aberration_px: 0.20,
        distortion: -0.02,
        vignette: 0.18,
        edge_softness_px: 0.70,
        flare_strength: 0.25,
        dirt: 0.10,
        halation_bias: 0.12,
        lens_bloom: 0.12,
        flare_ghosts: 0.12,
        anamorphic_squeeze: 1.0,
        coma: 0.12,
        cat_eye_bokeh: 0.12,
        focus_breathing: 0.12,
    },
    LensProfile2d {
        id: "expired_compact_zoom",
        label: "Expired Compact Zoom",
        focal_length_mm: 38.0,
        aberration_px: 0.62,
        distortion: 0.09,
        vignette: 0.40,
        edge_softness_px: 0.95,
        flare_strength: 0.22,
        dirt: 0.30,
        halation_bias: 0.16,
        lens_bloom: 0.16,
        flare_ghosts: 0.16,
        anamorphic_squeeze: 1.0,
        coma: 0.16,
        cat_eye_bokeh: 0.16,
        focus_breathing: 0.16,
    },
    LensProfile2d {
        id: "security_monitor_soft",
        label: "Security Monitor Soft",
        focal_length_mm: 30.0,
        aberration_px: 0.48,
        distortion: 0.14,
        vignette: 0.46,
        edge_softness_px: 1.45,
        flare_strength: 0.08,
        dirt: 0.22,
        halation_bias: 0.04,
        lens_bloom: 0.04,
        flare_ghosts: 0.04,
        anamorphic_squeeze: 1.0,
        coma: 0.04,
        cat_eye_bokeh: 0.04,
        focus_breathing: 0.04,
    },
    LensProfile2d {
        id: "wet_window_macro",
        label: "Wet Window Macro",
        focal_length_mm: 60.0,
        aberration_px: 0.50,
        distortion: 0.04,
        vignette: 0.30,
        edge_softness_px: 0.88,
        flare_strength: 0.48,
        dirt: 0.62,
        halation_bias: 0.32,
        lens_bloom: 0.32,
        flare_ghosts: 0.32,
        anamorphic_squeeze: 1.0,
        coma: 0.32,
        cat_eye_bokeh: 0.32,
        focus_breathing: 0.32,
    },
    LensProfile2d {
        id: "old_news_camera",
        label: "Old News Camera",
        focal_length_mm: 32.0,
        aberration_px: 0.34,
        distortion: 0.08,
        vignette: 0.26,
        edge_softness_px: 0.62,
        flare_strength: 0.18,
        dirt: 0.26,
        halation_bias: 0.14,
        lens_bloom: 0.14,
        flare_ghosts: 0.14,
        anamorphic_squeeze: 1.0,
        coma: 0.14,
        cat_eye_bokeh: 0.14,
        focus_breathing: 0.14,
    },
    LensProfile2d {
        id: "documentary_16mm_gate",
        label: "Documentary 16mm Gate",
        focal_length_mm: 25.0,
        aberration_px: 0.44,
        distortion: 0.10,
        vignette: 0.36,
        edge_softness_px: 0.82,
        flare_strength: 0.30,
        dirt: 0.36,
        halation_bias: 0.20,
        lens_bloom: 0.20,
        flare_ghosts: 0.20,
        anamorphic_squeeze: 1.0,
        coma: 0.20,
        cat_eye_bokeh: 0.20,
        focus_breathing: 0.20,
    },
    LensProfile2d {
        id: "polaroid_soft_edges",
        label: "Polaroid Soft Edges",
        focal_length_mm: 38.0,
        aberration_px: 0.58,
        distortion: 0.08,
        vignette: 0.52,
        edge_softness_px: 1.55,
        flare_strength: 0.28,
        dirt: 0.20,
        halation_bias: 0.18,
        lens_bloom: 0.18,
        flare_ghosts: 0.18,
        anamorphic_squeeze: 1.0,
        coma: 0.18,
        cat_eye_bokeh: 0.18,
        focus_breathing: 0.18,
    },
    LensProfile2d {
        id: "gritty_club_lens",
        label: "Gritty Club Lens",
        focal_length_mm: 35.0,
        aberration_px: 0.60,
        distortion: 0.06,
        vignette: 0.42,
        edge_softness_px: 0.76,
        flare_strength: 0.62,
        dirt: 0.56,
        halation_bias: 0.38,
        lens_bloom: 0.38,
        flare_ghosts: 0.38,
        anamorphic_squeeze: 1.0,
        coma: 0.38,
        cat_eye_bokeh: 0.38,
        focus_breathing: 0.38,
    },
    LensProfile2d {
        id: "night_bus_window",
        label: "Night Bus Window",
        focal_length_mm: 40.0,
        aberration_px: 0.46,
        distortion: 0.04,
        vignette: 0.34,
        edge_softness_px: 0.90,
        flare_strength: 0.52,
        dirt: 0.70,
        halation_bias: 0.30,
        lens_bloom: 0.30,
        flare_ghosts: 0.30,
        anamorphic_squeeze: 1.0,
        coma: 0.30,
        cat_eye_bokeh: 0.30,
        focus_breathing: 0.30,
    },
    LensProfile2d {
        id: "lofi_phone_2007",
        label: "Lo-fi Phone 2007",
        focal_length_mm: 29.0,
        aberration_px: 0.92,
        distortion: 0.16,
        vignette: 0.50,
        edge_softness_px: 1.35,
        flare_strength: 0.16,
        dirt: 0.16,
        halation_bias: 0.08,
        lens_bloom: 0.08,
        flare_ghosts: 0.08,
        anamorphic_squeeze: 1.0,
        coma: 0.08,
        cat_eye_bokeh: 0.08,
        focus_breathing: 0.08,
    },
    LensProfile2d {
        id: "dream_glass_bloom",
        label: "Dream Glass Bloom",
        focal_length_mm: 50.0,
        aberration_px: 0.30,
        distortion: 0.02,
        vignette: 0.26,
        edge_softness_px: 1.10,
        flare_strength: 0.70,
        dirt: 0.22,
        halation_bias: 0.50,
        lens_bloom: 0.50,
        flare_ghosts: 0.50,
        anamorphic_squeeze: 1.0,
        coma: 0.50,
        cat_eye_bokeh: 0.50,
        focus_breathing: 0.50,
    },
];

pub const BUILTIN_FILM_STOCKS_2D: &[FilmStockProfile2d] = &[
    FilmStockProfile2d {
        id: "neutral_digital_400",
        label: "Neutral Digital 400",
        base_iso: 400.0,
        color_shift: 0.00,
        contrast: 1.00,
        saturation: 1.00,
        flicker: 0.00,
        vignette: 0.02,
        opacity: 0.12,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::clean_digital(),
    },
    FilmStockProfile2d {
        id: "polish_1994_push_800",
        label: "Polish 1994 Push 800",
        base_iso: 800.0,
        color_shift: 0.05,
        contrast: 1.14,
        saturation: 0.78,
        flicker: 0.14,
        vignette: 0.12,
        opacity: 0.48,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fast_color_negative(),
    },
    FilmStockProfile2d {
        id: "expired_orwo_400",
        label: "Expired ORWO 400",
        base_iso: 400.0,
        color_shift: 0.09,
        contrast: 0.92,
        saturation: 0.62,
        flicker: 0.12,
        vignette: 0.10,
        opacity: 0.42,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::dirty_scan(),
    },
    FilmStockProfile2d {
        id: "kodak_gold_200_soft",
        label: "Kodak Gold 200 Soft",
        base_iso: 200.0,
        color_shift: 0.02,
        contrast: 1.04,
        saturation: 1.12,
        flicker: 0.02,
        vignette: 0.05,
        opacity: 0.24,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::modern_color_negative(),
    },
    FilmStockProfile2d {
        id: "fuji_green_800_night",
        label: "Fuji Green 800 Night",
        base_iso: 800.0,
        color_shift: 0.07,
        contrast: 1.06,
        saturation: 0.86,
        flicker: 0.08,
        vignette: 0.09,
        opacity: 0.42,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fast_color_negative(),
    },
    FilmStockProfile2d {
        id: "ilford_delta_3200_bw",
        label: "Ilford Delta 3200 BW",
        base_iso: 3200.0,
        color_shift: 0.00,
        contrast: 1.28,
        saturation: 0.00,
        flicker: 0.04,
        vignette: 0.12,
        opacity: 0.62,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::bw_silver_pushed(),
    },
    FilmStockProfile2d {
        id: "ektachrome_cold_100",
        label: "Ektachrome Cold 100",
        base_iso: 100.0,
        color_shift: 0.04,
        contrast: 1.18,
        saturation: 1.10,
        flicker: 0.01,
        vignette: 0.04,
        opacity: 0.20,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fine_reversal(),
    },
    FilmStockProfile2d {
        id: "portra_400_warm",
        label: "Portra 400 Warm",
        base_iso: 400.0,
        color_shift: 0.02,
        contrast: 0.96,
        saturation: 1.04,
        flicker: 0.01,
        vignette: 0.04,
        opacity: 0.22,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::modern_color_negative(),
    },
    FilmStockProfile2d {
        id: "cinestill_800t_halation",
        label: "Cinestill 800T Halation",
        base_iso: 800.0,
        color_shift: 0.06,
        contrast: 1.08,
        saturation: 0.94,
        flicker: 0.04,
        vignette: 0.08,
        opacity: 0.40,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fast_color_negative(),
    },
    FilmStockProfile2d {
        id: "agfa_vista_200",
        label: "Agfa Vista 200",
        base_iso: 200.0,
        color_shift: 0.03,
        contrast: 1.08,
        saturation: 0.95,
        flicker: 0.02,
        vignette: 0.05,
        opacity: 0.26,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::modern_color_negative(),
    },
    FilmStockProfile2d {
        id: "superia_1600_under",
        label: "Superia 1600 Underexposed",
        base_iso: 1600.0,
        color_shift: 0.08,
        contrast: 1.20,
        saturation: 0.72,
        flicker: 0.10,
        vignette: 0.14,
        opacity: 0.56,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fast_color_negative(),
    },
    FilmStockProfile2d {
        id: "tri_x_400_pushed",
        label: "Tri-X 400 Pushed",
        base_iso: 400.0,
        color_shift: 0.00,
        contrast: 1.32,
        saturation: 0.00,
        flicker: 0.03,
        vignette: 0.10,
        opacity: 0.50,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::bw_silver_pushed(),
    },
    FilmStockProfile2d {
        id: "lomography_800",
        label: "Lomography 800",
        base_iso: 800.0,
        color_shift: 0.08,
        contrast: 1.05,
        saturation: 1.18,
        flicker: 0.06,
        vignette: 0.16,
        opacity: 0.44,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fast_color_negative(),
    },
    FilmStockProfile2d {
        id: "expired_polaroid_600",
        label: "Expired Polaroid 600",
        base_iso: 600.0,
        color_shift: 0.12,
        contrast: 0.86,
        saturation: 0.70,
        flicker: 0.06,
        vignette: 0.20,
        opacity: 0.46,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::dirty_scan(),
    },
    FilmStockProfile2d {
        id: "soviet_slide_64",
        label: "Soviet Slide 64",
        base_iso: 64.0,
        color_shift: 0.07,
        contrast: 1.22,
        saturation: 0.82,
        flicker: 0.03,
        vignette: 0.08,
        opacity: 0.24,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fine_reversal(),
    },
    FilmStockProfile2d {
        id: "cheap_lab_scan_2001",
        label: "Cheap Lab Scan 2001",
        base_iso: 400.0,
        color_shift: 0.10,
        contrast: 0.95,
        saturation: 0.74,
        flicker: 0.05,
        vignette: 0.07,
        opacity: 0.36,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::dirty_scan(),
    },
    FilmStockProfile2d {
        id: "newsprint_bleach_bypass",
        label: "Newsprint Bleach Bypass",
        base_iso: 800.0,
        color_shift: 0.02,
        contrast: 1.42,
        saturation: 0.42,
        flicker: 0.08,
        vignette: 0.11,
        opacity: 0.54,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::dirty_scan(),
    },
    FilmStockProfile2d {
        id: "surveillance_tape_color",
        label: "Surveillance Tape Color",
        base_iso: 1600.0,
        color_shift: 0.11,
        contrast: 1.12,
        saturation: 0.58,
        flicker: 0.20,
        vignette: 0.15,
        opacity: 0.60,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::dirty_scan(),
    },
    FilmStockProfile2d {
        id: "noir_mono_soft",
        label: "Noir Mono Soft",
        base_iso: 800.0,
        color_shift: 0.00,
        contrast: 1.16,
        saturation: 0.00,
        flicker: 0.05,
        vignette: 0.18,
        opacity: 0.44,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::bw_silver_pushed(),
    },
    FilmStockProfile2d {
        id: "rotten_neon_push_1600",
        label: "Rotten Neon Push 1600",
        base_iso: 1600.0,
        color_shift: 0.10,
        contrast: 1.24,
        saturation: 0.66,
        flicker: 0.16,
        vignette: 0.17,
        opacity: 0.62,
        toe: 0.45,
        shoulder: 0.65,
        black_lift: 0.02,
        print_fade: 0.08,
        dust: 0.08,
        scratches: 0.03,
        push_pull: 0.0,
        gate_weave: 0.02,
        scan_softness: 0.08,
        grain: FilmGrainProfile2d::fast_color_negative(),
    },
];

pub fn lens_profile_2d(id: &str) -> Option<LensProfile2d> {
    BUILTIN_LENS_PROFILES_2D
        .iter()
        .find(|profile| profile.id == id)
        .cloned()
}

pub fn film_stock_2d(id: &str) -> Option<FilmStockProfile2d> {
    BUILTIN_FILM_STOCKS_2D
        .iter()
        .find(|profile| profile.id == id)
        .cloned()
}

pub fn lens_profile_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<LensProfile2d> {
    lens_profile_2d(id_or_key).or_else(|| {
        let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
        lens_profile_2d_from_prepared(&prepared)
    })
}

pub fn film_stock_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<FilmStockProfile2d> {
    film_stock_2d(id_or_key).or_else(|| {
        let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
        film_stock_2d_from_prepared(&prepared)
    })
}

pub fn rain_glass_profile_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<RainGlass2d> {
    let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
    rain_glass_profile_2d_from_prepared(&prepared)
}

pub fn look_profile_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<ColorRamp2d> {
    let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
    look_profile_2d_from_prepared(&prepared)
}

pub fn lens_profile_2d_from_prepared(prepared: &PreparedAsset) -> Option<LensProfile2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-lens-profile-2d"
    {
        return None;
    }

    let id = metadata_string(prepared, "id")?;
    Some(LensProfile2d {
        id: Box::leak(id.into_boxed_str()),
        label: Box::leak(
            metadata_string(prepared, "label")
                .unwrap_or_else(|| "Custom Lens Profile".to_owned())
                .into_boxed_str(),
        ),
        focal_length_mm: metadata_f32(prepared, "focal_length_mm").unwrap_or(35.0),
        aberration_px: metadata_f32(prepared, "aberration_px").unwrap_or(0.0),
        distortion: metadata_f32(prepared, "distortion").unwrap_or(0.0),
        vignette: metadata_f32(prepared, "vignette").unwrap_or(0.0),
        edge_softness_px: metadata_f32(prepared, "edge_softness_px").unwrap_or(0.0),
        flare_strength: metadata_f32(prepared, "flare_strength").unwrap_or(0.0),
        dirt: metadata_f32(prepared, "dirt").unwrap_or(0.0),
        halation_bias: metadata_f32(prepared, "halation_bias").unwrap_or(0.0),
        lens_bloom: metadata_f32(prepared, "lens_bloom").unwrap_or(0.0),
        flare_ghosts: metadata_f32(prepared, "flare_ghosts").unwrap_or(0.0),
        anamorphic_squeeze: metadata_f32(prepared, "anamorphic_squeeze").unwrap_or(1.0),
        coma: metadata_f32(prepared, "coma").unwrap_or(0.0),
        cat_eye_bokeh: metadata_f32(prepared, "cat_eye_bokeh").unwrap_or(0.0),
        focus_breathing: metadata_f32(prepared, "focus_breathing").unwrap_or(0.0),
    })
}

pub fn film_stock_2d_from_prepared(prepared: &PreparedAsset) -> Option<FilmStockProfile2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-film-stock-2d"
    {
        return None;
    }

    let id = metadata_string(prepared, "id")?;
    Some(FilmStockProfile2d {
        id: Box::leak(id.into_boxed_str()),
        label: Box::leak(
            metadata_string(prepared, "label")
                .unwrap_or_else(|| "Custom Film Stock".to_owned())
                .into_boxed_str(),
        ),
        base_iso: metadata_f32(prepared, "base_iso").unwrap_or(400.0),
        color_shift: metadata_f32(prepared, "color_shift").unwrap_or(0.0),
        contrast: metadata_f32(prepared, "contrast").unwrap_or(1.0),
        saturation: metadata_f32(prepared, "saturation").unwrap_or(1.0),
        flicker: metadata_f32(prepared, "flicker").unwrap_or(0.0),
        vignette: metadata_f32(prepared, "vignette").unwrap_or(0.0),
        opacity: metadata_f32(prepared, "opacity").unwrap_or(0.25),
        toe: metadata_f32(prepared, "toe").unwrap_or(0.45),
        shoulder: metadata_f32(prepared, "shoulder").unwrap_or(0.65),
        black_lift: metadata_f32(prepared, "black_lift").unwrap_or(0.02),
        print_fade: metadata_f32(prepared, "print_fade").unwrap_or(0.08),
        dust: metadata_f32(prepared, "dust").unwrap_or(0.0),
        scratches: metadata_f32(prepared, "scratches").unwrap_or(0.0),
        push_pull: metadata_f32(prepared, "push_pull").unwrap_or(0.0),
        gate_weave: metadata_f32(prepared, "gate_weave").unwrap_or(0.0),
        scan_softness: metadata_f32(prepared, "scan_softness").unwrap_or(0.0),
        grain: film_grain_profile_2d_from_prepared(prepared, "grain"),
    })
}

fn film_grain_profile_2d_from_prepared(
    prepared: &PreparedAsset,
    prefix: &str,
) -> FilmGrainProfile2d {
    let model = metadata_string(prepared, &format!("{prefix}.model"))
        .unwrap_or_else(|| "modern_color_negative".to_owned())
        .to_ascii_lowercase();
    let mut grain = match model.as_str() {
        "clean" | "clean_digital" | "digital" => FilmGrainProfile2d::clean_digital(),
        "fast" | "fast_color" | "fast_color_negative" | "portra_800" | "vision3_500t" => {
            FilmGrainProfile2d::fast_color_negative()
        }
        "bw" | "b&w" | "silver" | "silver_halide" | "bw_silver_pushed" | "tri_x" | "hp5" => {
            FilmGrainProfile2d::bw_silver_pushed()
        }
        "reversal" | "slide" | "ektachrome" | "fine_reversal" => {
            FilmGrainProfile2d::fine_reversal()
        }
        "dirty" | "expired" | "dirty_scan" | "lab_scan" => FilmGrainProfile2d::dirty_scan(),
        _ => FilmGrainProfile2d::modern_color_negative(),
    };

    grain.luma_amount = metadata_f32(prepared, &format!("{prefix}.luma_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.luma")))
        .unwrap_or(grain.luma_amount);
    grain.chroma_amount = metadata_f32(prepared, &format!("{prefix}.chroma_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.chroma")))
        .unwrap_or(grain.chroma_amount);
    grain.shadow_amount = metadata_f32(prepared, &format!("{prefix}.shadow_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.shadows")))
        .unwrap_or(grain.shadow_amount);
    grain.midtone_amount = metadata_f32(prepared, &format!("{prefix}.midtone_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.midtones")))
        .unwrap_or(grain.midtone_amount);
    grain.highlight_amount = metadata_f32(prepared, &format!("{prefix}.highlight_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.highlights")))
        .unwrap_or(grain.highlight_amount);
    grain.highlight_suppression =
        metadata_f32(prepared, &format!("{prefix}.highlight_suppression"))
            .unwrap_or(grain.highlight_suppression);
    grain.fine_grain_px = metadata_f32(prepared, &format!("{prefix}.fine_grain_px"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.fine_px")))
        .unwrap_or(grain.fine_grain_px);
    grain.medium_grain_px = metadata_f32(prepared, &format!("{prefix}.medium_grain_px"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.medium_px")))
        .unwrap_or(grain.medium_grain_px);
    grain.coarse_grain_px = metadata_f32(prepared, &format!("{prefix}.coarse_grain_px"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.coarse_px")))
        .unwrap_or(grain.coarse_grain_px);
    grain.clumpiness =
        metadata_f32(prepared, &format!("{prefix}.clumpiness")).unwrap_or(grain.clumpiness);
    grain.softness =
        metadata_f32(prepared, &format!("{prefix}.softness")).unwrap_or(grain.softness);
    grain.underexposure_boost = metadata_f32(prepared, &format!("{prefix}.underexposure_boost"))
        .unwrap_or(grain.underexposure_boost);
    grain.push_process_boost = metadata_f32(prepared, &format!("{prefix}.push_process_boost"))
        .unwrap_or(grain.push_process_boost);
    grain.density_pivot =
        metadata_f32(prepared, &format!("{prefix}.density_pivot")).unwrap_or(grain.density_pivot);
    grain.channel_balance[0] =
        metadata_f32(prepared, &format!("{prefix}.channel_r")).unwrap_or(grain.channel_balance[0]);
    grain.channel_balance[1] =
        metadata_f32(prepared, &format!("{prefix}.channel_g")).unwrap_or(grain.channel_balance[1]);
    grain.channel_balance[2] =
        metadata_f32(prepared, &format!("{prefix}.channel_b")).unwrap_or(grain.channel_balance[2]);
    grain.temporal_jitter = metadata_f32(prepared, &format!("{prefix}.temporal_jitter"))
        .unwrap_or(grain.temporal_jitter);
    grain.regenerate_per_frame = metadata_bool(prepared, &format!("{prefix}.regenerate_per_frame"))
        .or_else(|| metadata_bool(prepared, &format!("{prefix}.per_frame")))
        .or_else(|| metadata_bool(prepared, &format!("{prefix}.animated")))
        .unwrap_or(grain.regenerate_per_frame);

    grain
}

pub fn rain_glass_profile_2d_from_prepared(prepared: &PreparedAsset) -> Option<RainGlass2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-rain-glass-profile-2d"
    {
        return None;
    }

    let mut rain = RainGlass2d::default();

    if let Some(value) = metadata_bool(prepared, "spawn.enabled") {
        rain.enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "spawn.spawn_rate") {
        rain.spawn_rate = value;
    }
    if let Some(value) = metadata_u32(prepared, "spawn.spawn_limit") {
        rain.spawn_limit = value;
    }
    if let Some(value) = metadata_u32(prepared, "spawn.seed") {
        rain.seed = value;
    }

    if let Some(value) = metadata_f32(prepared, "droplets.min_radius_px") {
        rain.min_radius_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.max_radius_px") {
        rain.max_radius_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.gravity_px_per_sec2") {
        rain.gravity_px_per_sec2 = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.slip_rate") {
        rain.slip_rate = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.initial_spread") {
        rain.initial_spread = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.shrink_rate") {
        rain.shrink_rate = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.evaporate") {
        rain.evaporate = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.velocity_spread") {
        rain.velocity_spread = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.motion_interval_min") {
        rain.motion_interval_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.motion_interval_max") {
        rain.motion_interval_max = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.x_shift_min") {
        rain.x_shift_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.x_shift_max") {
        rain.x_shift_max = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.collider_scale") {
        rain.collider_scale = value;
    }

    if let Some(value) = metadata_bool(prepared, "trails.enabled") {
        rain.trails_enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_drop_density") {
        rain.trail_drop_density = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_drop_size_min") {
        rain.trail_drop_size_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_drop_size_max") {
        rain.trail_drop_size_max = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_distance_min_px") {
        rain.trail_distance_min_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_distance_max_px") {
        rain.trail_distance_max_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_spread") {
        rain.trail_spread = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_shrink_rate") {
        rain.trail_shrink_rate = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_evaporate") {
        rain.trail_evaporate = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_taper") {
        rain.trail_taper = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_refract_scale") {
        rain.trail_refract_scale = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_opacity") {
        rain.trail_opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.streak_boost") {
        rain.streak_boost = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.streak_length") {
        rain.streak_length = value;
    }

    if let Some(value) = metadata_bool(prepared, "micro_droplets.enabled") {
        rain.micro_droplets_enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "micro_droplets.micro_droplets_per_second") {
        rain.micro_droplets_per_second = value;
    }
    if let Some(value) = metadata_f32(prepared, "micro_droplets.micro_droplet_min_px") {
        rain.micro_droplet_min_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "micro_droplets.micro_droplet_max_px") {
        rain.micro_droplet_max_px = value;
    }

    if let Some(value) = metadata_bool(prepared, "mist.enabled") {
        rain.mist_enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_opacity") {
        rain.mist_opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_blur_px") {
        rain.mist_blur_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_accumulation") {
        rain.mist_accumulation = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_time") {
        rain.mist_time = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_color_strength") {
        rain.mist_color_strength = value;
    }
    if let Some(value) = metadata_u32(prepared, "mist.mist_blur_step") {
        rain.mist_blur_step = value;
    }

    if let Some(value) = metadata_f32(prepared, "optics.refract_base") {
        rain.refract_base = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.refract_scale") {
        rain.refract_scale = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.distortion_px") {
        rain.distortion_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.normal_strength") {
        rain.normal_strength = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.chromatic_aberration") {
        rain.chromatic_aberration = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.focus_blur_strength") {
        rain.focus_blur_strength = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.background_blur_px") {
        rain.background_blur_px = value;
    }
    if let Some(value) = metadata_u32(prepared, "optics.background_blur_steps") {
        rain.background_blur_steps = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.smooth_edge_min") {
        rain.smooth_edge_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.smooth_edge_max") {
        rain.smooth_edge_max = value;
    }

    if let Some(value) = metadata_f32(prepared, "compose.opacity") {
        rain.opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.body_opacity") {
        rain.body_opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.scene_blend") {
        rain.scene_blend = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.scene_darken") {
        rain.scene_darken = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.drop_plane_blur_px") {
        rain.drop_plane_blur_px = value;
    }
    if let Some(value) = metadata_bool(prepared, "compose.reference_mode") {
        rain.reference_mode = value;
    }
    if let Some(value) = metadata_string(prepared, "compose.raindrop_compose") {
        rain.raindrop_compose = parse_rain_glass_raindrop_compose(&value);
    }
    if let Some(value) = metadata_vec2(prepared, "compose.raindrop_eraser_size") {
        rain.raindrop_eraser_size = value;
    }

    if let Some(value) = metadata_bool(prepared, "lighting.receives_scene_light") {
        rain.receives_scene_light = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.scene_light_response") {
        rain.scene_light_response = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.scene_light_tint_strength") {
        rain.scene_light_tint_strength = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.scene_shadow_floor") {
        rain.scene_shadow_floor = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.rim_strength") {
        rain.rim_strength = value;
    }
    if let Some(value) = metadata_vec4(prepared, "lighting.light_pos") {
        rain.light_pos = value;
    }
    if let Some(value) = metadata_vec3(prepared, "lighting.diffuse_light") {
        rain.diffuse_light = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.shadow_offset") {
        rain.shadow_offset = value;
    }
    if let Some(value) = metadata_vec3(prepared, "lighting.specular_light") {
        rain.specular_light = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.specular_shininess") {
        rain.specular_shininess = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.light_bump") {
        rain.light_bump = value;
    }

    if let Some(value) = metadata_vec3(prepared, "contamination.blood_tint") {
        rain.blood_tint = value;
    }
    if let Some(value) = metadata_f32(prepared, "contamination.blood_amount") {
        rain.blood_amount = value;
    }

    if let Some(value) = metadata_string(prepared, "debug.view") {
        rain.debug_view = parse_rain_glass_debug_view(Some(&value));
    }

    Some(rain.normalized())
}

pub fn look_profile_2d_from_prepared(prepared: &PreparedAsset) -> Option<ColorRamp2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-look-profile-2d"
    {
        return None;
    }

    let _id = metadata_string(prepared, "id")?;
    Some(
        ColorRamp2d {
            palette_size: metadata_u32(prepared, "palette_size")
                .or_else(|| metadata_u32(prepared, "colors"))
                .unwrap_or(32),
            dither_strength: metadata_f32(prepared, "dither_strength")
                .or_else(|| metadata_f32(prepared, "dither"))
                .unwrap_or(0.12),
            dither_scale: metadata_f32(prepared, "dither_scale")
                .or_else(|| metadata_f32(prepared, "scale"))
                .unwrap_or(1.0),
            layered_dither: metadata_f32(prepared, "layered_dither")
                .or_else(|| metadata_f32(prepared, "layered"))
                .unwrap_or(0.22),
            opacity: metadata_f32(prepared, "opacity").unwrap_or(1.0),
            luma_preserve: metadata_f32(prepared, "luma_preserve")
                .or_else(|| metadata_f32(prepared, "luma"))
                .unwrap_or(0.55),
            highlight_bias: metadata_f32(prepared, "highlight_bias")
                .or_else(|| metadata_f32(prepared, "highlight"))
                .or_else(|| metadata_f32(prepared, "light_bias"))
                .unwrap_or(0.0),
            shadow_bias: metadata_f32(prepared, "shadow_bias")
                .or_else(|| metadata_f32(prepared, "shadow"))
                .unwrap_or(0.0),
            contrast: metadata_f32(prepared, "contrast").unwrap_or(1.0),
            saturation: metadata_f32(prepared, "saturation")
                .or_else(|| metadata_f32(prepared, "sat"))
                .unwrap_or(1.0),
            gamma: metadata_f32(prepared, "gamma").unwrap_or(1.0),
            seed: metadata_u32(prepared, "seed").unwrap_or(0),
        }
        .normalized(),
    )
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Option<f32> {
    prepared.metadata.get(key)?.parse::<f32>().ok()
}

fn metadata_bool(prepared: &PreparedAsset, key: &str) -> Option<bool> {
    prepared.metadata.get(key)?.parse::<bool>().ok()
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Option<u32> {
    prepared.metadata.get(key)?.parse::<u32>().ok()
}

fn metadata_string(prepared: &PreparedAsset, key: &str) -> Option<String> {
    let value = prepared.metadata.get(key)?.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_owned())
}

fn metadata_vec2(prepared: &PreparedAsset, key: &str) -> Option<[f32; 2]> {
    Some([
        metadata_f32(prepared, &format!("{key}.0"))?,
        metadata_f32(prepared, &format!("{key}.1"))?,
    ])
}

fn metadata_vec3(prepared: &PreparedAsset, key: &str) -> Option<[f32; 3]> {
    Some([
        metadata_f32(prepared, &format!("{key}.0"))?,
        metadata_f32(prepared, &format!("{key}.1"))?,
        metadata_f32(prepared, &format!("{key}.2"))?,
    ])
}

fn metadata_vec4(prepared: &PreparedAsset, key: &str) -> Option<[f32; 4]> {
    Some([
        metadata_f32(prepared, &format!("{key}.0"))?,
        metadata_f32(prepared, &format!("{key}.1"))?,
        metadata_f32(prepared, &format!("{key}.2"))?,
        metadata_f32(prepared, &format!("{key}.3"))?,
    ])
}

fn parse_rain_glass_raindrop_compose(value: &str) -> RainGlassRaindropCompose {
    match value.trim().to_ascii_lowercase().as_str() {
        "hard" | "harder" => RainGlassRaindropCompose::Harder,
        _ => RainGlassRaindropCompose::Smoother,
    }
}

fn parse_rain_glass_debug_view(value: Option<&str>) -> RainGlassDebugView {
    match value
        .unwrap_or("final")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "scene" | "scene_input" => RainGlassDebugView::SceneInput,
        "blur" | "blurred" | "blurred_scene" => RainGlassDebugView::BlurredScene,
        "raindrop_map" | "raindrops" => RainGlassDebugView::RaindropMap,
        "droplet_map" | "droplets" => RainGlassDebugView::DropletMap,
        "trail_map" | "trails" => RainGlassDebugView::TrailMap,
        "drop_normals" | "normals" => RainGlassDebugView::DropNormals,
        "drop_mask" | "mask" => RainGlassDebugView::DropMask,
        "mist" => RainGlassDebugView::Mist,
        "refraction" => RainGlassDebugView::Refraction,
        _ => RainGlassDebugView::Final,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn has_exactly_twenty_lens_profiles() {
        assert_eq!(BUILTIN_LENS_PROFILES_2D.len(), 20);
    }

    #[test]
    fn has_exactly_twenty_film_stocks() {
        assert_eq!(BUILTIN_FILM_STOCKS_2D.len(), 20);
    }

    #[test]
    fn resolves_default_profiles() {
        assert!(lens_profile_2d("clean_modern_35mm").is_some());
        assert!(film_stock_2d("neutral_digital_400").is_some());
    }

    #[test]
    fn parses_custom_lens_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/lens/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/lens/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-lens-profile-2d".to_owned()),
            label: Some("Custom Lens".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("id".to_owned(), "custom_lens".to_owned()),
                ("label".to_owned(), "Custom Lens".to_owned()),
                ("focal_length_mm".to_owned(), "42".to_owned()),
                ("flare_strength".to_owned(), "0.5".to_owned()),
            ]),
        };

        let profile = lens_profile_2d_from_prepared(&prepared).expect("custom lens should parse");
        assert_eq!(profile.id, "custom_lens");
        assert_eq!(profile.focal_length_mm, 42.0);
        assert_eq!(profile.flare_strength, 0.5);
    }

    #[test]
    fn parses_custom_film_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/film/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/film/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-film-stock-2d".to_owned()),
            label: Some("Custom Film".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("id".to_owned(), "custom_film".to_owned()),
                ("label".to_owned(), "Custom Film".to_owned()),
                ("base_iso".to_owned(), "500".to_owned()),
                ("saturation".to_owned(), "0.8".to_owned()),
            ]),
        };

        let profile = film_stock_2d_from_prepared(&prepared).expect("custom film should parse");
        assert_eq!(profile.id, "custom_film");
        assert_eq!(profile.base_iso, 500.0);
        assert_eq!(profile.saturation, 0.8);
    }

    #[test]
    fn parses_custom_look_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/look/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/look/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-look-profile-2d".to_owned()),
            label: Some("Custom Look".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("id".to_owned(), "custom_look".to_owned()),
                ("label".to_owned(), "Custom Look".to_owned()),
                ("palette_size".to_owned(), "24".to_owned()),
                ("contrast".to_owned(), "1.12".to_owned()),
                ("shadow_bias".to_owned(), "-0.15".to_owned()),
            ]),
        };

        let profile = look_profile_2d_from_prepared(&prepared).expect("custom look should parse");
        assert_eq!(profile.palette_size, 24);
        assert_eq!(profile.contrast, 1.12);
        assert_eq!(profile.shadow_bias, 0.0);
    }

    #[test]
    fn parses_custom_rain_glass_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/rain/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/rain/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-rain-glass-profile-2d".to_owned()),
            label: Some("Custom Rain".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("spawn.spawn_rate".to_owned(), "9.5".to_owned()),
                ("droplets.min_radius_px".to_owned(), "28".to_owned()),
                ("optics.refract_scale".to_owned(), "0.9".to_owned()),
                ("mist.enabled".to_owned(), "false".to_owned()),
                ("debug.view".to_owned(), "refraction".to_owned()),
            ]),
        };

        let profile =
            rain_glass_profile_2d_from_prepared(&prepared).expect("custom rain should parse");
        assert_eq!(profile.spawn_rate, 9.5);
        assert_eq!(profile.min_radius_px, 28.0);
        assert_eq!(profile.refract_scale, 0.9);
        assert!(!profile.mist_enabled);
        assert_eq!(profile.debug_view, RainGlassDebugView::Refraction);
    }
}
