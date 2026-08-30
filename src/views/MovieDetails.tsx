import React, { useState, useEffect } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { MovieWithMedia } from "../types";
import * as ipc from "../ipc";
import { usePlayback } from "../state/PlaybackContext";
import { useLibrary } from "../state/LibraryContext";
import { MetadataMatchModal } from "../components/MetadataMatchModal";
import {
  Play,
  Bookmark,
  Star,
  Clock,
  Film,
  HardDrive,
  Sliders,
  X,
} from "lucide-react";

interface MovieDetailsProps {
  movieId: string;
  onClose: () => void;
}

export const MovieDetails: React.FC<MovieDetailsProps> = ({ movieId, onClose }) => {
  const [data, setData] = useState<MovieWithMedia | null>(null);
  const [selectedMediaId, setSelectedMediaId] = useState<string>("");
  const [showMatchModal, setShowMatchModal] = useState(false);

  const { startMovie } = usePlayback();
  const { watchlist, toggleWatchlist } = useLibrary();

  useEffect(() => {
    ipc.getMovie(movieId).then((res) => {
      if (res) {
        setData(res);
        if (res.media.length > 0) {
          setSelectedMediaId(res.media[0].id);
        }
      }
    });
  }, [movieId]);

  if (!data) return null;

  const { movie, media } = data;
  const isSaved = watchlist.some((m) => m.id === movie.id);
  const backdropUrl = movie.backdrop_path
    ? movie.backdrop_path.startsWith("http")
      ? movie.backdrop_path
      : convertFileSrc(movie.backdrop_path)
    : undefined;
  const posterUrl = movie.poster_path
    ? movie.poster_path.startsWith("http")
      ? movie.poster_path
      : convertFileSrc(movie.poster_path)
    : undefined;

  const selectedMedia = media.find((m) => m.id === selectedMediaId) || media[0];

  const formatFileSize = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) return `${gb.toFixed(1)} GB`;
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(0)} MB`;
  };

  const formatDuration = (seconds?: number | null) => {
    if (!seconds) return "2h";
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return hrs > 0 ? `${hrs}h ${mins}m` : `${mins}m`;
  };

  return (
    <div className="modal-backdrop" style={{ padding: 0 }}>
      <div
        className="glass-panel"
        style={{
          width: "100%",
          maxWidth: 1040,
          maxHeight: "90vh",
          overflowY: "auto",
          position: "relative",
          background: "var(--bg-surface)",
          borderRadius: "var(--radius-lg)",
          boxShadow: "0 32px 64px rgba(0, 0, 0, 0.9)",
          margin: "24px",
        }}
      >
        {/* Close button */}
        <button
          className="btn-icon"
          onClick={onClose}
          style={{
            position: "absolute",
            top: 20,
            right: 20,
            zIndex: 10,
            background: "rgba(0, 0, 0, 0.6)",
            backdropFilter: "blur(8px)",
          }}
        >
          <X size={18} />
        </button>

        {/* Backdrop Banner Header */}
        <div style={{ position: "relative", height: 360, width: "100%", overflow: "hidden" }}>
          {backdropUrl ? (
            <img
              src={backdropUrl}
              alt=""
              style={{ width: "100%", height: "100%", objectFit: "cover", filter: "brightness(0.55)" }}
            />
          ) : (
            <div
              style={{
                width: "100%",
                height: "100%",
                background: "linear-gradient(135deg, #1f233a 0%, #0d0e17 100%)",
              }}
            />
          )}
          <div
            style={{
              position: "absolute",
              inset: 0,
              background: "linear-gradient(to top, var(--bg-surface) 0%, transparent 80%)",
            }}
          />
        </div>

        {/* Main Content Layout */}
        <div
          style={{
            display: "flex",
            gap: 36,
            padding: "0 40px 40px 40px",
            marginTop: -160,
            position: "relative",
            zIndex: 2,
          }}
        >
          {/* Poster */}
          <div style={{ flex: "0 0 240px" }}>
            <div
              style={{
                width: "100%",
                aspectRatio: "2 / 3",
                borderRadius: "var(--radius-md)",
                overflow: "hidden",
                boxShadow: "0 16px 36px rgba(0, 0, 0, 0.8)",
                border: "1px solid var(--border-active)",
                background: "#181926",
              }}
            >
              {posterUrl ? (
                <img src={posterUrl} alt={movie.title} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
              ) : (
                <div className="movie-card-fallback-poster">
                  <Film size={48} color="var(--text-muted)" />
                </div>
              )}
            </div>

            {/* Match Status Badge */}
            <div
              style={{
                marginTop: 16,
                padding: "8px 12px",
                background: "rgba(255, 255, 255, 0.04)",
                border: "1px solid var(--border-subtle)",
                borderRadius: "var(--radius-md)",
                fontSize: "0.8rem",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
              }}
            >
              <span style={{ color: "var(--text-muted)" }}>Metadata:</span>
              <span
                style={{
                  fontWeight: 600,
                  color:
                    movie.metadata_status === "auto_matched" || movie.metadata_status === "manually_matched"
                      ? "#10b981"
                      : "var(--text-muted)",
                }}
              >
                {movie.metadata_status.replace("_", " ")}
              </span>
            </div>
            <button
              className="btn btn-secondary"
              style={{ width: "100%", marginTop: 8, fontSize: "0.82rem", padding: "8px" }}
              onClick={() => setShowMatchModal(true)}
            >
              <Sliders size={14} /> Correct Match
            </button>
          </div>

          {/* Details Body */}
          <div style={{ flex: 1 }}>
            <h1 style={{ fontSize: "2.6rem", fontWeight: 800, lineHeight: 1.1, marginBottom: 8 }}>
              {movie.title}
            </h1>
            {movie.original_title && movie.original_title !== movie.title && (
              <div style={{ fontSize: "1rem", color: "var(--text-muted)", marginBottom: 12 }}>
                Original Title: {movie.original_title}
              </div>
            )}

            {/* Quick Meta Badges */}
            <div style={{ display: "flex", alignItems: "center", gap: 16, marginBottom: 20, flexWrap: "wrap" }}>
              {movie.year && <span style={{ fontWeight: 600 }}>{movie.year}</span>}
              <span style={{ color: "var(--text-muted)" }}>•</span>
              <span style={{ display: "flex", alignItems: "center", gap: 4, color: "var(--text-secondary)" }}>
                <Clock size={15} /> {formatDuration(selectedMedia?.duration_seconds)}
              </span>
              {movie.rating && (
                <>
                  <span style={{ color: "var(--text-muted)" }}>•</span>
                  <span style={{ display: "flex", alignItems: "center", gap: 4, color: "var(--accent-gold)" }}>
                    <Star size={15} fill="currentColor" /> {movie.rating.toFixed(1)} / 10
                  </span>
                </>
              )}
            </div>

            {/* Play & Watchlist Action Buttons */}
            <div style={{ display: "flex", alignItems: "center", gap: 14, marginBottom: 28 }}>
              <button
                className="btn btn-primary"
                style={{ padding: "12px 28px", fontSize: "1.05rem" }}
                onClick={() => {
                  startMovie(movie, selectedMedia?.id);
                  onClose();
                }}
              >
                <Play size={20} fill="currentColor" /> Play Movie
              </button>
              <button
                className={`btn ${isSaved ? "btn-primary" : "btn-secondary"}`}
                onClick={() => toggleWatchlist(movie.id)}
              >
                <Bookmark size={18} fill={isSaved ? "currentColor" : "none"} />
                {isSaved ? "In Watchlist" : "Add to Watchlist"}
              </button>
            </div>

            {/* Genres */}
            {movie.genres.length > 0 && (
              <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 20 }}>
                {movie.genres.map((g) => (
                  <span
                    key={g}
                    style={{
                      padding: "4px 12px",
                      background: "rgba(255, 255, 255, 0.08)",
                      borderRadius: "var(--radius-full)",
                      fontSize: "0.82rem",
                      fontWeight: 500,
                    }}
                  >
                    {g}
                  </span>
                ))}
              </div>
            )}

            {/* Overview */}
            <div style={{ marginBottom: 28 }}>
              <h3 style={{ fontSize: "1.1rem", marginBottom: 8, color: "var(--text-secondary)" }}>
                Synopsis
              </h3>
              <p style={{ lineHeight: 1.65, color: "#cbd5e1", fontSize: "1rem" }}>
                {movie.description || "No overview available."}
              </p>
            </div>

            {/* Cast & Director */}
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20, marginBottom: 28 }}>
              {movie.director && (
                <div>
                  <h4 style={{ fontSize: "0.85rem", color: "var(--text-muted)", textTransform: "uppercase", marginBottom: 4 }}>
                    Director
                  </h4>
                  <div style={{ fontWeight: 600 }}>{movie.director}</div>
                </div>
              )}
              {movie.cast.length > 0 && (
                <div>
                  <h4 style={{ fontSize: "0.85rem", color: "var(--text-muted)", textTransform: "uppercase", marginBottom: 4 }}>
                    Starring
                  </h4>
                  <div style={{ fontWeight: 500, color: "#e2e8f0" }}>{movie.cast.join(", ")}</div>
                </div>
              )}
            </div>

            {/* Media Versions Section (Multi-version support per PRD §7.5) */}
            <div style={{ borderTop: "1px solid var(--border-subtle)", paddingTop: 20 }}>
              <h3 style={{ fontSize: "1.1rem", marginBottom: 12 }}>
                Available Versions & Files ({media.length})
              </h3>
              <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
                {media.map((m) => (
                  <div
                    key={m.id}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "12px 16px",
                      background:
                        selectedMediaId === m.id
                          ? "rgba(229, 9, 20, 0.12)"
                          : "rgba(255, 255, 255, 0.03)",
                      border: `1px solid ${
                        selectedMediaId === m.id ? "var(--accent-primary)" : "var(--border-subtle)"
                      }`,
                      borderRadius: "var(--radius-md)",
                      cursor: "pointer",
                    }}
                    onClick={() => setSelectedMediaId(m.id)}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                      <HardDrive size={18} color="var(--text-muted)" />
                      <div>
                        <div style={{ fontWeight: 600, fontSize: "0.92rem" }}>
                          {m.resolution_height ? `${m.resolution_height}p` : "HD"} • {m.video_codec || "Video"} • {formatFileSize(m.size_bytes)}
                        </div>
                        <div style={{ fontSize: "0.78rem", color: "var(--text-muted)", marginTop: 2 }}>
                          {m.path}
                        </div>
                      </div>
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                      <span className="badge badge-resolution">
                        {m.resolution_width && m.resolution_width >= 3840 ? "4K UHD" : "1080p"}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>

      {showMatchModal && (
        <MetadataMatchModal
          movie={movie}
          onClose={() => setShowMatchModal(false)}
          onMatched={(updated) => setData((prev) => (prev ? { ...prev, movie: updated } : prev))}
        />
      )}
    </div>
  );
};
