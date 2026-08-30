import React from "react";
import { Film, Home, Clapperboard, Tv, Bookmark, Settings, Search, FolderPlus } from "lucide-react";
import { useLibrary } from "../state/LibraryContext";
import { ScanProgressBadge } from "./ScanProgressBadge";

export type NavTab = "home" | "movies" | "series" | "watchlist" | "search" | "settings";

interface NavbarProps {
  currentTab: NavTab;
  onSelectTab: (tab: NavTab) => void;
  searchQuery: string;
  onSearchChange: (q: string) => void;
}

export const Navbar: React.FC<NavbarProps> = ({
  currentTab,
  onSelectTab,
  searchQuery,
  onSearchChange,
}) => {
  const { addSourceByDialog } = useLibrary();

  return (
    <header className="glass-nav">
      {/* Brand / Logo */}
      <div
        style={{ display: "flex", alignItems: "center", gap: 10, cursor: "pointer" }}
        onClick={() => onSelectTab("home")}
      >
        <div
          style={{
            width: 38,
            height: 38,
            borderRadius: "var(--radius-md)",
            background: "var(--accent-gradient)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            boxShadow: "var(--shadow-glow)",
          }}
        >
          <Film size={22} color="#ffffff" />
        </div>
        <div style={{ display: "flex", flexDirection: "column" }}>
          <span style={{ fontSize: "1.25rem", fontWeight: 800, letterSpacing: "-0.03em" }}>
            Movie<span style={{ color: "var(--accent-primary)" }}>Stream</span>
          </span>
          <span style={{ fontSize: "0.65rem", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase" }}>
            Personal Cinema
          </span>
        </div>
      </div>

      {/* Nav Links */}
      <nav style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <button
          className={`btn ${currentTab === "home" ? "btn-primary" : "btn-secondary"}`}
          style={{ padding: "8px 16px", fontSize: "0.88rem" }}
          onClick={() => onSelectTab("home")}
        >
          <Home size={16} /> Home
        </button>
        <button
          className={`btn ${currentTab === "movies" ? "btn-primary" : "btn-secondary"}`}
          style={{ padding: "8px 16px", fontSize: "0.88rem" }}
          onClick={() => onSelectTab("movies")}
        >
          <Clapperboard size={16} /> Movies
        </button>
        <button
          className={`btn ${currentTab === "series" ? "btn-primary" : "btn-secondary"}`}
          style={{ padding: "8px 16px", fontSize: "0.88rem" }}
          onClick={() => onSelectTab("series")}
        >
          <Tv size={16} /> TV Series
        </button>
        <button
          className={`btn ${currentTab === "watchlist" ? "btn-primary" : "btn-secondary"}`}
          style={{ padding: "8px 16px", fontSize: "0.88rem" }}
          onClick={() => onSelectTab("watchlist")}
        >
          <Bookmark size={16} /> Watchlist
        </button>
        <button
          className={`btn ${currentTab === "settings" ? "btn-primary" : "btn-secondary"}`}
          style={{ padding: "8px 16px", fontSize: "0.88rem" }}
          onClick={() => onSelectTab("settings")}
        >
          <Settings size={16} /> Settings
        </button>
      </nav>

      {/* Right Controls: Scan Badge, Search, Add Source */}
      <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
        <ScanProgressBadge />

        <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
          <Search
            size={16}
            color="var(--text-muted)"
            style={{ position: "absolute", left: 14, pointerEvents: "none" }}
          />
          <input
            type="text"
            className="input-search"
            placeholder="Search movies, TV series, cast…"
            value={searchQuery}
            onChange={(e) => {
              onSearchChange(e.target.value);
              if (currentTab !== "search" && e.target.value.trim().length > 0) {
                onSelectTab("search");
              }
            }}
            onFocus={() => {
              if (searchQuery.trim().length > 0) {
                onSelectTab("search");
              }
            }}
          />
        </div>

        <button
          className="btn btn-secondary"
          style={{ padding: "8px 14px", fontSize: "0.88rem" }}
          onClick={addSourceByDialog}
          title="Add folder to library"
        >
          <FolderPlus size={16} /> Add Folder
        </button>
      </div>
    </header>
  );
};
