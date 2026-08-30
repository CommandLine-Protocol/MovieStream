import React, { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Movie } from "../types";
import * as ipc from "../ipc";
import { useLibrary } from "../state/LibraryContext";
import { useToast } from "../state/ToastContext";
import { Search, Check, X, Film, Loader2 } from "lucide-react";

interface MetadataMatchModalProps {
  movie: Movie;
  onClose: () => void;
  onMatched: (updatedMovie: Movie) => void;
}

export const MetadataMatchModal: React.FC<MetadataMatchModalProps> = ({
  movie,
  onClose,
  onMatched,
}) => {
  const [query, setQuery] = useState(movie.title);
  const [year, setYear] = useState<string>(movie.year ? movie.year.toString() : "");
  const [isSearching, setIsSearching] = useState(false);
  const [candidates, setCandidates] = useState<any[]>([]);
  const [isApplying, setIsApplying] = useState(false);

  const { refreshLibrary } = useLibrary();
  const { showToast } = useToast();

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;

    setIsSearching(true);
    try {
      // In V1, we search local or online candidates
      const results = await ipc.searchMovies(query);
      setCandidates(results);
    } catch (err) {
      showToast({ title: "Search Failed", message: String(err), type: "error" });
    } finally {
      setIsSearching(false);
    }
  };

  const handleSelectMatch = async (candidateId: string) => {
    setIsApplying(true);
    try {
      const updated = await ipc.setMetadataMatch(movie.id, candidateId);
      showToast({ title: "Metadata Updated", message: `Updated to ${updated.title}`, type: "success" });
      onMatched(updated);
      await refreshLibrary();
      onClose();
    } catch (err) {
      showToast({ title: "Failed to Apply Match", message: String(err), type: "error" });
    } finally {
      setIsApplying(false);
    }
  };

  return (
    <div className="modal-backdrop">
      <div className="modal-card" style={{ maxWidth: 580 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h3 style={{ fontSize: "1.3rem" }}>Fix Metadata Match</h3>
          <button className="btn-icon" onClick={onClose} style={{ width: 32, height: 32 }}>
            <X size={16} />
          </button>
        </div>

        <form onSubmit={handleSearch} style={{ display: "flex", gap: 10, marginBottom: 20 }}>
          <input
            type="text"
            className="input-search"
            style={{ flex: 1, width: "auto", borderRadius: "var(--radius-md)" }}
            placeholder="Movie Title"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <input
            type="text"
            className="input-search"
            style={{ width: 100, borderRadius: "var(--radius-md)" }}
            placeholder="Year"
            value={year}
            onChange={(e) => setYear(e.target.value)}
          />
          <button type="submit" className="btn btn-primary" disabled={isSearching}>
            {isSearching ? <Loader2 size={16} className="animate-spin" /> : <Search size={16} />} Search
          </button>
        </form>

        <div style={{ maxHeight: 360, overflowY: "auto", display: "flex", flexDirection: "column", gap: 10 }}>
          {candidates.length === 0 && !isSearching && (
            <div style={{ textAlign: "center", padding: "32px 0", color: "var(--text-muted)", fontSize: "0.9rem" }}>
              Search for the correct movie title to update posters and metadata.
            </div>
          )}

          {candidates.map((c) => (
            <div
              key={c.id}
              style={{
                display: "flex",
                gap: 14,
                padding: 12,
                background: "rgba(255, 255, 255, 0.04)",
                border: "1px solid var(--border-subtle)",
                borderRadius: "var(--radius-md)",
                alignItems: "center",
                cursor: "pointer",
                transition: "all var(--transition-fast)",
              }}
              onClick={() => handleSelectMatch(c.id)}
            >
              <div
                style={{
                  width: 44,
                  height: 64,
                  background: "#222",
                  borderRadius: "var(--radius-sm)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  overflow: "hidden",
                }}
              >
                {c.poster_path ? (
                  <img
                    src={c.poster_path.startsWith("http") ? c.poster_path : convertFileSrc(c.poster_path)}
                    alt=""
                    style={{ width: "100%", height: "100%", objectFit: "cover" }}
                  />
                ) : (
                  <Film size={20} color="var(--text-muted)" />
                )}
              </div>
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: 600, fontSize: "0.95rem" }}>{c.title}</div>
                <div style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>{c.year || "Unknown Year"}</div>
              </div>
              <button className="btn btn-secondary" style={{ padding: "6px 12px", fontSize: "0.8rem" }} disabled={isApplying}>
                <Check size={14} /> Select
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
