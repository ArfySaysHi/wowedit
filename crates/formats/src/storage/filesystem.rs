use crate::storage::{ReadSeek, Storage};
use anyhow::Result;
use std::path::PathBuf;

pub struct FsStorage {
    root: PathBuf,
}

impl Storage for FsStorage {
    fn open(&self, path: &str) -> Result<Box<dyn ReadSeek>> {
        let full = self.root.join(normalise_path_to_fs(path));
        Ok(Box::new(std::fs::File::open(full)?))
    }

    fn exists(&self, _path: &str) -> bool {
        todo!()
    }

    fn list_files(&self, _pattern: &str) -> Result<Vec<String>, anyhow::Error> {
        todo!()
    }
}

fn normalise_path_to_fs(path: &str) -> PathBuf {
    path.replace('\\', "/").into()
}
