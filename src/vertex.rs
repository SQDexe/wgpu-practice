use {
    bytemuck::{
        Pod,
        Zeroable
        },
    wgpu::*,
    std::mem::size_of,
    crate::utils::*
    };

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: Vec3<f32>,
    texture_coords: Vec2<f32>
    }

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelVertex {
    position: Vec3<f32>,
    texture_coords: Vec2<f32>,
    normal_coords: Vec3<f32>
    }

impl Vertex {
    pub const fn new(position: Vec3<f32>, texture_coords: Vec2<f32>) -> Self {
        Self { position, texture_coords }
        }
    }

impl ModelVertex {
    pub const fn new(position: Vec3<f32>, texture_coords: Vec2<f32>, normal_coords: Vec3<f32>) -> Self {
        Self { position, texture_coords, normal_coords }
        }
    }

impl VertexInfo for Vertex {
    const DESC: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2
            ]
        };
    }

impl VertexInfo for ModelVertex {
    const DESC: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
            2 => Float32x3
            ]
        };
    }