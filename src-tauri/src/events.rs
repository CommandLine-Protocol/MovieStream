use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Movie, SourceStatus};

pub const EVENT_SCAN_PROGRESS: &str = "library://scan-progress";
pub const EVENT_SOURCE_STATUS_CHANGED: &str = "library://source-status-changed";
pub const EVENT_MOVIE_ADDED: &str = "library://movie-added";
pub const EVENT_PLAYBACK_POSITION: &str = "playback://position";
pub const EVENT_PLAYBACK_ERROR: &str = "playback://error";
pub const EVENT_PLAYBACK_STATE: &str = "playback://state";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgressPayload {
    pub source_id: Uuid,
    pub files_discovered: u32,
    pub movies_identified: u32,
    pub phase: String, // "scanning" | "analyzing" | "matching" | "completed" | "error"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStatusPayload {
    pub source_id: Uuid,
    pub status: SourceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieAddedPayload {
    pub movie: Movie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPositionPayload {
    pub movie_id: Uuid,
    pub media_id: Uuid,
    pub position_seconds: u32,
    pub duration_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackErrorPayload {
    pub movie_id: Option<Uuid>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStatePayload {
    pub state: String, // "playing" | "paused" | "stopped" | "ended"
}
