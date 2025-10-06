use crate::{core, graphics};

pub struct Scene {
    pub entities: Vec<core::Entity>,
    graphics_scene: graphics::Scene,
}
