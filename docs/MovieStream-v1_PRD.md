# Product Requirements Document — MovieStream V1

## 1. Product Overview

**Working name:** MovieStream
**Version:** V1 — Personal Movie Library
**Platform:** macOS, Windows, Linux
**Application type:** Cross-platform desktop application
**Primary technology direction:** Tauri + web-based UI + Rust/native application layer + VLC/libVLC media engine

### Product vision

MovieStream transforms a user's existing collection of locally stored movies into a polished, interactive, streaming-service-style desktop experience.

The application does **not** attempt to replace VLC's media engine.

Instead:

> **MovieStream is the product; VLC/libVLC is the media engine.**

The application should delegate as much low-level media functionality as reasonably possible to VLC/libVLC while concentrating development effort on:

* Library management
* Movie discovery
* Metadata
* Organization
* Search
* Watch state
* User experience
* Application-level intelligence
* Cross-platform desktop integration

---

# 2. Problem Statement

Traditional personal movie collections are often just folders containing files:

```text
Movies/
├── movie1.mkv
├── movie2.mp4
├── movie3.avi
├── movie4.mkv
└── movie5.mp4
```

Although VLC provides excellent playback, the experience surrounding the media is relatively utilitarian.

Users may have to:

* navigate folders manually;
* remember filenames;
* search through files;
* determine which movies they have already watched;
* remember where they stopped watching;
* manually locate subtitles;
* manually identify different versions of the same movie;
* use separate tools for metadata and artwork.

MovieStream addresses this by introducing a **library/product layer around VLC**.

---

# 3. Product Goals

## Primary goals

MovieStream V1 must:

1. Provide an attractive personal movie-library experience.
2. Automatically discover movies from user-selected folders.
3. Organize discovered media into meaningful movie entries.
4. Retrieve and display useful movie metadata.
5. Provide posters and backdrops.
6. Provide powerful but simple search/filtering.
7. Provide reliable VLC-powered playback.
8. Remember playback progress.
9. Provide subtitle support.
10. Provide multiple audio-track support.
11. Maintain watch history.
12. Provide a watchlist.
13. Work offline for core functionality.
14. Work on macOS, Windows and Linux.
15. Avoid requiring users to separately install VLC.
16. Remain lightweight relative to comparable Electron applications.
17. Have an architecture capable of evolving into V2–V5 without requiring a fundamental rewrite.

---

# 4. Non-Goals for V1

V1 will **not** attempt to provide:

* User accounts
* Cloud synchronization
* A public streaming service
* A hosted movie catalogue
* DRM
* Social networking
* Recommendations powered by AI
* Mobile applications
* Smart TV applications
* A dedicated streaming server
* Large-scale transcoding infrastructure
* Adaptive bitrate streaming infrastructure
* Content distribution/CDN infrastructure
* Subscription management
* Payments
* User-generated movie hosting
* Piracy-related functionality

The application is intended for media the user has the legal right to access.

---

# 5. Target Users

### Primary user

A person who has a collection of movies stored on:

* laptop
* desktop
* external SSD/HDD
* NAS-mounted directories
* other locally accessible storage

and wants a better way to browse and watch them.

### Secondary user

A technically inclined user who wants a highly customizable personal media application and may eventually connect it to their own media server.

---

# 6. Core User Experience

The intended experience:

```text
Install MovieStream
        │
        ▼
Launch
        │
        ▼
Add Movie Folder
        │
        ▼
MovieStream scans folder
        │
        ▼
Movies discovered
        │
        ▼
Media analyzed
        │
        ▼
Movies identified
        │
        ▼
Metadata/artwork retrieved
        │
        ▼
Library populated
        │
        ▼
User browses library
        │
        ├──────────────┐
        ▼              ▼
   Movie details    Continue Watching
        │              │
        └──────┬───────┘
               ▼
            Playback
               │
               ▼
        VLC/libVLC engine
               │
               ▼
        Progress recorded
```

The experience should feel closer to:

> **"My own little Netflix"**

than:

> **"A fancy file browser."**

---

# 7. Functional Requirements

# 7.1 Library Management

The application must allow users to manage their movie sources.

### Add folders

Users can:

* select a folder through the native file picker;
* add multiple folders;
* add folders from different drives;
* add nested movie directories.

Example:

```text
/Users/user/Movies
/Volumes/ExternalDrive/Films
/Volumes/NAS/Movies
```

Each folder becomes a **library source**.

### Remove folders

Removing a source must not delete the actual movie files.

It only removes the source from MovieStream's index.

The application should clearly communicate this.

### Source status

The application should be able to indicate:

* available
* unavailable
* scanning
* indexing
* inaccessible
* disconnected

This becomes particularly important for external drives.

---

# 7.2 Automatic Scanning

When a source is added:

```text
Folder
   ↓
Recursive scanner
   ↓
Candidate files
   ↓
Media validation
   ↓
Media analysis
   ↓
Movie identification
```

The scanner should recursively inspect supported directories.

The application must avoid freezing the UI during scanning.

Scanning should therefore occur asynchronously/backgrounded.

The UI should show progress such as:

> Scanning Movies…
> 1,284 files discovered
> 73 movies identified

---

# 7.3 Incremental Indexing

The application should **not perform a full scan every time it launches**.

After initial indexing:

```text
Library
   │
   ▼
Detect changes
   │
   ├── New file
   ├── Removed file
   ├── Modified file
   └── Moved/renamed file
```

Only relevant changes should be processed.

This is important for users with large libraries.

---

# 7.4 Movie Detection

The system should determine whether a discovered file is likely to represent a movie.

It should understand common naming conventions such as:

```text
Inception.2010.mkv
Inception (2010).mp4
Dune.Part.Two.2024.1080p.mkv
The.Matrix.1999.4K.mkv
```

The filename parser should attempt to extract:

* title
* year
* edition/version information where useful

The system should not blindly trust filenames.

---

# 7.5 Duplicate Detection

The application should avoid creating multiple movie records simply because:

```text
Dune.2021.1080p.mkv
Dune.2021.2160p.mkv
```

exist.

The conceptual model should support:

```text
Movie
 │
 ├── Media Version A
 └── Media Version B
```

This prepares the architecture for multiple versions of the same movie.

---

# 7.6 Media Analysis

VLC/libVLC should be leveraged wherever practical to obtain media information.

Possible information includes:

* duration
* dimensions
* codecs
* audio tracks
* subtitle tracks
* video tracks
* media format
* other available stream information

The application should not recreate media-analysis functionality unnecessarily.

---

# 7.7 Metadata Extraction

Movie metadata should be separated into two categories.

### File/media metadata

Obtained from the actual media file.

Examples:

* duration
* resolution
* codec
* audio tracks
* subtitle tracks

### Movie metadata

Obtained from a metadata provider.

Examples:

* official title
* release year
* synopsis
* genres
* cast
* director
* poster
* backdrop
* rating

The application should use a provider abstraction:

```text
MetadataService
       │
 ┌─────┼─────┐
 │     │     │
Provider A Provider B Local
```

The application should not permanently hard-code itself to one provider.

---

# 7.8 Metadata Matching

Automatic metadata matching should be attempted when a movie is indexed.

Example:

```text
Dune.Part.Two.2024.1080p.mkv
             │
             ▼
       title + year
             │
             ▼
     Metadata provider
             │
             ▼
       Dune: Part Two
```

The application should account for imperfect matches.

Where automatic identification fails, the user should be able to:

* search manually;
* select the correct movie;
* correct metadata;
* potentially identify the movie later.

---

# 7.9 Posters and Backdrops

The library should display cinematic artwork.

Each movie may have:

* poster
* backdrop
* thumbnail
* title
* year
* metadata

Artwork should preferably be cached locally.

The application should not repeatedly download the same poster every time it opens.

---

# 7.10 Library Interface

The library should provide multiple useful views.

### Home

Potential sections:

```text
Continue Watching

Recently Added

Recently Watched

Watchlist

Unwatched
```

### All Movies

Grid-based movie browsing.

### Search

Search by relevant movie information.

### Filters

Potential V1 filters:

* genre
* year
* watched/unwatched
* watchlist
* source
* rating where metadata exists

### Sorting

Potential options:

* title
* release year
* date added
* recently watched
* rating

The exact UI can evolve independently from the underlying library architecture.

---

# 7.11 Movie Details

Selecting a movie should open a detailed view.

Example:

```text
┌─────────────────────────────────────┐
│              BACKDROP               │
│                                     │
│   DUNE: PART TWO                    │
│   2024 • 2h 46m                     │
│                                     │
│   [ Play ] [ Add to Watchlist ]     │
│                                     │
│   Description...                    │
│                                     │
│   Genres • Cast • Director          │
└─────────────────────────────────────┘
```

The movie details page should expose useful information without overwhelming the user.

---

# 7.12 Playback

Playback must be powered by VLC/libVLC.

The application should not implement its own media decoder.

The playback layer should support, where provided by VLC:

* play
* pause
* stop
* seek
* volume
* mute
* fullscreen
* playback speed
* audio selection
* subtitle selection
* supported video tracks
* chapters where available

---

# 7.13 Custom Player UI

The application should **not simply reproduce the standard VLC interface**.

The player interface belongs to MovieStream.

This allows us to develop:

* custom controls;
* animations;
* contextual information;
* gestures;
* keyboard shortcuts;
* cinematic transitions;
* custom overlays;
* responsive layouts.

The media engine remains VLC.

---

# 7.14 Resume Playback

When playback begins:

```text
Movie
 ↓
PlaybackState
 ↓
VLC
```

During playback, the application records position.

For example:

```text
Movie: Interstellar
Position: 01:23:17
Duration: 02:49:00
```

When the movie is opened again:

```text
Resume from 1:23:17?
```

The user should be able to:

* resume;
* restart from beginning.

---

# 7.15 Watch History

The application must maintain watch history.

History can record:

* movie
* last playback position
* last watched timestamp
* completion state

The UI can then expose:

```text
Recently Watched
```

and:

```text
Continue Watching
```

---

# 7.16 Completion Detection

The application should determine when a movie has effectively been watched.

It should not necessarily require reaching exactly 100%.

A configurable completion threshold can eventually be used.

For example:

```text
> 90–95% watched
       ↓
Consider movie completed
```

The exact threshold should be configurable internally and potentially exposed later.

---

# 7.17 Watchlist

Users can add/remove movies from a watchlist.

The watchlist should be independent of watch history.

Example:

```text
Watchlist
├── Oppenheimer
├── Dune
└── Blade Runner 2049
```

A movie can be:

```text
Watchlist + Watched
```

because the user may want to revisit it.

---

# 7.18 Subtitle Support

MovieStream should use VLC's subtitle capabilities.

It should support subtitles recognized by VLC, subject to VLC's supported formats.

The user should be able to:

* enable/disable subtitles;
* choose among subtitle tracks;
* load an external subtitle file where supported;
* change relevant subtitle settings where practical.

Where subtitle files are automatically discovered alongside media, MovieStream should consider associating them with the movie.

Example:

```text
Dune.mkv
Dune.en.srt
Dune.fr.srt
```

---

# 7.19 Multiple Audio Tracks

If a movie contains:

```text
English
French
Spanish
```

the player should allow the user to select the desired track.

VLC handles the underlying media functionality.

MovieStream provides the interface.

---

# 7.20 External Media

V1 should primarily be **local-first**.

However, the architecture should not assume:

```text
Media = local file
```

Instead:

```text
MediaSource
├── LocalFile
├── NetworkSource
└── FutureRemoteStream
```

V1 may support VLC-compatible network media where practical, but a dedicated MovieStream media server is explicitly future functionality.

---

# 8. Library/Indexing Architecture

This is one of the most important V1 subsystems.

## Proposed conceptual pipeline

```text
               Library Manager
                      │
                      ▼
                 Source Manager
                      │
                      ▼
                   Scanner
                      │
                      ▼
                File Candidates
                      │
                      ▼
               Media Analyzer
                      │
                      ▼
              Movie Identification
                      │
                      ▼
               Metadata Resolver
                      │
                      ▼
                 Library DB
```

---

# 9. Core data model

The database should conceptually separate:

### Movie

Represents the logical movie.

```text
Movie
├── id
├── title
├── original_title
├── year
├── description
├── poster
├── backdrop
├── genres
├── cast
├── director
└── metadata
```

### Media

Represents an actual playable representation.

```text
Media
├── id
├── movie_id
├── source_id
├── path
├── size
├── duration
├── format
├── resolution
├── video information
├── audio information
└── subtitle information
```

### Library Source

```text
LibrarySource
├── id
├── path
├── name
├── status
└── last_scanned
```

### Playback State

```text
PlaybackState
├── movie_id
├── position
├── duration
├── completed
└── updated_at
```

### Watch History

```text
WatchHistory
├── movie_id
├── started_at
├── completed_at
└── last_position
```

### Watchlist

```text
Watchlist
├── movie_id
└── added_at
```

This is conceptual rather than a final SQL schema.

---

# 10. Why this data model matters

The architecture deliberately avoids:

```text
Movie = File
```

because that would create problems later.

Instead:

```text
Movie
  │
  ├── Media
  ├── Playback state
  ├── Watch history
  ├── Metadata
  └── Watchlist state
```

This allows us to eventually support:

```text
Movie
 ├── 1080p local file
 ├── 4K local file
 ├── network stream
 └── remote server stream
```

without redesigning the entire application.

---

# 11. Database Choice

V1 should use **SQLite**.

Reasons:

* local-first
* no server required
* lightweight
* reliable
* mature
* fast enough for very large personal libraries
* easy to back up
* works well with desktop applications

The application should access the database through a repository/data-access abstraction rather than allowing UI components to issue SQL directly.

---

# 12. Application Architecture

The architecture should enforce clear boundaries.

```text
┌──────────────────────────────────────────┐
│                  UI                      │
│              React/Web UI                │
└──────────────────┬───────────────────────┘
                   │
             Tauri bridge
                   │
┌──────────────────▼───────────────────────┐
│          APPLICATION LAYER              │
│                                          │
│ Library • Playback • Search • Watchlist │
│ History • Metadata • Settings            │
└───────────┬────────────────┬─────────────┘
            │                │
            ▼                ▼
   ┌────────────────┐ ┌────────────────┐
   │ Repository     │ │ Media Player   │
   │ abstractions   │ │ abstraction    │
   └───────┬────────┘ └───────┬────────┘
           │                  │
           ▼                  ▼
        SQLite             VLC Adapter
                                │
                                ▼
                             libVLC
                                │
                                ▼
                           VLC Engine
```

---

# 13. The VLC Adapter

This should be one of the strongest architectural boundaries.

The rest of the application should not constantly interact directly with raw libVLC APIs.

Instead:

```text
Application
     │
     ▼
MediaPlayer interface
     │
     ▼
VLC implementation
     │
     ▼
libVLC
```

This means another developer could eventually implement:

```text
MediaPlayer
   ├── VLC
   ├── MPV
   └── Future backend
```

without rewriting the application layer.

---

# 14. Metadata Adapter

Likewise:

```text
MetadataService
      │
      ├── Provider A
      ├── Provider B
      └── Local/manual metadata
```

The library should never care which external provider supplied the metadata.

---

# 15. Repository Abstraction

Similarly:

```text
MovieRepository
      │
      ├── SQLite
      └── Future server repository
```

This is important for V3 when the architecture becomes client/server.

---

# 16. Filesystem Abstraction

The library should interact with a filesystem abstraction:

```text
MediaSource
      │
      ├── Local filesystem
      ├── Mounted network filesystem
      └── Future remote source
```

Again, this prevents the application from assuming everything will forever be a local path.

---

# 17. Tauri Architecture

The intended desktop structure is:

```text
┌───────────────────────────────┐
│          React UI             │
│                               │
│ Library / Player / Settings   │
└───────────────┬───────────────┘
                │
          Tauri commands
          + events
                │
┌───────────────▼───────────────┐
│          Rust Layer           │
│                               │
│ Application services          │
│ Library / DB / filesystem     │
│ VLC integration               │
└───────────────┬───────────────┘
                │
          native interfaces
                │
        ┌───────┴────────┐
        ▼                ▼
     SQLite           libVLC
                         │
                         ▼
                    VLC Engine
```

The exact binding/FFI mechanism remains a **technical design decision**, not a product requirement.

---

# 18. VLC Bundling Requirement

A core requirement is:

> **The end user should not have to install VLC separately.**

The application distribution should therefore investigate and implement bundling of the necessary VLC/libVLC runtime and associated modules.

Conceptually:

```text
MovieStream Installer
        │
        ├── MovieStream
        ├── VLC/libVLC runtime
        └── required VLC modules
```

The exact packaging mechanism must be validated independently for:

```text
macOS
Windows
Linux
```

including architecture considerations such as ARM64 and x86-64 where supported.

---

# 19. Cross-Platform Requirements

V1 must target:

### macOS

Potential architectures:

* Apple Silicon
* Intel where practical

### Windows

At minimum:

* modern 64-bit Windows

### Linux

The project should define supported distributions/packaging formats during implementation.

Potential distribution mechanisms may include:

* AppImage
* `.deb`
* Flatpak

The final selection is a technical/distribution decision.

---

# 20. Performance Requirements

Performance is a first-class requirement.

The application should:

* launch quickly;
* remain responsive during scanning;
* avoid blocking the UI;
* avoid unnecessary rescans;
* avoid unnecessary metadata requests;
* cache artwork;
* avoid unnecessary file hashing;
* use background workers for expensive operations;
* minimize memory consumption;
* release resources appropriately.

A large library should not make the interface unusable.

---

# 21. Offline-First Behavior

The core application must remain usable without internet access.

Offline functionality should include:

* browsing existing library;
* searching;
* filtering;
* playing local movies;
* subtitles;
* playback history;
* watchlist;
* resume playback.

Internet connectivity should primarily be required for optional services such as metadata/artwork retrieval.

If metadata retrieval fails:

> **The movie should still work.**

This is an important resilience requirement.

---

# 22. Error Handling

The application must gracefully handle:

### Missing media

```text
Movie indexed
     ↓
File removed
     ↓
Movie unavailable
```

The application should not crash.

### Disconnected drive

```text
External Drive
     ↓
Disconnected
     ↓
Library remains visible
     ↓
Movie marked unavailable
```

### Metadata failure

Movie remains playable.

### Corrupt media

VLC's error should be surfaced in a useful manner.

### Unsupported media

The user should receive a meaningful explanation rather than a generic application failure.

---

# 23. Settings

V1 should have a settings/preferences area.

Potential settings:

### Library

* scan behavior
* watched-state behavior
* default library locations

### Playback

* default volume
* playback speed
* subtitle preferences
* audio preferences
* resume behavior

### Appearance

* theme
* interface preferences
* animation preferences where practical

### Metadata

* metadata provider
* refresh metadata
* artwork caching

### Application

* startup behavior
* notifications where appropriate
* diagnostic/logging options

---

# 24. Search

Search should feel instantaneous for the local library.

It should search fields such as:

* title
* original title
* year
* genres
* cast/director where available

Search should not require an internet request.

---

# 25. UI/UX Philosophy

The application should prioritize:

### Cinematic

Movies should feel like the primary content.

### Interactive

The UI can contain:

* animated transitions;
* hover states;
* dynamic backgrounds;
* rich movie cards;
* contextual controls.

### Lightweight

Visual sophistication should not result in an unnecessarily bloated application.

### Accessible

The application should remain usable with:

* keyboard navigation;
* readable typography;
* appropriate contrast;
* clear controls;
* sensible focus behavior.

### Consistent

All screens should follow a coherent visual language.

---

# 26. "Fun/Witty" Features

These are encouraged, but they must serve the product.

Potential examples:

### Dynamic empty state

Instead of:

> No movies found.

Something more engaging:

> **The cinema is empty. Add a movie folder to begin.**

### Continue Watching

Instead of merely listing files:

> **Pick up where you left off.**

### Recently Added

> **Fresh arrivals**

The exact copy is a design decision.

The principle is:

> **Personality without sacrificing usability.**

---

# 27. V1 Acceptance Criteria

V1 can be considered functionally complete when a user can:

1. Install MovieStream without separately installing VLC.
2. Launch it on macOS, Windows or Linux.
3. Add a movie directory.
4. Have the directory scanned without freezing the application.
5. Have supported movie files identified.
6. Have movies represented independently from their physical files.
7. Retrieve metadata when internet access is available.
8. Display posters/backdrops.
9. Browse the movie library.
10. Search the library.
11. Filter and sort the library.
12. Open movie details.
13. Play a movie through VLC/libVLC.
14. Pause, seek, change volume and enter fullscreen.
15. Select supported audio tracks.
16. Select supported subtitle tracks.
17. Resume an interrupted movie.
18. See recently watched movies.
19. Add/remove movies from a watchlist.
20. Continue using the core application offline.
21. Handle unavailable files without crashing.
22. Detect newly added media during subsequent scans.
23. Avoid unnecessary full-library rescans.
24. Maintain a responsive interface during background operations.

---

# 28. V1 Architecture Principle

The single most important architectural rule is:

> **The product must not become coupled to VLC, SQLite, a specific metadata provider, or the local filesystem at the application level.**

Instead:

```text
              APPLICATION
                   │
        ┌──────────┼───────────┐
        │          │           │
   MediaPlayer  Repository  Metadata
        │          │           │
        ▼          ▼           ▼
       VLC       SQLite     Provider
```

This is what gives us room to experiment.

---

# 29. Future Evolution — V2 → V5

The following is **not V1 scope**. It is the planned evolution path.

## V2 — Remote Media Sources

Goal:

> Allow MovieStream to access media that isn't physically stored on the same computer.

Potential sources:

```text
Desktop
   │
   ├── Local files
   ├── NAS
   ├── Network shares
   └── Remote media URLs
```

Features may include:

* network libraries;
* remote media sources;
* improved network playback;
* remote metadata;
* source management.

The V1 `MediaSource` abstraction should make this possible without rewriting the library.

---

# 30. V3 — MovieStream Media Server

Introduce a dedicated server.

```text
                 MovieStream Server
                        │
             ┌──────────┼──────────┐
             │          │          │
           Media       DB       Streaming
           Storage
             │
             ▼
       Desktop Clients
```

The server could provide:

* authentication;
* centralized library;
* remote access;
* user profiles;
* watch-state synchronization;
* API;
* media discovery;
* storage management.

The desktop client becomes one client of the platform.

---

# 31. V4 — Transcoding & Adaptive Streaming

This is where FFmpeg becomes much more important.

```text
Original Movie
      │
      ▼
    FFmpeg
      │
 ┌────┼────┐
 ▼    ▼    ▼
1080 720  480
 │    │    │
 └────┼────┘
      ▼
 HLS / DASH
      │
      ▼
   Client
```

Potential capabilities:

* transcoding;
* HLS;
* MPEG-DASH;
* adaptive bitrate;
* hardware-accelerated transcoding;
* bandwidth-aware playback;
* streaming optimization.

VLC/libVLC can remain a playback component.

---

# 32. V5 — Full Personal Streaming Platform

The long-term system could evolve into:

```text
                       MovieStream
                            │
             ┌──────────────┼──────────────┐
             │              │              │
          Desktop         Mobile           TV
             │              │              │
             └──────────────┼──────────────┘
                            │
                         API
                            │
                     Media Server
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
       Database          Storage          Streaming
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
                       FFmpeg/etc.
```

Potential V5 functionality:

* multiple users;
* profiles;
* synchronized watch history;
* multiple clients;
* remote access;
* advanced media management;
* recommendations;
* collections;
* playlists;
* server administration;
* sophisticated streaming controls.

At that point MovieStream would have evolved from:

> **a desktop movie library**

into:

> **a self-hosted personal media streaming platform.**

---

# 33. Evolution Strategy

The intended evolution is:

```text
V1
Personal Library
      │
      ▼
V2
Remote Sources
      │
      ▼
V3
Media Server
      │
      ▼
V4
Transcoding / Adaptive Streaming
      │
      ▼
V5
Multi-device Streaming Platform
```

The critical point is that each version should **extend the previous architecture rather than invalidate it**.

---

# 34. Rewrite Resistance

The architecture should specifically protect against several future changes.

### VLC → another media engine

Should require:

```text
New MediaPlayer adapter
```

rather than rewriting the application.

### SQLite → server database

Should require:

```text
New repository implementation
```

rather than rewriting the UI.

### Local → remote media

Should require:

```text
New MediaSource implementation
```

rather than rewriting Movie entities.

### Metadata provider changes

Should require:

```text
New metadata adapter
```

rather than modifying the entire library.

### New UI

The business logic should remain usable even if developers completely redesign the interface.

### New developers experimenting

A contributor should be able to experiment with:

* different player implementations;
* different metadata providers;
* different databases;
* different library algorithms;
* alternative UI designs;

without being forced to understand the entire codebase first.

---

# 35. Architectural Rule for Contributors

A useful project rule will eventually be:

> **Changes should be made at the narrowest appropriate abstraction boundary.**

For example:

If someone wants to change metadata providers:

**Don't modify LibraryManager everywhere.**

Implement/change the metadata adapter.

If someone wants to change playback:

**Don't modify Movie, Watchlist and History.**

Change the media-player implementation.

If someone wants a new UI:

**Don't rewrite the database.**

Consume the application services.

This makes the project much more welcoming to experimentation.

---

# 36. V1 Success Definition

V1 succeeds if a user can install the application, point it at a folder containing their movies, walk away while it indexes, and return to a **beautiful, responsive movie library where they can browse, discover, organize and watch their collection without thinking about the underlying files or VLC.**

The ideal reaction is not:

> "This is a better VLC."

It is:

> **"This feels like a streaming service, but these are my movies."**

---

## One architectural decision I'd make now

I would **not** commit the project to a giant monolithic Rust backend or a giant React frontend.

Keep the boundaries explicit:

```text
UI
 │
 ▼
Application Services
 │
 ├──────────────┬───────────────┐
 ▼              ▼               ▼
Library       Playback       Metadata
 │              │               │
 ▼              ▼               ▼
Repository   MediaPlayer     Provider
 │              │
 ▼              ▼
SQLite         VLC/libVLC
```

That structure is intentionally boring underneath, so that the **experience can be extremely creative on top**.
