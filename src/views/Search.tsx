import React, { useState, useEffect } from "react";
import { Movie } from "../types";
import * as ipc from "../ipc";
import { MovieCard } from "../components/MovieCard";
import { Search as SearchIcon, Film } from "lucide-react";

interface SearchProps {
  initialQuery?: string;
}

export const Search: React.FC<SearchProps> = ({ initialQuery = "" }) => {
  const [query, setQuery] = useState(initialQuery);
  const [results, setResults] = useState<Movie[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  useEffect(() => {
    setQuery(initialQuery);
  }, [initialQuery]);

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      return;
    }

    let active = true;
    setIsSearching(true);

    ipc.searchMovies(query).then((res) => {
      if (active) {
        setResults(res);
        setIsSearching(false);
      }
    });

    return () => {
      active = false;
    };
  }, [query]);

  return (
    <div style={{ padding: "32px 48px", maxWidth: 1600, margin: "0 auto" }}>
      {/* Search Header */}
      <div style={{ marginBottom: 32 }}>
        <h2 style={{ fontSize: "2rem", fontWeight: 800, marginBottom: 16 }}>Search Library</h2>
        <div style={{ position: "relative", maxWidth: 640 }}>
          <SearchIcon
            size={20}
            color="var(--text-muted)"
            style={{ position: "absolute", left: 16, top: 16, pointerEvents: "none" }}
          />
          <input
            type="text"
            className="input-search"
            style={{
              width: "100%",
              padding: "14px 20px 14px 48px",
              fontSize: "1.05rem",
              borderRadius: "var(--radius-lg)",
            }}
            placeholder="Search by title, original title, director, cast, year, genre…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            autoFocus
          />
        </div>
      </div>

      {query.trim() && (
        <div style={{ marginBottom: 20, fontSize: "0.95rem", color: "var(--text-secondary)" }}>
          Found <strong style={{ color: "#fff" }}>{results.length}</strong> {results.length === 1 ? "match" : "matches"} for "{query}"
        </div>
      )}

      {results.length > 0 ? (
        <div className="movie-grid">
          {results.map((movie) => (
            <MovieCard key={movie.id} movie={movie} />
          ))}
        </div>
      ) : query.trim() && !isSearching ? (
        <div
          style={{
            textAlign: "center",
            padding: "80px 24px",
            background: "rgba(255, 255, 255, 0.02)",
            border: "1px dashed var(--border-subtle)",
            borderRadius: "var(--radius-lg)",
          }}
        >
          <Film size={48} color="var(--text-muted)" style={{ marginBottom: 16 }} />
          <h3 style={{ fontSize: "1.4rem", marginBottom: 8 }}>No matches found</h3>
          <p style={{ color: "var(--text-secondary)" }}>
            We couldn't find any movie matching "{query}" in your local library.
          </p>
        </div>
      ) : (
        <div style={{ textAlign: "center", padding: "80px 24px", color: "var(--text-muted)" }}>
          Type a movie title, actor, director, or genre to instantly search your collection.
        </div>
      )}
    </div>
  );
};
