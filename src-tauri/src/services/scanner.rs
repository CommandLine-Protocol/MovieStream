use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::abstractions::{
    LibrarySourceRepository, MediaRepository, MediaSource, MovieRepository, TvRepository,
};
use crate::domain::{
    LibrarySource, Media, MediaAvailability, MetadataStatus, SourceStatus, TvEpisode, TvSeason,
    TvSeries,
};
use crate::error::{AppError, AppResult};
use crate::events::ScanProgressPayload;
use crate::services::duplicate_resolver::DuplicateResolver;
use crate::services::filename_parser::{FilenameParser, ParsedMediaType};
use crate::services::media_analyzer::MediaAnalyzer;
use crate::services::metadata_resolver::MetadataResolver;
use crate::services::tmdb_service::TmdbService;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "mov", "m4v", "webm", "flv", "ts", "wmv",
];

pub type ProgressEmitter = Arc<dyn Fn(ScanProgressPayload) + Send + Sync>;

pub struct Scanner {
    source_repo: Arc<dyn LibrarySourceRepository>,
    media_repo: Arc<dyn MediaRepository>,
    _movie_repo: Arc<dyn MovieRepository>,
    tv_repo: Option<Arc<dyn TvRepository>>,
    tmdb_service: Option<Arc<TmdbService>>,
    media_source: Arc<dyn MediaSource>,
    media_analyzer: Arc<MediaAnalyzer>,
    metadata_resolver: Arc<MetadataResolver>,
    duplicate_resolver: Arc<DuplicateResolver>,
}

impl Scanner {
    pub fn new(
        source_repo: Arc<dyn LibrarySourceRepository>,
        media_repo: Arc<dyn MediaRepository>,
        movie_repo: Arc<dyn MovieRepository>,
        media_source: Arc<dyn MediaSource>,
        media_analyzer: Arc<MediaAnalyzer>,
        metadata_resolver: Arc<MetadataResolver>,
        duplicate_resolver: Arc<DuplicateResolver>,
    ) -> Self {
        Self {
            source_repo,
            media_repo,
            _movie_repo: movie_repo,
            tv_repo: None,
            tmdb_service: None,
            media_source,
            media_analyzer,
            metadata_resolver,
            duplicate_resolver,
        }
    }

    pub fn with_tv_support(
        source_repo: Arc<dyn LibrarySourceRepository>,
        media_repo: Arc<dyn MediaRepository>,
        movie_repo: Arc<dyn MovieRepository>,
        tv_repo: Arc<dyn TvRepository>,
        tmdb_service: Arc<TmdbService>,
        media_source: Arc<dyn MediaSource>,
        media_analyzer: Arc<MediaAnalyzer>,
        metadata_resolver: Arc<MetadataResolver>,
        duplicate_resolver: Arc<DuplicateResolver>,
    ) -> Self {
        Self {
            source_repo,
            media_repo,
            _movie_repo: movie_repo,
            tv_repo: Some(tv_repo),
            tmdb_service: Some(tmdb_service),
            media_source,
            media_analyzer,
            metadata_resolver,
            duplicate_resolver,
        }
    }

    pub async fn scan_source(
        &self,
        source: &LibrarySource,
        progress_emitter: Option<ProgressEmitter>,
    ) -> AppResult<()> {
        let emit = |phase: &str, files_discovered: u32, movies_identified: u32| {
            if let Some(ref emitter) = progress_emitter {
                emitter(ScanProgressPayload {
                    source_id: source.id,
                    files_discovered,
                    movies_identified,
                    phase: phase.to_string(),
                });
            }
        };

        self.source_repo.set_status(&source.id, SourceStatus::Scanning)?;
        emit("scanning", 0, 0);

        if !self.media_source.is_available(&source.path) {
            self.source_repo.set_status(&source.id, SourceStatus::Inaccessible)?;
            self.media_repo.set_source_media_availability(&source.id, MediaAvailability::Unavailable)?;
            emit("error", 0, 0);
            return Err(AppError::Source(format!("Source path is inaccessible: {}", source.path)));
        }

        // List files on disk
        let disk_files = self.media_source.list_files(&source.path, SUPPORTED_EXTENSIONS)?;
        let total_discovered = disk_files.len() as u32;
        emit("analyzing", total_discovered, 0);

        // Get existing media for this source in database
        let existing_media = self.media_repo.list_media_for_source(&source.id)?;
        let mut existing_by_path: std::collections::HashMap<String, Media> = existing_media
            .into_iter()
            .map(|m| (m.path.clone(), m))
            .collect();

        let mut items_identified = 0u32;
        let mut processed_paths = std::collections::HashSet::new();

        for candidate in disk_files {
            processed_paths.insert(candidate.path.clone());

            if let Some(existing) = existing_by_path.remove(&candidate.path) {
                // Check if file was modified
                let mtime_changed = (candidate.mtime - existing.file_mtime).num_seconds().abs() > 2;
                let size_changed = candidate.size_bytes != existing.size_bytes;

                if mtime_changed || size_changed {
                    let parsed = FilenameParser::parse(&candidate.filename);
                    let analyzed = self.media_analyzer.analyze(&candidate.path, parsed.resolution_guess.as_deref())?;

                    let mut updated_media = existing;
                    updated_media.size_bytes = candidate.size_bytes;
                    updated_media.file_mtime = candidate.mtime;
                    updated_media.duration_seconds = analyzed.duration_seconds;
                    updated_media.container_format = analyzed.container_format;
                    updated_media.video_codec = analyzed.video_codec;
                    updated_media.resolution_width = analyzed.resolution_width;
                    updated_media.resolution_height = analyzed.resolution_height;
                    updated_media.audio_tracks = analyzed.audio_tracks;
                    updated_media.subtitle_tracks = analyzed.subtitle_tracks;
                    updated_media.availability = MediaAvailability::Available;
                    updated_media.updated_at = Utc::now();

                    self.media_repo.upsert_media(&updated_media)?;
                } else if existing.availability == MediaAvailability::Unavailable {
                    self.media_repo.set_availability(&existing.id, MediaAvailability::Available)?;
                }
                items_identified += 1;
            } else {
                // New file discovered -> full pipeline
                let parsed = FilenameParser::parse(&candidate.filename);
                let analyzed = self.media_analyzer.analyze(&candidate.path, parsed.resolution_guess.as_deref())?;
                let media_id = Uuid::new_v4();
                let now = Utc::now();

                match parsed.media_type {
                    ParsedMediaType::Episode { season_number, episode_number } => {
                        let episode_id = self.resolve_or_create_episode(
                            &parsed.title_guess,
                            parsed.year_guess,
                            season_number,
                            episode_number,
                            analyzed.duration_seconds,
                        ).await?;

                        let new_media = Media {
                            id: media_id,
                            movie_id: None,
                            episode_id: Some(episode_id),
                            source_id: source.id,
                            path: candidate.path,
                            size_bytes: candidate.size_bytes,
                            duration_seconds: analyzed.duration_seconds,
                            container_format: analyzed.container_format,
                            video_codec: analyzed.video_codec,
                            resolution_width: analyzed.resolution_width,
                            resolution_height: analyzed.resolution_height,
                            audio_tracks: analyzed.audio_tracks,
                            subtitle_tracks: analyzed.subtitle_tracks,
                            file_hash: None,
                            file_mtime: candidate.mtime,
                            availability: MediaAvailability::Available,
                            created_at: now,
                            updated_at: now,
                        };

                        self.media_repo.upsert_media(&new_media)?;
                    }
                    ParsedMediaType::Movie => {
                        let resolved_movie = self.metadata_resolver.resolve_with_file_path(
                            &parsed.title_guess,
                            parsed.year_guess,
                            Some(&candidate.path),
                        );
                        let movie_id = self.duplicate_resolver.match_or_create_movie(resolved_movie)?;

                        let new_media = Media {
                            id: media_id,
                            movie_id: Some(movie_id),
                            episode_id: None,
                            source_id: source.id,
                            path: candidate.path,
                            size_bytes: candidate.size_bytes,
                            duration_seconds: analyzed.duration_seconds,
                            container_format: analyzed.container_format,
                            video_codec: analyzed.video_codec,
                            resolution_width: analyzed.resolution_width,
                            resolution_height: analyzed.resolution_height,
                            audio_tracks: analyzed.audio_tracks,
                            subtitle_tracks: analyzed.subtitle_tracks,
                            file_hash: None,
                            file_mtime: candidate.mtime,
                            availability: MediaAvailability::Available,
                            created_at: now,
                            updated_at: now,
                        };

                        self.media_repo.upsert_media(&new_media)?;
                    }
                }

                items_identified += 1;
            }

            emit("indexing", total_discovered, items_identified);
        }

        // Mark missing media as unavailable
        for (_, missing) in existing_by_path {
            self.media_repo.set_availability(&missing.id, MediaAvailability::Unavailable)?;
        }

        self.source_repo.update_last_scanned(&source.id, Utc::now())?;
        self.source_repo.set_status(&source.id, SourceStatus::Available)?;
        emit("completed", total_discovered, items_identified);

        Ok(())
    }

    async fn resolve_or_create_episode(
        &self,
        series_title: &str,
        year: Option<u16>,
        season_number: u32,
        episode_number: u32,
        duration_seconds: Option<u32>,
    ) -> AppResult<Uuid> {
        let tv_repo = match self.tv_repo {
            Some(ref r) => r.clone(),
            None => return Err(AppError::Database("TV repository not initialized".to_string())),
        };

        // 1. Find or create TvSeries
        let series = if let Some(existing) = tv_repo.find_series_by_title(series_title)? {
            existing
        } else {
            // Search TMDB for series metadata if available
            let (tmdb_id, title, desc, poster, backdrop, genres, rating, year_val) = if let Some(ref tmdb) = self.tmdb_service {
                if let Ok(results) = tmdb.search_tv(series_title, year).await {
                    if let Some(top) = results.first() {
                        (
                            Some(top.id),
                            top.name.clone(),
                            top.overview.clone(),
                            top.poster_path.clone(),
                            top.backdrop_path.clone(),
                            vec!["Drama".to_string()],
                            top.vote_average,
                            year,
                        )
                    } else {
                        (None, series_title.to_string(), None, None, None, vec![], None, year)
                    }
                } else {
                    (None, series_title.to_string(), None, None, None, vec![], None, year)
                }
            } else {
                (None, series_title.to_string(), None, None, None, vec![], None, year)
            };

            let now = Utc::now();
            let new_series = TvSeries {
                id: Uuid::new_v4(),
                tmdb_id,
                title,
                original_title: None,
                year: year_val,
                description: desc,
                poster_path: poster,
                backdrop_path: backdrop,
                genres,
                rating,
                metadata_provider_id: tmdb_id.map(|id| format!("tmdb-{}", id)),
                metadata_status: if tmdb_id.is_some() { MetadataStatus::AutoMatched } else { MetadataStatus::Unmatched },
                created_at: now,
                updated_at: now,
            };

            tv_repo.upsert_series(&new_series)?;
            new_series
        };

        // 2. Find or create TvSeason
        let seasons = tv_repo.list_seasons_by_series(&series.id)?;
        let season = if let Some(existing_season) = seasons.into_iter().find(|s| s.season_number == season_number) {
            existing_season
        } else {
            let new_season = TvSeason {
                id: Uuid::new_v4(),
                series_id: series.id,
                season_number,
                name: format!("Season {}", season_number),
                overview: None,
                poster_path: series.poster_path.clone(),
                episode_count: 0,
                created_at: Utc::now(),
            };
            tv_repo.upsert_season(&new_season)?;
            new_season
        };

        // 3. Find or create TvEpisode
        let episodes = tv_repo.list_episodes_by_season(&season.id)?;
        if let Some(existing_ep) = episodes.into_iter().find(|e| e.episode_number == episode_number) {
            Ok(existing_ep.id)
        } else {
            // Try fetching episode details from TMDB
            let (ep_title, ep_overview, ep_still, ep_air_date, ep_rating) = if let (Some(tmdb_id), Some(ref tmdb)) = (series.tmdb_id, self.tmdb_service.as_ref()) {
                if let Ok(tmdb_eps) = tmdb.get_tv_episodes(tmdb_id, season_number).await {
                    if let Some(found) = tmdb_eps.into_iter().find(|e| e.episode_number == episode_number) {
                        (found.name, found.overview, found.still_path, found.air_date, found.vote_average)
                    } else {
                        (format!("Episode {}", episode_number), None, None, None, None)
                    }
                } else {
                    (format!("Episode {}", episode_number), None, None, None, None)
                }
            } else {
                (format!("Episode {}", episode_number), None, None, None, None)
            };

            let now = Utc::now();
            let new_ep = TvEpisode {
                id: Uuid::new_v4(),
                series_id: series.id,
                season_id: season.id,
                season_number,
                episode_number,
                title: ep_title,
                overview: ep_overview,
                still_path: ep_still,
                air_date: ep_air_date,
                duration_seconds,
                rating: ep_rating,
                created_at: now,
                updated_at: now,
            };

            tv_repo.upsert_episode(&new_ep)?;
            Ok(new_ep.id)
        }
    }
}
