use crate::graphics;
use crate::core;

pub struct Engine {
    current_scene: core::Scene,
    renderer: graphics::Renderer,
    modules: Vec<Box<dyn core::Module>>,
}

pub struct Context<'a> {
    pub scene: &'a mut core::Scene,
    pub renderer: &'a mut graphics::Renderer,
}

impl Engine {
    pub fn register_module(&mut self, module: impl core::Module) {
        self.modules.push(Box::new(module));
    }

    pub fn run(&mut self) {
        for module in &mut self.modules {
            module.init();
        }

        // Main loop (simplified)
        loop {
            self.update();
            // self.current_scene.render(&mut self.renderer);
            // Break condition for the loop would go here
            break; // Placeholder to prevent infinite loop in this example
        }

        for module in &mut self.modules {
            module.shutdown();
        }
    }

    pub fn update(&mut self) {
        let modules_count = self.modules.len();

        let mut context = Context {
                scene: &mut self.current_scene,
                renderer: &mut self.renderer,
            };
        
        for idx in 0..modules_count {
            self.modules[idx].update(&mut context, 0.016); // Assuming a fixed delta time for simplicity
        }
    }
}
