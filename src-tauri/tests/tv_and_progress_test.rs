use chrono::Utc;
use moviestream_lib::abstractions::{
    LibrarySourceRepository, MediaRepository, ProgressRepository, TvRepository,
};
use moviestream_lib::adapters::{
    SqliteDb, SqliteMediaRepository, SqliteProgressRepository, SqliteTvRepository,
};
use moviestream_lib::domain::{
    Media, MediaAvailability, MediaProgress, MediaType, MetadataStatus, TvEpisode, TvSeason,
    TvSeries,
};
use uuid::Uuid;

#[test]
fn test_tv_series_season_episode_and_progress_flow() {
    let db = SqliteDb::new_in_memory().expect("in-memory sqlite db");
    let tv_repo = SqliteTvRepository::new(db.clone());
    let media_repo = SqliteMediaRepository::new(db.clone());
    let progress_repo = SqliteProgressRepository::new(db.clone());

    let series_id = Uuid::new_v4();
    let season_id = Uuid::new_v4();
    let ep1_id = Uuid::new_v4();
    let ep2_id = Uuid::new_v4();

    // 1. Insert TV Series
    let series = TvSeries {
        id: series_id,
        tmdb_id: Some(1399),
        title: "Game of Thrones".to_string(),
        original_title: None,
        year: Some(2011),
        description: Some("Nine noble families fight for control over the lands of Westeros.".to_string()),
        poster_path: None,
        backdrop_path: None,
        genres: vec!["Drama".to_string(), "Sci-Fi & Fantasy".to_string()],
        rating: Some(8.4),
        metadata_provider_id: Some("tmdb-1399".to_string()),
        metadata_status: MetadataStatus::AutoMatched,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    tv_repo.upsert_series(&series).unwrap();

    // 2. Insert Season 1
    let season = TvSeason {
        id: season_id,
        series_id,
        season_number: 1,
        name: "Season 1".to_string(),
        overview: None,
        poster_path: None,
        episode_count: 2,
        created_at: Utc::now(),
    };
    tv_repo.upsert_season(&season).unwrap();

    // 3. Insert Episode 1 & Episode 2
    let ep1 = TvEpisode {
        id: ep1_id,
        series_id,
        season_id,
        season_number: 1,
        episode_number: 1,
        title: "Winter Is Coming".to_string(),
        overview: Some("Lord Eddard Stark is asked to become Hand of the King.".to_string()),
        still_path: None,
        air_date: Some("2011-04-17".to_string()),
        duration_seconds: Some(3700),
        rating: Some(8.9),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    tv_repo.upsert_episode(&ep1).unwrap();

    let ep2 = TvEpisode {
        id: ep2_id,
        series_id,
        season_id,
        season_number: 1,
        episode_number: 2,
        title: "The Kingsroad".to_string(),
        overview: Some("Bran awakens from his coma.".to_string()),
        still_path: None,
        air_date: Some("2011-04-24".to_string()),
        duration_seconds: Some(3360),
        rating: Some(8.7),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    tv_repo.upsert_episode(&ep2).unwrap();

    // 4. Attach Media to Episode 1
    let source_repo = moviestream_lib::adapters::SqliteLibrarySourceRepository::new(db.clone());
    let source_id = Uuid::new_v4();
    source_repo.upsert_source(&moviestream_lib::domain::LibrarySource {
        id: source_id,
        path: "/shows".to_string(),
        name: "TV Shows".to_string(),
        status: moviestream_lib::domain::SourceStatus::Available,
        last_scanned_at: None,
        created_at: Utc::now(),
    }).unwrap();

    let media1_id = Uuid::new_v4();
    let media1 = Media {
        id: media1_id,
        movie_id: None,
        episode_id: Some(ep1_id),
        source_id,
        path: "/shows/Game of Thrones/Season 1/S01E01.mkv".to_string(),
        size_bytes: 3_500_000_000,
        duration_seconds: Some(3700),
        container_format: Some("mkv".to_string()),
        video_codec: Some("HEVC".to_string()),
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

    // 5. Test Series Details
    let details = tv_repo.get_series_details(&series_id).unwrap().unwrap();
    assert_eq!(details.series.title, "Game of Thrones");
    assert_eq!(details.seasons.len(), 1);
    assert_eq!(details.seasons[0].episodes.len(), 2);
    assert_eq!(details.seasons[0].episodes[0].episode.title, "Winter Is Coming");
    assert_eq!(details.seasons[0].episodes[0].media_id, Some(media1_id));

    // 6. Test Next Episode Resolution (S01E01 -> S01E02)
    let next_ep = tv_repo.get_next_episode(&series_id, 1, 1).unwrap().unwrap();
    assert_eq!(next_ep.episode_number, 2);
    assert_eq!(next_ep.title, "The Kingsroad");

    // 7. Test Unified Progress and Continue Watching
    let progress_id = Uuid::new_v4();
    let progress = MediaProgress {
        id: progress_id,
        media_type: MediaType::Episode,
        media_id: media1_id,
        movie_id: None,
        series_id: Some(series_id),
        season_number: Some(1),
        episode_number: Some(1),
        episode_id: Some(ep1_id),
        position_seconds: 1200,
        duration_seconds: 3700,
        progress_percentage: 32.43,
        completed: false,
        last_watched: Utc::now(),
    };
    progress_repo.save_progress(&progress).unwrap();

    let cw = progress_repo.get_continue_watching(10).unwrap();
    assert_eq!(cw.len(), 1);
    assert_eq!(cw[0].series_title.as_deref(), Some("Game of Thrones"));
    assert_eq!(cw[0].episode_title.as_deref(), Some("Winter Is Coming"));
    assert_eq!(cw[0].progress.position_seconds, 1200);

    // 8. Test Completion Marking
    progress_repo.mark_completed(&media1_id).unwrap();
    let completed_cw = progress_repo.get_continue_watching(10).unwrap();
    assert_eq!(completed_cw.len(), 0);
}
