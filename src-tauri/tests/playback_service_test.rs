use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use uuid::Uuid;

use moviestream_lib::abstractions::{
    LibrarySourceRepository, MediaRepository, MediaPlayer, MovieRepository,
    PlaybackStateRepository, WatchHistoryRepository,
};
use moviestream_lib::adapters::{
    SqliteDb, SqliteLibrarySourceRepository, SqliteMediaRepository, SqliteMovieRepository,
    SqlitePlaybackStateRepository, SqliteWatchHistoryRepository, VlcMediaPlayer,
};
use moviestream_lib::domain::{
    LibrarySource, Media, MediaAvailability, MetadataStatus, Movie, SourceStatus,
};
use moviestream_lib::services::PlaybackService;

#[test]
fn test_playback_service_session_resume_and_completion() {
    let tmp = tempdir().unwrap();
    let media_file = tmp.path().join("test_video.mkv");
    std::fs::write(&media_file, b"fake video bytes").unwrap();

    let db = SqliteDb::new_in_memory().unwrap();
    let source_repo = Arc::new(SqliteLibrarySourceRepository::new(db.clone())) as Arc<dyn LibrarySourceRepository>;
    let movie_repo = Arc::new(SqliteMovieRepository::new(db.clone())) as Arc<dyn MovieRepository>;
    let media_repo = Arc::new(SqliteMediaRepository::new(db.clone())) as Arc<dyn MediaRepository>;
    let playback_repo = Arc::new(SqlitePlaybackStateRepository::new(db.clone())) as Arc<dyn PlaybackStateRepository>;
    let history_repo = Arc::new(SqliteWatchHistoryRepository::new(db)) as Arc<dyn WatchHistoryRepository>;

    let source_id = Uuid::new_v4();
    source_repo
        .upsert_source(&LibrarySource {
            id: source_id,
            path: tmp.path().to_string_lossy().to_string(),
            name: "Test".to_string(),
            status: SourceStatus::Available,
            last_scanned_at: None,
            created_at: chrono::Utc::now(),
        })
        .unwrap();

    let movie_id = Uuid::new_v4();
    let media_id = Uuid::new_v4();

    movie_repo
        .upsert_movie(&Movie {
            id: movie_id,
            title: "Test Movie".to_string(),
            original_title: None,
            year: Some(2024),
            description: None,
            poster_path: None,
            backdrop_path: None,
            genres: vec!["Action".to_string()],
            cast: vec![],
            director: None,
            rating: None,
            metadata_provider_id: None,
            metadata_status: MetadataStatus::AutoMatched,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .unwrap();

    media_repo
        .upsert_media(&Media {
            id: media_id,
            movie_id: Some(movie_id),
            episode_id: None,
            source_id,
            path: media_file.to_string_lossy().to_string(),
            size_bytes: 1000,
            duration_seconds: Some(7200),
            container_format: Some("mkv".to_string()),
            video_codec: Some("AVC".to_string()),
            resolution_width: Some(1920),
            resolution_height: Some(1080),
            audio_tracks: vec![],
            subtitle_tracks: vec![],
            file_hash: None,
            file_mtime: chrono::Utc::now(),
            availability: MediaAvailability::Available,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .unwrap();

    let player = Arc::new(Mutex::new(Box::new(VlcMediaPlayer::new()) as Box<dyn MediaPlayer>));
    let playback_service = PlaybackService::new(player, playback_repo.clone(), history_repo.clone());

    // 1. Start playback from beginning
    let session = playback_service
        .start(movie_id, media_id, &media_file.to_string_lossy())
        .unwrap();
    assert_eq!(session.movie_id, Some(movie_id));
    assert_eq!(session.position_seconds, 0);
    assert!(!session.requires_resume_prompt);

    // 2. Simulate progress tick to 1800s (30 minutes in)
    playback_service.record_position_tick(1800).unwrap();

    let state = playback_repo.get_state(&movie_id).unwrap().unwrap();
    assert_eq!(state.position_seconds, 1800);
    assert!(!state.completed);

    // 3. Stop playback
    playback_service.stop().unwrap();

    // 4. Start playback again -> Should detect resume point and require resume prompt
    let session2 = playback_service
        .start(movie_id, media_id, &media_file.to_string_lossy())
        .unwrap();
    assert!(session2.requires_resume_prompt);
    assert_eq!(session2.resume_position_seconds, 1800);

    // 5. User accepts resume -> seek to 1800s
    playback_service.resume_from(session2.resume_position_seconds).unwrap();
    let active = playback_service.get_active_session().unwrap();
    assert_eq!(active.position_seconds, 1800);
    assert!(active.is_playing);

    // 6. Simulate watching past completion threshold (95% of 7200s duration = 6840s)
    playback_service.record_position_tick(7000).unwrap();
    let completed_state = playback_repo.get_state(&movie_id).unwrap().unwrap();
    assert!(completed_state.completed);

    let recent = history_repo.get_recent(5).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].id, movie_id);
}
