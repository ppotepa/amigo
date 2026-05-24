use crate::{
    math::clamp01,
    pipeline::{ContourKind, Mark, RenderFrame},
};
use glam::Vec2;
use image::{Rgba, RgbaImage};
use std::{fs, path::Path};

pub fn save_png(frame: &RenderFrame, path: &Path) -> anyhow::Result<()> {
    let image = rasterize_frame(frame, frame.width, frame.height);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    image.save(path)?;
    Ok(())
}

pub fn save_atlas(frames: &[(&str, RenderFrame)], path: &Path) -> anyhow::Result<()> {
    if frames.is_empty() {
        anyhow::bail!("no frames for atlas");
    }
    let cell_w = 600;
    let cell_h = 340;
    let cols = 3u32;
    let rows = (frames.len() as u32).div_ceil(cols);
    let mut atlas = RgbaImage::from_pixel(cols * cell_w, rows * cell_h, Rgba([246, 242, 232, 255]));
    for (i, (label, frame)) in frames.iter().enumerate() {
        let src = rasterize_frame(frame, cell_w, cell_h);
        let ox = i as u32 % cols * cell_w;
        let oy = i as u32 / cols * cell_h;
        overlay(&mut atlas, &src, ox, oy);
        draw_label(&mut atlas, ox + 14, oy + 14, label);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    atlas.save(path)?;
    Ok(())
}

fn rasterize_frame(frame: &RenderFrame, width: u32, height: u32) -> RgbaImage {
    let bg = rgba8(frame.paper, 1.0);
    let mut img = RgbaImage::from_pixel(width.max(1), height.max(1), bg);
    let sx = width as f32 / frame.width.max(1) as f32;
    let sy = height as f32 / frame.height.max(1) as f32;
    for region in &frame.paint_regions {
        if region.points.len() >= 3 {
            for i in 1..region.points.len() - 1 {
                fill_tri(
                    &mut img,
                    scale(region.points[0], sx, sy),
                    scale(region.points[i], sx, sy),
                    scale(region.points[i + 1], sx, sy),
                    region.color,
                    region.alpha,
                );
            }
        }
    }
    for contour in &frame.contours {
        let color = match contour.kind {
            ContourKind::Contour => [0.09, 0.07, 0.04, 1.0],
            ContourKind::Crease => [0.17, 0.13, 0.09, 1.0],
            ContourKind::Suggestive => [0.24, 0.21, 0.18, 1.0],
            ContourKind::Hidden => [0.40, 0.35, 0.30, 1.0],
        };
        draw_line(
            &mut img,
            scale(contour.a, sx, sy),
            scale(contour.b, sx, sy),
            1.4,
            color,
            if contour.visible { 0.86 } else { 0.26 },
        );
    }
    for mark in &frame.marks {
        match mark {
            Mark::Line {
                pts,
                color,
                width,
                alpha,
            } => {
                for pair in pts.windows(2) {
                    draw_line(
                        &mut img,
                        scale(pair[0], sx, sy),
                        scale(pair[1], sx, sy),
                        *width,
                        *color,
                        *alpha,
                    );
                }
            }
            Mark::Dot {
                center,
                radius,
                color,
                alpha,
            } => fill_circle(
                &mut img,
                scale(*center, sx, sy),
                radius * sx.min(sy),
                *color,
                *alpha,
            ),
        }
    }
    img
}

fn scale(p: Vec2, sx: f32, sy: f32) -> Vec2 {
    Vec2::new(p.x * sx, p.y * sy)
}

fn rgba8(color: [f32; 4], alpha_mul: f32) -> Rgba<u8> {
    Rgba([
        (clamp01(color[0]) * 255.0) as u8,
        (clamp01(color[1]) * 255.0) as u8,
        (clamp01(color[2]) * 255.0) as u8,
        (clamp01(color[3] * alpha_mul) * 255.0) as u8,
    ])
}

fn blend(dst: &mut Rgba<u8>, src: Rgba<u8>) {
    let a = src[3] as f32 / 255.0;
    let inv = 1.0 - a;
    dst[0] = (src[0] as f32 * a + dst[0] as f32 * inv) as u8;
    dst[1] = (src[1] as f32 * a + dst[1] as f32 * inv) as u8;
    dst[2] = (src[2] as f32 * a + dst[2] as f32 * inv) as u8;
    dst[3] = 255;
}

fn fill_tri(img: &mut RgbaImage, a: Vec2, b: Vec2, c: Vec2, color: [f32; 4], alpha: f32) {
    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as u32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(img.width() as f32 - 1.0) as u32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as u32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min(img.height() as f32 - 1.0) as u32;
    let den = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if den.abs() < 1.0e-6 {
        return;
    }
    let src = rgba8(color, alpha);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let u = ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / den;
            let v = ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / den;
            let w = 1.0 - u - v;
            if u >= -0.002 && v >= -0.002 && w >= -0.002 {
                blend(img.get_pixel_mut(x, y), src);
            }
        }
    }
}

fn draw_line(img: &mut RgbaImage, a: Vec2, b: Vec2, width: f32, color: [f32; 4], alpha: f32) {
    let steps = a.distance(b).ceil().max(1.0) as usize;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        fill_circle(img, a.lerp(b, t), width.max(0.5) * 0.5, color, alpha);
    }
}

fn fill_circle(img: &mut RgbaImage, center: Vec2, radius: f32, color: [f32; 4], alpha: f32) {
    let r = radius.max(0.5);
    let min_x = (center.x - r).floor().max(0.0) as u32;
    let max_x = (center.x + r).ceil().min(img.width() as f32 - 1.0) as u32;
    let min_y = (center.y - r).floor().max(0.0) as u32;
    let max_y = (center.y + r).ceil().min(img.height() as f32 - 1.0) as u32;
    let src = rgba8(color, alpha);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            if p.distance_squared(center) <= r * r {
                blend(img.get_pixel_mut(x, y), src);
            }
        }
    }
}

fn overlay(dst: &mut RgbaImage, src: &RgbaImage, ox: u32, oy: u32) {
    for y in 0..src.height() {
        for x in 0..src.width() {
            if ox + x < dst.width() && oy + y < dst.height() {
                *dst.get_pixel_mut(ox + x, oy + y) = *src.get_pixel(x, y);
            }
        }
    }
}

fn draw_label(img: &mut RgbaImage, x: u32, y: u32, text: &str) {
    let w = (text.len() as u32 * 7 + 22).min(280);
    for yy in y..(y + 28).min(img.height()) {
        for xx in x..(x + w).min(img.width()) {
            blend(img.get_pixel_mut(xx, yy), Rgba([246, 242, 232, 225]));
        }
    }
    // Lightweight marker blocks instead of bundling a font rasterizer.
    for (i, byte) in text.bytes().take(32).enumerate() {
        let bx = x + 10 + i as u32 * 7;
        let h = 5 + (byte % 13) as u32;
        for yy in (y + 20 - h)..(y + 20) {
            for xx in bx..(bx + 4) {
                if xx < img.width() && yy < img.height() {
                    *img.get_pixel_mut(xx, yy) = Rgba([23, 17, 11, 210]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mesh::Mesh, pipeline::compute_frame, state::AppState};

    fn sample_frame() -> RenderFrame {
        let mesh =
            Mesh::from_obj_text("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n", "export-tri").unwrap();
        compute_frame(&mesh, &AppState::default(), 96, 64)
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "char_3d_rust_impl_{}_{}_{}.png",
            name,
            std::process::id(),
            stamp
        ))
    }

    fn temp_nested_path(name: &str) -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "char_3d_rust_impl_nested_{}_{}",
                std::process::id(),
                stamp
            ))
            .join(format!("{name}.png"))
    }

    #[test]
    fn png_export_writes_non_empty_file() {
        let path = temp_path("frame");
        let _ = std::fs::remove_file(&path);

        save_png(&sample_frame(), &path).unwrap();

        let len = std::fs::metadata(&path).unwrap().len();
        let _ = std::fs::remove_file(&path);
        assert!(len > 0);
    }

    #[test]
    fn atlas_export_writes_non_empty_file() {
        let path = temp_path("atlas");
        let _ = std::fs::remove_file(&path);
        let frame = sample_frame();

        save_atlas(&[("sample", frame)], &path).unwrap();

        let len = std::fs::metadata(&path).unwrap().len();
        let _ = std::fs::remove_file(&path);
        assert!(len > 0);
    }

    #[test]
    fn png_export_creates_parent_directory() {
        let path = temp_nested_path("frame");
        let parent = path.parent().unwrap().to_owned();
        let _ = std::fs::remove_dir_all(&parent);

        save_png(&sample_frame(), &path).unwrap();

        let len = std::fs::metadata(&path).unwrap().len();
        let _ = std::fs::remove_dir_all(&parent);
        assert!(len > 0);
    }
}
