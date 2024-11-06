use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sysinfo::Disk;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub kind: String,
    pub file_system: String,
    pub mount_point: Option<String>,
    pub total_space: u64,
    pub available_space: u64,
    pub is_removable: bool,
    pub is_read_only: bool,
}

impl From<&Disk> for DiskInfo {
    fn from(disk: &Disk) -> Self {
        Self{
            kind: disk.kind().to_string(),
            name: disk.name().to_string_lossy().to_string(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            mount_point: Some(disk.mount_point().to_string_lossy().to_string()),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            is_read_only: disk.is_read_only(),
            is_removable: disk.is_removable(),
        }
    }
}
