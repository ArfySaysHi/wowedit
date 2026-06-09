pub mod casc;
pub mod filesystem;
pub mod mpq;

use anyhow::Result;
use std::io::{Read, Seek};

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

pub trait Storage: Send + Sync {
    fn exists(&self, path: &str) -> bool;

    fn open(&self, path: &str) -> Result<Box<dyn ReadSeek>>;

    fn read_to_end(&self, path: &str) -> Result<Vec<u8>> {
        let mut handle = self.open(path)?;
        let mut buffer = Vec::new();
        handle.read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    fn list_files(&self, _pattern: &str) -> Result<Vec<String>, anyhow::Error>;
}

// For determining the end result of grouped MPQs, not used for CASC
pub struct CompoundStorage {
    layers: Vec<Box<dyn Storage>>,
}

impl Storage for CompoundStorage {
    fn exists(&self, path: &str) -> bool {
        self.layers.iter().any(|s| s.exists(path))
    }

    fn open(&self, path: &str) -> Result<Box<dyn ReadSeek>> {
        for layer in &self.layers {
            if layer.exists(path) {
                return layer.open(path);
            }
        }
        anyhow::bail!("file not found: {}", path)
    }

    fn list_files(&self, pattern: &str) -> Result<Vec<String>> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for layer in &self.layers {
            for f in layer.list_files(pattern)? {
                if seen.insert(f.clone()) {
                    result.push(f);
                }
            }
        }
        Ok(result)
    }
}
