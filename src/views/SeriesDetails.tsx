import React, { useState, useEffect } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { SeriesDetails as ISeriesDetails, EpisodeWithMedia } from "../types";
import * as ipc from "../ipc";
import { usePlayback } from "../state/PlaybackContext";
import { X, Play, Star, Tv, Film } from "lucide-react";

interface SeriesDetailsProps {
  seriesId: string;
  onClose: () => void;
}

export const SeriesDetails: React.FC<SeriesDetailsProps> = ({ seriesId, onClose }) => {
  const [data, setData] = useState<ISeriesDetails | null>(null);
  const [selectedSeasonNumber, setSelectedSeasonNumber] = useState<number>(1);
  const { startEpisode } = usePlayback();

  useEffect(() => {
    ipc.getSeriesDetails(seriesId).then((res) => {
      if (res) {
        setData(res);
        if (res.seasons.length > 0) {
          setSelectedSeasonNumber(res.seasons[0].season.season_number);
        }
      }
    });
  }, [seriesId]);

  if (!data) return null;

  const { series, seasons } = data;
  const currentSeason = seasons.find((s) => s.season.season_number === selectedSeasonNumber) || seasons[0];

  const backdropUrl = series.backdrop_path
    ? series.backdrop_path.startsWith("http")
      ? series.backdrop_path
      : convertFileSrc(series.backdrop_path)
    : undefined;

  const posterUrl = series.poster_path
    ? series.poster_path.startsWith("http")
      ? series.poster_path
      : convertFileSrc(series.poster_path)
    : undefined;

  const formatDuration = (secs?: number | null) => {
    if (!secs) return "45m";
    const mins = Math.floor(secs / 60);
    return `${mins}m`;
  };

  const handlePlayEpisode = (epWithMedia: EpisodeWithMedia) => {
    if (epWithMedia.media_id) {
      startEpisode(epWithMedia.episode.id, epWithMedia.media_id);
    }
  };

  return (
    <div className="details-backdrop" onClick={onClose}>
      <div className="details-container" onClick={(e) => e.stopPropagation()}>
        {/* Close Button */}
        <button className="details-close" onClick={onClose} title="Close">
          <X size={20} />
        </button>

        {/* Hero Area */}
        <div className="details-hero">
          {backdropUrl ? (
            <img src={backdropUrl} alt={series.title} className="details-hero-img" />
          ) : (
            <div
              className="details-hero-img"
              style={{ background: "radial-gradient(circle at 60% 40%, #1e2238 0%, #0a0b10 100%)" }}
            />
          )}
          <div className="details-hero-gradient" />

          {/* Hero Content */}
          <div className="details-hero-content">
            <div style={{ display: "flex", gap: 32, alignItems: "flex-end" }}>
              {/* Poster thumbnail */}
              {posterUrl ? (
                <img
                  src={posterUrl}
                  alt={series.title}
                  style={{
                    width: 140,
                    height: 210,
                    borderRadius: "var(--radius-md)",
                    objectFit: "cover",
                    boxShadow: "0 12px 32px rgba(0, 0, 0, 0.8)",
                    border: "1px solid var(--border-subtle)",
                    flexShrink: 0,
                  }}
                />
              ) : (
                <div
                  style={{
                    width: 140,
                    height: 210,
                    borderRadius: "var(--radius-md)",
                    background: "rgba(255, 255, 255, 0.05)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    flexShrink: 0,
                  }}
                >
                  <Tv size={36} color="var(--text-muted)" />
                </div>
              )}

              <div style={{ flex: 1 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 8, flexWrap: "wrap" }}>
                  {series.year && (
                    <span style={{ fontSize: "0.95rem", fontWeight: 700, color: "var(--text-secondary)" }}>
                      {series.year}
                    </span>
                  )}
                  {series.rating && (
                    <div className="movie-card-badge" style={{ position: "static", transform: "none" }}>
                      <Star size={12} fill="currentColor" />
                      <span>{series.rating.toFixed(1)}</span>
                    </div>
                  )}
                  <span style={{ fontSize: "0.85rem", color: "var(--text-muted)" }}>
                    {seasons.length} Season{seasons.length !== 1 ? "s" : ""} • {data.total_episodes} Episode{data.total_episodes !== 1 ? "s" : ""}
                  </span>
                </div>

                <h1 style={{ fontSize: "2.4rem", fontWeight: 800, marginBottom: 12 }}>{series.title}</h1>

                {/* Genre Tags */}
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 14 }}>
                  {series.genres.map((g) => (
                    <span
                      key={g}
                      style={{
                        padding: "3px 10px",
                        background: "rgba(255, 255, 255, 0.08)",
                        borderRadius: "var(--radius-full)",
                        fontSize: "0.78rem",
                        fontWeight: 600,
                      }}
                    >
                      {g}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Details Body */}
        <div className="details-body">
          {/* Overview */}
          {series.description && (
            <p style={{ fontSize: "0.98rem", color: "var(--text-secondary)", lineHeight: 1.6, marginBottom: 28, maxWidth: 840 }}>
              {series.description}
            </p>
          )}

          {/* Season Selector Tabs */}
          <div style={{ display: "flex", gap: 10, borderBottom: "1px solid var(--border-subtle)", paddingBottom: 16, marginBottom: 24, overflowX: "auto" }}>
            {seasons.map((s) => (
              <button
                key={s.season.id}
                className={`filter-pill ${selectedSeasonNumber === s.season.season_number ? "active" : ""}`}
                style={{ fontSize: "0.9rem", padding: "8px 18px", borderRadius: "var(--radius-md)" }}
                onClick={() => setSelectedSeasonNumber(s.season.season_number)}
              >
                {s.season.name || `Season ${s.season.season_number}`} ({s.episodes.length})
              </button>
            ))}
          </div>

          {/* Episode List */}
          <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
            {currentSeason?.episodes.map((epItem) => {
              const ep = epItem.episode;
              const hasMedia = !!epItem.media_id;
              const stillUrl = ep.still_path
                ? ep.still_path.startsWith("http")
                  ? ep.still_path
                  : convertFileSrc(ep.still_path)
                : undefined;

              const progressRatio = epItem.duration_seconds > 0
                ? epItem.progress_seconds / epItem.duration_seconds
                : 0;

              return (
                <div
                  key={ep.id}
                  style={{
                    display: "flex",
                    gap: 20,
                    padding: 16,
                    background: "rgba(255, 255, 255, 0.03)",
                    border: "1px solid var(--border-subtle)",
                    borderRadius: "var(--radius-md)",
                    alignItems: "center",
                    transition: "all 0.2s ease",
                    opacity: hasMedia ? 1 : 0.6,
                  }}
                >
                  {/* Episode Still */}
                  <div
                    style={{
                      width: 160,
                      height: 90,
                      borderRadius: "var(--radius-sm)",
                      background: "#181a24",
                      position: "relative",
                      overflow: "hidden",
                      flexShrink: 0,
                      cursor: hasMedia ? "pointer" : "default",
                    }}
                    onClick={() => hasMedia && handlePlayEpisode(epItem)}
                  >
                    {stillUrl ? (
                      <img src={stillUrl} alt={ep.title} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
                    ) : (
                      <div style={{ width: "100%", height: "100%", display: "flex", alignItems: "center", justifyContent: "center" }}>
                        <Film size={28} color="var(--text-muted)" />
                      </div>
                    )}

                    {hasMedia && (
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
                        <Play size={28} fill="#fff" />
                      </div>
                    )}

                    {/* Progress bar */}
                    {progressRatio > 0 && (
                      <div className="movie-card-progress" style={{ bottom: 0 }}>
                        <div className="movie-card-progress-fill" style={{ width: `${progressRatio * 100}%` }} />
                      </div>
                    )}
                  </div>

                  {/* Episode Info */}
                  <div style={{ flex: 1 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 4 }}>
                      <span style={{ fontSize: "0.82rem", fontWeight: 700, color: "var(--accent-primary)" }}>
                        EPISODE {ep.episode_number}
                      </span>
                      {ep.duration_seconds && (
                        <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>
                          • {formatDuration(ep.duration_seconds)}
                        </span>
                      )}
                      {ep.air_date && (
                        <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>
                          • {ep.air_date}
                        </span>
                      )}
                    </div>
                    <h4 style={{ fontSize: "1.05rem", fontWeight: 700, marginBottom: 6 }}>{ep.title}</h4>
                    {ep.overview && (
                      <p style={{ fontSize: "0.88rem", color: "var(--text-secondary)", lineHeight: 1.5, margin: 0 }}>
                        {ep.overview}
                      </p>
                    )}
                  </div>

                  {/* Play Action */}
                  <div>
                    {hasMedia ? (
                      <button
                        className="btn btn-primary"
                        style={{ padding: "8px 16px", fontSize: "0.88rem" }}
                        onClick={() => handlePlayEpisode(epItem)}
                      >
                        <Play size={14} fill="currentColor" /> {epItem.progress_seconds > 10 ? "Resume" : "Play"}
                      </button>
                    ) : (
                      <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>File Missing</span>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
