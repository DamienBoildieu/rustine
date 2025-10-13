use crate::{
    core::{Asset, Engine},
    graphics::Model,
};

pub trait Component {
    fn init(&mut self, engine: &mut Engine);
}

pub struct ModelComponent {
    pub model: Asset<Model>,
}
