use glam::Vec3;
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnimClipError {
    #[error("AMC read failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("AMC invalid: {0}")]
    Invalid(String),
}

#[derive(Clone, Debug)]
pub struct AnimMeshClip {
    pub fps: f32,
    pub duration: f32,
    pub vertex_count: usize,
    pub faces: Vec<[usize; 3]>,
    pub frames: Vec<Vec<Vec3>>,
}

impl AnimMeshClip {
    pub fn from_amc_file(path: &Path) -> Result<Self, AnimClipError> {
        let bytes = fs::read(path)?;
        Self::from_amc_bytes(&bytes)
    }

    pub fn from_amc_bytes(bytes: &[u8]) -> Result<Self, AnimClipError> {
        if bytes.len() < 32 || &bytes[0..4] != b"AMC1" {
            return Err(AnimClipError::Invalid("bad magic".to_owned()));
        }

        let version = read_u32(bytes, 4)?;
        if version != 1 {
            return Err(AnimClipError::Invalid(format!(
                "unsupported version {version}"
            )));
        }

        let fps = read_f32(bytes, 8)?;
        let duration = read_f32(bytes, 12)?;
        let vertex_count = read_u32(bytes, 16)? as usize;
        let face_count = read_u32(bytes, 20)? as usize;
        let frame_count = read_u32(bytes, 24)? as usize;

        if fps <= 0.0 || duration <= 0.0 || vertex_count == 0 || frame_count == 0 {
            return Err(AnimClipError::Invalid("empty clip".to_owned()));
        }

        let expected_len =
            32usize
                .checked_add(face_count.checked_mul(12).ok_or_else(|| {
                    AnimClipError::Invalid("face section size overflow".to_owned())
                })?)
                .and_then(|len| {
                    len.checked_add(frame_count.checked_mul(vertex_count)?.checked_mul(12)?)
                })
                .ok_or_else(|| AnimClipError::Invalid("vertex section size overflow".to_owned()))?;
        if bytes.len() < expected_len {
            return Err(AnimClipError::Invalid("truncated clip".to_owned()));
        }

        let mut offset = 32usize;
        let mut faces = Vec::with_capacity(face_count);
        for _ in 0..face_count {
            let a = read_u32(bytes, offset)? as usize;
            offset += 4;
            let b = read_u32(bytes, offset)? as usize;
            offset += 4;
            let c = read_u32(bytes, offset)? as usize;
            offset += 4;
            if a >= vertex_count || b >= vertex_count || c >= vertex_count {
                return Err(AnimClipError::Invalid("face index out of range".to_owned()));
            }
            faces.push([a, b, c]);
        }

        let mut frames = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            let mut verts = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                let x = read_f32(bytes, offset)?;
                offset += 4;
                let y = read_f32(bytes, offset)?;
                offset += 4;
                let z = read_f32(bytes, offset)?;
                offset += 4;
                verts.push(Vec3::new(x, y, z));
            }
            frames.push(verts);
        }

        Ok(Self {
            fps,
            duration,
            vertex_count,
            faces,
            frames,
        })
    }

    pub fn sample_vertices(&self, time: f32) -> Vec<Vec3> {
        if self.frames.len() == 1 {
            return self.frames[0].clone();
        }

        let wrapped = wrap_time(time, self.duration);
        let frame_f = wrapped * self.fps;
        let a = frame_f.floor() as usize % self.frames.len();
        let b = (a + 1) % self.frames.len();
        let t = frame_f.fract();

        self.frames[a]
            .iter()
            .zip(self.frames[b].iter())
            .map(|(pa, pb)| pa.lerp(*pb, t))
            .collect()
    }
}

fn wrap_time(value: f32, duration: f32) -> f32 {
    if duration <= 0.0 {
        return value;
    }
    let mut out = value % duration;
    if out < 0.0 {
        out += duration;
    }
    out
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, AnimClipError> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| AnimClipError::Invalid("truncated u32".to_owned()))?;
    Ok(u32::from_le_bytes(raw.try_into().unwrap()))
}

fn read_f32(bytes: &[u8], offset: usize) -> Result<f32, AnimClipError> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| AnimClipError::Invalid("truncated f32".to_owned()))?;
    Ok(f32::from_le_bytes(raw.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::AnimMeshClip;

    #[test]
    fn amc_loader_samples_interpolated_vertices() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"AMC1");
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        for id in [0u32, 1, 2] {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        for value in [
            0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 2.0, 1.0,
            0.0,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        let clip = AnimMeshClip::from_amc_bytes(&bytes).unwrap();
        let sample = clip.sample_vertices(0.25);
        assert_eq!(clip.vertex_count, 3);
        assert_eq!(clip.faces, vec![[0, 1, 2]]);
        assert!((sample[0].x - 1.0).abs() < 0.001);
        assert!((sample[1].x - 2.0).abs() < 0.001);
    }
}
