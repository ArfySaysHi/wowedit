#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WoWVersion {
    WotLK,
    // Other versions, maybe never
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Mpq,
    // Casc later...
}

impl WoWVersion {
    pub const fn storage_type(&self) -> StorageType {
        match self {
            WoWVersion::WotLK => StorageType::Mpq,
        }
    }
}
