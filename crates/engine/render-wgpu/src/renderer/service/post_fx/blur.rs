use amigo_2d_post_fx::PostFxBlur2d;
use image::RgbaImage;

pub(crate) fn apply_blur(source: RgbaImage, blur: PostFxBlur2d) -> RgbaImage {
    let blur = blur.normalized();
    let (source_width, source_height) = source.dimensions();
    if source_width == 0 || source_height == 0 || !blur.is_active() {
        return source;
    }

    let downsample = blur.downsample.clamp(0.125, 1.0);
    let work_width = ((source_width as f32 * downsample).round() as u32).max(1);
    let work_height = ((source_height as f32 * downsample).round() as u32).max(1);
    let work_source = if work_width == source_width && work_height == source_height {
        source
    } else {
        image::imageops::resize(
            &source,
            work_width,
            work_height,
            image::imageops::FilterType::Triangle,
        )
    };

    let width = work_width as usize;
    let height = work_height as usize;
    let pixels = extract_lightmap_pixels(&work_source);
    let radius = ((blur.radius * downsample).round() as usize).clamp(1, 96);
    let sigma = (radius as f32 / 2.5).max(0.75);
    let blurred = gaussian_blur_rgba(&pixels, width, height, radius, sigma);
    write_lightmap_pixels(width as u32, height as u32, &blurred, blur.intensity)
}

fn extract_lightmap_pixels(source: &RgbaImage) -> Vec<[f32; 4]> {
    source
        .pixels()
        .map(|pixel| {
            let [r, g, b, a] = pixel.0;
            let a = a as f32 / 255.0;
            let r = r as f32 / 255.0;
            let g = g as f32 / 255.0;
            let b = b as f32 / 255.0;
            let light = r.max(g).max(b) * a;
            if light < 0.018 {
                [0.0, 0.0, 0.0, 0.0]
            } else {
                [r * a, g * a, b * a, light]
            }
        })
        .collect()
}

fn gaussian_blur_rgba(
    source: &[[f32; 4]],
    width: usize,
    height: usize,
    radius: usize,
    sigma: f32,
) -> Vec<[f32; 4]> {
    let kernel = gaussian_kernel(radius, sigma);
    let mut temp = vec![[0.0; 4]; source.len()];
    let mut output = vec![[0.0; 4]; source.len()];

    for y in 0..height {
        for x in 0..width {
            let out = y * width + x;
            for (offset, weight) in kernel.iter().enumerate() {
                let sample_x = (x as isize + offset as isize - radius as isize)
                    .clamp(0, width.saturating_sub(1) as isize)
                    as usize;
                let sample = source[y * width + sample_x];
                for channel in 0..4 {
                    temp[out][channel] += sample[channel] * weight;
                }
            }
        }
    }

    for y in 0..height {
        for x in 0..width {
            let out = y * width + x;
            for (offset, weight) in kernel.iter().enumerate() {
                let sample_y = (y as isize + offset as isize - radius as isize)
                    .clamp(0, height.saturating_sub(1) as isize)
                    as usize;
                let sample = temp[sample_y * width + x];
                for channel in 0..4 {
                    output[out][channel] += sample[channel] * weight;
                }
            }
        }
    }

    output
}

fn gaussian_kernel(radius: usize, sigma: f32) -> Vec<f32> {
    let mut kernel = Vec::with_capacity(radius * 2 + 1);
    let mut sum = 0.0;
    let sigma2 = 2.0 * sigma * sigma;
    for index in 0..=(radius * 2) {
        let x = index as f32 - radius as f32;
        let value = (-x * x / sigma2).exp();
        kernel.push(value);
        sum += value;
    }
    if sum > 0.0 {
        for value in &mut kernel {
            *value /= sum;
        }
    }
    kernel
}

fn write_lightmap_pixels(
    width: u32,
    height: u32,
    pixels: &[[f32; 4]],
    intensity: f32,
) -> RgbaImage {
    let mut image = RgbaImage::new(width, height);
    for (pixel, source) in image.pixels_mut().zip(pixels.iter()) {
        let r = (source[0] * intensity).clamp(0.0, 1.0);
        let g = (source[1] * intensity).clamp(0.0, 1.0);
        let b = (source[2] * intensity).clamp(0.0, 1.0);
        let a = (source[3] * intensity).clamp(0.0, 1.0);
        *pixel = image::Rgba([
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
            (a * 255.0).round() as u8,
        ]);
    }
    image
}
