use crate::core;

pub trait Module : 'static {
    fn init(&mut self);
    fn update(&mut self, engine: &mut core::Context, delta_time: f32);
    fn shutdown(&mut self);
}