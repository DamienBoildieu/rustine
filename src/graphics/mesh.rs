use crate::graphics::Vertex;

// When in doubt about transformation types see: https://nalgebra.rs/docs/user_guide/points_and_transformations#transformations

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub trait Meshable {
    fn build_mesh(&self) -> Mesh;
}

impl<T> From<T> for Mesh
where
    T: Meshable,
{
    fn from(meshable: T) -> Self {     
        meshable.build_mesh()
    }
}
