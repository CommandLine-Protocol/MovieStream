import React, { createContext, useContext, useState, useEffect, useCallback } from "react";
import {
  ContinueWatchingItem,
  LibrarySource,
  Movie,
  MovieFilter,
  MovieSort,
  ScanProgressPayload,
  TvSeries,
} from "../types";
import * as ipc from "../ipc";
import { useToast } from "./ToastContext";

interface LibraryContextType {
  sources: LibrarySource[];
  movies: Movie[];
  series: TvSeries[];
  continueWatching: ContinueWatchingItem[];
  recentlyWatched: Movie[];
  watchlist: Movie[];
  scanProgress: ScanProgressPayload | null;
  isLoading: boolean;
  filter: MovieFilter;
  sort: MovieSort;
  selectedMovieId: string | null;
  selectedSeriesId: string | null;
  setFilter: React.Dispatch<React.SetStateAction<MovieFilter>>;
  setSort: React.Dispatch<React.SetStateAction<MovieSort>>;
  openMovieDetails: (id: string) => void;
  closeMovieDetails: () => void;
  openSeriesDetails: (id: string) => void;
  closeSeriesDetails: () => void;
  refreshLibrary: () => Promise<void>;
  refreshSources: () => Promise<void>;
  addSourceByDialog: () => Promise<void>;
  removeSource: (id: string) => Promise<void>;
  rescanSource: (id: string) => Promise<void>;
  rescanAll: () => Promise<void>;
  toggleWatchlist: (movieId: string) => Promise<void>;
}

const LibraryContext = createContext<LibraryContextType | undefined>(undefined);

export const LibraryProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [sources, setSources] = useState<LibrarySource[]>([]);
  const [movies, setMovies] = useState<Movie[]>([]);
  const [series, setSeries] = useState<TvSeries[]>([]);
  const [continueWatching, setContinueWatching] = useState<ContinueWatchingItem[]>([]);
  const [recentlyWatched, setRecentlyWatched] = useState<Movie[]>([]);
  const [watchlist, setWatchlist] = useState<Movie[]>([]);
  const [scanProgress, setScanProgress] = useState<ScanProgressPayload | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [filter, setFilter] = useState<MovieFilter>({});
  const [sort, setSort] = useState<MovieSort>("title_asc");
  const [selectedMovieId, setSelectedMovieId] = useState<string | null>(null);
  const [selectedSeriesId, setSelectedSeriesId] = useState<string | null>(null);

  const { showToast } = useToast();

  const refreshSources = useCallback(async () => {
    try {
      const srcList = await ipc.listSources();
      setSources(srcList);
    } catch (err) {
      console.error("Failed to list sources:", err);
    }
  }, []);

  const refreshLibrary = useCallback(async () => {
    setIsLoading(true);
    try {
      const [allMovies, allSeries, cw, rw, wl] = await Promise.all([
        ipc.listMovies(filter, sort),
        ipc.listTvSeries(),
        ipc.getContinueWatching(20),
        ipc.getRecentlyWatched(10),
        ipc.listWatchlist(),
      ]);
      setMovies(allMovies);
      setSeries(allSeries);
      setContinueWatching(cw);
      setRecentlyWatched(rw);
      setWatchlist(wl);
    } catch (err) {
      console.error("Failed to refresh library:", err);
    } finally {
      setIsLoading(false);
    }
  }, [filter, sort]);

  useEffect(() => {
    refreshLibrary();
  }, [filter, sort]);

  useEffect(() => {
    refreshSources();
    refreshLibrary();
  }, [refreshSources, refreshLibrary]);

  // Progressive refresh interval while a scan is in progress
  useEffect(() => {
    if (!scanProgress || scanProgress.phase === "completed" || scanProgress.phase === "error") return;

    const interval = setInterval(() => {
      refreshLibrary();
    }, 1500);

    return () => clearInterval(interval);
  }, [scanProgress, refreshLibrary]);

  // Listen for scan progress events
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    ipc.onScanProgress((payload) => {
      setScanProgress(payload);
      if (payload.phase === "completed") {
        showToast({
          title: "Library Indexing Complete",
          message: `Identified ${payload.movies_identified} items from ${payload.files_discovered} files.`,
          type: "success",
        });
        setTimeout(() => setScanProgress(null), 3000);
        refreshLibrary();
        refreshSources();
      } else if (payload.phase === "indexing" || payload.phase === "analyzing") {
        refreshLibrary();
      } else if (payload.phase === "error") {
        showToast({
          title: "Scan Error",
          message: "Could not access movie/TV folder.",
          type: "error",
        });
        setScanProgress(null);
        refreshSources();
      }
    }).then((unsub) => {
      unlisten = unsub;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [refreshLibrary, refreshSources, showToast]);

  const addSourceByDialog = async () => {
    try {
      const newSource = await ipc.pickAndAddSource();
      if (newSource) {
        showToast({
          title: "Folder Added",
          message: `Scanning ${newSource.name} in background...`,
          type: "info",
        });
        await refreshSources();
        await refreshLibrary();
      }
    } catch (err) {
      showToast({
        title: "Failed to Add Folder",
        message: String(err),
        type: "error",
      });
    }
  };

  const removeSource = async (id: string) => {
    try {
      await ipc.removeSource(id);
      showToast({ title: "Folder Removed", message: "Source and indexed media removed", type: "info" });
      await refreshSources();
      await refreshLibrary();
    } catch (err) {
      showToast({ title: "Failed to Remove Folder", message: String(err), type: "error" });
    }
  };

  const rescanSource = async (id: string) => {
    try {
      await ipc.rescanSource(id);
      showToast({ title: "Rescanning Folder", message: "Checking for new/modified media...", type: "info" });
    } catch (err) {
      showToast({ title: "Rescan Failed", message: String(err), type: "error" });
    }
  };

  const rescanAll = async () => {
    try {
      await ipc.rescanAll();
      showToast({ title: "Rescanning All Folders", message: "Library scan started...", type: "info" });
    } catch (err) {
      showToast({ title: "Rescan All Failed", message: String(err), type: "error" });
    }
  };

  const toggleWatchlist = async (movieId: string) => {
    const isSaved = watchlist.some((m) => m.id === movieId);
    try {
      if (isSaved) {
        await ipc.removeFromWatchlist(movieId);
        setWatchlist((prev) => prev.filter((m) => m.id !== movieId));
        showToast({ title: "Removed from Watchlist", message: "Item removed", type: "info" });
      } else {
        await ipc.addToWatchlist(movieId);
        const movie = movies.find((m) => m.id === movieId);
        if (movie) setWatchlist((prev) => [...prev, movie]);
        showToast({ title: "Saved to Watchlist", message: "Added to your list", type: "success" });
      }
    } catch (err) {
      showToast({ title: "Watchlist Error", message: String(err), type: "error" });
    }
  };

  const openMovieDetails = (id: string) => setSelectedMovieId(id);
  const closeMovieDetails = () => setSelectedMovieId(null);

  const openSeriesDetails = (id: string) => setSelectedSeriesId(id);
  const closeSeriesDetails = () => setSelectedSeriesId(null);

  return (
    <LibraryContext.Provider
      value={{
        sources,
        movies,
        series,
        continueWatching,
        recentlyWatched,
        watchlist,
        scanProgress,
        isLoading,
        filter,
        sort,
        selectedMovieId,
        selectedSeriesId,
        setFilter,
        setSort,
        openMovieDetails,
        closeMovieDetails,
        openSeriesDetails,
        closeSeriesDetails,
        refreshLibrary,
        refreshSources,
        addSourceByDialog,
        removeSource,
        rescanSource,
        rescanAll,
        toggleWatchlist,
      }}
    >
      {children}
    </LibraryContext.Provider>
  );
};

export const useLibrary = (): LibraryContextType => {
  const ctx = useContext(LibraryContext);
  if (!ctx) throw new Error("useLibrary must be used within a LibraryProvider");
  return ctx;
};
