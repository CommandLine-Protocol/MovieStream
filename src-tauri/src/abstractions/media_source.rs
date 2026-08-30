use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaSourceKind {
    Local,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCandidate {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub mtime: DateTime<Utc>,
    pub extension: String,
}

pub trait MediaSource: Send + Sync {
    fn kind(&self) -> MediaSourceKind;
    fn list_files(&self, root: &str, supported_extensions: &[&str]) -> AppResult<Vec<FileCandidate>>;
    fn is_available(&self, root: &str) -> bool;
}
