use uuid::Uuid;
use crate::domain::{SeriesDetails, TvEpisode, TvSeason, TvSeries};
use crate::error::AppResult;

pub trait TvRepository: Send + Sync {
    fn upsert_series(&self, series: &TvSeries) -> AppResult<()>;
    fn get_series(&self, id: &Uuid) -> AppResult<Option<TvSeries>>;
    fn list_series(&self) -> AppResult<Vec<TvSeries>>;
    fn find_series_by_title(&self, title: &str) -> AppResult<Option<TvSeries>>;
    fn delete_series(&self, id: &Uuid) -> AppResult<()>;

    fn upsert_season(&self, season: &TvSeason) -> AppResult<()>;
    fn get_season(&self, id: &Uuid) -> AppResult<Option<TvSeason>>;
    fn list_seasons_by_series(&self, series_id: &Uuid) -> AppResult<Vec<TvSeason>>;

    fn upsert_episode(&self, episode: &TvEpisode) -> AppResult<()>;
    fn get_episode(&self, id: &Uuid) -> AppResult<Option<TvEpisode>>;
    fn list_episodes_by_season(&self, season_id: &Uuid) -> AppResult<Vec<TvEpisode>>;
    fn get_series_details(&self, series_id: &Uuid) -> AppResult<Option<SeriesDetails>>;
    fn get_next_episode(&self, series_id: &Uuid, season_number: u32, episode_number: u32) -> AppResult<Option<TvEpisode>>;
}
