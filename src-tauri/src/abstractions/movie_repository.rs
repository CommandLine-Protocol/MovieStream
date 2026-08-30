use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    AppSettings, LibrarySource, Media, MediaAvailability, Movie, MovieFilter, MovieSort,
    PlaybackState, SourceStatus, WatchHistoryEntry,
};
use crate::error::AppResult;

pub trait MovieRepository: Send + Sync {
    fn upsert_movie(&self, movie: &Movie) -> AppResult<()>;
    fn get_movie(&self, id: &Uuid) -> AppResult<Option<Movie>>;
    fn find_by_title_year(&self, title: &str, year: Option<u16>) -> AppResult<Option<Movie>>;
    fn find_by_provider_id(&self, provider_id: &str) -> AppResult<Option<Movie>>;
    fn list_movies(&self, filter: &MovieFilter, sort: MovieSort) -> AppResult<Vec<Movie>>;
    fn search_movies(&self, query: &str) -> AppResult<Vec<Movie>>;
    fn delete_movie(&self, id: &Uuid) -> AppResult<()>;
    fn count_movies(&self) -> AppResult<u32>;
}

pub trait MediaRepository: Send + Sync {
    fn upsert_media(&self, media: &Media) -> AppResult<()>;
    fn get_media(&self, id: &Uuid) -> AppResult<Option<Media>>;
    fn find_by_path(&self, path: &str) -> AppResult<Option<Media>>;
    fn list_media_for_movie(&self, movie_id: &Uuid) -> AppResult<Vec<Media>>;
    fn list_media_for_episode(&self, episode_id: &Uuid) -> AppResult<Vec<Media>>;
    fn list_media_for_source(&self, source_id: &Uuid) -> AppResult<Vec<Media>>;
    fn set_availability(&self, media_id: &Uuid, availability: MediaAvailability) -> AppResult<()>;
    fn set_source_media_availability(&self, source_id: &Uuid, availability: MediaAvailability) -> AppResult<()>;
    fn delete_media(&self, id: &Uuid) -> AppResult<()>;
}

pub trait LibrarySourceRepository: Send + Sync {
    fn upsert_source(&self, source: &LibrarySource) -> AppResult<()>;
    fn get_source(&self, id: &Uuid) -> AppResult<Option<LibrarySource>>;
    fn find_by_path(&self, path: &str) -> AppResult<Option<LibrarySource>>;
    fn list_sources(&self) -> AppResult<Vec<LibrarySource>>;
    fn set_status(&self, id: &Uuid, status: SourceStatus) -> AppResult<()>;
    fn update_last_scanned(&self, id: &Uuid, time: DateTime<Utc>) -> AppResult<()>;
    fn delete_source(&self, id: &Uuid) -> AppResult<()>;
}

pub trait PlaybackStateRepository: Send + Sync {
    fn upsert_state(&self, state: &PlaybackState) -> AppResult<()>;
    fn get_state(&self, movie_id: &Uuid) -> AppResult<Option<PlaybackState>>;
    fn list_in_progress(&self) -> AppResult<Vec<(Movie, PlaybackState)>>;
    fn delete_state(&self, movie_id: &Uuid) -> AppResult<()>;
}

pub trait WatchHistoryRepository: Send + Sync {
    fn add_entry(&self, entry: &WatchHistoryEntry) -> AppResult<()>;
    fn update_entry(&self, entry: &WatchHistoryEntry) -> AppResult<()>;
    fn get_recent(&self, limit: usize) -> AppResult<Vec<Movie>>;
    fn list_entries_for_movie(&self, movie_id: &Uuid) -> AppResult<Vec<WatchHistoryEntry>>;
}

pub trait WatchlistRepository: Send + Sync {
    fn add_to_watchlist(&self, movie_id: &Uuid) -> AppResult<()>;
    fn remove_from_watchlist(&self, movie_id: &Uuid) -> AppResult<()>;
    fn is_in_watchlist(&self, movie_id: &Uuid) -> AppResult<bool>;
    fn list_watchlist(&self) -> AppResult<Vec<Movie>>;
}

pub trait SettingsRepository: Send + Sync {
    fn get_settings(&self) -> AppResult<AppSettings>;
    fn save_settings(&self, settings: &AppSettings) -> AppResult<()>;
}
