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

        *(projection.as_projective() * view).matrix()
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: na::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        // We're using Vector4 because of the uniforms 16 byte spacing requirement
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_proj =
            (utils::OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}
