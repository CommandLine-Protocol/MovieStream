import React, { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useLibrary } from "../state/LibraryContext";
import { Tv, Star } from "lucide-react";

export const AllSeries: React.FC = () => {
  const { series, openSeriesDetails, isLoading } = useLibrary();
  const [selectedGenre, setSelectedGenre] = useState<string>("All");

  const allGenres = Array.from(new Set(series.flatMap((s) => s.genres))).sort();

  const filteredSeries = series.filter((s) => {
    if (selectedGenre !== "All" && !s.genres.includes(selectedGenre)) {
      return false;
    }
    return true;
  });

  return (
    <div style={{ padding: "0 0 40px 0" }}>
      {/* Header */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 24,
          flexWrap: "wrap",
          gap: 16,
        }}
      >
        <div>
          <h1 style={{ fontSize: "2rem", fontWeight: 800, letterSpacing: "-0.02em" }}>
            TV Series
          </h1>
          <div style={{ fontSize: "0.9rem", color: "var(--text-secondary)", marginTop: 4 }}>
            {filteredSeries.length} series in your library
          </div>
        </div>

        {/* Genre Filter Pills */}
        <div style={{ display: "flex", gap: 8, overflowX: "auto", maxWidth: "100%", paddingBottom: 4 }}>
          <button
            className={`filter-pill ${selectedGenre === "All" ? "active" : ""}`}
            onClick={() => setSelectedGenre("All")}
          >
            All
          </button>
          {allGenres.map((g) => (
            <button
              key={g}
              className={`filter-pill ${selectedGenre === g ? "active" : ""}`}
              onClick={() => setSelectedGenre(g)}
            >
              {g}
            </button>
          ))}
        </div>
      </div>

      {/* Empty State */}
      {series.length === 0 && !isLoading && (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            minHeight: 300,
            textAlign: "center",
          }}
        >
          <Tv size={48} color="var(--text-muted)" style={{ marginBottom: 16 }} />
          <h3 style={{ fontSize: "1.3rem", fontWeight: 700, marginBottom: 8 }}>No TV Series Found</h3>
          <p style={{ color: "var(--text-secondary)", maxWidth: 420, fontSize: "0.92rem" }}>
            Add folders containing TV shows (named with <code>S01E01</code> or <code>1x01</code>) to start streaming seasons and episodes.
          </p>
        </div>
      )}

      {/* Series Grid */}
      <div className="movie-grid">
        {filteredSeries.map((s) => {
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
                    {s.year && (
                      <span style={{ fontSize: "0.75rem", color: "var(--text-muted)", marginTop: 4 }}>
                        {s.year}
                      </span>
                    )}
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
  );
};
