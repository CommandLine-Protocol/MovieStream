use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::abstractions::{
    LibrarySourceRepository, MediaRepository, MovieRepository,
};
use crate::domain::{LibrarySource, Media, Movie, MovieFilter, MovieSort, SourceStatus};
use crate::error::{AppError, AppResult};
use crate::services::metadata_resolver::MetadataResolver;
use crate::services::scanner::{ProgressEmitter, Scanner};

pub struct LibraryService {
    source_repo: Arc<dyn LibrarySourceRepository>,
    media_repo: Arc<dyn MediaRepository>,
    movie_repo: Arc<dyn MovieRepository>,
    scanner: Arc<Scanner>,
    metadata_resolver: Arc<MetadataResolver>,
}

impl LibraryService {
    pub fn new(
        source_repo: Arc<dyn LibrarySourceRepository>,
        media_repo: Arc<dyn MediaRepository>,
        movie_repo: Arc<dyn MovieRepository>,
        scanner: Arc<Scanner>,
        metadata_resolver: Arc<MetadataResolver>,
    ) -> Self {
        Self {
            source_repo,
            media_repo,
            movie_repo,
            scanner,
            metadata_resolver,
        }
    }

    pub async fn add_source(
        &self,
        path: &str,
        progress_emitter: Option<ProgressEmitter>,
    ) -> AppResult<LibrarySource> {
        let p = Path::new(path);
        if !p.exists() || !p.is_dir() {
            return Err(AppError::Source(format!("Invalid directory path: {}", path)));
        }

        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let source = if let Some(existing) = self.source_repo.find_by_path(path)? {
            existing
        } else {
            let id = Uuid::new_v4();
            let new_source = LibrarySource {
                id,
                path: path.to_string(),
                name,
                status: SourceStatus::Available,
                last_scanned_at: None,
                created_at: Utc::now(),
            };
            self.source_repo.upsert_source(&new_source)?;
            new_source
        };

        // Scan asynchronously
        let scanner = self.scanner.clone();
        let src_clone = source.clone();
        tokio::spawn(async move {
            if let Err(err) = scanner.scan_source(&src_clone, progress_emitter).await {
                tracing::error!("Failed background scan for source {}: {}", src_clone.path, err);
            }
        });

        Ok(source)
    }

    pub async fn pick_and_add_source(
        &self,
        progress_emitter: Option<ProgressEmitter>,
    ) -> AppResult<Option<LibrarySource>> {
        let folder = rfd::AsyncFileDialog::new()
            .set_title("Select Movie Folder")
            .pick_folder()
            .await;

        if let Some(handle) = folder {
            let path_str = handle.path().to_string_lossy().to_string();
            let source = self.add_source(&path_str, progress_emitter).await?;
            Ok(Some(source))
        } else {
            Ok(None)
        }
    }

    pub fn remove_source(&self, source_id: &Uuid) -> AppResult<()> {
        self.source_repo.delete_source(source_id)?;
        Ok(())
    }

    pub fn list_sources(&self) -> AppResult<Vec<LibrarySource>> {
        self.source_repo.list_sources()
    }

    pub async fn rescan_source(
        &self,
        source_id: &Uuid,
        progress_emitter: Option<ProgressEmitter>,
    ) -> AppResult<()> {
        let source = self
            .source_repo
            .get_source(source_id)?
            .ok_or_else(|| AppError::NotFound(format!("Source {} not found", source_id)))?;

        let scanner = self.scanner.clone();
        tokio::spawn(async move {
            if let Err(err) = scanner.scan_source(&source, progress_emitter).await {
                tracing::error!("Failed scan for source {}: {}", source.path, err);
            }
        });

        Ok(())
    }

    pub async fn rescan_all(&self, progress_emitter: Option<ProgressEmitter>) -> AppResult<()> {
        let sources = self.source_repo.list_sources()?;
        for source in sources {
            let scanner = self.scanner.clone();
            let emitter = progress_emitter.clone();
            tokio::spawn(async move {
                let _ = scanner.scan_source(&source, emitter).await;
            });
        }
        Ok(())
    }

    pub fn get_movie_with_media(&self, movie_id: &Uuid) -> AppResult<Option<(Movie, Vec<Media>)>> {
        if let Some(movie) = self.movie_repo.get_movie(movie_id)? {
            let media_list = self.media_repo.list_media_for_movie(movie_id)?;
            Ok(Some((movie, media_list)))
        } else {
            Ok(None)
        }
    }

    pub fn list_movies(&self, filter: &MovieFilter, sort: MovieSort) -> AppResult<Vec<Movie>> {
        self.movie_repo.list_movies(filter, sort)
    }

    pub fn set_manual_metadata(&self, movie_id: &Uuid, provider_id: &str) -> AppResult<Movie> {
        let existing = self
            .movie_repo
            .get_movie(movie_id)?
            .ok_or_else(|| AppError::NotFound(format!("Movie {} not found", movie_id)))?;

        let updated = self.metadata_resolver.resolve_manual(&existing, provider_id)?;
        self.movie_repo.upsert_movie(&updated)?;
        Ok(updated)
    }
}
