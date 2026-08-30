use chrono::Utc;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

use moviestream_lib::abstractions::{
    LibrarySourceRepository, MediaRepository, MovieMetadata, MovieRepository,
};
use moviestream_lib::adapters::{
    LocalFileSystemSource, MockMetadataProvider, SqliteDb, SqliteLibrarySourceRepository,
    SqliteMediaRepository, SqliteMovieRepository,
};
use moviestream_lib::domain::{LibrarySource, MediaAvailability, SourceStatus};
use moviestream_lib::services::{
    DuplicateResolver, MediaAnalyzer, MetadataResolver, Scanner,
};

#[tokio::test]
async fn test_scanner_end_to_end_and_incremental_indexing() {
    let tmp = tempdir().unwrap();
    let movies_dir = tmp.path().join("Movies");
    std::fs::create_dir_all(&movies_dir).unwrap();

    // Create 2 test movie files
    let movie1_path = movies_dir.join("Inception.2010.1080p.mkv");
    let movie2_path = movies_dir.join("Dune.Part.Two.2024.mkv");
    std::fs::write(&movie1_path, b"fake video content 1").unwrap();
    std::fs::write(&movie2_path, b"fake video content 2").unwrap();

    let db = SqliteDb::new_in_memory().unwrap();
    let source_repo = Arc::new(SqliteLibrarySourceRepository::new(db.clone())) as Arc<dyn LibrarySourceRepository>;
    let media_repo = Arc::new(SqliteMediaRepository::new(db.clone())) as Arc<dyn MediaRepository>;
    let movie_repo = Arc::new(SqliteMovieRepository::new(db)) as Arc<dyn MovieRepository>;

    let media_source = Arc::new(LocalFileSystemSource::new());
    let media_analyzer = Arc::new(MediaAnalyzer::new());

    let mock_provider = Arc::new(
        MockMetadataProvider::new()
            .with_mock_movie(MovieMetadata {
                id: "1001".to_string(),
                title: "Inception".to_string(),
                original_title: None,
                year: Some(2010),
                description: Some("Mind bending dream heist".to_string()),
                genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
                cast: vec!["Leonardo DiCaprio".to_string()],
                director: Some("Christopher Nolan".to_string()),
                rating: Some(8.8),
                poster_url: None,
                backdrop_url: None,
            })
            .with_mock_movie(MovieMetadata {
                id: "1002".to_string(),
                title: "Dune Part Two".to_string(),
                original_title: None,
                year: Some(2024),
                description: Some("Paul Atreides unites with the Fremen".to_string()),
                genres: vec!["Sci-Fi".to_string(), "Adventure".to_string()],
                cast: vec!["Timothée Chalamet".to_string()],
                director: Some("Denis Villeneuve".to_string()),
                rating: Some(8.6),
                poster_url: None,
                backdrop_url: None,
            }),
    );

    let cache_dir = tmp.path().join("artwork");
    let metadata_resolver = Arc::new(MetadataResolver::new(mock_provider, cache_dir));
    let duplicate_resolver = Arc::new(DuplicateResolver::new(movie_repo.clone()));

    let scanner = Scanner::new(
        source_repo.clone(),
        media_repo.clone(),
        movie_repo.clone(),
        media_source,
        media_analyzer,
        metadata_resolver,
        duplicate_resolver,
    );

    let source_id = Uuid::new_v4();
    let source = LibrarySource {
        id: source_id,
        path: movies_dir.to_string_lossy().to_string(),
        name: "Test Movies".to_string(),
        status: SourceStatus::Available,
        last_scanned_at: None,
        created_at: Utc::now(),
    };
    source_repo.upsert_source(&source).unwrap();

    // 1. Initial Scan
    scanner.scan_source(&source, None).await.unwrap();

    let movie_count = movie_repo.count_movies().unwrap();
    assert_eq!(movie_count, 2);

    let media_items = media_repo.list_media_for_source(&source_id).unwrap();
    assert_eq!(media_items.len(), 2);
    assert_eq!(media_items[0].availability, MediaAvailability::Available);

    // 2. Incremental Scan: Add a new movie version (multi-version test)
    let movie1_4k_path = movies_dir.join("Inception.2010.4K.UHD.mkv");
    std::fs::write(&movie1_4k_path, b"fake 4k video content").unwrap();

    scanner.scan_source(&source, None).await.unwrap();

    // Movie count should STILL be 2 (Inception grouped), but media count is now 3
    let movie_count_after = movie_repo.count_movies().unwrap();
    assert_eq!(movie_count_after, 2);

    let media_after = media_repo.list_media_for_source(&source_id).unwrap();
    assert_eq!(media_after.len(), 3);

    // 3. Incremental Scan: Remove a file from disk
    std::fs::remove_file(&movie2_path).unwrap();

    scanner.scan_source(&source, None).await.unwrap();

    // Media row for Dune should now be marked Unavailable (not deleted from db)
    let dune_media = media_repo.find_by_path(&movie2_path.to_string_lossy()).unwrap().unwrap();
    assert_eq!(dune_media.availability, MediaAvailability::Unavailable);
}
