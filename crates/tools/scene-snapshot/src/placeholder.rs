use crate::{SceneSnapshotError, SceneSnapshotImage, SceneSnapshotRequest, SceneSnapshotService};

#[derive(Clone, Debug, Default)]
pub struct PlaceholderSceneSnapshotService;

impl SceneSnapshotService for PlaceholderSceneSnapshotService {
    fn capture(
        &self,
        request: SceneSnapshotRequest,
    ) -> Result<SceneSnapshotImage, SceneSnapshotError> {
        if request.width == 0 || request.height == 0 {
            return Err(SceneSnapshotError::new("Snapshot dimensions must be non-zero"));
        }

        let mut pixels = vec![0; request.width as usize * request.height as usize * 4];
        fill(&mut pixels, request.width, request.height, [248, 249, 250, 255]);
        draw_grid(&mut pixels, request.width, request.height, 48, [232, 238, 242, 255]);

        let seed = hash(&format!("{}:{}", request.mod_id, request.scene_id));
        for index in 0..9 {
            let local = seed.wrapping_add(index as u64 * 0x9e37_79b9);
            let w = 52 + (local as u32 % 120);
            let h = 32 + ((local >> 8) as u32 % 76);
            let max_x = request.width.saturating_sub(w + 16).max(1);
            let max_y = request.height.saturating_sub(h + 16).max(1);
            let x = 16 + ((local >> 16) as u32 % max_x);
            let y = 42 + ((local >> 28) as u32 % max_y.saturating_sub(24).max(1));
            let color = match index % 4 {
                0 => [36, 123, 160, 205],
                1 => [112, 193, 179, 200],
                2 => [255, 22, 84, 190],
                _ => [44, 62, 80, 175],
            };
            draw_rect(&mut pixels, request.width, request.height, x, y, w, h, color);
        }

        Ok(SceneSnapshotImage {
            width: request.width,
            height: request.height,
            diagnostic_label: format!(
                "placeholder scene snapshot: {} / {}",
                request.mod_id, request.scene_id
            ),
            request,
            pixels_rgba8: pixels,
        })
    }
}

fn fill(pixels: &mut [u8], width: u32, height: u32, color: [u8; 4]) {
    for y in 0..height {
        for x in 0..width {
            set_pixel(pixels, width, x, y, color);
        }
    }
}

fn draw_grid(pixels: &mut [u8], width: u32, height: u32, step: u32, color: [u8; 4]) {
    if step == 0 {
        return;
    }
    let mut x = 0;
    while x < width {
        for y in 0..height {
            set_pixel(pixels, width, x, y, color);
        }
        x += step;
    }
    let mut y = 0;
    while y < height {
        for x in 0..width {
            set_pixel(pixels, width, x, y, color);
        }
        y += step;
    }
}

fn draw_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: [u8; 4],
) {
    let x1 = x.saturating_add(w).min(width);
    let y1 = y.saturating_add(h).min(height);
    for py in y..y1 {
        for px in x..x1 {
            blend_pixel(pixels, width, px, py, color);
        }
    }
}

fn set_pixel(pixels: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 4]) {
    let offset = (y as usize * width as usize + x as usize) * 4;
    if offset + 3 >= pixels.len() {
        return;
    }
    pixels[offset] = color[0];
    pixels[offset + 1] = color[1];
    pixels[offset + 2] = color[2];
    pixels[offset + 3] = color[3];
}

fn blend_pixel(pixels: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 4]) {
    let offset = (y as usize * width as usize + x as usize) * 4;
    if offset + 3 >= pixels.len() {
        return;
    }
    let alpha = color[3] as f32 / 255.0;
    pixels[offset] = blend_channel(pixels[offset], color[0], alpha);
    pixels[offset + 1] = blend_channel(pixels[offset + 1], color[1], alpha);
    pixels[offset + 2] = blend_channel(pixels[offset + 2], color[2], alpha);
    pixels[offset + 3] = 255;
}

fn blend_channel(dst: u8, src: u8, alpha: f32) -> u8 {
    (dst as f32 * (1.0 - alpha) + src as f32 * alpha).round() as u8
}

fn hash(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}
