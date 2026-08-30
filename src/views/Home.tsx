import React from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useLibrary } from "../state/LibraryContext";
import { usePlayback } from "../state/PlaybackContext";
import { HeroBanner } from "../components/HeroBanner";
import { MovieRow } from "../components/MovieRow";
import { Film, FolderPlus, Play, Tv, Star } from "lucide-react";
import { ContinueWatchingItem } from "../types";

export const Home: React.FC = () => {
  const {
    movies,
    series,
    continueWatching,
    recentlyWatched,
    watchlist,
    addSourceByDialog,
    openMovieDetails,
    openSeriesDetails,
    isLoading,
  } = useLibrary();

  const { startMovie, startEpisode } = usePlayback();

  const featuredMovie = movies.length > 0 ? movies[0] : null;

  const recentlyAdded = [...movies].sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  const handleContinueWatchingClick = (item: ContinueWatchingItem) => {
    if (item.progress.media_type === "movie" && item.progress.movie_id) {
      const movie = movies.find((m) => m.id === item.progress.movie_id);
      if (movie) {
        startMovie(movie, item.progress.media_id);
      } else {
        openMovieDetails(item.progress.movie_id);
      }
    } else if (item.progress.media_type === "episode" && item.progress.episode_id) {
      startEpisode(item.progress.episode_id, item.progress.media_id);
    }
  };

  if (!isLoading && movies.length === 0 && series.length === 0) {
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          minHeight: "calc(100vh - 120px)",
          padding: "32px",
          textAlign: "center",
        }}
      >
        <div
          style={{
            width: 80,
            height: 80,
            borderRadius: "var(--radius-full)",
            background: "rgba(229, 9, 20, 0.12)",
            border: "1px solid rgba(229, 9, 20, 0.3)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            marginBottom: 24,
            boxShadow: "0 0 32px rgba(229, 9, 20, 0.2)",
          }}
        >
          <Film size={40} color="var(--accent-primary)" />
        </div>
        <h2 style={{ fontSize: "2.2rem", fontWeight: 800, marginBottom: 12 }}>
          The cinema is empty.
        </h2>
        <p
          style={{
            color: "var(--text-secondary)",
            fontSize: "1.1rem",
            maxWidth: 480,
            lineHeight: 1.6,
            marginBottom: 28,
          }}
        >
          Add a folder to begin scanning movies and TV shows into your personal streaming library.
        </p>
        <button className="btn btn-primary" onClick={addSourceByDialog} style={{ padding: "14px 28px", fontSize: "1.05rem" }}>
          <FolderPlus size={20} /> Add Media Folder
        </button>
      </div>
    );
  }

  return (
    <div>
      {/* Cinematic Hero */}
      <HeroBanner movie={featuredMovie} />

      <div style={{ marginTop: 24 }}>
        {/* Unified Continue Watching Section */}
        {continueWatching.length > 0 && (
          <div className="movie-row-container">
            <h2 className="movie-row-title">Continue Watching</h2>
            <div className="movie-row-track">
              {continueWatching.map((item) => {
                const isMovie = item.progress.media_type === "movie";
                const title = isMovie ? item.movie_title || "Movie" : item.series_title || "TV Show";
                const subtitle = isMovie
                  ? item.movie_year ? `${item.movie_year}` : "Movie"
                  : `S${(item.progress.season_number || 1).toString().padStart(2, "0")}E${(item.progress.episode_number || 1).toString().padStart(2, "0")} • ${item.episode_title || "Episode"}`;

                const posterRaw = isMovie ? item.movie_poster : (item.episode_still || item.series_poster);
                const posterUrl = posterRaw
                  ? posterRaw.startsWith("http")
                    ? posterRaw
                    : convertFileSrc(posterRaw)
                  : undefined;

                const progressPct = item.progress.duration_seconds > 0
                  ? (item.progress.position_seconds / item.progress.duration_seconds) * 100
                  : 0;

                return (
                  <div
                    key={item.progress.id}
                    className="movie-card"
                    style={{ width: 220, flexShrink: 0 }}
                    onClick={() => handleContinueWatchingClick(item)}
                  >
                    <div className="movie-card-poster-wrapper" style={{ height: 130 }}>
                      {posterUrl ? (
                        <img src={posterUrl} alt={title} className="movie-card-poster" />
                      ) : (
                        <div className="movie-card-fallback-poster">
                          {isMovie ? <Film size={32} color="var(--text-muted)" /> : <Tv size={32} color="var(--text-muted)" />}
                        </div>
                      )}

                      <div
                        style={{
                          position: "absolute",
                          inset: 0,
                          background: "rgba(0, 0, 0, 0.4)",
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          opacity: 0,
                          transition: "opacity 0.2s ease",
                        }}
                        onMouseEnter={(e) => (e.currentTarget.style.opacity = "1")}
                        onMouseLeave={(e) => (e.currentTarget.style.opacity = "0")}
                      >
                        <div
                          style={{
                            width: 44,
                            height: 44,
                            borderRadius: "var(--radius-full)",
                            background: "var(--accent-gradient)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            boxShadow: "0 0 16px rgba(229, 9, 20, 0.6)",
                          }}
                        >
                          <Play size={20} fill="#fff" style={{ marginLeft: 2 }} />
                        </div>
                      </div>

                      {/* Progress Bar */}
                      <div className="movie-card-progress">
                        <div className="movie-card-progress-fill" style={{ width: `${progressPct}%` }} />
                      </div>
                    </div>

                    <div className="movie-card-info">
                      <div className="movie-card-title" title={title}>{title}</div>
                      <div className="movie-card-meta" style={{ fontSize: "0.78rem" }}>{subtitle}</div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* TV Series Row */}
        {series.length > 0 && (
          <div className="movie-row-container">
            <h2 className="movie-row-title">TV Series</h2>
            <div className="movie-row-track">
              {series.map((s) => {
                const posterUrl = s.poster_path
                  ? s.poster_path.startsWith("http")
                    ? s.poster_path
                    : convertFileSrc(s.poster_path)
                  : undefined;

                return (
                  <div key={s.id} className="movie-card" onClick={() => openSeriesDetails(s.id)}>
                    <div className="movie-card-poster-wrapper">
                      {posterUrl ? (
                        <img src={posterUrl} alt={s.title} className="movie-card-poster" />
                      ) : (
                        <div className="movie-card-fallback-poster">
                          <Tv size={36} color="var(--text-muted)" style={{ marginBottom: 8 }} />
                          <span style={{ fontSize: "0.85rem", fontWeight: 600 }}>{s.title}</span>
                        </div>
                      )}
                      {s.rating && s.rating > 0 && (
                        <div className="movie-card-badge">
                          <Star size={11} fill="currentColor" />
                          <span>{s.rating.toFixed(1)}</span>
                        </div>
                      )}
                    </div>
                    <div className="movie-card-info">
                      <div className="movie-card-title">{s.title}</div>
                      <div className="movie-card-meta">
                        {s.year && <span>{s.year}</span>}
                        {s.genres.length > 0 && <span>• {s.genres[0]}</span>}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Fresh Arrivals */}
        <MovieRow
          title="Fresh Movie Arrivals"
          movies={recentlyAdded.slice(0, 15)}
        />

        {/* Watchlist */}
        {watchlist.length > 0 && (
          <MovieRow
            title="Your Watchlist"
            movies={watchlist}
          />
        )}

        {/* Recently Watched */}
        {recentlyWatched.length > 0 && (
          <MovieRow
            title="Recently Watched"
            movies={recentlyWatched}
          />
        )}
      </div>
    </div>
  );
};
