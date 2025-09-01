pub type Vec2<T> = [T; 2];
pub type Vec3<T> = [T; 3];
pub type Vec4<T> = [T; 4];

pub type Mat4<T> = [[T; 4]; 4];

pub const fn array_to_point3<T: Sized + Copy>(value: Vec3<T>) -> cgmath::Point3<T> {
    let [x, y, z] = value;
    cgmath::Point3::new(x, y, z)
    }
pub const fn matrix4_to_array<T: Sized + Copy>(value: cgmath::Matrix4<T>) -> Mat4<T> {
    let cgmath::Matrix4 { x, y, z, w } = value;
    [ 
        [x.x, x.y, x.z, x.w],
        [y.x, y.y, y.z, y.w],
        [z.x, z.y, z.z, z.w],
        [w.x, w.y, w.z, w.w]
    ]
    }

pub trait VertexInfo {
    const DESC: wgpu::VertexBufferLayout<'static>;
    }