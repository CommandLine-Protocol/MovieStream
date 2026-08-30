CREATE TABLE IF NOT EXISTS library_source (
  id TEXT PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'available',
  last_scanned_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS movie (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  original_title TEXT,
  year INTEGER,
  description TEXT,
  poster_path TEXT,
  backdrop_path TEXT,
  genres TEXT,              -- JSON array
  [cast] TEXT,              -- JSON array
  director TEXT,
  rating REAL,
  metadata_provider_id TEXT,
  metadata_status TEXT NOT NULL DEFAULT 'unmatched',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tv_series (
  id TEXT PRIMARY KEY,
  tmdb_id INTEGER,
  title TEXT NOT NULL,
  original_title TEXT,
  year INTEGER,
  description TEXT,
  poster_path TEXT,
  backdrop_path TEXT,
  genres TEXT,              -- JSON array
  rating REAL,
  metadata_provider_id TEXT,
  metadata_status TEXT NOT NULL DEFAULT 'unmatched',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tv_season (
  id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES tv_series(id) ON DELETE CASCADE,
  season_number INTEGER NOT NULL,
  name TEXT NOT NULL,
  overview TEXT,
  poster_path TEXT,
  episode_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tv_episode (
  id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES tv_series(id) ON DELETE CASCADE,
  season_id TEXT NOT NULL REFERENCES tv_season(id) ON DELETE CASCADE,
  season_number INTEGER NOT NULL,
  episode_number INTEGER NOT NULL,
  title TEXT NOT NULL,
  overview TEXT,
  still_path TEXT,
  air_date TEXT,
  duration_seconds INTEGER,
  rating REAL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS media (
  id TEXT PRIMARY KEY,
  movie_id TEXT REFERENCES movie(id) ON DELETE CASCADE,
  episode_id TEXT REFERENCES tv_episode(id) ON DELETE CASCADE,
  source_id TEXT NOT NULL REFERENCES library_source(id) ON DELETE CASCADE,
  path TEXT NOT NULL UNIQUE,
  size_bytes INTEGER NOT NULL,
  duration_seconds INTEGER,
  container_format TEXT,
  video_codec TEXT,
  resolution_width INTEGER,
  resolution_height INTEGER,
  audio_tracks TEXT,          -- JSON array
  subtitle_tracks TEXT,       -- JSON array
  file_hash TEXT,
  file_mtime TEXT NOT NULL,
  availability TEXT NOT NULL DEFAULT 'available',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Unified Playback Progress (Movies & TV Episodes)
CREATE TABLE IF NOT EXISTS playback_progress (
  id TEXT PRIMARY KEY,
  media_type TEXT NOT NULL, -- 'movie' | 'episode'
  media_id TEXT NOT NULL REFERENCES media(id) ON DELETE CASCADE,
  movie_id TEXT REFERENCES movie(id) ON DELETE CASCADE,
  series_id TEXT REFERENCES tv_series(id) ON DELETE CASCADE,
  season_number INTEGER,
  episode_number INTEGER,
  episode_id TEXT REFERENCES tv_episode(id) ON DELETE CASCADE,
  position_seconds INTEGER NOT NULL DEFAULT 0,
  duration_seconds INTEGER NOT NULL DEFAULT 0,
  progress_percentage REAL NOT NULL DEFAULT 0.0,
  completed INTEGER NOT NULL DEFAULT 0,
  last_watched TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Legacy table kept for compatibility
CREATE TABLE IF NOT EXISTS playback_state (
  movie_id TEXT PRIMARY KEY REFERENCES movie(id) ON DELETE CASCADE,
  media_id TEXT NOT NULL REFERENCES media(id) ON DELETE CASCADE,
  position_seconds INTEGER NOT NULL DEFAULT 0,
  duration_seconds INTEGER NOT NULL DEFAULT 0,
  completed INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS watch_history (
  id TEXT PRIMARY KEY,
  movie_id TEXT REFERENCES movie(id) ON DELETE CASCADE,
  episode_id TEXT REFERENCES tv_episode(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL,
  completed_at TEXT,
  last_position_seconds INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS watchlist (
  movie_id TEXT PRIMARY KEY REFERENCES movie(id) ON DELETE CASCADE,
  added_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL   -- JSON value
);

CREATE INDEX IF NOT EXISTS idx_media_movie_id ON media(movie_id);
CREATE INDEX IF NOT EXISTS idx_media_episode_id ON media(episode_id);
CREATE INDEX IF NOT EXISTS idx_media_source_id ON media(source_id);
CREATE INDEX IF NOT EXISTS idx_movie_title ON movie(title);
CREATE INDEX IF NOT EXISTS idx_tv_series_title ON tv_series(title);
CREATE INDEX IF NOT EXISTS idx_tv_season_series ON tv_season(series_id);
CREATE INDEX IF NOT EXISTS idx_tv_episode_season ON tv_episode(season_id);
CREATE INDEX IF NOT EXISTS idx_tv_episode_series ON tv_episode(series_id);
CREATE INDEX IF NOT EXISTS idx_playback_progress_last_watched ON playback_progress(last_watched DESC);
CREATE INDEX IF NOT EXISTS idx_playback_progress_media_id ON playback_progress(media_id);
CREATE INDEX IF NOT EXISTS idx_watch_history_movie_id ON watch_history(movie_id);
