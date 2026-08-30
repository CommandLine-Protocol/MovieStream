use chrono::{DateTime, Utc};
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::abstractions::{FileCandidate, MediaSource, MediaSourceKind};
use crate::error::{AppError, AppResult};

pub struct LocalFileSystemSource;

impl LocalFileSystemSource {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalFileSystemSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaSource for LocalFileSystemSource {
    fn kind(&self) -> MediaSourceKind {
        MediaSourceKind::Local
    }

    fn is_available(&self, root: &str) -> bool {
        let path = Path::new(root);
        path.exists() && path.is_dir()
    }

    fn list_files(&self, root: &str, supported_extensions: &[&str]) -> AppResult<Vec<FileCandidate>> {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(AppError::Source(format!("Source path does not exist: {}", root)));
        }

        let mut candidates = Vec::new();
        let lower_extensions: Vec<String> = supported_extensions
            .iter()
            .map(|ext| ext.trim_start_matches('.').to_lowercase())
            .collect();

        for entry_res in WalkDir::new(root_path).follow_links(true).into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    tracing::warn!("Error reading directory entry: {}", err);
                    continue;
                }
            };

            // Skip directories and hidden files
            if entry.file_type().is_dir() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with('.') {
                continue;
            }

            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if lower_extensions.contains(&ext_str) {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let size_bytes = metadata.len();
                    let mtime: DateTime<Utc> = metadata
                        .modified()
                        .ok()
                        .and_then(|sys_time| {
                            let duration = sys_time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
                            DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
                        })
                        .unwrap_or_else(Utc::now);

                    let path_str = entry.path().to_string_lossy().to_string();

                    candidates.push(FileCandidate {
                        path: path_str,
                        filename: file_name,
                        size_bytes,
                        mtime,
                        extension: ext_str,
                    });
                }
            }
        }

        Ok(candidates)
    }
}
