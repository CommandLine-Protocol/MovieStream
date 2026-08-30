import React from "react";
import { useLibrary } from "../state/LibraryContext";
import { Loader2 } from "lucide-react";

export const ScanProgressBadge: React.FC = () => {
  const { scanProgress } = useLibrary();

  if (!scanProgress) return null;

  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 8,
        padding: "6px 14px",
        background: "rgba(229, 9, 20, 0.15)",
        border: "1px solid rgba(229, 9, 20, 0.4)",
        borderRadius: "var(--radius-full)",
        color: "#fff",
        fontSize: "0.82rem",
        fontWeight: 600,
        boxShadow: "0 0 16px rgba(229, 9, 20, 0.3)",
      }}
    >
      <Loader2 size={14} className="animate-spin" style={{ animation: "spin 1s linear infinite" }} />
      <span>
        {scanProgress.phase === "scanning" && "Scanning folders…"}
        {scanProgress.phase === "analyzing" && `Analyzing ${scanProgress.files_discovered} files…`}
        {scanProgress.phase === "matching" && `Identified ${scanProgress.movies_identified} movies…`}
        {scanProgress.phase === "completed" && "Library up to date"}
      </span>
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
};
