use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatchlistEntry {
    pub movie_id: Uuid,
    pub added_at: DateTime<Utc>,
}
