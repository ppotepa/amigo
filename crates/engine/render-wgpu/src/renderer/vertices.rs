use std::mem::size_of;

use amigo_math::{ColorRgba, Vec2};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ColorVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: [f32; 4],
}

impl ColorVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4
    ];

    pub(crate) fn new(position: Vec2, color: ColorRgba) -> Self {
        Self {
            position: [position.x, position.y],
            color: [color.r, color.g, color.b, color.a],
        }
    }

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ColorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TextureVertex {
    pub(crate) position: [f32; 2],
    pub(crate) uv: [f32; 2],
    pub(crate) color: [f32; 4],
}

impl TextureVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4
    ];

    pub(crate) fn new(position: Vec2, uv: Vec2, color: ColorRgba) -> Self {
        Self {
            position: [position.x, position.y],
            uv: [uv.x, uv.y],
            color: [color.r, color.g, color.b, color.a],
        }
    }

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<TextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NprStrokeSegmentVertex {
    pub(crate) start: [f32; 2],
    pub(crate) end: [f32; 2],
    pub(crate) color: [f32; 4],
    pub(crate) width_px: f32,
    pub(crate) offset_px: f32,
    pub(crate) overshoot_start_px: f32,
    pub(crate) overshoot_end_px: f32,
    pub(crate) viewport_half: [f32; 2],
    pub(crate) end_width_px: f32,
    pub(crate) end_alpha: f32,
}

impl NprStrokeSegmentVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 10] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4,
        3 => Float32,
        4 => Float32,
        5 => Float32,
        6 => Float32,
        7 => Float32x2,
        8 => Float32,
        9 => Float32
    ];

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<NprStrokeSegmentVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NprStrokeSegmentVertex;

    #[test]
    fn npr_stroke_segment_vertex_matches_wgsl_storage_stride() {
        assert_eq!(std::mem::size_of::<NprStrokeSegmentVertex>(), 64);
        assert_eq!(NprStrokeSegmentVertex::layout().array_stride, 64);
    }
}
