use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    Available,
    Unavailable,
    Scanning,
    Indexing,
    Inaccessible,
    Disconnected,
}

impl Default for SourceStatus {
    fn default() -> Self {
        SourceStatus::Available
    }
}

impl std::fmt::Display for SourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceStatus::Available => write!(f, "available"),
            SourceStatus::Unavailable => write!(f, "unavailable"),
            SourceStatus::Scanning => write!(f, "scanning"),
            SourceStatus::Indexing => write!(f, "indexing"),
            SourceStatus::Inaccessible => write!(f, "inaccessible"),
            SourceStatus::Disconnected => write!(f, "disconnected"),
        }
    }
}

impl std::str::FromStr for SourceStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "available" => Ok(SourceStatus::Available),
            "unavailable" => Ok(SourceStatus::Unavailable),
            "scanning" => Ok(SourceStatus::Scanning),
            "indexing" => Ok(SourceStatus::Indexing),
            "inaccessible" => Ok(SourceStatus::Inaccessible),
            "disconnected" => Ok(SourceStatus::Disconnected),
            _ => Ok(SourceStatus::Available),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibrarySource {
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub status: SourceStatus,
    pub last_scanned_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
