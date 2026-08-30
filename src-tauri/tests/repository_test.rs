use chrono::Utc;
use uuid::Uuid;

use moviestream_lib::abstractions::{
    LibrarySourceRepository, MediaRepository, MovieRepository, PlaybackStateRepository,
    SettingsRepository, WatchHistoryRepository, WatchlistRepository,
};
use moviestream_lib::adapters::{
    SqliteDb, SqliteLibrarySourceRepository, SqliteMediaRepository, SqliteMovieRepository,
    SqlitePlaybackStateRepository, SqliteSettingsRepository, SqliteWatchHistoryRepository,
    SqliteWatchlistRepository,
};
use moviestream_lib::domain::{
    LibrarySource, Media, MediaAvailability, MetadataStatus, Movie, MovieFilter,
    MovieSort, PlaybackState, SourceStatus, WatchHistoryEntry,
};

#[test]
fn test_movie_and_media_crud_and_relationships() {
    let db = SqliteDb::new_in_memory().expect("in-memory db failed");
    let movie_repo = SqliteMovieRepository::new(db.clone());
    let media_repo = SqliteMediaRepository::new(db.clone());
    let source_repo = SqliteLibrarySourceRepository::new(db);

    let source_id = Uuid::new_v4();
    let source = LibrarySource {
        id: source_id,
        path: "/test/movies".to_string(),
        name: "Test Movies".to_string(),
        status: SourceStatus::Available,
        last_scanned_at: None,
        created_at: Utc::now(),
    };
    source_repo.upsert_source(&source).unwrap();

    let movie_id = Uuid::new_v4();
    let movie = Movie {
        id: movie_id,
        title: "Interstellar".to_string(),
        original_title: Some("Interstellar".to_string()),
        year: Some(2014),
        description: Some("A team of explorers travel through a wormhole...".to_string()),
        poster_path: None,
        backdrop_path: None,
        genres: vec!["Sci-Fi".to_string(), "Adventure".to_string()],
        cast: vec!["Matthew McConaughey".to_string(), "Anne Hathaway".to_string()],
        director: Some("Christopher Nolan".to_string()),
        rating: Some(8.7),
        metadata_provider_id: Some("157336".to_string()),
        metadata_status: MetadataStatus::AutoMatched,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    movie_repo.upsert_movie(&movie).unwrap();

    // Attach 1080p media
    let media1_id = Uuid::new_v4();
    let media1 = Media {
        id: media1_id,
        movie_id: Some(movie_id),
        episode_id: None,
        source_id,
        path: "/test/movies/Interstellar.2014.1080p.mkv".to_string(),
        size_bytes: 8_000_000_000,
        duration_seconds: Some(10140),
        container_format: Some("mkv".to_string()),
        video_codec: Some("AVC".to_string()),
        resolution_width: Some(1920),
        resolution_height: Some(1080),
        audio_tracks: vec![],
        subtitle_tracks: vec![],
        file_hash: None,
        file_mtime: Utc::now(),
        availability: MediaAvailability::Available,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    media_repo.upsert_media(&media1).unwrap();

    // Attach 2160p media (multi-version)
    let media2_id = Uuid::new_v4();
    let media2 = Media {
        id: media2_id,
        movie_id: Some(movie_id),
        episode_id: None,
        source_id,
        path: "/test/movies/Interstellar.2014.2160p.mkv".to_string(),
        size_bytes: 35_000_000_000,
        duration_seconds: Some(10140),
        container_format: Some("mkv".to_string()),
        video_codec: Some("HEVC".to_string()),
        resolution_width: Some(3840),
        resolution_height: Some(2160),
        audio_tracks: vec![],
        subtitle_tracks: vec![],
        file_hash: None,
        file_mtime: Utc::now(),
        availability: MediaAvailability::Available,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    media_repo.upsert_media(&media2).unwrap();

    let fetched_movie = movie_repo.get_movie(&movie_id).unwrap().unwrap();
    assert_eq!(fetched_movie.title, "Interstellar");
    assert_eq!(fetched_movie.genres.len(), 2);

    let movie_media = media_repo.list_media_for_movie(&movie_id).unwrap();
    assert_eq!(movie_media.len(), 2);
    assert_eq!(movie_media[0].resolution_height, Some(2160)); // Ordered by resolution desc

    // Test search
    let search_results = movie_repo.search_movies("inter").unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].id, movie_id);

    let search_by_director = movie_repo.search_movies("Nolan").unwrap();
    assert_eq!(search_by_director.len(), 1);

    // Test filter
    let filter_genre = MovieFilter {
        genre: Some("Sci-Fi".to_string()),
        ..Default::default()
    };
    let genre_movies = movie_repo.list_movies(&filter_genre, MovieSort::TitleAsc).unwrap();
    assert_eq!(genre_movies.len(), 1);
}

#[test]
fn test_playback_and_watch_history() {
    let db = SqliteDb::new_in_memory().expect("in-memory db failed");
    let movie_repo = SqliteMovieRepository::new(db.clone());
    let media_repo = SqliteMediaRepository::new(db.clone());
    let source_repo = SqliteLibrarySourceRepository::new(db.clone());
    let playback_repo = SqlitePlaybackStateRepository::new(db.clone());
    let history_repo = SqliteWatchHistoryRepository::new(db);

    let source_id = Uuid::new_v4();
    source_repo
        .upsert_source(&LibrarySource {
            id: source_id,
            path: "/media/films".to_string(),
            name: "Films".to_string(),
            status: SourceStatus::Available,
            last_scanned_at: None,
            created_at: Utc::now(),
        })
        .unwrap();

    let movie_id = Uuid::new_v4();
    movie_repo
        .upsert_movie(&Movie {
            id: movie_id,
            title: "Dune".to_string(),
            original_title: None,
            year: Some(2021),
            description: None,
            poster_path: None,
            backdrop_path: None,
            genres: vec!["Sci-Fi".to_string()],
            cast: vec![],
            director: Some("Denis Villeneuve".to_string()),
            rating: Some(8.0),
            metadata_provider_id: None,
            metadata_status: MetadataStatus::AutoMatched,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .unwrap();

    let media_id = Uuid::new_v4();
    media_repo
        .upsert_media(&Media {
            id: media_id,
            movie_id: Some(movie_id),
            episode_id: None,
            source_id,
            path: "/media/films/Dune.2021.mkv".to_string(),
            size_bytes: 5000,
            duration_seconds: Some(9300),
            container_format: Some("mkv".to_string()),
            video_codec: Some("AVC".to_string()),
            resolution_width: Some(1920),
            resolution_height: Some(1080),
            audio_tracks: vec![],
            subtitle_tracks: vec![],
            file_hash: None,
            file_mtime: Utc::now(),
            availability: MediaAvailability::Available,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .unwrap();

    let state = PlaybackState {
        movie_id,
        media_id,
        position_seconds: 3600,
        duration_seconds: 9300,
        completed: false,
        updated_at: Utc::now(),
    };
    playback_repo.upsert_state(&state).unwrap();

    let fetched_state = playback_repo.get_state(&movie_id).unwrap().unwrap();
    assert_eq!(fetched_state.position_seconds, 3600);
    assert!(!fetched_state.completed);

    let in_progress = playback_repo.list_in_progress().unwrap();
    assert_eq!(in_progress.len(), 1);
    assert_eq!(in_progress[0].0.title, "Dune");

    let history_entry = WatchHistoryEntry {
        id: Uuid::new_v4(),
        movie_id,
        started_at: Utc::now(),
        completed_at: None,
        last_position_seconds: 3600,
    };
    history_repo.add_entry(&history_entry).unwrap();

    let recent = history_repo.get_recent(10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].title, "Dune");
}

#[test]
fn test_watchlist_and_settings() {
    let db = SqliteDb::new_in_memory().expect("in-memory db failed");
    let movie_repo = SqliteMovieRepository::new(db.clone());
    let watchlist_repo = SqliteWatchlistRepository::new(db.clone());
    let settings_repo = SqliteSettingsRepository::new(db);

    let movie_id = Uuid::new_v4();
    movie_repo
        .upsert_movie(&Movie {
            id: movie_id,
            title: "Oppenheimer".to_string(),
            original_title: None,
            year: Some(2023),
            description: None,
            poster_path: None,
            backdrop_path: None,
            genres: vec!["Biography".to_string(), "Drama".to_string()],
            cast: vec![],
            director: Some("Christopher Nolan".to_string()),
            rating: Some(8.9),
            metadata_provider_id: None,
            metadata_status: MetadataStatus::AutoMatched,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .unwrap();

    assert!(!watchlist_repo.is_in_watchlist(&movie_id).unwrap());
    watchlist_repo.add_to_watchlist(&movie_id).unwrap();
    assert!(watchlist_repo.is_in_watchlist(&movie_id).unwrap());

    let wl = watchlist_repo.list_watchlist().unwrap();
    assert_eq!(wl.len(), 1);
    assert_eq!(wl[0].title, "Oppenheimer");

    watchlist_repo.remove_from_watchlist(&movie_id).unwrap();
    assert!(!watchlist_repo.is_in_watchlist(&movie_id).unwrap());

    // Settings
    let mut settings = settings_repo.get_settings().unwrap();
    assert_eq!(settings.playback.default_volume, 80);
    settings.playback.default_volume = 95;
    settings_repo.save_settings(&settings).unwrap();

    let updated_settings = settings_repo.get_settings().unwrap();
    assert_eq!(updated_settings.playback.default_volume, 95);
}
