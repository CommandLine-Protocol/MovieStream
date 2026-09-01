# MovieStream

<div align="center">

![MovieStream Banner](https://img.shields.io/badge/MovieStream-Personal%20Cinema-e50914?style=for-the-badge&logo=film)

**Your Personal Local Streaming Platform**

Transform local video directories into a cinematic streaming experience with zero cloud dependencies.

[![Developed by CommandLine-Protocol](https://img.shields.io/badge/Developer-CommandLine--Protocol-181717?style=flat-square&logo=github)](https://github.com/CommandLine-Protocol)
[![Sponsored by Yimatt Technologies](https://img.shields.io/badge/Sponsor-Yimatt%20Technologies-e50914?style=flat-square&logo=safari)](https://yimatt.com)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8.svg?style=flat-square&logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61DAFB.svg?style=flat-square&logo=react)](https://reactjs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-blue.svg?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-3-003B57.svg?style=flat-square&logo=sqlite)](https://www.sqlite.org/)
[![VLC](https://img.shields.io/badge/Playback%20Engine-libVLC-FF8800.svg?style=flat-square&logo=vlc-media-player)](https://www.videolan.org/vlc/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)

</div>

---

## Overview

**MovieStream** is a lightweight, local-first desktop media application built with **Rust**, **Tauri 2**, and **React**. It indexes movie files stored on internal drives, external HDDs, and network storage into a modern, responsive streaming interface modeled after platforms like Netflix and Apple TV+.

MovieStream pairs an **offline-first SQLite database** and **libVLC media engine** with a cinematic frontend, providing full format support without requiring complex server infrastructure or transcoding overhead.

---

## Importance & Rationale

Managing personal movie libraries across local folders, external storage, and removable drives is often fragmented:
- **Generic File Browsers**: Lack metadata, cast information, posters, watch tracking, and resume points.
- **Complex Media Servers**: Require dedicated background servers, heavy memory footprints, user accounts, and CPU-intensive transcoding for basic playback.
- **Privacy & Ownership**: MovieStream operates strictly on your machine. Your library index, watch history, and playback stats remain local.
- **True Universal Playback**: Embedded hardware-accelerated playback with libVLC abstraction plays virtually any container, video codec (H.264, HEVC/H.265, AV1), and audio format.
- **Removable Media Resiliency**: Disconnecting an external hard drive preserves your watch history, watchlist entries, and progress. Reconnecting the drive instantly restores availability.

---

## Key Features

- **Cinematic Streaming Interface**:
  - Spotlight Hero featuring backdrop artwork, synopsis, genres, and instant playback.
  - *Continue Watching* row with exact progress bars and resume points.
  - *Fresh Arrivals*, *Your Watchlist*, and *Recently Watched* carousels.
  - Filterable library grid with instant genre pills, multi-criteria sorting, and availability indicators.
- **Intelligent Background Scanner**:
  - Non-blocking asynchronous directory scanner running off the UI thread.
  - Live progress updates with automatic real-time UI population.
  - Change detection for new, modified, and removed files.
- **Zero-Config Metadata & Cover Art Engine**:
  - **Zero Configuration Required**: Uses built-in public metadata sources (iTunes Search API) to fetch high-resolution posters, backdrops, cast, director, and overview data out-of-the-box.
  - **Local Artwork Scraper**: Automatically detects `poster.jpg`, `cover.jpg`, `folder.jpg`, and `<movie>.jpg` in your media directories.
  - **Optional TMDb API Key**: Easily configure personal TMDb keys via `.env` or in the Settings view.
- **Multi-Version Media Grouping**:
  - Consolidates multiple files for the same title (e.g., 4K HDR, 1080p BluRay, Extended Edition) under a single movie entry.
- **Comprehensive Playback Controls**:
  - Hardware-accelerated video streaming with precise timeline scrubbing and timecode display.
  - Multi-track audio selection and embedded/external subtitle switching (`.srt`, `.vtt`).
  - Playback speed adjustment (0.5x to 2.0x).
  - Configurable resume prompts and automatic completion threshold marking (90%+).

---

## Quick Start & Usage

### 1. Launching MovieStream
Download the latest desktop binary for your operating system or compile from source (see [Developer Guide](#developer-guide)).

### 2. Adding Media Folders
1. On initial launch, click **Add Movie Folder** from the Home view or navigate to **Settings > Library Sources**.
2. Select any local folder or external drive containing your movie collection (`.mkv`, `.mp4`, `.avi`, `.mov`, `.webm`, etc.).
3. The background scanner will automatically index files, parse metadata, and populate your library in real-time.

### 3. Playing Media & Managing Library
- Click **Play** on any title to begin playback.
- If you previously stopped watching a movie partway through, MovieStream will prompt you to resume or restart from the beginning.
- Use the **Bookmark** button to add titles to your personal Watchlist.
- Use the search bar to query titles, directors, actors, and genres.

---

## Architecture & Design

MovieStream is structured using Ports and Adapters (Hexagonal Architecture) with clean domain boundaries:

```
┌────────────────────────────────────────────────────────┐
│                   React 18 + TypeScript                │
│   Home • All Movies • Search • Movie Details • Player  │
└───────────────────────────┬────────────────────────────┘
                            │ Tauri IPC (Commands & Events)
┌───────────────────────────▼────────────────────────────┐
│              Application Services (Rust)               │
│   Library • Scanner • MediaAnalyzer • FilenameParser   │
│   MetadataResolver • Playback • Search • History       │
└───────┬───────────────────┬───────────────────┬────────┘
        │                   │                   │
┌───────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
│ MediaPlayer    │  │ Repository     │  │ Metadata       │
│ Abstraction    │  │ Abstraction    │  │ Abstraction    │
└───────┬────────┘  └───────┬────────┘  └───────┬────────┘
        │                   │                   │
┌───────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
│ VLC Adapter    │  │ SQLite DB      │  │ TMDB / Mock    │
└────────────────┘  └────────────────┘  └────────────────┘
```

- **Domain Layer (`src-tauri/src/domain/`)**: Pure domain models (`Movie`, `Media`, `LibrarySource`, `PlaybackState`, `WatchlistEntry`, `WatchHistoryEntry`, `AppSettings`).
- **Abstractions Layer (`src-tauri/src/abstractions/`)**: Trait contracts decoupling business logic from third-party libraries (`MediaPlayer`, `MovieRepository`, `MediaRepository`, `MetadataProvider`, `MediaSource`).
- **Adapters Layer (`src-tauri/src/adapters/`)**: Concrete implementations for SQLite storage, local filesystem scanning, libVLC playback, and metadata fetching.
- **Services Layer (`src-tauri/src/services/`)**: Orchestration of core business workflows (scanning, duplicate resolution, playback lifecycle).
- **Frontend Layer (`src/`)**: Component hierarchy, typed IPC interfaces, and React contexts.

---

## Developer Guide

### Prerequisites

Ensure you have the following installed on your system:
- **Node.js**: `v18.0.0` or higher
- **npm** or **yarn** / **pnpm**
- **Rust**: `v1.75.0` or higher (`rustup update stable`)
- **Tauri Prerequisites**: Follow the [Tauri 2 Prerequisites Guide](https://v2.tauri.app/start/prerequisites/) for your operating system.
- **VLC / libVLC**: Standard VLC installation for media playback bindings.

### Project Setup

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/CommandLine-Protocol/MovieStream.git
   cd MovieStream
   ```

2. **Install Frontend Dependencies**:
   ```bash
   npm install
   ```

3. **Configure Environment Variables (Optional)**:
   MovieStream works out of the box with zero configuration. If you wish to use your personal TMDb API key:
   ```bash
   cp .env.example .env
   ```
   Add your key to `.env`:
   ```env
   TMDB_API_KEY=your_tmdb_api_key_here
   ```

4. **Run in Development Mode**:
   ```bash
   npm run tauri dev
   ```
   *Alternatively, start the Vite dev server individually:*
   ```bash
   npm run dev
   ```

### Running Tests

Execute the automated Rust unit and integration test suite:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Build and validate the TypeScript frontend bundle:
```bash
npm run build
```

---

## Contributing

Contributions are welcome. To get started:
1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m 'feat: add amazing feature'`).
4. Ensure all Rust tests and frontend builds pass (`cargo test --manifest-path src-tauri/Cargo.toml && npm run build`).
5. Push to your branch (`git push origin feature/amazing-feature`).
6. Open a Pull Request.

## Credits & Attribution

- **Developed by**: [CommandLine-Protocol](https://github.com/CommandLine-Protocol)
- **Sponsored by**: [Yimatt Technologies](https://yimatt.com)

---

## Licensing

This project is dually licensed. You may choose to use, distribute, and modify this software under the terms of either:

* The **GNU General Public License v2.0** (`LICENSE-GPLv2`)
* The **GNU Lesser General Public License v2.1** (`LICENSE-LGPLv2.1`)

For more details, please refer to the respective license files in the root directory.

