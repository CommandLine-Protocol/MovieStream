import React from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Movie } from "../types";
import { Play, Bookmark, Star, Film, AlertCircle } from "lucide-react";
import { usePlayback } from "../state/PlaybackContext";
import { useLibrary } from "../state/LibraryContext";

interface MovieCardProps {
  movie: Movie;
  progressRatio?: number; // 0 to 1
  isUnavailable?: boolean;
}

export const MovieCard: React.FC<MovieCardProps> = ({
  movie,
  progressRatio,
  isUnavailable,
}) => {
  const { startMovie } = usePlayback();
  const { toggleWatchlist, watchlist, openMovieDetails } = useLibrary();

  const isSaved = watchlist.some((m) => m.id === movie.id);
  const posterUrl = movie.poster_path
    ? movie.poster_path.startsWith("http")
      ? movie.poster_path
      : convertFileSrc(movie.poster_path)
    : undefined;

  const handlePlayClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    startMovie(movie);
  };

  const handleWatchlistClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    toggleWatchlist(movie.id);
  };

  return (
    <div className="movie-card" onClick={() => openMovieDetails(movie.id)}>
      <div className="movie-card-poster-wrapper">
        {posterUrl ? (
          <img src={posterUrl} alt={movie.title} className="movie-card-poster" />
        ) : (
          <div className="movie-card-fallback-poster">
            <Film size={36} color="var(--text-muted)" style={{ marginBottom: 8 }} />
            <span style={{ fontSize: "0.85rem", fontWeight: 600 }}>{movie.title}</span>
            {movie.year && (
              <span style={{ fontSize: "0.75rem", color: "var(--text-muted)", marginTop: 4 }}>
                {movie.year}
              </span>
            )}
          </div>
        )}

        {/* Top badges */}
        <div style={{ position: "absolute", top: 8, left: 8, right: 8, display: "flex", justifyContent: "space-between" }}>
          {isUnavailable ? (
            <span className="badge badge-unavailable" style={{ display: "flex", alignItems: "center", gap: 3 }}>
              <AlertCircle size={10} /> Disconnected
            </span>
          ) : (
            <span />
          )}

          {movie.rating ? (
            <span
              className="badge"
              style={{
                background: "rgba(0, 0, 0, 0.75)",
                color: "var(--accent-gold)",
                backdropFilter: "blur(6px)",
                display: "flex",
                alignItems: "center",
                gap: 3,
              }}
            >
              <Star size={10} fill="currentColor" />
              {movie.rating.toFixed(1)}
            </span>
          ) : null}
        </div>

        {/* Hover Overlay with Quick Actions */}
        <div className="movie-card-overlay">
          <div style={{ display: "flex", gap: 8, justifyContent: "center" }}>
            <button
              className="btn-icon"
              style={{ background: "var(--accent-primary)", border: "none" }}
              onClick={handlePlayClick}
              title="Play"
            >
              <Play size={18} fill="#fff" color="#fff" />
            </button>
            <button
              className="btn-icon"
              onClick={handleWatchlistClick}
              title={isSaved ? "Remove from Watchlist" : "Add to Watchlist"}
            >
              <Bookmark size={16} fill={isSaved ? "var(--accent-primary)" : "none"} color={isSaved ? "var(--accent-primary)" : "#fff"} />
            </button>
          </div>
        </div>

        {/* Continue Watching Progress Bar */}
        {progressRatio !== undefined && progressRatio > 0 && (
          <div style={{ position: "absolute", bottom: 0, left: 0, right: 0, height: 4, background: "rgba(0,0,0,0.6)" }}>
            <div
              style={{
                height: "100%",
                width: `${Math.min(100, Math.round(progressRatio * 100))}%`,
                background: "var(--accent-primary)",
              }}
            />
          </div>
        )}
      </div>

      <div className="movie-card-info">
        <h4 className="movie-card-title" title={movie.title}>
          {movie.title}
        </h4>
        <div className="movie-card-meta">
          <span>{movie.year || "Movie"}</span>
          <span>{movie.genres[0] || ""}</span>
        </div>
      </div>
    </div>
  );
};
