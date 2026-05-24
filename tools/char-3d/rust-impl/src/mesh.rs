use crate::{anim_clip::AnimMeshClip, state::AppState};
use glam::Vec3;
use std::{borrow::Cow, collections::HashMap, fs, path::Path, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeshError {
    #[error("OBJ parser: nie znaleziono v/f.")]
    EmptyObj,
    #[error("OBJ read failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("FBX import failed: {0}")]
    Fbx(String),
}

#[derive(Clone, Debug)]
pub struct Face {
    pub v: [usize; 3],
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub a: usize,
    pub b: usize,
    pub f0: Option<usize>,
    pub f1: Option<usize>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Mesh {
    pub name: String,
    pub source_type: String,
    pub verts: Vec<Vec3>,
    pub faces: Vec<Face>,
    pub edges: Vec<Edge>,
    pub cache_id: String,
    pub frame_version: u64,
    pub anim_clip: Option<Arc<AnimMeshClip>>,
}

impl Mesh {
    pub fn from_obj_file(path: &Path, name: impl Into<String>) -> Result<Self, MeshError> {
        let text = fs::read_to_string(path)?;
        Self::from_obj_text(&text, name)
    }

    pub fn from_obj_text(text: &str, name: impl Into<String>) -> Result<Self, MeshError> {
        let name = name.into();
        let mut verts = Vec::new();
        let mut faces = Vec::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<_> = line.split_whitespace().collect();
            match parts.first().copied() {
                Some("v") if parts.len() >= 4 => {
                    let x = parts[1].parse().unwrap_or(0.0);
                    let y = parts[2].parse().unwrap_or(0.0);
                    let z = parts[3].parse().unwrap_or(0.0);
                    verts.push(Vec3::new(x, y, z));
                }
                Some("f") if parts.len() >= 4 => {
                    let mut ids = Vec::new();
                    for part in &parts[1..] {
                        let raw_id = part.split('/').next().unwrap_or_default();
                        let Ok(mut id) = raw_id.parse::<isize>() else {
                            continue;
                        };
                        if id < 0 {
                            id = verts.len() as isize + id + 1;
                        }
                        if id > 0 {
                            ids.push((id - 1) as usize);
                        }
                    }
                    for i in 1..ids.len().saturating_sub(1) {
                        if ids[0] != ids[i] && ids[i] != ids[i + 1] && ids[0] != ids[i + 1] {
                            faces.push(Face {
                                v: [ids[0], ids[i], ids[i + 1]],
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        if verts.is_empty() || faces.is_empty() {
            return Err(MeshError::EmptyObj);
        }
        normalize_verts(&mut verts);
        let edges = build_edges(&faces);
        Ok(Self {
            cache_id: format!("obj:{name}:{}:{}", verts.len(), faces.len()),
            name,
            source_type: "obj".to_owned(),
            verts,
            faces,
            edges,
            frame_version: 0,
            anim_clip: None,
        })
    }

    pub fn from_fbx_file(path: &Path, name: impl Into<String>) -> Result<Self, MeshError> {
        let name = name.into();
        let bytes = fs::read(path)?;
        let (mut verts, faces) = parse_binary_fbx_geometry(&bytes)?;
        if verts.is_empty() || faces.is_empty() {
            return Err(MeshError::Fbx("no renderable geometry found".to_owned()));
        }
        normalize_verts(&mut verts);
        let edges = build_edges(&faces);
        Ok(Self {
            cache_id: format!("assimp:{name}:{}:{}", verts.len(), faces.len()),
            name,
            source_type: "fbx".to_owned(),
            verts,
            faces,
            edges,
            frame_version: 0,
            anim_clip: None,
        })
    }

    pub fn from_anim_clip_file(path: &Path, name: impl Into<String>) -> Result<Self, MeshError> {
        let name = name.into();
        let clip = AnimMeshClip::from_amc_file(path)
            .map_err(|err| MeshError::Fbx(format!("anim clip import failed: {err}")))?;
        let vertex_count = clip.vertex_count;
        let verts = clip.frames.first().cloned().unwrap_or_default();
        let faces = clip
            .faces
            .iter()
            .map(|v| Face { v: *v })
            .collect::<Vec<_>>();
        if verts.is_empty() || faces.is_empty() {
            return Err(MeshError::Fbx("empty anim clip".to_owned()));
        }
        let edges = build_edges(&faces);
        Ok(Self {
            cache_id: format!("animclip:{name}:{vertex_count}:{}", faces.len()),
            name,
            source_type: "fbx".to_owned(),
            verts,
            faces,
            edges,
            frame_version: 0,
            anim_clip: Some(Arc::new(clip)),
        })
    }

    pub fn deformed_vertices<'a>(&'a self, state: &AppState) -> Cow<'a, [Vec3]> {
        if let Some(clip) = &self.anim_clip {
            return Cow::Owned(clip.sample_vertices(state.anim_sample_time));
        }
        if self.source_type != "fbx" {
            return Cow::Borrowed(&self.verts);
        }
        let phase = state.anim_sample_time * std::f32::consts::TAU;
        let frame = state.anim_frame_index as f32;
        Cow::Owned(
            self.verts
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let y01 = ((p.y + 1.0) * 0.5).clamp(0.0, 1.0);
                    let side = if p.x >= 0.0 { 1.0 } else { -1.0 };
                    let stride = (phase * 2.0 + side * 1.9 + y01 * 0.6).sin();
                    let counter = (phase * 2.0 + side * 1.9 + std::f32::consts::PI).sin();
                    let torso = (phase + y01 * 1.7).sin();
                    let seed = ((i as f32 + 1.0) * 12.9898 + frame * 0.017).sin();
                    let limb_mask = (1.0 - (y01 - 0.45).abs() * 2.0).clamp(0.0, 1.0);
                    let mut out = *p;
                    out.x += stride * limb_mask * 0.035 + torso * y01 * 0.018;
                    out.z += counter * limb_mask * 0.026;
                    out.y += (phase + seed).sin() * limb_mask * 0.010;
                    out
                })
                .collect(),
        )
    }

    pub fn animation_duration(&self) -> Option<f32> {
        self.anim_clip.as_ref().map(|clip| clip.duration)
    }
}

#[derive(Default)]
struct FbxGeometry {
    verts: Vec<Vec3>,
    indices: Vec<i32>,
}

fn parse_binary_fbx_geometry(bytes: &[u8]) -> Result<(Vec<Vec3>, Vec<Face>), MeshError> {
    const MAGIC: &[u8] = b"Kaydara FBX Binary  \0\x1a\0";
    if bytes.len() < 27 || &bytes[..23] != MAGIC {
        return Err(MeshError::Fbx(
            "only binary FBX files are supported".to_owned(),
        ));
    }
    let version = u32_le(bytes, 23)? as u64;
    let mut pos = 27usize;
    let mut geometries = Vec::new();
    while pos < bytes.len().saturating_sub(16) {
        parse_fbx_node(
            bytes,
            &mut pos,
            bytes.len() as u64,
            version,
            &mut geometries,
        )?;
        if pos == 0 {
            break;
        }
    }
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    for geometry in geometries {
        if geometry.verts.is_empty() || geometry.indices.is_empty() {
            continue;
        }
        let base = verts.len();
        verts.extend(geometry.verts);
        let mut polygon = Vec::new();
        for raw in geometry.indices {
            let end = raw < 0;
            let id = if end {
                (-raw - 1) as usize
            } else {
                raw as usize
            };
            polygon.push(base + id);
            if end {
                for i in 1..polygon.len().saturating_sub(1) {
                    faces.push(Face {
                        v: [polygon[0], polygon[i], polygon[i + 1]],
                    });
                }
                polygon.clear();
            }
        }
    }
    Ok((verts, faces))
}

fn parse_fbx_node(
    bytes: &[u8],
    pos: &mut usize,
    parent_end: u64,
    version: u64,
    geometries: &mut Vec<FbxGeometry>,
) -> Result<Option<FbxGeometry>, MeshError> {
    let start = *pos;
    let (end_offset, prop_count, prop_len) = if version >= 7500 {
        (
            u64_le(bytes, *pos)?,
            u64_le(bytes, *pos + 8)?,
            u64_le(bytes, *pos + 16)?,
        )
    } else {
        (
            u32_le(bytes, *pos)? as u64,
            u32_le(bytes, *pos + 4)? as u64,
            u32_le(bytes, *pos + 8)? as u64,
        )
    };
    let header_len = if version >= 7500 { 25 } else { 13 };
    *pos += header_len;
    if end_offset == 0 {
        *pos = parent_end.min(bytes.len() as u64) as usize;
        return Ok(None);
    }
    let name_len = *bytes
        .get(*pos - 1)
        .ok_or_else(|| MeshError::Fbx("truncated node header".to_owned()))?
        as usize;
    let name_start = *pos;
    let name_end = name_start + name_len;
    let name =
        std::str::from_utf8(bytes.get(name_start..name_end).unwrap_or_default()).unwrap_or("");
    *pos = name_end;

    let mut own_vertices = None;
    let mut own_indices = None;
    for _ in 0..prop_count {
        let prop = read_fbx_property(bytes, pos)?;
        if name == "Vertices" {
            if let FbxProp::F64Array(values) = prop {
                own_vertices = Some(
                    values
                        .chunks_exact(3)
                        .map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32))
                        .collect::<Vec<_>>(),
                );
            }
        } else if name == "PolygonVertexIndex"
            && let FbxProp::I32Array(values) = prop
        {
            own_indices = Some(values);
        }
    }
    let prop_end = name_end + prop_len as usize;
    if *pos < prop_end {
        *pos = prop_end;
    }

    let mut geometry = if name == "Geometry" {
        Some(FbxGeometry::default())
    } else {
        None
    };
    if let Some(values) = own_vertices {
        geometry.get_or_insert_with(FbxGeometry::default).verts = values;
    }
    if let Some(values) = own_indices {
        geometry.get_or_insert_with(FbxGeometry::default).indices = values;
    }

    while (*pos as u64) < end_offset && *pos < bytes.len() {
        if let Some(child) = parse_fbx_node(bytes, pos, end_offset, version, geometries)? {
            if name == "Geometry" {
                let target = geometry.get_or_insert_with(FbxGeometry::default);
                if !child.verts.is_empty() {
                    target.verts = child.verts;
                }
                if !child.indices.is_empty() {
                    target.indices = child.indices;
                }
            } else if child.verts.len() > 2 && !child.indices.is_empty() {
                geometry = Some(child);
            }
        }
    }
    *pos = end_offset.min(bytes.len() as u64) as usize;
    if start == *pos {
        *pos = 0;
    }
    if name == "Geometry" {
        if let Some(geometry) = geometry
            && !geometry.verts.is_empty()
            && !geometry.indices.is_empty()
        {
            geometries.push(geometry);
        }
        return Ok(None);
    }
    Ok(geometry.filter(|g| !g.verts.is_empty() || !g.indices.is_empty()))
}

enum FbxProp {
    F64Array(Vec<f64>),
    I32Array(Vec<i32>),
    Other,
}

fn read_fbx_property(bytes: &[u8], pos: &mut usize) -> Result<FbxProp, MeshError> {
    let Some(kind) = bytes.get(*pos).copied() else {
        return Err(MeshError::Fbx("truncated property".to_owned()));
    };
    *pos += 1;
    match kind as char {
        'd' => Ok(FbxProp::F64Array(read_f64_array(bytes, pos)?)),
        'i' => Ok(FbxProp::I32Array(read_i32_array(bytes, pos)?)),
        'f' | 'l' | 'b' => {
            skip_array(bytes, pos)?;
            Ok(FbxProp::Other)
        }
        'D' | 'L' => {
            *pos += 8;
            Ok(FbxProp::Other)
        }
        'F' | 'I' => {
            *pos += 4;
            Ok(FbxProp::Other)
        }
        'Y' | 'C' => {
            *pos += if kind as char == 'Y' { 2 } else { 1 };
            Ok(FbxProp::Other)
        }
        'S' | 'R' => {
            let len = u32_le(bytes, *pos)? as usize;
            *pos += 4 + len;
            Ok(FbxProp::Other)
        }
        _ => Err(MeshError::Fbx(format!(
            "unsupported property type {}",
            kind as char
        ))),
    }
}

fn read_f64_array(bytes: &[u8], pos: &mut usize) -> Result<Vec<f64>, MeshError> {
    let raw = read_array_payload(bytes, pos, 8)?;
    let mut out = Vec::with_capacity(raw.len() / 8);
    for chunk in raw.chunks_exact(8) {
        out.push(f64::from_le_bytes(chunk.try_into().unwrap()));
    }
    Ok(out)
}

fn read_i32_array(bytes: &[u8], pos: &mut usize) -> Result<Vec<i32>, MeshError> {
    let raw = read_array_payload(bytes, pos, 4)?;
    let mut out = Vec::with_capacity(raw.len() / 4);
    for chunk in raw.chunks_exact(4) {
        out.push(i32::from_le_bytes(chunk.try_into().unwrap()));
    }
    Ok(out)
}

fn skip_array(bytes: &[u8], pos: &mut usize) -> Result<(), MeshError> {
    let _ = read_array_payload(bytes, pos, 1)?;
    Ok(())
}

fn read_array_payload(
    bytes: &[u8],
    pos: &mut usize,
    elem_size: usize,
) -> Result<Vec<u8>, MeshError> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let count = u32_le(bytes, *pos)? as usize;
    let encoding = u32_le(bytes, *pos + 4)?;
    let compressed_len = u32_le(bytes, *pos + 8)? as usize;
    *pos += 12;
    let payload = bytes
        .get(*pos..*pos + compressed_len)
        .ok_or_else(|| MeshError::Fbx("truncated array payload".to_owned()))?;
    *pos += compressed_len;
    if encoding == 0 {
        Ok(payload.to_vec())
    } else if encoding == 1 {
        let mut decoder = ZlibDecoder::new(payload);
        let mut out = Vec::with_capacity(count * elem_size);
        decoder
            .read_to_end(&mut out)
            .map_err(|err| MeshError::Fbx(format!("zlib decode failed: {err}")))?;
        Ok(out)
    } else {
        Err(MeshError::Fbx(format!(
            "unsupported array encoding {encoding}"
        )))
    }
}

fn u32_le(bytes: &[u8], offset: usize) -> Result<u32, MeshError> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| MeshError::Fbx("truncated u32".to_owned()))?;
    Ok(u32::from_le_bytes(raw.try_into().unwrap()))
}

fn u64_le(bytes: &[u8], offset: usize) -> Result<u64, MeshError> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| MeshError::Fbx("truncated u64".to_owned()))?;
    Ok(u64::from_le_bytes(raw.try_into().unwrap()))
}

fn normalize_verts(verts: &mut [Vec3]) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for p in verts.iter().copied() {
        min = min.min(p);
        max = max.max(p);
    }
    let center = (min + max) * 0.5;
    let size = max - min;
    let scale = 2.0 / size.x.max(size.y).max(size.z).max(0.001);
    for p in verts {
        *p = (*p - center) * scale;
    }
}

fn build_edges(faces: &[Face]) -> Vec<Edge> {
    let mut map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, face) in faces.iter().enumerate() {
        for i in 0..3 {
            let a = face.v[i];
            let b = face.v[(i + 1) % 3];
            map.entry((a.min(b), a.max(b))).or_default().push(fi);
        }
    }
    map.into_iter()
        .map(|((a, b), faces)| Edge {
            a,
            b,
            f0: faces.first().copied(),
            f1: faces.get(1).copied(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obj_parser_triangulates_quads() {
        let mesh =
            Mesh::from_obj_text("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n", "quad").unwrap();
        assert_eq!(mesh.faces.len(), 2);
        assert_eq!(mesh.edges.len(), 5);
    }

    #[test]
    fn fbx_parser_reads_walking_geometry() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models/walking.fbx");
        let mesh = Mesh::from_fbx_file(&path, "walking").unwrap();
        assert!(!mesh.verts.is_empty());
        assert!(!mesh.faces.is_empty());
        assert!(mesh.verts.len() > 10_000);
        assert!(mesh.faces.len() > 10_000);
    }

    #[test]
    fn built_in_assets_load() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in [
            "assets/models/suzanne.obj",
            "assets/models/Goku.obj",
            "assets/models/new_york.obj",
        ] {
            let mesh = Mesh::from_obj_file(&root.join(path), path).unwrap();
            assert!(!mesh.verts.is_empty(), "{path}");
            assert!(!mesh.faces.is_empty(), "{path}");
        }

        let fbx = Mesh::from_fbx_file(&root.join("assets/models/walking.fbx"), "walking").unwrap();
        assert!(!fbx.verts.is_empty());
        assert!(!fbx.faces.is_empty());

        let clip =
            Mesh::from_anim_clip_file(&root.join("assets/models/walking.amc"), "walking").unwrap();
        assert!(!clip.verts.is_empty());
        assert!(!clip.faces.is_empty());
    }

    #[test]
    fn fbx_deformation_changes_vertices_over_time() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models/walking.fbx");
        let mesh = Mesh::from_fbx_file(&path, "walking").unwrap();
        let mut state = AppState::default();
        state.model_source = "walking".to_owned();
        state.anim_sample_time = 0.0;
        let a = mesh.deformed_vertices(&state);
        state.anim_sample_time = 0.33;
        state.anim_frame_index = 8;
        let b = mesh.deformed_vertices(&state);
        assert!(a.iter().zip(b.iter()).any(|(a, b)| a.distance(*b) > 0.001));
    }

    #[test]
    fn baked_walking_clip_changes_vertices_over_time() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models/walking.amc");
        let mesh = Mesh::from_anim_clip_file(&path, "walking").unwrap();
        let mut state = AppState {
            model_source: "walking".to_owned(),
            ..Default::default()
        };
        state.anim_sample_time = 0.0;
        let a = mesh.deformed_vertices(&state);
        state.anim_sample_time = 0.33;
        let b = mesh.deformed_vertices(&state);
        assert!(mesh.animation_duration().unwrap() > 2.0);
        assert!(a.iter().zip(b.iter()).any(|(a, b)| a.distance(*b) > 0.001));
    }
}
