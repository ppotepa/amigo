#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoldenImageTolerance {
    pub max_channel_delta: u8,
    pub max_mismatched_pixels: usize,
}

impl GoldenImageTolerance {
    pub const EXACT: Self = Self {
        max_channel_delta: 0,
        max_mismatched_pixels: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoldenImageDiff {
    pub total_pixels: usize,
    pub mismatched_pixels: usize,
    pub max_channel_delta: u8,
}

impl GoldenImageDiff {
    pub fn passes(self, tolerance: GoldenImageTolerance) -> bool {
        self.mismatched_pixels <= tolerance.max_mismatched_pixels
            && self.max_channel_delta <= tolerance.max_channel_delta
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoldenImageError {
    DimensionOverflow { width: u32, height: u32 },
    ExpectedLength { expected: usize, actual: usize },
    ObservedLength { expected: usize, actual: usize },
}

pub fn compare_golden_rgba8(
    width: u32,
    height: u32,
    expected: &[u8],
    observed: &[u8],
) -> Result<GoldenImageDiff, GoldenImageError> {
    let pixels = (width as usize)
        .checked_mul(height as usize)
        .ok_or(GoldenImageError::DimensionOverflow { width, height })?;
    let expected_len = pixels
        .checked_mul(4)
        .ok_or(GoldenImageError::DimensionOverflow { width, height })?;
    if expected.len() != expected_len {
        return Err(GoldenImageError::ExpectedLength {
            expected: expected_len,
            actual: expected.len(),
        });
    }
    if observed.len() != expected_len {
        return Err(GoldenImageError::ObservedLength {
            expected: expected_len,
            actual: observed.len(),
        });
    }

    let mut mismatched_pixels = 0usize;
    let mut max_channel_delta = 0u8;
    for (expected_pixel, observed_pixel) in expected.chunks_exact(4).zip(observed.chunks_exact(4)) {
        let mut pixel_delta = 0u8;
        for channel in 0..4 {
            pixel_delta =
                pixel_delta.max(expected_pixel[channel].abs_diff(observed_pixel[channel]));
        }
        if pixel_delta != 0 {
            mismatched_pixels += 1;
            max_channel_delta = max_channel_delta.max(pixel_delta);
        }
    }

    Ok(GoldenImageDiff {
        total_pixels: pixels,
        mismatched_pixels,
        max_channel_delta,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_golden_match_passes() {
        let expected = [0, 20, 40, 255, 10, 30, 50, 255];
        let diff = compare_golden_rgba8(2, 1, &expected, &expected).expect("valid image");
        assert!(diff.passes(GoldenImageTolerance::EXACT));
    }

    #[test]
    fn tolerance_can_accept_small_pixel_drift() {
        let expected = [10, 20, 30, 255, 40, 50, 60, 255];
        let observed = [11, 20, 30, 255, 40, 53, 60, 255];
        let diff = compare_golden_rgba8(2, 1, &expected, &observed).expect("valid image");
        assert_eq!(diff.mismatched_pixels, 2);
        assert_eq!(diff.max_channel_delta, 3);
        assert!(diff.passes(GoldenImageTolerance {
            max_channel_delta: 3,
            max_mismatched_pixels: 2,
        }));
        assert!(!diff.passes(GoldenImageTolerance::EXACT));
    }

    #[test]
    fn rejects_invalid_buffer_dimensions() {
        assert!(matches!(
            compare_golden_rgba8(2, 2, &[0; 4], &[0; 16]),
            Err(GoldenImageError::ExpectedLength { .. })
        ));
    }
}
