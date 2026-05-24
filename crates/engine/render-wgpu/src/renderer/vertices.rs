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
