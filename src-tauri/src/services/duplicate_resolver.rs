use std::sync::Arc;
use uuid::Uuid;

use crate::abstractions::MovieRepository;
use crate::domain::Movie;
use crate::error::AppResult;

pub struct DuplicateResolver {
    movie_repo: Arc<dyn MovieRepository>,
}

impl DuplicateResolver {
    pub fn new(movie_repo: Arc<dyn MovieRepository>) -> Self {
        Self { movie_repo }
    }

    pub fn match_or_create_movie(&self, candidate_movie: Movie) -> AppResult<Uuid> {
        // 1. Check if movie already exists with same metadata_provider_id
        if let Some(ref provider_id) = candidate_movie.metadata_provider_id {
            if let Some(existing) = self.movie_repo.find_by_provider_id(provider_id)? {
                return Ok(existing.id);
            }
        }

        // 2. Check if movie already exists with same title & year
        if let Some(existing) = self.movie_repo.find_by_title_year(&candidate_movie.title, candidate_movie.year)? {
            return Ok(existing.id);
        }

        // 3. Otherwise, create new Movie record
        let movie_id = candidate_movie.id;
        self.movie_repo.upsert_movie(&candidate_movie)?;
        Ok(movie_id)
    }
}
