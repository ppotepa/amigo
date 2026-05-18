use super::*;

pub(super) fn translated_transform2(transform: Transform2, offset: Vec2) -> Transform2 {
    Transform2 {
        translation: Vec2::new(
            transform.translation.x + offset.x,
            transform.translation.y + offset.y,
        ),
        ..transform
    }
}

pub(super) fn color_with_alpha_mul(color: ColorRgba, alpha: f32) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, color.a * alpha.clamp(0.0, 1.0))
}

pub(super) fn text2d_effect_offsets(radius: f32) -> [(f32, f32); 8] {
    [
        (-radius, 0.0),
        (radius, 0.0),
        (0.0, -radius),
        (0.0, radius),
        (-radius, -radius),
        (radius, -radius),
        (-radius, radius),
        (radius, radius),
    ]
}
