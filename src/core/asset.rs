use std::{collections::HashMap};

pub enum Handle<T> {
    Strong(Asset<T>),
    Weak(Asset<T>),
}

pub struct Asset<T> {
    pub id: u64,
    pub data: T,
}

pub struct AssetManager<T> {
    assets: HashMap<u64, Asset<T>>,
    next_id: u64,
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<T> AssetManager<T> {
    pub fn create_asset(&mut self, data: T) -> &Asset<T> {
        let id = self.next_id;
        self.next_id += 1;
        let asset = Asset {
            id,
            data
        };
        self.assets.insert(id, asset);
        self.assets.get(&id).unwrap()
    }

    pub fn get_asset(&self, id: u64) -> Option<&Asset<T>> {
        self.assets.get(&id)
    }
}
