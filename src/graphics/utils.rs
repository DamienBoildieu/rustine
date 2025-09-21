use nalgebra as na;

pub(super) const OPENGL_TO_WGPU_MATRIX: na::Matrix4<f32> = na::matrix![1.0, 0.0, 0.0, 0.0;
                                                                       0.0, 1.0, 0.0, 0.0;
                                                                       0.0, 0.0, 0.5, 0.0;
                                                                       0.0, 0.0, 0.5, 1.0];
