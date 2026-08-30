use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_COMPLETION_THRESHOLD: f32 = 0.90;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaybackState {
    pub movie_id: Uuid,
    pub media_id: Uuid,
    pub position_seconds: u32,
    pub duration_seconds: u32,
    pub completed: bool,
    pub updated_at: DateTime<Utc>,
}

impl PlaybackState {
    pub fn completion_ratio(&self) -> f32 {
        if self.duration_seconds == 0 {
            0.0
        } else {
            self.position_seconds as f32 / self.duration_seconds as f32
        }
    }

    pub fn is_effectively_completed(&self, threshold: f32) -> bool {
        self.completed || self.completion_ratio() >= threshold
    }
}
