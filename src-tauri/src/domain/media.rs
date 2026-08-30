use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaAvailability {
    Available,
    Unavailable,
}

impl Default for MediaAvailability {
    fn default() -> Self {
        MediaAvailability::Available
    }
}

impl std::fmt::Display for MediaAvailability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAvailability::Available => write!(f, "available"),
            MediaAvailability::Unavailable => write!(f, "unavailable"),
        }
    }
}

impl std::str::FromStr for MediaAvailability {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "available" => Ok(MediaAvailability::Available),
            "unavailable" => Ok(MediaAvailability::Unavailable),
            _ => Ok(MediaAvailability::Available),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioTrackInfo {
    pub id: String,
    pub name: String,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub channels: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubtitleTrackInfo {
    pub id: String,
    pub name: String,
    pub language: Option<String>,
    pub is_external: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Media {
    pub id: Uuid,
    pub movie_id: Option<Uuid>,
    pub episode_id: Option<Uuid>,
    pub source_id: Uuid,
    pub path: String,
    pub size_bytes: u64,
    pub duration_seconds: Option<u32>,
    pub container_format: Option<String>,
    pub video_codec: Option<String>,
    pub resolution_width: Option<u32>,
    pub resolution_height: Option<u32>,
    pub audio_tracks: Vec<AudioTrackInfo>,
    pub subtitle_tracks: Vec<SubtitleTrackInfo>,
    pub file_hash: Option<String>,
    pub file_mtime: DateTime<Utc>,
    pub availability: MediaAvailability,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
