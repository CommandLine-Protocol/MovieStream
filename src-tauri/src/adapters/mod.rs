pub mod local_fs;
pub mod metadata;
pub mod sqlite;
pub mod vlc;

pub use local_fs::LocalFileSystemSource;
pub use metadata::{MockMetadataProvider, OpenMovieMetadataProvider};
pub use sqlite::{
    SqliteDb, SqliteLibrarySourceRepository, SqliteMediaRepository, SqliteMovieRepository,
    SqlitePlaybackStateRepository, SqliteProgressRepository, SqliteSettingsRepository,
    SqliteTvRepository, SqliteWatchHistoryRepository, SqliteWatchlistRepository,
};
pub use vlc::VlcMediaPlayer;
