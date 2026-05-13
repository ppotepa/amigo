use amigo_2d_post_fx::PostFxEmbossEdges2d;
use image::RgbaImage;

pub(crate) fn apply_emboss_edges(source: RgbaImage, emboss: PostFxEmbossEdges2d) -> RgbaImage {
    let emboss = emboss.normalized();
    let (width, height) = source.dimensions();
    if width == 0 || height == 0 || !emboss.is_active() {
        return source;
    }

    let mut output = RgbaImage::new(width, height);
    let offset = emboss.sample_offset_px.round().max(1.0) as i32;

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let sample = |sx: i32, sy: i32| -> [u8; 4] {
                let px = sx.clamp(0, width as i32 - 1) as u32;
                let py = sy.clamp(0, height as i32 - 1) as u32;
                source.get_pixel(px, py).0
            };

            let center = sample(x, y);
            let a = center[3] as f32 / 255.0;
            if a <= 0.0 {
                output.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 0]));
                continue;
            }

            let hi = sample(x + offset, y + offset);
            let lo = sample(x - offset, y - offset);
            let luma = |p: [u8; 4]| -> f32 {
                let r = p[0] as f32 / 255.0;
                let g = p[1] as f32 / 255.0;
                let b = p[2] as f32 / 255.0;
                0.2126 * r + 0.7152 * g + 0.0722 * b
            };
            let center_luma = luma(center);
            let diff = (luma(hi) - luma(lo)) * emboss.edge_strength;
            let edge = diff.abs().clamp(0.0, 1.0);
            let highlight = diff.max(0.0).clamp(0.0, 1.0);
            let light_gate = ((center_luma - emboss.luma_threshold).max(0.0)
                / (1.0 - emboss.luma_threshold).max(0.0001))
            .powf(emboss.luma_gamma);
            let local_light = local_light_proximity(
                &sample,
                &luma,
                x,
                y,
                width as i32,
                height as i32,
                emboss.specular_radius_px,
                emboss.distance_falloff,
            );
            let specular = (local_light * (0.35 + edge * 0.65)).clamp(0.0, 1.0);
            let signal = (edge * 0.55 + highlight * 0.25 + specular * 0.45)
                * light_gate
                * emboss.intensity.clamp(0.0, 2.0);
            let value = signal.clamp(0.0, 1.0);
            let alpha = (value * a).clamp(0.0, 1.0);
            let r = (value * emboss.tint[0]).clamp(0.0, 1.0);
            let g = (value * emboss.tint[1]).clamp(0.0, 1.0);
            let b = (value * emboss.tint[2]).clamp(0.0, 1.0);

            output.put_pixel(
                x as u32,
                y as u32,
                image::Rgba([
                    (r * 255.0).round() as u8,
                    (g * 255.0).round() as u8,
                    (b * 255.0).round() as u8,
                    (alpha * 255.0).round() as u8,
                ]),
            );
        }
    }

    output
}

fn local_light_proximity(
    sample: &impl Fn(i32, i32) -> [u8; 4],
    luma: &impl Fn([u8; 4]) -> f32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    radius_px: f32,
    distance_falloff: f32,
) -> f32 {
    let radius = radius_px.round().max(1.0) as i32;
    let mut best = 0.0_f32;

    for oy in -radius..=radius {
        for ox in -radius..=radius {
            let sx = (x + ox).clamp(0, width - 1);
            let sy = (y + oy).clamp(0, height - 1);
            let dist2 = (ox * ox + oy * oy) as f32;
            let falloff = 1.0 / (1.0 + dist2 * distance_falloff.max(0.0001));
            let lum = luma(sample(sx, sy));
            let weighted = lum * falloff;
            if weighted > best {
                best = weighted;
            }
        }
    }

    best.clamp(0.0, 1.0)
}

