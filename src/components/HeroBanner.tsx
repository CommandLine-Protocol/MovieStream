import React from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Movie } from "../types";
import { Play, Bookmark, Info, Star } from "lucide-react";
import { usePlayback } from "../state/PlaybackContext";
import { useLibrary } from "../state/LibraryContext";

interface HeroBannerProps {
  movie: Movie | null;
}

export const HeroBanner: React.FC<HeroBannerProps> = ({ movie }) => {
  const { startMovie } = usePlayback();
  const { toggleWatchlist, watchlist, openMovieDetails } = useLibrary();

  if (!movie) {
    return (
      <div className="hero-banner" style={{ background: "#0f1019", justifyContent: "center" }}>
        <div style={{ textAlign: "center", maxWidth: 500, padding: 32 }}>
          <h2 style={{ fontSize: "2rem", marginBottom: 12 }}>Welcome to MovieStream</h2>
          <p style={{ color: "var(--text-secondary)", marginBottom: 20 }}>
            Your personal streaming library. Add a movie folder to begin scanning your collection.
          </p>
        </div>
      </div>
    );
  }

  const isSaved = watchlist.some((m) => m.id === movie.id);

  // Convert local path to Tauri asset URL or fallback
  const backdropUrl = movie.backdrop_path
    ? movie.backdrop_path.startsWith("http")
      ? movie.backdrop_path
      : convertFileSrc(movie.backdrop_path)
    : undefined;

  return (
    <div className="hero-banner">
      {backdropUrl ? (
        <img src={backdropUrl} alt={movie.title} className="hero-backdrop" />
      ) : (
        <div
          className="hero-backdrop"
          style={{
            background: "radial-gradient(circle at 70% 30%, #1e2238 0%, #0a0b10 80%)",
          }}
        />
      )}
      <div className="hero-gradient" />

      <div className="hero-content">
        <div
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 6,
            padding: "4px 10px",
            background: "rgba(255, 255, 255, 0.12)",
            backdropFilter: "blur(8px)",
            borderRadius: "var(--radius-sm)",
            fontSize: "0.75rem",
            fontWeight: 700,
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            marginBottom: 12,
          }}
        >
          <Star size={12} fill="var(--accent-gold)" color="var(--accent-gold)" />
          <span>Featured Spotlight</span>
        </div>

        <h1 className="hero-title">{movie.title}</h1>

        <div className="hero-meta">
          {movie.year && <span>{movie.year}</span>}
          {movie.rating && (
            <span style={{ display: "flex", alignItems: "center", gap: 4, color: "var(--accent-gold)" }}>
              <Star size={14} fill="currentColor" /> {movie.rating.toFixed(1)}
            </span>
          )}
          {movie.genres.length > 0 && (
            <span>• {movie.genres.slice(0, 3).join(", ")}</span>
          )}
        </div>

        <p className="hero-overview">
          {movie.description || "No overview available for this title."}
        </p>

        <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
          <button className="btn btn-primary" onClick={() => startMovie(movie)}>
            <Play size={18} fill="currentColor" /> Play Now
          </button>
          <button
            className={`btn ${isSaved ? "btn-primary" : "btn-secondary"}`}
            onClick={() => toggleWatchlist(movie.id)}
          >
            <Bookmark size={16} fill={isSaved ? "currentColor" : "none"} />
            {isSaved ? "In Watchlist" : "Add to Watchlist"}
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => openMovieDetails(movie.id)}
          >
            <Info size={16} /> Details
          </button>
        </div>
      </div>
    </div>
  );
};
