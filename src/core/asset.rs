use std::rc::{Rc, Weak};

pub enum Handle<T> {
    Strong(Rc<T>),
    Weak(Weak<T>),
}

pub struct Asset<T> {
    pub data: T,
}

pub struct AssetManager<T> {
    // Asset storage and management fields would go here
    pub assets: Vec<Asset<T>>,
}

impl<T> AssetManager<T> {
    pub fn add_asset(&mut self, data: Asset<T>) -> &Asset<T> {
        self.assets.push(data);
        self.assets.last().unwrap()
    }
}

impl<T> From<T> for Asset<T> {
    fn from(value: T) -> Self {
        Self { data: value }
    }
}
