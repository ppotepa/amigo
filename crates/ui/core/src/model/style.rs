use amigo_math::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub struct UiStyle {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: f32,
    pub gap: f32,
    pub background: Option<ColorRgba>,
    pub color: Option<ColorRgba>,
    pub border_color: Option<ColorRgba>,
    pub opacity: Option<f32>,
    pub border_width: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub word_wrap: bool,
    pub fit_to_width: bool,
    pub align: UiTextAlign,
    pub blend: Option<UiBlendMode>,
    pub text_shadow: Option<UiTextShadow>,
    pub text_outline: Option<UiTextOutline>,
    pub text_glow: Option<UiTextGlow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextAlign {
    Start,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiBlendMode {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTextShadow {
    pub color: ColorRgba,
    pub offset: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTextOutline {
    pub color: ColorRgba,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTextGlow {
    pub color: ColorRgba,
    pub radius: f32,
    pub intensity: f32,
    pub passes: u8,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
            padding: 0.0,
            gap: 0.0,
            background: None,
            color: None,
            border_color: None,
            opacity: None,
            border_width: 0.0,
            border_radius: 0.0,
            font_size: 16.0,
            word_wrap: false,
            fit_to_width: false,
            align: UiTextAlign::Start,
            blend: None,
            text_shadow: None,
            text_outline: None,
            text_glow: None,
        }
    }
}


