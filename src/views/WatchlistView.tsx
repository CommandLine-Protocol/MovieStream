import React from "react";
import { useLibrary } from "../state/LibraryContext";
import { MovieCard } from "../components/MovieCard";
import { Bookmark } from "lucide-react";

export const WatchlistView: React.FC = () => {
  const { watchlist } = useLibrary();

  return (
    <div style={{ padding: "32px 48px", maxWidth: 1600, margin: "0 auto" }}>
      <div style={{ display: "flex", alignItems: "baseline", gap: 12, marginBottom: 24 }}>
        <h2 style={{ fontSize: "2rem", fontWeight: 800 }}>Watchlist</h2>
        <span style={{ fontSize: "0.9rem", color: "var(--text-muted)", fontWeight: 600 }}>
          {watchlist.length} {watchlist.length === 1 ? "title" : "titles"}
        </span>
      </div>

      {watchlist.length === 0 ? (
        <div
          style={{
            textAlign: "center",
            padding: "80px 24px",
            background: "rgba(255, 255, 255, 0.02)",
            border: "1px dashed var(--border-subtle)",
            borderRadius: "var(--radius-lg)",
          }}
        >
          <Bookmark size={48} color="var(--text-muted)" style={{ marginBottom: 16 }} />
          <h3 style={{ fontSize: "1.4rem", marginBottom: 8 }}>Your watchlist is empty</h3>
          <p style={{ color: "var(--text-secondary)" }}>
            Click the bookmark icon on any movie card or details page to add it to your watchlist.
          </p>
        </div>
      ) : (
        <div className="movie-grid">
          {watchlist.map((movie) => (
            <MovieCard key={movie.id} movie={movie} />
          ))}
        </div>
      )}
    </div>
  );
};
