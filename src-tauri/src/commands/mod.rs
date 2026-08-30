pub mod history;
pub mod library;
pub mod movies;
pub mod playback;
pub mod progress;
pub mod settings;
pub mod tmdb;
pub mod tv;
pub mod watchlist;

pub use history::{continue_watching, recently_watched, ContinueWatchingItem};
pub use library::{add_source, list_sources, pick_and_add_source, remove_source, rescan_all, rescan_source};
pub use movies::{get_movie, list_movies, search_movies, set_metadata_match, MovieWithMedia};
pub use playback::{
    get_active_session, load_external_subtitle, pause, play, record_position, resume_at, seek,
    select_audio_track, select_subtitle_track, set_fullscreen, set_mute, set_playback_speed,
    set_volume, start_playback, stop,
};
pub use progress::{get_continue_watching, get_next_episode, get_playback_progress, mark_media_completed};
pub use settings::{get_settings, update_settings};
pub use tmdb::{
    get_movie_details, get_popular_movies, get_popular_tv, get_trending, get_tv_details,
    get_tv_episodes, get_tv_seasons, search_tmdb_movies, search_tv,
};
pub use tv::{get_series_details, list_tv_series, start_episode_playback};
pub use watchlist::{add_to_watchlist, is_in_watchlist, list_watchlist, remove_from_watchlist};
