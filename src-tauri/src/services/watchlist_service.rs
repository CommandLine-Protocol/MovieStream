use std::sync::Arc;
use uuid::Uuid;

use crate::abstractions::WatchlistRepository;
use crate::domain::Movie;
use crate::error::AppResult;

pub struct WatchlistService {
    watchlist_repo: Arc<dyn WatchlistRepository>,
}

impl WatchlistService {
    pub fn new(watchlist_repo: Arc<dyn WatchlistRepository>) -> Self {
        Self { watchlist_repo }
    }

    pub fn add(&self, movie_id: &Uuid) -> AppResult<()> {
        self.watchlist_repo.add_to_watchlist(movie_id)
    }

    pub fn remove(&self, movie_id: &Uuid) -> AppResult<()> {
        self.watchlist_repo.remove_from_watchlist(movie_id)
    }

    pub fn is_in_watchlist(&self, movie_id: &Uuid) -> AppResult<bool> {
        self.watchlist_repo.is_in_watchlist(movie_id)
    }

    pub fn list(&self) -> AppResult<Vec<Movie>> {
        self.watchlist_repo.list_watchlist()
    }
}
