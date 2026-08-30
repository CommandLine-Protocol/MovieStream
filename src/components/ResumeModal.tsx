import React from "react";
import { usePlayback } from "../state/PlaybackContext";
import { Play, RotateCcw, Clock } from "lucide-react";

export const ResumeModal: React.FC = () => {
  const {
    showResumePrompt,
    resumePosition,
    resumeCountdown,
    session,
    activeMovie,
    acceptResume,
    restartFromBeginning,
  } = usePlayback();

  if (!showResumePrompt || !session) return null;

  const formatTime = (seconds: number) => {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hrs > 0) {
      return `${hrs}:${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const title = session.title || activeMovie?.title || "Video";
  const subtitle = session.subtitle_info || (activeMovie?.year ? `${activeMovie.year}` : "");

  return (
    <div className="modal-backdrop" style={{ zIndex: 1000 }}>
      <div className="modal-card" style={{ textAlign: "center", maxWidth: 440, padding: 32 }}>
        <div
          style={{
            width: 60,
            height: 60,
            borderRadius: "var(--radius-full)",
            background: "rgba(229, 9, 20, 0.15)",
            border: "1px solid rgba(229, 9, 20, 0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            margin: "0 auto 16px auto",
            boxShadow: "0 0 24px rgba(229, 9, 20, 0.3)",
          }}
        >
          <Clock size={28} color="var(--accent-primary)" />
        </div>

        <h3 style={{ fontSize: "1.45rem", fontWeight: 800, marginBottom: 6 }}>Resume Playback?</h3>
        <p style={{ color: "var(--text-secondary)", fontSize: "0.92rem", marginBottom: 20 }}>
          You previously watched <strong style={{ color: "#fff" }}>{title}</strong>
          {subtitle ? ` (${subtitle})` : ""} up to{" "}
          <strong style={{ color: "var(--accent-primary)" }}>{formatTime(resumePosition)}</strong>.
        </p>

        {/* 15-Second Countdown Timer Badge */}
        <div
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 8,
            padding: "6px 14px",
            background: "rgba(255, 255, 255, 0.06)",
            border: "1px solid var(--border-subtle)",
            borderRadius: "var(--radius-full)",
            fontSize: "0.82rem",
            color: "var(--text-secondary)",
            marginBottom: 24,
          }}
        >
          <span>Auto-resuming in</span>
          <span style={{ fontWeight: 800, color: "var(--accent-primary)", fontSize: "0.95rem" }}>
            {resumeCountdown}s
          </span>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <button className="btn btn-primary" onClick={acceptResume} style={{ width: "100%", padding: "12px", fontSize: "1rem" }}>
            <Play size={18} fill="currentColor" /> Resume from {formatTime(resumePosition)}
          </button>
          <button className="btn btn-secondary" onClick={restartFromBeginning} style={{ width: "100%", padding: "12px", fontSize: "0.9rem" }}>
            <RotateCcw size={16} /> Start from Beginning
          </button>
        </div>
      </div>
    </div>
  );
};
