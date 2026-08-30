import React from "react";
import { useLibrary } from "../state/LibraryContext";
import { FilterBar } from "../components/FilterBar";
import { MovieCard } from "../components/MovieCard";
import { Clapperboard, FolderPlus } from "lucide-react";

export const AllMovies: React.FC = () => {
  const { movies, filter, sort, setFilter, setSort, addSourceByDialog } = useLibrary();

  return (
    <div style={{ padding: "32px 48px", maxWidth: 1600, margin: "0 auto" }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: 24,
        }}
      >
        <div style={{ display: "flex", alignItems: "baseline", gap: 12 }}>
          <h2 style={{ fontSize: "2rem", fontWeight: 800 }}>All Movies</h2>
          <span
            style={{
              fontSize: "0.9rem",
              color: "var(--text-muted)",
              fontWeight: 600,
            }}
          >
            {movies.length} {movies.length === 1 ? "title" : "titles"}
          </span>
        </div>
      </div>

      <FilterBar
        filter={filter}
        sort={sort}
        onFilterChange={setFilter}
        onSortChange={setSort}
      />

      {movies.length === 0 ? (
        <div
          style={{
            textAlign: "center",
            padding: "80px 24px",
            background: "rgba(255, 255, 255, 0.02)",
            border: "1px dashed var(--border-subtle)",
            borderRadius: "var(--radius-lg)",
          }}
        >
          <Clapperboard size={48} color="var(--text-muted)" style={{ marginBottom: 16 }} />
          <h3 style={{ fontSize: "1.4rem", marginBottom: 8 }}>No movies match your filters</h3>
          <p style={{ color: "var(--text-secondary)", marginBottom: 20 }}>
            Try resetting your filters or add more folders to your movie collection.
          </p>
          <button className="btn btn-secondary" onClick={addSourceByDialog}>
            <FolderPlus size={16} /> Add Folder
          </button>
        </div>
      ) : (
        <div className="movie-grid">
          {movies.map((movie) => (
            <MovieCard key={movie.id} movie={movie} />
          ))}
        </div>
      )}
    </div>
  );
};
