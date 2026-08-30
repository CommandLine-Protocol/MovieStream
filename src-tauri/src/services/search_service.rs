use std::sync::Arc;

use crate::abstractions::MovieRepository;
use crate::domain::Movie;
use crate::error::AppResult;

pub struct SearchService {
    movie_repo: Arc<dyn MovieRepository>,
}

impl SearchService {
    pub fn new(movie_repo: Arc<dyn MovieRepository>) -> Self {
        Self { movie_repo }
    }

    pub fn search(&self, query: &str) -> AppResult<Vec<Movie>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        self.movie_repo.search_movies(query)
    }
}
