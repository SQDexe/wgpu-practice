use {
    bytemuck::{
        Pod,
        Zeroable
        },
    cgmath::*,
    winit::keyboard::*,
    crate::utils::*
    };

const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
    );

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    // up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32
    }

impl Camera {
    pub const fn new(eye: Vec3<f32>, target: Vec3<f32>, aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            eye: array_to_point3(eye),
            target: array_to_point3(target),
            // up: VEC3_UNIT_Y,
            aspect,
            fovy,
            znear,
            zfar
            }
        }

    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, Vector3::unit_y());
        let projection = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * projection * view
        }
    }

// For correct representation in shader
#[repr(C)]
// For storing in buffer
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    // Matrix4 can not be used inside buffers so a medium ground is needed
    view_projection: Mat4<f32>
    }

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection: matrix4_to_array(Matrix4::identity())
            }
        }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = matrix4_to_array(camera.build_view_projection_matrix())
        }
    }

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool
    }

impl CameraController {
    pub const fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false
            }
        }

    pub fn handle_key(&mut self, key: KeyCode, is_pressed: bool) -> bool {
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
                },
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
                },
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
                },
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
                },
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
                },
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.is_down_pressed = is_pressed;
                true
                },
            _ => false
            }
        }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevent glitches while too close to the centre of the scene
        if self.is_forward_pressed && self.speed < forward_mag {
            camera.eye += forward_norm * self.speed;
            }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
            }

        let right = forward_norm.cross(Vector3::unit_y());

        // Double-check in case front/back is pressed
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        // Rescale the distance between the target, and the eye, so that the eye lies on a cricle around the target
        if self.is_right_pressed {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
            }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
            }
        }
    }