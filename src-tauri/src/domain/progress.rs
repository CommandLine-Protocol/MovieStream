use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Movie,
    Episode,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Movie => write!(f, "movie"),
            MediaType::Episode => write!(f, "episode"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProgress {
    pub id: Uuid,
    pub media_type: MediaType,
    pub media_id: Uuid,
    pub movie_id: Option<Uuid>,
    pub series_id: Option<Uuid>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub episode_id: Option<Uuid>,
    pub position_seconds: u32,
    pub duration_seconds: u32,
    pub progress_percentage: f32,
    pub completed: bool,
    pub last_watched: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinueWatchingItem {
    pub progress: MediaProgress,
    // Movie fields (if movie)
    pub movie_title: Option<String>,
    pub movie_poster: Option<String>,
    pub movie_backdrop: Option<String>,
    pub movie_year: Option<u16>,
    // TV Episode fields (if TV series)
    pub series_title: Option<String>,
    pub series_poster: Option<String>,
    pub episode_title: Option<String>,
    pub episode_still: Option<String>,
}
