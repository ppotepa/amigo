use std::collections::BTreeMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::renderer::CachedMeshGeometry3d;

use super::{
    gpu_edges_from_geometry, gpu_triangles_from_geometry, gpu_vertices_from_geometry,
    GpuNprEndpointEntry3d, GpuNprFrameUniforms3d,
};

#[derive(Debug)]
pub(crate) struct NprGpuMeshTopologyBuffers3d {
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub edge_count: u32,
    pub vertices: wgpu::Buffer,
    pub triangles: wgpu::Buffer,
    pub edges: wgpu::Buffer,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct NprGpuFaceIdTarget3d {
    pub width: u32,
    pub height: u32,
    pub face_id: wgpu::Texture,
    pub face_id_view: wgpu::TextureView,
    pub depth: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

#[derive(Debug)]
pub(crate) struct NprGpuFrameBuffers3d {
    pub projected_vertices: wgpu::Buffer,
    pub visible_segments: wgpu::Buffer,
    pub endpoint_heads: wgpu::Buffer,
    pub endpoint_entries: wgpu::Buffer,
    pub path_links: wgpu::Buffer,
    pub stroke_segments: wgpu::Buffer,
    pub indirect_args: wgpu::Buffer,
    pub uniforms: wgpu::Buffer,
    pub projected_vertices_capacity: u64,
    pub visible_segments_capacity: u64,
    pub endpoint_heads_capacity: u64,
    pub endpoint_entries_capacity: u64,
    pub path_links_capacity: u64,
    pub stroke_segments_capacity: u64,
}

#[derive(Debug, Default)]
pub(crate) struct NprGpuResources3d {
    pub(crate) topology_cache: BTreeMap<String, NprGpuMeshTopologyBuffers3d>,
    pub(crate) face_id_target: Option<NprGpuFaceIdTarget3d>,
    pub(crate) frame_buffers: Option<NprGpuFrameBuffers3d>,
}

impl NprGpuResources3d {
    pub(crate) fn ensure_topology_uploaded(
        &mut self,
        device: &wgpu::Device,
        mesh_key: &str,
        geometry: &Arc<CachedMeshGeometry3d>,
    ) -> bool {
        let cache_key = topology_cache_key(mesh_key, geometry);
        if self.topology_cache.contains_key(&cache_key) {
            return false;
        }

        let vertices = gpu_vertices_from_geometry(geometry);
        let triangles = gpu_triangles_from_geometry(geometry);
        let edges = gpu_edges_from_geometry(geometry);
        let topology = NprGpuMeshTopologyBuffers3d {
            vertex_count: vertices.len() as u32,
            triangle_count: triangles.len() as u32,
            edge_count: edges.len() as u32,
            vertices: create_buffer(device, "amigo-npr-gpu-topology-vertices", &vertices),
            triangles: create_buffer(device, "amigo-npr-gpu-topology-triangles", &triangles),
            edges: create_buffer(device, "amigo-npr-gpu-topology-edges", &edges),
        };
        self.topology_cache.insert(cache_key, topology);
        true
    }

    pub(crate) fn ensure_face_id_target(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> &NprGpuFaceIdTarget3d {
        let recreate = self.face_id_target.as_ref().is_none_or(|target| {
            target.width != width || target.height != height
        });
        if recreate {
            self.face_id_target = Some(create_face_id_target(device, width, height));
        }
        self.face_id_target
            .as_ref()
            .expect("face id target should exist after creation")
    }

    pub(crate) fn ensure_frame_buffers(
        &mut self,
        device: &wgpu::Device,
        projected_vertices_capacity: u64,
        visible_segments_capacity: u64,
        endpoint_heads_capacity: u64,
        endpoint_entries_capacity: u64,
        path_links_capacity: u64,
        stroke_segments_capacity: u64,
    ) -> &NprGpuFrameBuffers3d {
        let recreate = self.frame_buffers.as_ref().is_none_or(|buffers| {
                buffers.projected_vertices_capacity < projected_vertices_capacity
                || buffers.visible_segments_capacity < visible_segments_capacity
                || buffers.endpoint_heads_capacity < endpoint_heads_capacity
                || buffers.endpoint_entries_capacity < endpoint_entries_capacity
                || buffers.path_links_capacity < path_links_capacity
                || buffers.stroke_segments_capacity < stroke_segments_capacity
        });
        if recreate {
            self.frame_buffers = Some(create_frame_buffers(
                device,
                projected_vertices_capacity,
                visible_segments_capacity,
                endpoint_heads_capacity,
                endpoint_entries_capacity,
                path_links_capacity,
                stroke_segments_capacity,
            ));
        }
        self.frame_buffers
            .as_ref()
            .expect("frame buffers should exist after creation")
    }

    pub(crate) fn frame_buffer_capacity_bytes(&self) -> u64 {
        self.frame_buffers
            .as_ref()
            .map(|buffers| {
                buffers.projected_vertices_capacity
                    + buffers.visible_segments_capacity
                    + buffers.endpoint_heads_capacity
                    + buffers.endpoint_entries_capacity
                    + buffers.path_links_capacity
                    + buffers.stroke_segments_capacity
            })
            .unwrap_or(0)
    }
}

fn topology_cache_key(mesh_key: &str, geometry: &CachedMeshGeometry3d) -> String {
    format!(
        "{mesh_key}:{}:{}:{}",
        geometry.vertex_count(),
        geometry.triangle_count(),
        geometry.edge_count()
    )
}

fn create_buffer<T>(device: &wgpu::Device, label: &'static str, data: &[T]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: slice_as_bytes(data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

fn slice_as_bytes<T>(data: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            std::mem::size_of_val(data),
        )
    }
}

fn create_face_id_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> NprGpuFaceIdTarget3d {
    let size = wgpu::Extent3d {
        width: width.max(1),
        height: height.max(1),
        depth_or_array_layers: 1,
    };
    let face_id = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("amigo-npr-gpu-face-id"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Uint,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("amigo-npr-gpu-face-depth"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    NprGpuFaceIdTarget3d {
        width: size.width,
        height: size.height,
        face_id_view: face_id.create_view(&wgpu::TextureViewDescriptor::default()),
        depth_view: depth.create_view(&wgpu::TextureViewDescriptor::default()),
        face_id,
        depth,
    }
}

fn create_frame_buffers(
    device: &wgpu::Device,
    projected_vertices_capacity: u64,
    visible_segments_capacity: u64,
    endpoint_heads_capacity: u64,
    endpoint_entries_capacity: u64,
    path_links_capacity: u64,
    stroke_segments_capacity: u64,
) -> NprGpuFrameBuffers3d {
    NprGpuFrameBuffers3d {
        projected_vertices: create_empty_buffer(
            device,
            "amigo-npr-projected-vertices",
            projected_vertices_capacity,
        ),
        visible_segments: create_empty_buffer(
            device,
            "amigo-npr-visible-segments",
            visible_segments_capacity,
        ),
        endpoint_heads: create_empty_buffer(
            device,
            "amigo-npr-endpoint-heads",
            endpoint_heads_capacity,
        ),
        endpoint_entries: create_empty_buffer(
            device,
            "amigo-npr-endpoint-entries",
            endpoint_entries_capacity.max(std::mem::size_of::<GpuNprEndpointEntry3d>() as u64),
        ),
        path_links: create_empty_buffer(
            device,
            "amigo-npr-path-links",
            path_links_capacity,
        ),
        stroke_segments: create_empty_buffer(
            device,
            "amigo-npr-stroke-segments",
            stroke_segments_capacity,
        ),
        indirect_args: create_empty_buffer(device, "amigo-npr-indirect-args", 64),
        uniforms: create_uniform_buffer(
            device,
            "amigo-npr-frame-uniforms",
            std::mem::size_of::<GpuNprFrameUniforms3d>() as u64,
        ),
        projected_vertices_capacity,
        visible_segments_capacity,
        endpoint_heads_capacity,
        endpoint_entries_capacity,
        path_links_capacity,
        stroke_segments_capacity,
    }
}

fn create_empty_buffer(device: &wgpu::Device, label: &'static str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(4),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::INDIRECT,
        mapped_at_creation: false,
    })
}

fn create_uniform_buffer(device: &wgpu::Device, label: &'static str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
