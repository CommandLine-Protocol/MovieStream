# MovieStream V1 — Implementation Agent Prompt

You are the primary implementation agent for **MovieStream V1**.

Your task is to implement the MovieStream V1 desktop application in this repository according to the two authoritative documents stored in the repository's `docs/` directory.

## 1. Source of Truth

Before writing implementation code, read both documents in `docs/` completely.

The two documents are:

1. **MovieStream V1 PRD**

   * Product requirements, UX goals, functional scope, non-goals, future-version direction, and acceptance criteria.

2. **MovieStream V1 — Software Design & Technical Documentation**

   * Concrete architecture, domain model, repository structure, SQLite schema, Rust traits, services, adapters, Tauri command/event surface, indexing pipeline, playback behavior, error handling, performance requirements, and Definition of Done.

Treat these documents as the project's authoritative specification.

If the two documents conflict:

1. Prefer explicit requirements over implementation suggestions.
2. Prefer the more specific technical-design requirement when the PRD leaves the decision open.
3. Do not silently invent product requirements.
4. Identify genuine conflicts or blocking ambiguities before making a consequential architectural change.
5. For documented **Decision Points**, use the proposed default unless there is a strong technical reason not to.

Do not expand V1 into V2/V3/V4/V5 functionality.

---

# 2. Implementation Objective

Build a working, production-quality **MovieStream V1** end to end.

The implementation must satisfy the Definition of Done in the technical documentation, not merely create placeholder architecture.

The result should be a functioning cross-platform desktop application with:

* Tauri desktop shell
* React frontend
* Rust application/backend layer
* SQLite persistence
* VLC/libVLC playback through an adapter
* Local filesystem media sources
* Background library scanning/indexing
* Metadata-provider abstraction
* Metadata/artwork caching
* Movie/media separation
* Search
* Filtering/sorting
* Movie details
* Playback
* Resume playback
* Watch history
* Continue Watching
* Watchlist
* Subtitle selection
* Audio-track selection
* Offline core functionality
* Incremental indexing
* Graceful missing/disconnected media handling
* Cross-platform packaging strategy

Do not implement fake functionality merely to make tests pass.

---

# 3. Required Architectural Boundaries

These boundaries are mandatory.

## UI

React must communicate with the Rust layer through Tauri commands/events.

The UI must never directly access:

* SQLite
* libVLC
* filesystem persistence
* metadata-provider APIs

Use typed IPC wrappers under the frontend `src/ipc/` layer.

---

## Application Services

Application services contain product/application logic.

Examples:

* `LibraryService`
* `PlaybackService`
* `SearchService`
* `WatchlistService`
* `HistoryService`
* `MetadataService` / resolver
* `SettingsService`

Services must depend on abstractions rather than concrete infrastructure implementations.

---

## MediaPlayer

Define and use the `MediaPlayer` abstraction.

The V1 implementation is the VLC/libVLC adapter.

Do not leak VLC-specific types into:

* domain entities
* application services
* React
* Tauri command contracts

---

## Repository

Define repository traits for persistence.

SQLite is the V1 implementation.

Services must not contain raw SQL.

React must never access SQLite.

---

## MetadataProvider

Metadata access must occur through the metadata-provider abstraction.

Do not couple application services to a specific provider SDK/API.

Metadata failure must never prevent local media from being indexed and played.

---

## MediaSource

Use the `MediaSource` abstraction for filesystem access.

V1 implements the local filesystem source.

Do not build remote/network media-server functionality as part of V1.

---

# 4. Repository Structure

Use the structure specified by the technical documentation as the baseline:

```text
moviestream/
├── docs/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   ├── services/
│   │   ├── domain/
│   │   ├── abstractions/
│   │   ├── adapters/
│   │   ├── events.rs
│   │   └── db/
│   │       ├── migrations/
│   │       └── schema.sql
│   └── Cargo.toml
├── src/
│   ├── views/
│   ├── components/
│   ├── ipc/
│   └── state/
└── package.json
```

Adapt the exact structure if the existing repository already contains a sensible Tauri/React scaffold, but preserve the architectural boundaries.

Do not unnecessarily reorganize unrelated existing project files.

---

# 5. Implementation Process

Work incrementally.

## Phase 1 — Repository inspection

First inspect the repository.

Determine:

* existing application structure;
* existing Tauri setup;
* React setup;
* Rust version;
* package manager;
* existing dependencies;
* existing tests;
* existing build scripts;
* existing configuration;
* existing documentation.

Do not overwrite an existing working project structure without understanding it.

---

## Phase 2 — Architecture foundation

Implement:

* domain entities;
* abstraction traits;
* repository interfaces;
* service interfaces/structure;
* adapter boundaries;
* application state management;
* error types;
* configuration;
* database initialization;
* migrations.

The architecture must compile before proceeding to higher-level functionality.

---

## Phase 3 — SQLite persistence

Implement the SQLite adapter and migrations.

Use the schema in the technical documentation as the baseline.

Requirements include:

* foreign keys;
* appropriate indexes;
* migration support;
* transactions where appropriate;
* safe serialization/deserialization of JSON fields;
* repository-level error handling.

Services must not contain raw SQL.

---

## Phase 4 — Local media source and scanner

Implement:

* source registration;
* native folder selection;
* recursive scanning;
* supported-extension filtering;
* incremental indexing;
* source status;
* missing-file detection;
* background scanning;
* progress events.

Do not block the UI thread.

Do not perform an unnecessary full-library scan on every launch.

---

## Phase 5 — Filename parsing and media analysis

Implement the filename parser according to the documented rules.

It should handle examples such as:

```text
Inception.2010.mkv
Inception (2010).mp4
Dune.Part.Two.2024.1080p.mkv
The.Matrix.1999.4K.mkv
```

Treat parsed information as a guess.

Implement media analysis through the appropriate media abstraction/adapter.

Do not duplicate functionality unnecessarily if libVLC already provides the required information.

---

## Phase 6 — Metadata

Implement:

* metadata-provider abstraction;
* provider adapter;
* search/matching;
* manual matching path;
* metadata status;
* artwork retrieval;
* local artwork cache;
* offline/error behavior.

The exact external provider must follow the documented project decision or, if still genuinely undecided, isolate the provider behind the abstraction and make the provider configuration replaceable.

Do not make metadata availability a prerequisite for playback.

---

## Phase 7 — Library UI

Implement the core React experience:

* Home;
* All Movies;
* Search;
* Movie Details;
* Watchlist;
* Continue Watching;
* Recently Watched;
* Settings.

Prioritize a polished cinematic interface rather than a generic CRUD interface.

The UI should remain functional while metadata is unavailable.

---

## Phase 8 — VLC/libVLC playback

Implement the V1 `MediaPlayer` adapter.

Support the documented playback requirements:

* load;
* play;
* pause;
* stop;
* seek;
* volume;
* mute;
* fullscreen;
* playback speed where practical;
* audio-track selection;
* subtitle-track selection;
* external subtitles where supported;
* duration;
* current position;
* player events/errors.

Keep VLC-specific implementation details inside the adapter.

---

## Phase 9 — Playback state and history

Implement:

* resume state;
* resume prompt;
* periodic progress persistence;
* pause/stop persistence;
* application-close persistence where practical;
* completion detection;
* configurable completion threshold;
* watch history;
* Continue Watching;
* Recently Watched.

Do not merge playback state and watch history into one persistence model.

---

## Phase 10 — Watchlist and settings

Implement:

* add/remove watchlist;
* watchlist view;
* settings persistence;
* relevant playback settings;
* library settings;
* appearance settings;
* metadata settings.

Use the repository abstraction.

---

## Phase 11 — Error and offline behavior

Explicitly test and implement:

* removed media;
* disconnected drives;
* inaccessible sources;
* corrupt media;
* unsupported media;
* metadata-provider failure;
* network loss;
* missing artwork;
* incomplete metadata.

The application should degrade gracefully.

Core local functionality must continue without network access.

---

# 6. UI/UX Requirements

Do not interpret "functional" as "visually minimal."

The product goal is a polished personal streaming-library experience.

Aim for:

* cinematic artwork;
* strong visual hierarchy;
* attractive movie cards;
* smooth navigation;
* useful hover/focus states;
* clear playback controls;
* polished loading states;
* meaningful empty states;
* clear unavailable-media states;
* responsive layouts;
* keyboard accessibility.

Avoid excessive animation that harms performance.

Do not add unrelated features merely for visual complexity.

---

# 7. Background Work

The following must not block the UI:

* directory scanning;
* media analysis;
* metadata fetching;
* artwork downloads;
* expensive indexing operations.

Use appropriate Rust async/background execution.

Long-running operations should communicate progress through Tauri events.

The UI should remain responsive during large-library indexing.

---

# 8. Events

Implement typed events corresponding to the documented event surface, including where applicable:

```text
library://scan-progress
library://source-status-changed
library://movie-added
playback://position
playback://error
```

Keep event payloads typed and versionable.

Avoid passing arbitrary unstructured blobs between backend and frontend when a defined payload is appropriate.

---

# 9. Commands

Implement the documented command surface or an equivalent typed command structure covering:

```text
library.add_source
library.remove_source
library.list_sources
library.rescan

movies.list
movies.get
movies.search
movies.set_metadata_match

playback.start
playback.pause
playback.resume
playback.seek
playback.set_audio_track
playback.set_subtitle_track
playback.set_volume
playback.set_fullscreen

watchlist.add
watchlist.remove
watchlist.list

history.recently_watched
history.continue_watching

settings.get
settings.update
```

Tauri command handlers should remain thin.

They should validate/translate IPC input and delegate to application services.

Do not put substantial business logic inside command handlers.

---

# 10. Incremental Indexing Rules

Follow these rules exactly unless a documented Decision Point is intentionally changed:

### New file

Run the required indexing pipeline.

### Removed file

Mark its media as unavailable.

Do not immediately destroy historical information.

### Modified file

If size or modification time changed:

* re-analyze media;
* do not automatically redo metadata matching unless the filename/title identity changed.

### Renamed/moved file

V1 may treat this as:

```text
old file removed
+
new file discovered
```

Do not implement expensive whole-library hashing merely to detect moves.

---

# 11. Duplicate/Multi-Version Rules

Never assume one movie equals one media file.

The correct model is:

```text
Movie
├── Media
├── Media
└── Media
```

For example:

```text
Dune.2021.1080p.mkv
Dune.2021.2160p.mkv
```

should normally result in:

```text
Dune (Movie)
├── 1080p Media
└── 2160p Media
```

rather than two independent movie entries.

---

# 12. Completion Threshold

Do not scatter a literal such as:

```text
0.90
```

through the codebase.

Use a named configuration value/constant.

The default may follow the technical documentation's proposed range, but it must be centralized.

---

# 13. Testing Requirements

Implement meaningful tests at appropriate layers.

At minimum, test:

### Domain

* entity serialization/deserialization where applicable;
* value-object behavior.

### Filename parser

Test:

```text
Inception.2010.mkv
Inception (2010).mp4
Dune.Part.Two.2024.1080p.mkv
The.Matrix.1999.4K.mkv
```

and edge cases.

### Repositories

Test:

* insert;
* update;
* retrieval;
* search;
* relationships;
* deletion behavior;
* watchlist;
* playback state;
* history.

### Incremental indexing

Test:

* new files;
* unchanged files;
* modified files;
* removed files.

### Playback

Mock the `MediaPlayer` abstraction where appropriate.

Test:

* resume;
* progress persistence;
* completion;
* playback errors.

### Metadata

Mock the `MetadataProvider`.

Test:

* successful matching;
* ambiguous matching;
* provider failure;
* offline behavior.

### Integration

Verify the major end-to-end flows where practical.

Do not write tests that depend unnecessarily on a developer's machine-specific filesystem.

---

# 14. VLC Bundling

The final application must not require a separately installed VLC application.

Investigate and implement the required libVLC runtime/plugin bundling for the supported platforms.

Validate:

* macOS;
* Windows;
* Linux;
* relevant CPU architectures.

Do not claim distribution readiness until the packaging has actually been tested.

Licensing implications of redistributing VLC/libVLC and associated components must be independently validated before release.

Do not make legal claims based solely on assumptions.

---

# 15. Security and Privacy

MovieStream is a local-first desktop application.

Avoid unnecessary collection or transmission of user data.

Do not introduce:

* accounts;
* analytics;
* tracking;
* telemetry;
* cloud synchronization;

unless explicitly required by the specification.

Metadata-provider network calls should only transmit the information required for metadata lookup.

Never log:

* API secrets;
* credentials;
* tokens;
* sensitive filesystem information unnecessarily.

---

# 16. Host-Machine Information Rule

This rule is mandatory.

**All reports, documentation, logs intended for developers, implementation summaries, test reports, and status updates must use repository-relative paths or sanitized repository paths.**

Never expose developer host-machine information.

Do not report paths such as:

```text
/Users/john/Documents/project/moviestream/src-tauri/...
/home/alice/projects/moviestream/...
C:\Users\John\Desktop\moviestream\...
```

Instead report:

```text
docs/...
src/...
src-tauri/...
src-tauri/src/...
package.json
Cargo.toml
```

If an absolute path appears in tool output, error output, compiler output, test output, or logs, sanitize it before including it in any report.

For example:

```text
/Users/developer/work/moviestream/src-tauri/src/services/library_service.rs
```

must be reported as:

```text
src-tauri/src/services/library_service.rs
```

This applies to:

* implementation reports;
* progress updates;
* test reports;
* bug reports;
* build reports;
* architecture notes;
* commit summaries;
* final handoff documentation.

Never expose:

* username;
* home directory;
* desktop path;
* workspace root outside the repository;
* machine hostname;
* private directory names;
* environment-specific absolute paths.

Repository-relative paths are the standard reporting format.

---

# 17. Documentation Rule

Keep implementation documentation inside the repository where appropriate.

The two provided source-of-truth documents remain in:

```text
docs/
```

Do not duplicate large portions of those documents into source files.

Add smaller technical documentation only when it improves maintainability.

Useful examples include:

```text
docs/
├── ...
├── architecture/
├── development/
└── testing/
```

Do not create documentation merely for the sake of creating files.

---

# 18. Dependency Discipline

Prefer mature, well-maintained dependencies.

Before introducing a dependency, ask:

1. Is it actually needed?
2. Does it solve a meaningful problem?
3. Does it fit the Tauri/Rust architecture?
4. Does it introduce significant binary size?
5. Does it complicate cross-platform packaging?
6. Does it create licensing concerns?
7. Can the functionality reasonably be implemented using existing dependencies?

Do not build an unnecessarily large dependency stack.

---

# 19. Do Not Over-Engineer V1

V1 should be extensible, but it should not be a framework for features that do not exist yet.

Do not implement:

* media-server APIs;
* user accounts;
* cloud synchronization;
* distributed databases;
* transcoding infrastructure;
* adaptive streaming;
* CDN support;
* mobile clients;
* TV clients;
* AI recommendation systems;
* subscription systems.

Create the abstraction seams required by the architecture, but implement only the V1 adapters.

---

# 20. Decision Points

When the documents identify an explicit Decision Point:

1. Use the proposed default unless there is a strong reason not to.
2. Keep the decision localized.
3. Avoid spreading an experimental choice through the codebase.
4. Document meaningful deviations.

Examples include:

* supported media extensions;
* playback progress persistence interval;
* Linux packaging format;
* completion threshold default;
* exact metadata provider;
* exact VLC/libVLC integration mechanism.

Do not turn every implementation choice into a project-wide architectural decision.

---

# 21. Definition of Done

Before declaring V1 complete, verify the documented Definition of Done end to end.

In particular, demonstrate that:

* the app builds;
* the app launches;
* VLC/libVLC is correctly integrated;
* a user can add a directory;
* scanning happens in the background;
* movies are indexed;
* metadata can be retrieved;
* artwork is cached;
* the library works offline;
* search works locally;
* filtering/sorting work;
* movie details work;
* playback works;
* resume works;
* subtitles work;
* audio-track selection works;
* watch history works;
* Continue Watching works;
* watchlist works;
* unavailable files are handled;
* incremental scanning works;
* large-library scanning does not freeze the UI.

Do not declare success based solely on compilation.

---

# 22. Validation Strategy

After implementation, perform validation in this order:

```text
Format
  ↓
Static checks
  ↓
Unit tests
  ↓
Integration tests
  ↓
Frontend build
  ↓
Rust build
  ↓
Tauri development run
  ↓
End-to-end flows
  ↓
Packaging validation
```

Fix failures rather than simply documenting them as expected.

If something cannot be validated because the environment lacks a required capability, clearly state that limitation.

---

# 23. Reporting Format

At the end of meaningful implementation phases, provide concise reports containing:

## Implemented

List completed functionality.

## Changed

List important repository-relative paths.

Example:

```text
src-tauri/src/services/library_service.rs
src-tauri/src/services/scanner.rs
src-tauri/src/adapters/sqlite/
src/views/Home.tsx
src/components/MovieCard.tsx
```

## Validation

Report commands/checks performed and their results.

Example:

```text
cargo test — passed
npm run build — passed
```

## Known Issues

List actual remaining issues.

## Decisions

List only meaningful deviations or implementation decisions that affect future work.

## Next Step

State the most useful next implementation step.

Do not include absolute host-machine paths.

---

# 24. Final Handoff Report

When V1 implementation is complete, produce a final implementation report containing:

1. Executive summary
2. Implemented features
3. Architecture implemented
4. Major repository changes
5. Database/migration changes
6. VLC/libVLC integration status
7. Metadata integration status
8. Library/indexing behavior
9. Playback/resume behavior
10. Offline behavior
11. Testing performed
12. Build/package validation
13. Known limitations
14. Deferred Decision Points
15. Remaining work before release

All paths in this report must be repository-relative.

Never include developer-machine paths.

---

# 25. Working Principle

Use this hierarchy when making implementation decisions:

```text
Product requirements
       ↓
Technical design
       ↓
Existing repository conventions
       ↓
Established engineering best practices
       ↓
Implementation convenience
```

Do not allow implementation convenience to override product requirements or architectural boundaries.

The objective is not merely to produce code that compiles.

The objective is to produce a maintainable MovieStream V1 that:

> **feels like a polished personal streaming service while remaining a lightweight, local-first desktop application built around VLC/libVLC.**

Begin by inspecting the repository and reading the two documents in `docs/`. Then establish the implementation plan and proceed incrementally, validating each major layer before building on it.
