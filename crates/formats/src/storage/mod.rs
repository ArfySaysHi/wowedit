pub mod casc;
pub mod filesystem;
pub mod mpq;

use crate::storage::mpq::MpqStorage;
use anyhow::Result;
use std::{
    io::{Read, Seek},
    path::{Path, PathBuf},
};

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

impl CompoundStorage {
    pub fn new(layers: Vec<Box<dyn Storage>>) -> Self {
        Self { layers }
    }

    pub fn from_wow_install(data_dir: impl AsRef<Path>, locale: &str) -> Result<Self> {
        let data = data_dir.as_ref();
        let locale_dir = data.join(locale);

        let mut layers: Vec<(i32, PathBuf)> = Vec::new();

        // Collect from both directories
        for (dir, is_locale) in [(&data.to_path_buf(), false), (&locale_dir, true)] {
            if !dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                if path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("mpq"))
                    .unwrap_or(false)
                {
                    let priority = infer_priority(&path, locale, is_locale);
                    layers.push((priority, path));
                }
            }
        }

        // Sort highest priority first
        layers.sort_by_key(|(p, _)| std::cmp::Reverse(*p));

        let storages: Vec<Box<dyn Storage>> = layers
            .into_iter()
            .filter_map(|(_, path)| match MpqStorage::open(&path) {
                Ok(s) => Some(Box::new(s) as Box<dyn Storage>),
                Err(e) => {
                    eprintln!("warning: skipping {:?}: {}", path, e);
                    None
                }
            })
            .collect();

        if storages.is_empty() {
            anyhow::bail!("no MPQ archives found in {}", data.display());
        }

        Ok(Self::new(storages))
    }
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

fn infer_priority(path: &Path, locale: &str, is_locale_dir: bool) -> i32 {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let locale_lower = locale.to_ascii_lowercase();

    if is_locale_dir && name.starts_with(&format!("patch-{locale_lower}")) {
        return 1000 + patch_number(&name);
    }

    if !is_locale_dir && name.starts_with("patch") {
        return 800 + patch_number(&name);
    }

    if is_locale_dir {
        return match name.as_str() {
            s if s.starts_with("lichking-locale") => 600,
            s if s.starts_with("expansion-locale") => 500,
            s if s.starts_with("locale") => 400,
            s if s.starts_with("speech") => 390,
            _ => 300,
        };
    }

    match name.as_str() {
        "lichking" => 200,
        "expansion" => 150,
        "common-2" => 100,
        "common" => 50,
        _ => 25,
    }
}

fn patch_number(name: &str) -> i32 {
    name.rsplit('-')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
