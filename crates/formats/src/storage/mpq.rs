use crate::mpq::archive::MpqArchive;
use crate::storage::{ReadSeek, Storage};
use anyhow::Result;
use std::io::Cursor;
use std::path::Path;

pub struct MpqStorage {
    archive: MpqArchive,
}

impl MpqStorage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            archive: MpqArchive::open(path)?,
        })
    }
}

impl Storage for MpqStorage {
    fn exists(&self, path: &str) -> bool {
        self.archive.read_file(path).is_ok()
    }

    fn open(&self, path: &str) -> Result<Box<dyn ReadSeek>> {
        let data = self.archive.read_file(path)?;
        Ok(Box::new(Cursor::new(data)))
    }

    fn list_files(&self, _pattern: &str) -> Result<Vec<String>> {
        self.archive.list_files()
    }
}
