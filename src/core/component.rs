use crate::core::Engine;

pub trait Component {
    fn init(&mut self, engine: &mut Engine);
}
