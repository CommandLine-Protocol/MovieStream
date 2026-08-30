use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::MetadataStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvSeries {
    pub id: Uuid,
    pub tmdb_id: Option<i64>,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<u16>,
    pub description: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub genres: Vec<String>,
    pub rating: Option<f32>,
    pub metadata_provider_id: Option<String>,
    pub metadata_status: MetadataStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvSeason {
    pub id: Uuid,
    pub series_id: Uuid,
    pub season_number: u32,
    pub name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub episode_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvEpisode {
    pub id: Uuid,
    pub series_id: Uuid,
    pub season_id: Uuid,
    pub season_number: u32,
    pub episode_number: u32,
    pub title: String,
    pub overview: Option<String>,
    pub still_path: Option<String>,
    pub air_date: Option<String>,
    pub duration_seconds: Option<u32>,
    pub rating: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeWithMedia {
    pub episode: TvEpisode,
    pub media_path: Option<String>,
    pub media_id: Option<Uuid>,
    pub progress_seconds: u32,
    pub duration_seconds: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonWithEpisodes {
    pub season: TvSeason,
    pub episodes: Vec<EpisodeWithMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesDetails {
    pub series: TvSeries,
    pub seasons: Vec<SeasonWithEpisodes>,
    pub total_episodes: u32,
}
