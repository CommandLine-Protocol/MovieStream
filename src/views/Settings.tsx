import React, { useState, useEffect } from "react";
import { useLibrary } from "../state/LibraryContext";
import { AppSettings } from "../types";
import * as ipc from "../ipc";
import { useToast } from "../state/ToastContext";
import {
  FolderPlus,
  RefreshCw,
  Trash2,
  HardDrive,
} from "lucide-react";

export const Settings: React.FC = () => {
  const { sources, addSourceByDialog, removeSource, rescanSource, rescanAll } = useLibrary();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const { showToast } = useToast();

  useEffect(() => {
    ipc.getSettings().then(setSettings).catch(console.error);
  }, []);

  const handleSaveSettings = async (newSettings: AppSettings) => {
    try {
      const updated = await ipc.updateSettings(newSettings);
      setSettings(updated);
      showToast({ title: "Settings Saved", type: "success" });
    } catch (err) {
      showToast({ title: "Failed to Save Settings", message: String(err), type: "error" });
    }
  };

  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return "Never";
    return new Date(dateStr).toLocaleString();
  };

  return (
    <div style={{ padding: "32px 48px", maxWidth: 1040, margin: "0 auto" }}>
      <h2 style={{ fontSize: "2rem", fontWeight: 800, marginBottom: 28 }}>Settings</h2>

      {/* Library Sources Management */}
      <section className="glass-panel" style={{ padding: 28, marginBottom: 28 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <div>
            <h3 style={{ fontSize: "1.25rem", fontWeight: 700 }}>Library Sources</h3>
            <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: 4 }}>
              MovieStream indexes movies from the local folders listed below.
            </p>
          </div>
          <div style={{ display: "flex", gap: 10 }}>
            <button className="btn btn-secondary" onClick={() => rescanAll()} title="Rescan All Sources">
              <RefreshCw size={16} /> Rescan All
            </button>
            <button className="btn btn-primary" onClick={addSourceByDialog}>
              <FolderPlus size={16} /> Add Folder
            </button>
          </div>
        </div>

        {sources.length === 0 ? (
          <div
            style={{
              padding: "32px",
              textAlign: "center",
              background: "rgba(255, 255, 255, 0.02)",
              border: "1px dashed var(--border-subtle)",
              borderRadius: "var(--radius-md)",
              color: "var(--text-muted)",
            }}
          >
            No folders configured. Click "Add Folder" to add your local movie directory.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {sources.map((src) => (
              <div
                key={src.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  padding: "16px",
                  background: "rgba(255, 255, 255, 0.03)",
                  border: "1px solid var(--border-subtle)",
                  borderRadius: "var(--radius-md)",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                  <div
                    style={{
                      width: 40,
                      height: 40,
                      borderRadius: "var(--radius-md)",
                      background: "rgba(255, 255, 255, 0.06)",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                    }}
                  >
                    <HardDrive size={20} color="var(--text-secondary)" />
                  </div>
                  <div>
                    <div style={{ fontWeight: 600, fontSize: "1rem" }}>{src.name}</div>
                    <div style={{ fontSize: "0.82rem", color: "var(--text-muted)", marginTop: 2 }}>
                      {src.path}
                    </div>
                    <div style={{ fontSize: "0.75rem", color: "var(--text-muted)", marginTop: 4 }}>
                      Last scanned: {formatDate(src.last_scanned_at)}
                    </div>
                  </div>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  <span
                    className="badge"
                    style={{
                      background:
                        src.status === "available"
                          ? "rgba(16, 185, 129, 0.15)"
                          : "rgba(239, 68, 68, 0.15)",
                      color: src.status === "available" ? "#10b981" : "#ef4444",
                    }}
                  >
                    {src.status}
                  </span>
                  <button
                    className="btn-icon"
                    onClick={() => rescanSource(src.id)}
                    title="Rescan this folder"
                  >
                    <RefreshCw size={15} />
                  </button>
                  <button
                    className="btn-icon"
                    onClick={() => removeSource(src.id)}
                    title="Remove from library"
                  >
                    <Trash2 size={15} color="#ef4444" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Playback Preferences */}
      {settings && (
        <section className="glass-panel" style={{ padding: 28, marginBottom: 28 }}>
          <h3 style={{ fontSize: "1.25rem", fontWeight: 700, marginBottom: 16 }}>
            Playback Preferences
          </h3>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20 }}>
            <div>
              <label style={{ display: "block", fontSize: "0.88rem", fontWeight: 600, marginBottom: 8 }}>
                Default Volume ({settings.playback.default_volume}%)
              </label>
              <input
                type="range"
                min={0}
                max={100}
                value={settings.playback.default_volume}
                onChange={(e) => {
                  const val = Number(e.target.value);
                  setSettings({
                    ...settings,
                    playback: { ...settings.playback, default_volume: val },
                  });
                }}
                onMouseUp={() => handleSaveSettings(settings)}
                style={{ width: "100%", accentColor: "var(--accent-primary)" }}
              />
            </div>

            <div>
              <label style={{ display: "block", fontSize: "0.88rem", fontWeight: 600, marginBottom: 8 }}>
                Resume Behavior
              </label>
              <select
                value={settings.playback.resume_behavior}
                onChange={(e) => {
                  const val = e.target.value as "prompt" | "always" | "never";
                  const updated = {
                    ...settings,
                    playback: { ...settings.playback, resume_behavior: val },
                  };
                  handleSaveSettings(updated);
                }}
                style={{
                  width: "100%",
                  padding: "8px 12px",
                  background: "rgba(255, 255, 255, 0.08)",
                  border: "1px solid var(--border-subtle)",
                  borderRadius: "var(--radius-md)",
                  color: "#fff",
                }}
              >
                <option value="prompt">Ask Before Resuming</option>
                <option value="always">Always Resume Automatically</option>
                <option value="never">Always Start from Beginning</option>
              </select>
            </div>
          </div>
        </section>
      )}

      {/* Metadata Preferences */}
      {settings && (
        <section className="glass-panel" style={{ padding: 28, marginBottom: 28 }}>
          <h3 style={{ fontSize: "1.25rem", fontWeight: 700, marginBottom: 8 }}>
            Metadata & Cover Art Provider
          </h3>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.88rem", marginBottom: 16 }}>
            MovieStream automatically discovers high-resolution cover art, backdrops, and synopses using built-in zero-config public metadata sources and local directory scrapers.
          </p>
          <div>
            <label style={{ display: "block", fontSize: "0.85rem", fontWeight: 600, marginBottom: 6 }}>
              TheMovieDB (TMDb) API Key (Optional)
            </label>
            <input
              type="password"
              placeholder="Leave blank for automatic zero-config metadata"
              value={settings.metadata.active_provider_id === "tmdb" ? "" : settings.metadata.active_provider_id}
              onChange={(e) => {
                const val = e.target.value;
                setSettings({
                  ...settings,
                  metadata: { ...settings.metadata, active_provider_id: val },
                });
              }}
              onBlur={() => handleSaveSettings(settings)}
              style={{
                width: "100%",
                maxWidth: 480,
                padding: "8px 12px",
                background: "rgba(255, 255, 255, 0.08)",
                border: "1px solid var(--border-subtle)",
                borderRadius: "var(--radius-md)",
                color: "#fff",
                fontSize: "0.9rem",
              }}
            />
            <span style={{ display: "block", fontSize: "0.78rem", color: "var(--text-muted)", marginTop: 6 }}>
              You can also specify <code>TMDB_API_KEY</code> in a <code>.env</code> file.
            </span>
          </div>
        </section>
      )}

      {/* About Box with Developer & Sponsor Attribution */}
      <section className="glass-panel" style={{ padding: 28, textAlign: "center" }}>
        <div style={{ fontSize: "1.25rem", fontWeight: 800, marginBottom: 6 }}>
          Movie<span style={{ color: "var(--accent-primary)" }}>Stream</span> V1
        </div>
        <div style={{ fontSize: "0.88rem", color: "var(--text-secondary)", marginBottom: 18 }}>
          Your Personal Cinematic Streaming Desktop Client
        </div>

        <div style={{ display: "flex", justifyContent: "center", alignItems: "center", gap: 24, flexWrap: "wrap" }}>
          {/* Developer Attribution */}
          <a
            href="https://github.com/CommandLine-Protocol"
            target="_blank"
            rel="noreferrer"
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 8,
              padding: "8px 16px",
              background: "rgba(255, 255, 255, 0.08)",
              border: "1px solid var(--border-subtle)",
              borderRadius: "var(--radius-full)",
              color: "#fff",
              textDecoration: "none",
              fontSize: "0.88rem",
              fontWeight: 600,
              transition: "all 0.2s ease",
            }}
          >
            {/* GitHub SVG Icon */}
            <svg height="18" width="18" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
            </svg>
            <span>Developed by <strong>CommandLine-Protocol</strong></span>
          </a>

          {/* Sponsor Attribution */}
          <a
            href="https://yimatt.com"
            target="_blank"
            rel="noreferrer"
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 8,
              padding: "8px 16px",
              background: "rgba(229, 9, 20, 0.12)",
              border: "1px solid rgba(229, 9, 20, 0.35)",
              borderRadius: "var(--radius-full)",
              color: "#fff",
              textDecoration: "none",
              fontSize: "0.88rem",
              fontWeight: 600,
              transition: "all 0.2s ease",
            }}
          >
            <span>Sponsored by <strong style={{ color: "var(--accent-primary)" }}>Yimatt Technologies</strong></span>
          </a>
        </div>

        <div style={{ fontSize: "0.78rem", color: "var(--text-muted)", marginTop: 16 }}>
          Powered by Tauri 2 • React • Rust • SQLite • VLC Media Engine
        </div>
      </section>
    </div>
  );
};
