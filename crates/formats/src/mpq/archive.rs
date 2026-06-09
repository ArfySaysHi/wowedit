use anyhow::Result;
use std::{path::Path, sync::Mutex};

pub struct MpqArchive {
    inner: Mutex<wow_mpq::Archive>,
}

impl MpqArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            inner: Mutex::new(wow_mpq::Archive::open(path)?),
        })
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("MPQ lock poisoned"))?;
        Ok(guard.read_file(&normalise(path))?)
    }

    pub fn list_files(&self) -> Result<Vec<String>> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("MPQ Lock poisoned"))?;
        Ok(guard
            .list_all()?
            .iter()
            .map(|fe| abnormalize(&fe.name))
            .collect())
    }
}

fn normalise(path: &str) -> String {
    path.replace('/', "\\").to_ascii_uppercase()
}

fn abnormalize(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}
