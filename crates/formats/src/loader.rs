use crate::{
    adt::{self, Adt},
    blp::BlpImage,
    m2::{
        m2_model::M2Model,
        m2_resolved_mesh::M2ResolvedMesh,
        m2_skin::{M2Skin, parse_skin},
        m2_texture::get_texture_path,
        parse_header, parse_texture_lookup, parse_textures, parse_vertices,
    },
    storage::Storage,
    version::WoWVersion,
};
use anyhow::Result;
use std::collections::HashMap;

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
        let mut adt = adt::parse(&data)?;
        adt.tile_x = x;
        adt.tile_y = y;

        Ok(adt)
    }

    fn load_skin(&self, path: &str) -> Result<M2Skin> {
        let data = self.storage.read_to_end(path)?;
        parse_skin(&data)
    }

    fn load_m2(&self, path: &str) -> Result<M2Model> {
        let data = self.storage.read_to_end(path)?;

        let header = parse_header(&data)?;

        let vertices = parse_vertices(
            &data,
            header.vertices_offset as usize,
            header.vertices_count as usize,
        )?;

        let textures = parse_textures(
            &data,
            header.textures_offset as usize,
            header.textures_count as usize,
        )?;

        let texture_lookup = parse_texture_lookup(
            &data,
            header.texture_lookup_offset as usize,
            header.texture_lookup_count as usize,
        )?;

        let texture_paths = textures
            .iter()
            .map(|tex| {
                get_texture_path(&data, tex.filename_offset as usize)
                    .unwrap_or_else(|_| "unknown.blp".to_string())
            })
            .collect::<Vec<_>>();

        Ok(M2Model {
            vertices,
            texture_paths,
            texture_lookup,
        })
    }

    pub fn load_m2_resolved(&self, path: &str) -> Result<M2ResolvedMesh> {
        let model = self.load_m2(path)?;
        let skin_path = path.to_uppercase().replace(".M2", "00.SKIN");
        let skin = self.load_skin(&skin_path)?;

        Ok(M2ResolvedMesh::new(&model, &skin))
    }
}
