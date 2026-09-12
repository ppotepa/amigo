use image::{ExtendedColorType, codecs::jpeg::JpegEncoder};

#[test]
fn turbojpeg_q92_preserves_reference_colours_orientation_and_ink_edges() {
    let (width, height) = (128usize, 72usize);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let colour = if x % 31 < 2 || y % 23 < 2 {
                [24, 25, 27]
            } else if y < height / 2 {
                [240, 225, 206]
            } else {
                [94, 140, 180]
            };
            rgba.extend_from_slice(&[colour[0], colour[1], colour[2], 255]);
        }
    }
    let rgb: Vec<u8> = rgba.chunks_exact(4).flat_map(|pixel| pixel[..3].iter().copied()).collect();
    let mut reference = Vec::new();
    JpegEncoder::new_with_quality(&mut reference, 92)
        .encode(&rgb, width as u32, height as u32, ExtendedColorType::Rgb8).unwrap();
    let mut compressor = turbojpeg::Compressor::new().unwrap();
    compressor.set_quality(92).unwrap();
    compressor.set_subsamp(turbojpeg::Subsamp::None).unwrap();
    let encoded = compressor.compress_to_vec(turbojpeg::Image {
        pixels: rgba.as_slice(), width, height, pitch: width * 4, format: turbojpeg::PixelFormat::RGBA,
    }).unwrap();
    let reference = image::load_from_memory(&reference).unwrap().into_rgb8();
    let actual = image::load_from_memory(&encoded).unwrap().into_rgb8();
    assert_eq!(actual.dimensions(), (width as u32, height as u32));
    let mean_error = actual.as_raw().iter().zip(reference.as_raw()).map(|(a, b)| a.abs_diff(*b) as f64).sum::<f64>() / rgb.len() as f64;
    assert!(mean_error < 3.0, "Q92 reference mismatch: {mean_error}");
    assert!(actual.get_pixel(10, 10)[0] > actual.get_pixel(10, 50)[0] + 100);
    assert!(actual.get_pixel(1, 10)[0] < 40);
}
