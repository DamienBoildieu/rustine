use crate::core::Component;

pub struct Entity {
    pub components: Vec<Box<dyn Component>>,
}