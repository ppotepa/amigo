use crate::renderer::{CachedMeshGeometry3d, Viewport};

pub(crate) fn scaled_face_id_dimensions(viewport: &Viewport, max_dimension_px: f32) -> (u32, u32) {
    let size = viewport.size();
    let scale = (max_dimension_px / size.x.max(size.y)).min(1.0);
    (
        (size.x * scale).round().max(1.0) as u32,
        (size.y * scale).round().max(1.0) as u32,
    )
}

pub(crate) fn slice_as_bytes<T>(data: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)) }
}

pub(crate) fn create_job_uniform_buffer(device: &wgpu::Device, uniform_size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("amigo-npr-job-uniforms"),
        size: uniform_size.max(16),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub(crate) fn workgroup_count(items: usize) -> u32 {
    ((items.max(1) as u32).saturating_add(63)) / 64
}

pub(crate) fn npr_gpu_endpoint_head_count(edge_count: usize) -> usize {
    let target = (edge_count.max(1) * 4).next_power_of_two();
    target.max(64)
}

pub(crate) fn topology_cache_key(mesh_key: &str, geometry: &CachedMeshGeometry3d) -> String {
    format!(
        "{mesh_key}:{}:{}:{}",
        geometry.vertex_count(),
        geometry.triangle_count(),
        geometry.edge_count()
    )
}

pub(crate) fn storage_binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

pub(crate) fn uniform_binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

pub(crate) fn texture_binding<'a>(
    binding: u32,
    view: &'a wgpu::TextureView,
) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}
