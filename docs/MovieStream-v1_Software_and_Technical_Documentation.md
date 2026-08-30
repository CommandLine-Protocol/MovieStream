# MovieStream V1 — Software Design & Technical Documentation

**Status:** Ready for implementation
**Source of truth:** MovieStream V1 PRD
**Audience:** Implementing developer(s) or an AI coding agent
**Scope:** This document translates the PRD's functional requirements and conceptual architecture into a concrete, implementable technical design. Where the PRD explicitly leaves a decision open, this document marks it as a **Decision Point** and proposes a default rather than inventing a requirement that was not in the PRD.

---

## 1. System Summary

MovieStream V1 is a cross-platform desktop application (macOS, Windows, Linux) built on:

- **Shell / packaging:** Tauri
- **Frontend:** React (web-based UI, communicating with the backend via Tauri commands/events)
- **Backend/native layer:** Rust
- **Media engine:** VLC / libVLC (bundled, not a separate user install)
- **Database:** SQLite, accessed only through a repository abstraction

MovieStream is a **library/product layer around VLC**, not a replacement media engine. All decoding/playback is delegated to libVLC. All product value (organization, metadata, discovery, watch state, UX) is built by MovieStream.

---

## 2. Architectural Layering (authoritative)

```
UI (React)
   │  Tauri commands + events
   ▼
Application Services Layer (Rust)
   │
   ├── Library Service        (sources, scanning, indexing)
   ├── Playback Service        (session state, resume, progress)
   ├── Search Service
   ├── Watchlist Service
   ├── History Service
   ├── Metadata Service
   └── Settings Service
   │
   ├────────────► Repository Abstraction ─► SQLite
   └────────────► MediaPlayer Abstraction ─► VLC Adapter ─► libVLC ─► VLC Engine
                Metadata Abstraction ─► Provider Adapter(s)
                Filesystem/MediaSource Abstraction ─► Local FS (V1) / Network FS (future)
```

**Non-negotiable rule (carried from PRD §28, §34, §35):** The Application Services layer must never call libVLC, SQLite, or a metadata provider's SDK/API directly. All access goes through the three abstractions:

1. `MediaPlayer` trait → VLC implementation
2. `MovieRepository` trait (+ related repositories) → SQLite implementation
3. `MetadataProvider` trait → concrete provider implementation(s)
4. `MediaSource` trait → local filesystem implementation (V1 only implements local)

Changes to playback, storage, or metadata must be made by adding/modifying an adapter, never by editing UI or cross-cutting application logic.

---

## 3. Repository / Directory Structure

```
moviestream/
├── src-tauri/                     # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/              # Tauri command handlers (thin, delegate to services)
│   │   │   ├── library.rs
│   │   │   ├── playback.rs
│   │   │   ├── search.rs
│   │   │   ├── watchlist.rs
│   │   │   ├── history.rs
│   │   │   ├── metadata.rs
│   │   │   └── settings.rs
│   │   ├── services/               # Application services layer
│   │   │   ├── library_service.rs
│   │   │   ├── scanner.rs
│   │   │   ├── media_analyzer.rs
│   │   │   ├── filename_parser.rs
│   │   │   ├── metadata_resolver.rs
│   │   │   ├── playback_service.rs
│   │   │   ├── search_service.rs
│   │   │   ├── watchlist_service.rs
│   │   │   ├── history_service.rs
│   │   │   └── settings_service.rs
│   │   ├── domain/                 # Entities + value objects (no persistence/IO)
│   │   │   ├── movie.rs
│   │   │   ├── media.rs
│   │   │   ├── library_source.rs
│   │   │   ├── playback_state.rs
│   │   │   ├── watch_history.rs
│   │   │   └── watchlist.rs
│   │   ├── abstractions/           # Trait definitions (interfaces)
│   │   │   ├── media_player.rs
│   │   │   ├── movie_repository.rs
│   │   │   ├── metadata_provider.rs
│   │   │   └── media_source.rs
│   │   ├── adapters/
│   │   │   ├── vlc/                # MediaPlayer implementation
│   │   │   ├── sqlite/             # Repository implementation
│   │   │   ├── metadata_provider_x/  # e.g. TMDb-style provider adapter
│   │   │   └── local_fs/           # MediaSource implementation
│   │   ├── events.rs                # Typed event payloads emitted to UI
│   │   └── db/
│   │       ├── migrations/
│   │       └── schema.sql
│   └── Cargo.toml
├── src/                            # React UI
│   ├── views/ (Home, AllMovies, Search, MovieDetails, Player, Settings)
│   ├── components/
│   ├── ipc/                        # Typed wrappers around Tauri invoke/listen
│   └── state/
└── package.json
```

---

## 4. Domain Model — Entities

Entities are plain data structures with no I/O or persistence logic (per PRD §9). They live in `domain/`.

### 4.1 Movie
```
Movie {
  id: Uuid
  title: String
  original_title: Option<String>
  year: Option<u16>
  description: Option<String>
  poster_path: Option<String>       // local cache path
  backdrop_path: Option<String>     // local cache path
  genres: Vec<String>
  cast: Vec<String>
  director: Option<String>
  rating: Option<f32>
  metadata_provider_id: Option<String>   // external id, for re-sync
  metadata_status: MetadataStatus        // Unmatched | AutoMatched | ManuallyMatched | Failed
  created_at: DateTime
  updated_at: DateTime
}
```

### 4.2 Media (a playable representation of a Movie)
```
Media {
  id: Uuid
  movie_id: Uuid
  source_id: Uuid
  path: String
  size_bytes: u64
  duration_seconds: Option<u32>
  container_format: Option<String>
  video_codec: Option<String>
  resolution_width: Option<u32>
  resolution_height: Option<u32>
  audio_tracks: Vec<AudioTrackInfo>
  subtitle_tracks: Vec<SubtitleTrackInfo>
  file_hash: Option<String>          // for change detection, see §7
  file_mtime: DateTime
  availability: MediaAvailability     // Available | Unavailable
  created_at: DateTime
  updated_at: DateTime
}
```

### 4.3 LibrarySource
```
LibrarySource {
  id: Uuid
  path: String
  name: String
  status: SourceStatus   // Available | Unavailable | Scanning | Indexing | Inaccessible | Disconnected
  last_scanned_at: Option<DateTime>
  created_at: DateTime
}
```

### 4.4 PlaybackState
```
PlaybackState {
  movie_id: Uuid
  media_id: Uuid
  position_seconds: u32
  duration_seconds: u32
  completed: bool
  updated_at: DateTime
}
```

### 4.5 WatchHistory
```
WatchHistoryEntry {
  id: Uuid
  movie_id: Uuid
  started_at: DateTime
  completed_at: Option<DateTime>
  last_position_seconds: u32
}
```

### 4.6 Watchlist
```
WatchlistEntry {
  movie_id: Uuid
  added_at: DateTime
}
```

---

## 5. Concrete SQLite Schema

The PRD's data model (§9) is explicitly conceptual. This section makes it concrete for implementation. Accessed only via the repository layer — no raw SQL from services or UI.

```sql
CREATE TABLE library_source (
  id TEXT PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'available',
  last_scanned_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE movie (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  original_title TEXT,
  year INTEGER,
  description TEXT,
  poster_path TEXT,
  backdrop_path TEXT,
  genres TEXT,              -- JSON array
  cast TEXT,                 -- JSON array
  director TEXT,
  rating REAL,
  metadata_provider_id TEXT,
  metadata_status TEXT NOT NULL DEFAULT 'unmatched',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE media (
  id TEXT PRIMARY KEY,
  movie_id TEXT NOT NULL REFERENCES movie(id) ON DELETE CASCADE,
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

CREATE TABLE playback_state (
  movie_id TEXT PRIMARY KEY REFERENCES movie(id) ON DELETE CASCADE,
  media_id TEXT NOT NULL REFERENCES media(id) ON DELETE CASCADE,
  position_seconds INTEGER NOT NULL DEFAULT 0,
  duration_seconds INTEGER NOT NULL DEFAULT 0,
  completed INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE watch_history (
  id TEXT PRIMARY KEY,
  movie_id TEXT NOT NULL REFERENCES movie(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL,
  completed_at TEXT,
  last_position_seconds INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE watchlist (
  movie_id TEXT PRIMARY KEY REFERENCES movie(id) ON DELETE CASCADE,
  added_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL   -- JSON value
);

CREATE INDEX idx_media_movie_id ON media(movie_id);
CREATE INDEX idx_media_source_id ON media(source_id);
CREATE INDEX idx_movie_title ON movie(title);
CREATE INDEX idx_watch_history_movie_id ON watch_history(movie_id);
```

**Notes:**
- `genres`, `cast`, `audio_tracks`, `subtitle_tracks` are stored as JSON text columns rather than normalized join tables. This is an implementation simplification consistent with V1's "not an unnecessarily huge system" principle; revisit only if query patterns require it.
- One `movie` row can have multiple `media` rows (PRD §7.5, §10 — duplicate/version handling).

---

## 6. Abstraction Interfaces (traits)

These are the contracts every adapter must satisfy. Defined in `abstractions/`, implemented in `adapters/`.

### 6.1 `MediaPlayer` (VLC adapter target)
```rust
trait MediaPlayer {
    fn load(&mut self, media_path: &str) -> Result<(), PlayerError>;
    fn play(&mut self) -> Result<(), PlayerError>;
    fn pause(&mut self) -> Result<(), PlayerError>;
    fn stop(&mut self) -> Result<(), PlayerError>;
    fn seek(&mut self, position_seconds: u32) -> Result<(), PlayerError>;
    fn set_volume(&mut self, level: u8) -> Result<(), PlayerError>;
    fn set_mute(&mut self, muted: bool) -> Result<(), PlayerError>;
    fn set_fullscreen(&mut self, enabled: bool) -> Result<(), PlayerError>;
    fn set_playback_speed(&mut self, speed: f32) -> Result<(), PlayerError>;

    fn list_audio_tracks(&self) -> Vec<AudioTrackInfo>;
    fn select_audio_track(&mut self, track_id: &str) -> Result<(), PlayerError>;

    fn list_subtitle_tracks(&self) -> Vec<SubtitleTrackInfo>;
    fn select_subtitle_track(&mut self, track_id: Option<&str>) -> Result<(), PlayerError>;
    fn load_external_subtitle(&mut self, path: &str) -> Result<(), PlayerError>;

    fn current_position(&self) -> u32;
    fn duration(&self) -> u32;
    fn on_event(&mut self, callback: PlayerEventCallback);  // position updates, errors, end-of-media
}
```

### 6.2 `MovieRepository` (SQLite adapter target)
```rust
trait MovieRepository {
    fn upsert_movie(&self, movie: &Movie) -> Result<(), RepoError>;
    fn get_movie(&self, id: &Uuid) -> Result<Option<Movie>, RepoError>;
    fn list_movies(&self, filter: MovieFilter, sort: MovieSort) -> Result<Vec<Movie>, RepoError>;
    fn search_movies(&self, query: &str) -> Result<Vec<Movie>, RepoError>;
    fn delete_movie(&self, id: &Uuid) -> Result<(), RepoError>;
}

trait MediaRepository {
    fn upsert_media(&self, media: &Media) -> Result<(), RepoError>;
    fn find_by_path(&self, path: &str) -> Result<Option<Media>, RepoError>;
    fn list_media_for_movie(&self, movie_id: &Uuid) -> Result<Vec<Media>, RepoError>;
    fn set_availability(&self, media_id: &Uuid, availability: MediaAvailability) -> Result<(), RepoError>;
    fn delete_media(&self, media_id: &Uuid) -> Result<(), RepoError>;
}

// Analogous repositories: LibrarySourceRepository, PlaybackStateRepository,
// WatchHistoryRepository, WatchlistRepository, SettingsRepository
```

### 6.3 `MetadataProvider`
```rust
trait MetadataProvider {
    fn search(&self, title_guess: &str, year_guess: Option<u16>) -> Result<Vec<MetadataCandidate>, ProviderError>;
    fn fetch_details(&self, provider_id: &str) -> Result<MovieMetadata, ProviderError>;
    fn fetch_poster(&self, provider_id: &str) -> Result<Vec<u8>, ProviderError>;
    fn fetch_backdrop(&self, provider_id: &str) -> Result<Vec<u8>, ProviderError>;
}
```
`MetadataService` (application layer) depends only on this trait, never on a specific provider SDK, satisfying PRD §7.7/§14's provider-abstraction requirement.

### 6.4 `MediaSource`
```rust
trait MediaSource {
    fn kind(&self) -> MediaSourceKind;  // Local (V1); NetworkSource / FutureRemoteStream reserved for V2+
    fn list_files(&self, root: &str) -> Result<Vec<FileCandidate>, SourceError>;
    fn is_available(&self) -> bool;
    fn watch_for_changes(&self, callback: FsChangeCallback) -> Result<(), SourceError>; // optional in V1, see §7.3
}
```
V1 implements only `LocalFileSystemSource`. The trait exists specifically so V2's `NetworkSource` can be added without touching `Scanner` or `LibraryService` (PRD §20, §29).

---

## 7. Library / Indexing Pipeline (implementation of PRD §8)

### 7.1 Pipeline stages
```
LibraryService.add_source(path)
   → SourceManager.register(path)
   → Scanner.scan(source)                [async, background thread/task]
        → recursively walk directory via MediaSource.list_files()
        → filter by supported extensions (see §7.5)
        → yield FileCandidate { path, size, mtime }
   → for each candidate:
        → MediaAnalyzer.analyze(path)     [uses libVLC media parsing]
             → duration, resolution, codecs, audio/subtitle track lists
        → FilenameParser.parse(filename)
             → title_guess, year_guess, edition_guess
        → MetadataResolver.resolve(title_guess, year_guess)
             → calls MetadataProvider.search() then fetch_details()
             → on ambiguous/failed match: mark Movie.metadata_status = Unmatched,
               media still persisted and playable (PRD §21 resilience rule)
        → DuplicateResolver.match_or_create_movie(...)
             → see §7.4
   → Repository.upsert_media(...) / upsert_movie(...)
   → emit progress events to UI (see §9)
```

Scanning MUST run off the UI thread (Tauri async command + background task/thread pool) — this directly implements PRD §7.2's "must avoid freezing the UI" requirement.

### 7.2 Progress reporting
Emit a Tauri event (e.g. `library://scan-progress`) with payload:
```
{ source_id, files_discovered: u32, movies_identified: u32, phase: "scanning" | "analyzing" | "matching" }
```
This directly implements the PRD §7.2 UI copy: "Scanning Movies… 1,284 files discovered, 73 movies identified."

### 7.3 Incremental indexing (PRD §7.3)
On subsequent app launches / re-scans of an existing source:
1. Compare current directory listing against `media.path` rows for that `source_id`.
2. **New file:** path not in DB → run full pipeline (§7.1) for that file only.
3. **Removed file:** DB path not found on disk → set `media.availability = Unavailable` (do not delete row immediately, so watch history/state is preserved — supports PRD §22 "disconnected drive" behavior).
4. **Modified file:** path matches but `file_mtime` or `size_bytes` differs → re-run `MediaAnalyzer` only (metadata match is not re-run unless the filename also changed).
5. **Moved/renamed file:** Decision Point — V1 default: treat as remove + new (simplest, matches PRD's "must avoid full scan every time" without requiring content hashing across the whole library). `file_hash` field is reserved on the `media` table if a future version wants stronger move-detection; V1 does not require populating it for correctness, only if cheap to compute.

A full re-scan of the entire library on every launch is explicitly disallowed by PRD §7.3.

### 7.4 Duplicate / multi-version handling (PRD §7.5)
When `MetadataResolver` returns a match, `DuplicateResolver`:
1. Looks up existing `Movie` rows with the same `metadata_provider_id` (if a match was found) or the same normalized `(title, year)` pair (if unmatched/manual).
2. If found → attach the new `Media` row to the existing `Movie` (do not create a second `Movie`).
3. If not found → create a new `Movie`.

This directly implements the `Movie → [Media A, Media B]` model in PRD §7.5/§10 (e.g. `Dune.2021.1080p.mkv` and `Dune.2021.2160p.mkv` become two `Media` rows under one `Movie`).

### 7.5 Filename parsing rules (PRD §7.4)
`FilenameParser` must extract `title`, `year`, and optional `edition/version` from common patterns, e.g.:
- `Inception.2010.mkv`
- `Inception (2010).mp4`
- `Dune.Part.Two.2024.1080p.mkv`
- `The.Matrix.1999.4K.mkv`

Approach: strip known technical tokens (resolution: `1080p`, `2160p`, `4K`; source tags; codec tags) from the filename stem, extract a 4-digit year token (bounded to a plausible range) as `year_guess`, and treat the remaining tokens (with separators normalized to spaces) as `title_guess`. Extracted title/year are **guesses only** and must not be trusted as final `Movie.title`/`year` — final values come from `MetadataProvider` on successful match, or remain as the filename guess with `metadata_status = Unmatched` otherwise (PRD §7.4: "should not blindly trust filenames").

### 7.6 Supported file extensions
Decision Point — not enumerated in the PRD. V1 default: rely on formats libVLC can parse/play; maintain an explicit allowlist of common containers (e.g. `.mkv`, `.mp4`, `.avi`, `.mov`, `.m4v`, `.webm`) in `Scanner` config rather than attempting to probe every file with libVLC (probing every file is expensive at scan time). This list should live in one config location so it can be extended without touching scanner logic.

---

## 8. Playback & Resume (implementation of PRD §7.12–§7.16)

### 8.1 Playback session flow
```
UI: user selects "Play" on a Movie
  → command: playback.start(movie_id, media_id)
  → PlaybackService:
      - loads PlaybackState for movie_id (if any)
      - if position_seconds > 0 and not completed → emit "resume-prompt" to UI
        (UI shows: "Resume from 1:23:17?" per PRD §7.14)
      - MediaPlayer.load(media.path)
      - if resume confirmed → MediaPlayer.seek(position_seconds)
      - MediaPlayer.play()
```

### 8.2 Progress persistence
`PlaybackService` subscribes to `MediaPlayer.on_event` position updates and periodically (e.g. every N seconds, Decision Point — suggested default 5–10s, not specified in PRD) writes to `PlaybackStateRepository.upsert(...)`. Also persist on pause/stop/app-close.

### 8.3 Completion detection (PRD §7.16)
```
completion_ratio = position_seconds / duration_seconds
if completion_ratio >= COMPLETION_THRESHOLD:   # configurable, default 0.90–0.95
    playback_state.completed = true
    write/close a WatchHistoryEntry with completed_at = now
```
`COMPLETION_THRESHOLD` must be a named constant/setting, not a hardcoded magic number, since PRD explicitly calls out it "should be configurable internally and potentially exposed later."

### 8.4 Watch History vs Playback State
- `PlaybackState` = one row per movie, latest resume point (mutable, overwritten).
- `WatchHistory` = append-style log of watch sessions (`started_at`/`completed_at`), used to populate "Recently Watched." These are deliberately separate tables per PRD §7.15, not derived from one another.

---

## 9. Tauri Command & Event Surface

Commands are the request/response boundary; events are the push channel for long-running/async work (scanning, playback position ticks).

### 9.1 Commands (indicative, one per PRD functional area)
```
library.add_source(path) -> LibrarySource
library.remove_source(source_id) -> ()
library.list_sources() -> Vec<LibrarySource>
library.rescan(source_id) -> ()

movies.list(filter, sort) -> Vec<Movie>
movies.get(movie_id) -> Movie
movies.search(query) -> Vec<Movie>
movies.set_metadata_match(movie_id, provider_id) -> Movie   // manual correction path (PRD §7.8)

playback.start(movie_id, media_id) -> PlaybackSession
playback.pause() -> ()
playback.resume() -> ()
playback.seek(position_seconds) -> ()
playback.set_audio_track(track_id) -> ()
playback.set_subtitle_track(track_id | null) -> ()
playback.set_volume(level) -> ()
playback.set_fullscreen(enabled) -> ()

watchlist.add(movie_id) -> ()
watchlist.remove(movie_id) -> ()
watchlist.list() -> Vec<Movie>

history.recently_watched() -> Vec<Movie>
history.continue_watching() -> Vec<(Movie, PlaybackState)>

settings.get() -> AppSettings
settings.update(partial) -> AppSettings
```

### 9.2 Events (push from backend to UI)
```
library://scan-progress        { source_id, files_discovered, movies_identified, phase }
library://source-status-changed { source_id, status }
library://movie-added          { movie }
playback://position            { movie_id, position_seconds, duration_seconds }
playback://error                { movie_id, message }
```

---

## 10. Error Handling Matrix (implementation of PRD §22)

| Condition | Backend behavior | UI behavior |
|---|---|---|
| Media file removed from disk | `Scanner`/incremental pass sets `media.availability = Unavailable` | Movie shown but marked unavailable; playback disabled with explanation |
| Source drive disconnected | `LibrarySource.status = Disconnected`; do not delete associated movies/media | Library remains visible; affected movies flagged unavailable |
| Metadata fetch fails | `Movie.metadata_status = Failed` or remains `Unmatched`; `Media` row still created and playable | Movie shown with filename-derived title, no artwork; playback unaffected |
| Corrupt media | `MediaPlayer` surfaces libVLC error via `PlayerEventCallback` | UI shows the surfaced error message, not a generic crash/failure screen |
| Unsupported media | `MediaAnalyzer`/`MediaPlayer` reports unsupported format | UI shows a meaningful explanation (not a raw stack trace or silent failure) |

General rule carried from PRD §21: **loss of network/metadata must never block local browsing, search, filtering, playback, subtitles, history, watchlist, or resume.** Any code path that couples core (offline) functionality to the `MetadataProvider` being reachable is a design defect.

---

## 11. Settings (PRD §23) — storage shape

Stored as key/value JSON rows in `app_settings` via `SettingsRepository`, exposed as one structured object to the UI:
```
AppSettings {
  library: { scan_behavior, default_locations: Vec<String> }
  playback: { default_volume, default_speed, resume_behavior, subtitle_prefs, audio_prefs }
  appearance: { theme, animations_enabled }
  metadata: { active_provider_id, artwork_caching_enabled }
  application: { launch_on_startup, notifications_enabled, log_level }
}
```
Individual services (`PlaybackService`, `MetadataResolver`, etc.) read only their relevant slice via `SettingsRepository`, not the whole blob directly from UI state.

---

## 12. Search (PRD §24)

`SearchService` queries `MovieRepository.search_movies(query)`, which must operate entirely against the local SQLite database (no network call in the hot path). Fields searched: `title`, `original_title`, `year`, `genres`, `cast`, `director` — matching the PRD's field list exactly. Use SQLite `LIKE`/FTS as appropriate to keep it responsive on large libraries; this is an implementation choice, not a new requirement.

---

## 13. VLC Bundling (PRD §18) — implementation checklist

This is flagged in the PRD as requiring independent validation; the technical task is:
1. Bundle the libVLC runtime and required VLC plugin/module set with the Tauri installer per OS (macOS `.app` bundle, Windows installer, Linux package).
2. Do not assume the end-user machine has VLC installed; the app must not depend on a system VLC install.
3. Validate on both Apple Silicon and Intel for macOS, 64-bit Windows, and whatever Linux packaging formats are selected (AppImage / `.deb` / Flatpak — final choice is an implementation decision, per PRD §19).
4. Separately validate GPL/LGPL and VLC redistribution licensing implications for the bundled runtime before shipping — this is called out explicitly in the PRD as something to check during implementation, not to assume.

---

## 14. Performance Constraints → Implementation Rules (PRD §20)

- All scanning/analysis/metadata-fetch work runs on background tasks/threads; Tauri commands that trigger them return immediately and report progress via events (§9.2).
- `Scanner` must not re-analyze unchanged files (see §7.3).
- Poster/backdrop bytes fetched from `MetadataProvider` must be cached to local disk (path stored in `movie.poster_path`/`backdrop_path`); `MetadataResolver` checks the cache before calling the provider again.
- Avoid file hashing across the whole library as a default behavior; `file_hash` is optional/reserved, not required for baseline correctness (see §7.3).

---

## 15. Cross-Cutting Rules Recap (do not violate)

1. UI never talks to SQLite, libVLC, or a metadata provider directly — always through Application Services.
2. `MediaPlayer`, `MovieRepository`(+siblings), `MetadataProvider`, `MediaSource` are the only four abstraction seams; new backends (MPV, server DB, new provider, network source) must be added as new implementations of these traits, not as special-cased branches inside services.
3. One `Movie` can own multiple `Media` rows (versions/duplicates) — never model a movie as a single file.
4. Metadata failure must never disable playback of an already-scanned file.
5. No full-library rescan on every launch — incremental indexing only.
6. Completion threshold is a named, configurable constant, not a hardcoded literal scattered across code.

---

## 16. Explicit Non-Goals for This Implementation Pass

Carried directly from PRD §4 — do not build any of the following in V1:
user accounts, cloud sync, public streaming service, hosted catalogue, DRM, social features, AI-powered recommendations, mobile/TV apps, a dedicated streaming server, large-scale transcoding, adaptive bitrate streaming, CDN infrastructure, subscriptions/payments, user-generated hosting, or anything piracy-related.

---

## 17. Definition of Done for V1 (mapped from PRD §27)

Implementation is complete when all of the following are demonstrably true end-to-end (not just unit-tested in isolation):

1. Installer includes bundled VLC runtime; no separate VLC install required.
2. App launches on macOS, Windows, Linux.
3. User can add a movie directory via native picker.
4. Directory scan runs without blocking the UI thread.
5. Supported files are identified per §7.5/§7.6 rules.
6. Movies are represented independently of their physical files (Movie/Media split, §4.1–4.2).
7. Metadata + artwork retrieved when online; app remains fully usable when offline (§21).
8. Posters/backdrops render from local cache.
9./10./11. Library browsing, search, filter, sort all function against local SQLite only.
12. Movie details view renders per §7.11.
13.–16. Playback via VLC adapter: play/pause/seek/volume/fullscreen, audio track selection, subtitle track selection all function.
17. Resume prompts and correctly seeks to last position.
18. "Recently Watched" / "Continue Watching" populate from `WatchHistory`/`PlaybackState`.
19. Watchlist add/remove works independently of watch state.
20. Core functionality (browse/search/filter/play/subtitles/history/watchlist/resume) works with no network connection.
21. Missing/unavailable files are handled per §10 without crashing.
22. Re-scanning an existing source detects new files without a full rescan.
23. Background scanning/analysis does not freeze the UI (verify under a large test library).

---

## 18. Seams Reserved for Future Versions (do not implement now, but do not block)

These are called out so implementation choices in V1 don't have to be revisited structurally later (PRD §29–§35):

- `MediaSource` trait already anticipates `NetworkSource`/`FutureRemoteStream` variants (V2) — V1 implements only `LocalFileSystemSource`.
- `MovieRepository`/sibling repositories are trait-based specifically so a server-backed repository (V3) can be substituted without UI changes.
- `MediaPlayer` trait is VLC-only in V1 but must not leak VLC-specific types into `Application Services` or UI, so an MPV or other backend could later be substituted.
- Transcoding/adaptive streaming (V4) and multi-client/server platform (V5) are out of scope entirely for this implementation pass; no code should be written toward them now.