use glam::{EulerRot, Mat4, Quat, Vec3};

// The camera scene object which will contain only the things we need for moving, rotating and
// otherwise messing with the scene object. NO RENDERING - Edna Mode
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // Y axis rotation (left/right)
    pub pitch: f32, // X axis rotation (up/down)
}

pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

impl Camera {
    pub const SPEED: f32 = 10.0;

    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
    // There is two matrices to view-proj, view and projection (mind blown)
    //
    // The view matrix' job is to tell me where the camera is in world space and where it's looking which
    // is the inverse of the camera's own transform. Since the camera stays stationary and the world moves
    // around it, the view matrix provides a transformation to be applied to the entire world, since it's
    // a 3D world we need a 4x4 matrix that describes the position and rotation change required to fit into
    // the camera's view which will be applied to ALL vertices in the scene.
    //
    // The projection matrix's job is to tell us how 3D depth turns into 2D perspective by encoding
    // your FOV, aspect ratio and near/far planes (the frustum) which makes far away things look
    // smaller by squishing the frustum pyramid into a -1/1 clip cube.
    //
    // If I don't write this down I will forget. I really wish I had gotten into A-level maths...
    pub fn build_view_proj(&self, aspect_ratio: f32) -> Mat4 {
        let rotation = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);

        // NEG_Z is our default forwards if we haven't rotated at all yet
        let direction = rotation * Vec3::NEG_Z;

        // Describes our cameras space in the world
        let view = Mat4::look_to_rh(self.position, direction, Vec3::Y);

        // Describes our frustum shape, location of near / far planes and how wide our lens is
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 10.0, 50000.0);

        // From right to left, we move the world relative to the camera then squish it into clip space
        proj * view
    }

    pub fn with_pitch(self, pitch: f32) -> Self {
        Self {
            position: self.position,
            yaw: self.yaw,
            pitch,
        }
    }

    pub fn look_at(position: Vec3, target: Vec3) -> Self {
        let dir = (target - position).normalize();
        let pitch = dir.y.asin();
        let yaw = dir.z.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
        Self {
            position,
            yaw,
            pitch,
        }
    }

    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        self.yaw -= delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity;

        // Prevent flipping when looking too far up/down
        let limit = std::f32::consts::FRAC_PI_2 - 0.01;
        self.pitch = self.pitch.clamp(-limit, limit);
    }

    pub fn process_keyboard(&mut self, direction: CameraMovement, speed: f32, dt: f32) {
        let rotation = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);

        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;
        let up = Vec3::Y;

        let velocity = speed * dt;

        match direction {
            CameraMovement::Forward => self.position += forward * velocity,
            CameraMovement::Backward => self.position -= forward * velocity,
            CameraMovement::Right => self.position += right * velocity,
            CameraMovement::Left => self.position -= right * velocity,
            CameraMovement::Up => self.position += up * velocity,
            CameraMovement::Down => self.position -= up * velocity,
        }
    }
}
