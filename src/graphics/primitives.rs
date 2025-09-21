use crate::graphics::Vertex;
use crate::graphics::{Mesh, Meshable};

use std::f32::consts;

pub struct Circle {
    pub radius: f32,
    pub nb_triangles: usize,
}

impl Circle {
    const DEFAULT_RADIUS: f32 = 0.5;
    const DEFAULT_NB_TRIANGLES: usize = 32;
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            radius: Circle::DEFAULT_RADIUS,
            nb_triangles: Circle::DEFAULT_NB_TRIANGLES,
        }
    }
}

impl Meshable for Circle {
    fn build_mesh(&self) -> Mesh {
        let mut vertices = Vec::with_capacity(self.nb_triangles + 1);

        vertices.push(Vertex {
            position: [0., 0., 0.],
            tex_coords: [0.5, 0.5],
        });

        for idx in 0..self.nb_triangles {
            let theta = idx as f32 * (consts::TAU / self.nb_triangles as f32);
            let (sin_theta, cos_theta) = theta.sin_cos();

            let x = self.radius * cos_theta;
            let y = self.radius * sin_theta;

            let x_tex = (x / self.radius + 1.0) * 0.5;
            // texture origin is top left while canva one is center
            let y_tex = (1.0 - y / self.radius) * 0.5;

            vertices.push(Vertex {
                position: [x, y, 0.],
                tex_coords: [x_tex, y_tex],
            });
        }

        let mut indices = Vec::with_capacity(self.nb_triangles * 3);
        for idx in 0..(self.nb_triangles-1) {
            indices.push(0);
            indices.push(idx as u16 + 1);
            indices.push(idx as u16 + 2);
        }

        indices.push(0);
        indices.push((vertices.len() - 1) as u16);
        indices.push(1);

        Mesh {
            vertices: vertices,
            indices: indices,
        }
    }
}
