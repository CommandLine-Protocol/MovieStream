use std::sync::Arc;

use crate::abstractions::{PlaybackStateRepository, WatchHistoryRepository};
use crate::domain::{Movie, PlaybackState};
use crate::error::AppResult;

pub struct HistoryService {
    history_repo: Arc<dyn WatchHistoryRepository>,
    playback_repo: Arc<dyn PlaybackStateRepository>,
}

impl HistoryService {
    pub fn new(
        history_repo: Arc<dyn WatchHistoryRepository>,
        playback_repo: Arc<dyn PlaybackStateRepository>,
    ) -> Self {
        Self {
            history_repo,
            playback_repo,
        }
    }

    pub fn recently_watched(&self, limit: usize) -> AppResult<Vec<Movie>> {
        self.history_repo.get_recent(limit)
    }

    pub fn continue_watching(&self) -> AppResult<Vec<(Movie, PlaybackState)>> {
        self.playback_repo.list_in_progress()
    }
}
