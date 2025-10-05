use std::collections::HashMap;

use crate::graphics::{Camera, Instance, LightUniform, Model, Renderer};

pub struct Scene {
    camera: Camera,
    instances: Vec<Instance>,
    obj_model: Model,
    light_uniform: LightUniform,
    models: HashMap<Model, Vec<Instance>>,
}

impl Scene {
    pub fn init(&self, renderer: &mut Renderer) {

    }

    pub fn render(&self, renderer: &mut Renderer) {

    }
}