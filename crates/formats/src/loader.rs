use crate::{
    adt::{self, Adt},
    storage::Storage,
    version::WoWVersion,
};
use anyhow::Result;

pub struct AssetLoader {
    storage: Box<dyn Storage>,
    pub version: WoWVersion,
}

impl AssetLoader {
    pub fn new(storage: Box<dyn Storage>, version: WoWVersion) -> Self {
        Self { storage, version }
    }

    pub fn load_adt(&self, map_name: &str, x: u8, y: u8) -> Result<Adt> {
        let path = format!("world/maps/{map_name}/{map_name}_{x}_{y}.adt");
        let data = self.storage.read_to_end(&path)?;
        adt::parse(&data)
    }
}
