pub mod abstractions;
pub mod adapters;
pub mod commands;
pub mod domain;
pub mod error;
pub mod events;
pub mod services;

use std::sync::{Arc, Mutex};
use tauri::Manager;

use abstractions::{
    LibrarySourceRepository, MediaRepository, MediaSource, MovieRepository,
    PlaybackStateRepository, ProgressRepository, SettingsRepository, TvRepository,
    WatchHistoryRepository, WatchlistRepository,
};
use adapters::{
    LocalFileSystemSource, OpenMovieMetadataProvider, SqliteDb, SqliteLibrarySourceRepository,
    SqliteMediaRepository, SqliteMovieRepository, SqlitePlaybackStateRepository,
    SqliteProgressRepository, SqliteSettingsRepository, SqliteTvRepository,
    SqliteWatchHistoryRepository, SqliteWatchlistRepository, VlcMediaPlayer,
};
use services::{
    DuplicateResolver, HistoryService, LibraryService, MediaAnalyzer, MetadataResolver,
    PlaybackService, Scanner, SearchService, SettingsService, TmdbService, WatchlistService,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Determine storage paths
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap().join(".moviestream"));
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir.join("moviestream.db");
            let artwork_cache_dir = app_data_dir.join("artworks");
            std::fs::create_dir_all(&artwork_cache_dir).ok();

            // Initialize SQLite DB
            let sqlite_db = SqliteDb::new(&db_path).expect("Failed to initialize SQLite database");

            // Initialize repositories
            let movie_repo: Arc<dyn MovieRepository> =
                Arc::new(SqliteMovieRepository::new(sqlite_db.clone()));
            let media_repo: Arc<dyn MediaRepository> =
                Arc::new(SqliteMediaRepository::new(sqlite_db.clone()));
            let source_repo: Arc<dyn LibrarySourceRepository> =
                Arc::new(SqliteLibrarySourceRepository::new(sqlite_db.clone()));
            let playback_repo: Arc<dyn PlaybackStateRepository> =
                Arc::new(SqlitePlaybackStateRepository::new(sqlite_db.clone()));
            let history_repo: Arc<dyn WatchHistoryRepository> =
                Arc::new(SqliteWatchHistoryRepository::new(sqlite_db.clone()));
            let watchlist_repo: Arc<dyn WatchlistRepository> =
                Arc::new(SqliteWatchlistRepository::new(sqlite_db.clone()));
            let settings_repo: Arc<dyn SettingsRepository> =
                Arc::new(SqliteSettingsRepository::new(sqlite_db.clone()));
            let tv_repo: Arc<dyn TvRepository> =
                Arc::new(SqliteTvRepository::new(sqlite_db.clone()));
            let progress_repo: Arc<dyn ProgressRepository> =
                Arc::new(SqliteProgressRepository::new(sqlite_db));

            // Initialize adapters & services
            let media_source: Arc<dyn MediaSource> = Arc::new(LocalFileSystemSource::new());
            let metadata_provider = Arc::new(OpenMovieMetadataProvider::new(None));
            let vlc_player = Arc::new(Mutex::new(Box::new(VlcMediaPlayer::new()) as Box<dyn abstractions::MediaPlayer>));
            let tmdb_service = Arc::new(TmdbService::new(None));

            let media_analyzer = Arc::new(MediaAnalyzer::new());
            let metadata_resolver = Arc::new(MetadataResolver::new(metadata_provider, artwork_cache_dir));
            let duplicate_resolver = Arc::new(DuplicateResolver::new(movie_repo.clone()));

            let scanner = Arc::new(Scanner::with_tv_support(
                source_repo.clone(),
                media_repo.clone(),
                movie_repo.clone(),
                tv_repo.clone(),
                tmdb_service.clone(),
                media_source,
                media_analyzer,
                metadata_resolver.clone(),
                duplicate_resolver,
            ));

            let library_service = Arc::new(LibraryService::new(
                source_repo.clone(),
                media_repo.clone(),
                movie_repo.clone(),
                scanner,
                metadata_resolver,
            ));

            let stream_server = tauri::async_runtime::block_on(async {
                crate::services::MediaStreamServer::start().await.expect("Failed to start MediaStreamServer")
            });

            let playback_service = Arc::new(PlaybackService::with_full_engine(
                vlc_player,
                playback_repo.clone(),
                history_repo.clone(),
                progress_repo.clone(),
                tv_repo.clone(),
                stream_server.clone(),
            ));

            let search_service = Arc::new(SearchService::new(movie_repo.clone()));
            let watchlist_service = Arc::new(WatchlistService::new(watchlist_repo));
            let history_service = Arc::new(HistoryService::new(history_repo, playback_repo));
            let settings_service = Arc::new(SettingsService::new(settings_repo));

            // Manage services in state
            app.manage(library_service);
            app.manage(playback_service);
            app.manage(search_service);
            app.manage(watchlist_service);
            app.manage(history_service);
            app.manage(settings_service);
            app.manage(media_repo);
            app.manage(movie_repo);
            app.manage(tv_repo);
            app.manage(progress_repo);
            app.manage(tmdb_service);
            app.manage(stream_server);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Library & Movies
            commands::library::add_source,
            commands::library::pick_and_add_source,
            commands::library::remove_source,
            commands::library::list_sources,
            commands::library::rescan_source,
            commands::library::rescan_all,
            commands::movies::list_movies,
            commands::movies::get_movie,
            commands::movies::search_movies,
            commands::movies::set_metadata_match,

            // TMDB Secure Endpoints
            commands::tmdb::search_tmdb_movies,
            commands::tmdb::get_movie_details,
            commands::tmdb::search_tv,
            commands::tmdb::get_tv_details,
            commands::tmdb::get_tv_seasons,
            commands::tmdb::get_tv_episodes,
            commands::tmdb::get_trending,
            commands::tmdb::get_popular_movies,
            commands::tmdb::get_popular_tv,

            // TV Series & Episodes
            commands::tv::list_tv_series,
            commands::tv::get_series_details,
            commands::tv::start_episode_playback,

            // Playback Progress & Continue Watching
            commands::progress::get_continue_watching,
            commands::progress::get_playback_progress,
            commands::progress::mark_media_completed,
            commands::progress::get_next_episode,

            // Playback Controls
            commands::playback::start_playback,
            commands::playback::play,
            commands::playback::pause,
            commands::playback::stop,
            commands::playback::seek,
            commands::playback::resume_at,
            commands::playback::set_volume,
            commands::playback::set_mute,
            commands::playback::set_fullscreen,
            commands::playback::set_playback_speed,
            commands::playback::select_audio_track,
            commands::playback::select_subtitle_track,
            commands::playback::load_external_subtitle,
            commands::playback::get_active_session,
            commands::playback::record_position,

            // Watchlist, History, Settings
            commands::watchlist::add_to_watchlist,
            commands::watchlist::remove_from_watchlist,
            commands::watchlist::is_in_watchlist,
            commands::watchlist::list_watchlist,
            commands::history::recently_watched,
            commands::history::continue_watching,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MovieStream application");
}
