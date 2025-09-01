use {
    bytemuck::{
        Pod,
        Zeroable
        },
    cgmath::*,
    wgpu::*,
    std::mem::size_of,
    crate::utils::*
    };

pub struct Instance {
    position: Vector3<f32>,
    rotation: Quaternion<f32>
    }

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InstanceRaw {
    model: Mat4<f32>
    }

impl Instance {
    pub const fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
        }

    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: matrix4_to_array(Matrix4::from_translation(self.position) * Matrix4::from(self.rotation))
            }
        }
    }

impl InstanceRaw {
    pub const DESC: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        // Shader will only start processing next Instance, after finishing previous one
        step_mode: VertexStepMode::Instance,
        // The Matrix4 is deconstructed here into corresponding rows
        attributes: &vertex_attr_array![
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4
            ]
        };
    }