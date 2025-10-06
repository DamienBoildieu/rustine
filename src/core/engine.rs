use crate::graphics;
use crate::core;

pub struct Engine {
    current_scene: core::Scene,
    renderer: graphics::Renderer,
}

impl Engine {
    fn update(&mut self) {
        // Update logic here
    }
}
