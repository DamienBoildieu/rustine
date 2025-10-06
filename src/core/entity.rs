use crate::core::Component;

pub struct Entity {
    // Use contiguous allocator at some point
    pub components: Vec<Box<dyn Component>>,
}