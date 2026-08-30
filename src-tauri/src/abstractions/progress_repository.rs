use uuid::Uuid;

use crate::domain::{ContinueWatchingItem, MediaProgress};
use crate::error::AppResult;

pub trait ProgressRepository: Send + Sync {
    fn save_progress(&self, progress: &MediaProgress) -> AppResult<()>;
    fn get_progress_by_media(&self, media_id: &Uuid) -> AppResult<Option<MediaProgress>>;
    fn get_progress_by_movie(&self, movie_id: &Uuid) -> AppResult<Option<MediaProgress>>;
    fn get_progress_by_episode(&self, episode_id: &Uuid) -> AppResult<Option<MediaProgress>>;
    fn mark_completed(&self, media_id: &Uuid) -> AppResult<()>;
    fn get_continue_watching(&self, limit: usize) -> AppResult<Vec<ContinueWatchingItem>>;
    fn delete_progress(&self, media_id: &Uuid) -> AppResult<()>;
}
