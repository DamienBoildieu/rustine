use crate::{core, graphics};

pub struct Scene {
    pub entities: Vec<core::Entity>,
    graphics_scene: graphics::Scene,
}

impl Scene {

    pub fn render(&self, renderer: &mut graphics::Renderer) {
        self.graphics_scene.render(renderer);
    }
}
