use crate::graphics::utils;

use nalgebra as na;

pub struct Camera {
    pub eye: na::Point3<f32>,
    pub target: na::Point3<f32>,
    pub up: na::Unit<na::Vector3<f32>>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> na::Matrix4<f32> {
        let view: nalgebra::Isometry<f32, nalgebra::Unit<nalgebra::Quaternion<f32>>, 3> =
            na::Isometry3::look_at_rh(&self.eye, &self.target, &self.up);
        let projection = na::Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar);

        utils::OPENGL_TO_WGPU_MATRIX * (projection.as_projective() * view).matrix()
    }
}
