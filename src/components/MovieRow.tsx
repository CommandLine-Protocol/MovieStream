import React, { useRef } from "react";
import { Movie } from "../types";
import { MovieCard } from "./MovieCard";
import { ChevronLeft, ChevronRight } from "lucide-react";

interface MovieRowProps {
  title: string;
  movies: Movie[];
  getProgressRatio?: (movie: Movie) => number | undefined;
  emptyMessage?: string;
}

export const MovieRow: React.FC<MovieRowProps> = ({
  title,
  movies,
  getProgressRatio,
  emptyMessage,
}) => {
  const trackRef = useRef<HTMLDivElement>(null);

  const scroll = (direction: "left" | "right") => {
    if (trackRef.current) {
      const scrollAmount = direction === "left" ? -400 : 400;
      trackRef.current.scrollBy({ left: scrollAmount, behavior: "smooth" });
    }
  };

  if (movies.length === 0) {
    if (!emptyMessage) return null;
    return (
      <section className="movie-row">
        <div className="movie-row-header">
          <h3 className="movie-row-title">{title}</h3>
        </div>
        <div
          style={{
            padding: "24px",
            background: "rgba(255, 255, 255, 0.03)",
            border: "1px dashed var(--border-subtle)",
            borderRadius: "var(--radius-md)",
            color: "var(--text-muted)",
            fontSize: "0.9rem",
          }}
        >
          {emptyMessage}
        </div>
      </section>
    );
  }

  return (
    <section className="movie-row">
      <div className="movie-row-header">
        <h3 className="movie-row-title">{title}</h3>
        <div style={{ display: "flex", gap: 6 }}>
          <button className="btn-icon" style={{ width: 32, height: 32 }} onClick={() => scroll("left")}>
            <ChevronLeft size={16} />
          </button>
          <button className="btn-icon" style={{ width: 32, height: 32 }} onClick={() => scroll("right")}>
            <ChevronRight size={16} />
          </button>
        </div>
      </div>

      <div className="movie-row-track" ref={trackRef}>
        {movies.map((movie) => (
          <MovieCard
            key={movie.id}
            movie={movie}
            progressRatio={getProgressRatio ? getProgressRatio(movie) : undefined}
          />
        ))}
      </div>
    </section>
  );
};
