import React from "react";
import { MovieFilter, MovieSort } from "../types";
import { ArrowUpDown } from "lucide-react";

interface FilterBarProps {
  filter: MovieFilter;
  sort: MovieSort;
  onFilterChange: (f: MovieFilter) => void;
  onSortChange: (s: MovieSort) => void;
}

const GENRES = [
  "All",
  "Action",
  "Adventure",
  "Animation",
  "Comedy",
  "Crime",
  "Documentary",
  "Drama",
  "Horror",
  "Mystery",
  "Sci-Fi",
  "Thriller",
];

export const FilterBar: React.FC<FilterBarProps> = ({
  filter,
  sort,
  onFilterChange,
  onSortChange,
}) => {
  const activeGenre = filter.genre || "All";

  const handleGenreClick = (genre: string) => {
    onFilterChange({
      ...filter,
      genre: genre === "All" ? undefined : genre,
    });
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 16,
        marginBottom: 28,
      }}
    >
      {/* Genre Pills */}
      <div
        style={{
          display: "flex",
          gap: 8,
          overflowX: "auto",
          paddingBottom: 4,
          scrollbarWidth: "none",
        }}
      >
        {GENRES.map((g) => (
          <button
            key={g}
            className={`filter-pill ${activeGenre === g ? "active" : ""}`}
            onClick={() => handleGenreClick(g)}
          >
            {g}
          </button>
        ))}
      </div>

      {/* Secondary Controls: Sort & Filter Options */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          flexWrap: "wrap",
          gap: 12,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          {/* Watched State */}
          <select
            value={
              filter.watched === undefined
                ? "all"
                : filter.watched
                ? "watched"
                : "unwatched"
            }
            onChange={(e) => {
              const val = e.target.value;
              onFilterChange({
                ...filter,
                watched: val === "all" ? undefined : val === "watched",
              });
            }}
            style={{
              background: "rgba(255, 255, 255, 0.08)",
              border: "1px solid var(--border-subtle)",
              color: "var(--text-primary)",
              padding: "6px 12px",
              borderRadius: "var(--radius-md)",
              outline: "none",
              fontSize: "0.85rem",
              cursor: "pointer",
            }}
          >
            <option value="all">All Watch States</option>
            <option value="unwatched">Unwatched Only</option>
            <option value="watched">Watched</option>
          </select>

          {/* Availability */}
          <select
            value={filter.is_available === undefined ? "all" : filter.is_available ? "available" : "unavailable"}
            onChange={(e) => {
              const val = e.target.value;
              onFilterChange({
                ...filter,
                is_available: val === "all" ? undefined : val === "available",
              });
            }}
            style={{
              background: "rgba(255, 255, 255, 0.08)",
              border: "1px solid var(--border-subtle)",
              color: "var(--text-primary)",
              padding: "6px 12px",
              borderRadius: "var(--radius-md)",
              outline: "none",
              fontSize: "0.85rem",
              cursor: "pointer",
            }}
          >
            <option value="all">All Media</option>
            <option value="available">Available Locally</option>
          </select>
        </div>

        {/* Sort dropdown */}
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <ArrowUpDown size={15} color="var(--text-muted)" />
          <span style={{ fontSize: "0.85rem", color: "var(--text-muted)" }}>Sort by:</span>
          <select
            value={sort}
            onChange={(e) => onSortChange(e.target.value as MovieSort)}
            style={{
              background: "rgba(255, 255, 255, 0.08)",
              border: "1px solid var(--border-subtle)",
              color: "var(--text-primary)",
              padding: "6px 12px",
              borderRadius: "var(--radius-md)",
              outline: "none",
              fontSize: "0.85rem",
              cursor: "pointer",
            }}
          >
            <option value="title_asc">Title (A - Z)</option>
            <option value="title_desc">Title (Z - A)</option>
            <option value="year_desc">Release Year (Newest)</option>
            <option value="year_asc">Release Year (Oldest)</option>
            <option value="date_added_desc">Date Added (Recent)</option>
            <option value="rating_desc">Rating (Highest)</option>
          </select>
        </div>
      </div>
    </div>
  );
};
