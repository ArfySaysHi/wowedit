use glam::{EulerRot, Mat4, Quat, Vec3};

// The camera scene object which will contain only the things we need for moving, rotating and
// otherwise messing with the scene object. NO RENDERING - Edna Mode
pub struct Camera {
    position: Vec3,
    yaw: f32,   // Y axis rotation (left/right)
    pitch: f32, // X axis rotation (up/down)
}

impl Camera {
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
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.1, 10000.0);

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
}
