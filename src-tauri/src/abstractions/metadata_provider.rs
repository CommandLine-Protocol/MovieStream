use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataCandidate {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<u16>,
    pub overview: Option<String>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieMetadata {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<u16>,
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub cast: Vec<String>,
    pub director: Option<String>,
    pub rating: Option<f32>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
}

pub trait MetadataProvider: Send + Sync {
    fn name(&self) -> &str;
    fn search(&self, title_guess: &str, year_guess: Option<u16>) -> AppResult<Vec<MetadataCandidate>>;
    fn fetch_details(&self, provider_id: &str) -> AppResult<MovieMetadata>;
    fn fetch_image(&self, url: &str) -> AppResult<Vec<u8>>;
}
