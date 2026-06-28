mod core;

pub(crate) use core::{COLOR_SHADER, NPR_STROKE_SEGMENT_SHADER, TEXTURE_SHADER};
pub(crate) const NPR_FACE_ID_SHADER: &str = include_str!("npr_face_id.wgsl");
pub(crate) const NPR_PROJECT_VERTICES_SHADER: &str = include_str!("npr_project_vertices.wgsl");
pub(crate) const NPR_CLASSIFY_EDGES_SHADER: &str = include_str!("npr_classify_edges.wgsl");
pub(crate) const NPR_CLEAR_ENDPOINT_HEADS_SHADER: &str =
    include_str!("npr_clear_endpoint_heads.wgsl");
pub(crate) const NPR_BUILD_ENDPOINT_BINS_SHADER: &str =
    include_str!("npr_build_endpoint_bins.wgsl");
pub(crate) const NPR_COMPACT_OWNERS_SHADER: &str = include_str!("npr_compact_owners.wgsl");
pub(crate) const NPR_CONNECT_PATHS_SHADER: &str = include_str!("npr_connect_paths.wgsl");
pub(crate) const NPR_RELAX_PATH_OWNERS_SHADER: &str = include_str!("npr_relax_path_owners.wgsl");
pub(crate) const NPR_EMIT_PATH_SEGMENTS_SHADER: &str = include_str!("npr_emit_path_segments.wgsl");
pub(crate) const NPR_BUILD_STROKES_SHADER: &str = include_str!("npr_build_strokes.wgsl");
pub(crate) const NPR_CLAMP_INDIRECT_ARGS_SHADER: &str =
    include_str!("npr_clamp_indirect_args.wgsl");
