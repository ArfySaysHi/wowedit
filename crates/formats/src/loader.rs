use std::collections::HashMap;

use crate::{
    adt::{self, Adt},
    blp::BlpImage,
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

    pub fn load_adt_textures(&self, adt: &Adt) -> Result<HashMap<u32, BlpImage>> {
        let mut textures: HashMap<u32, BlpImage> = HashMap::new();

        for (index, path) in adt.texture_paths.iter().enumerate() {
            let normalized = path.replace('\\', "/");

            match self.storage.read_to_end(&normalized) {
                Ok(data) => match crate::blp::decode(&data) {
                    Ok(image) => {
                        textures.insert(index as u32, image);
                    }
                    Err(e) => log::warn!("Failed to decode BLP {normalized}: {e}"),
                },
                Err(e) => log::warn!("Failed to load texture {normalized}: {e}"),
            }
        }

        Ok(textures)
    }

    pub fn load_adt(&self, map_name: &str, x: u8, y: u8) -> Result<Adt> {
        let path = format!("world/maps/{map_name}/{map_name}_{x}_{y}.adt");
        let data = self.storage.read_to_end(&path)?;
        adt::parse(&data)
    }
}
