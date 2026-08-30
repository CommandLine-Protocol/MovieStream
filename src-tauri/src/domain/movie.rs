use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataStatus {
    Unmatched,
    AutoMatched,
    ManuallyMatched,
    Failed,
}

impl Default for MetadataStatus {
    fn default() -> Self {
        MetadataStatus::Unmatched
    }
}

impl std::fmt::Display for MetadataStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataStatus::Unmatched => write!(f, "unmatched"),
            MetadataStatus::AutoMatched => write!(f, "auto_matched"),
            MetadataStatus::ManuallyMatched => write!(f, "manually_matched"),
            MetadataStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for MetadataStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unmatched" => Ok(MetadataStatus::Unmatched),
            "auto_matched" => Ok(MetadataStatus::AutoMatched),
            "manually_matched" => Ok(MetadataStatus::ManuallyMatched),
            "failed" => Ok(MetadataStatus::Failed),
            _ => Ok(MetadataStatus::Unmatched),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Movie {
    pub id: Uuid,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<u16>,
    pub description: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub genres: Vec<String>,
    pub cast: Vec<String>,
    pub director: Option<String>,
    pub rating: Option<f32>,
    pub metadata_provider_id: Option<String>,
    pub metadata_status: MetadataStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MovieFilter {
    pub genre: Option<String>,
    pub year: Option<u16>,
    pub watched: Option<bool>,
    pub in_watchlist: Option<bool>,
    pub source_id: Option<Uuid>,
    pub min_rating: Option<f32>,
    pub is_available: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MovieSort {
    TitleAsc,
    TitleDesc,
    YearAsc,
    YearDesc,
    DateAddedAsc,
    DateAddedDesc,
    RecentlyWatched,
    RatingDesc,
}

impl Default for MovieSort {
    fn default() -> Self {
        MovieSort::TitleAsc
    }
}
