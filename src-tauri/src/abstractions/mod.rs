pub mod media_player;
pub mod media_source;
pub mod metadata_provider;
pub mod movie_repository;
pub mod progress_repository;
pub mod tv_repository;

pub use media_player::{MediaPlayer, PlayerEvent, PlayerEventCallback};
pub use media_source::{FileCandidate, MediaSource, MediaSourceKind};
pub use metadata_provider::{MetadataCandidate, MetadataProvider, MovieMetadata};
pub use movie_repository::{
    LibrarySourceRepository, MediaRepository, MovieRepository, PlaybackStateRepository,
    SettingsRepository, WatchHistoryRepository, WatchlistRepository,
};
pub use progress_repository::ProgressRepository;
pub use tv_repository::TvRepository;
