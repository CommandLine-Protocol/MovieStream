import React, { useState } from "react";
import { ToastProvider } from "./state/ToastContext";
import { LibraryProvider, useLibrary } from "./state/LibraryContext";
import { PlaybackProvider } from "./state/PlaybackContext";
import { Navbar, NavTab } from "./components/Navbar";
import { ToastContainer } from "./components/Toast";
import { Home } from "./views/Home";
import { AllMovies } from "./views/AllMovies";
import { AllSeries } from "./views/AllSeries";
import { WatchlistView } from "./views/WatchlistView";
import { Search } from "./views/Search";
import { Settings } from "./views/Settings";
import { MovieDetails } from "./views/MovieDetails";
import { SeriesDetails } from "./views/SeriesDetails";
import { Player } from "./views/Player";
import { ResumeModal } from "./components/ResumeModal";

const AppContent: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<NavTab>("home");
  const [searchQuery, setSearchQuery] = useState("");
  const { selectedMovieId, closeMovieDetails, selectedSeriesId, closeSeriesDetails } = useLibrary();

  return (
    <div className="app-container">
      <Navbar
        currentTab={currentTab}
        onSelectTab={(tab) => {
          setCurrentTab(tab);
          if (tab !== "search") setSearchQuery("");
        }}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
      />

      <main className="main-content">
        {currentTab === "home" && <Home />}
        {currentTab === "movies" && <AllMovies />}
        {currentTab === "series" && <AllSeries />}
        {currentTab === "watchlist" && <WatchlistView />}
        {currentTab === "settings" && <Settings />}
        {currentTab === "search" && <Search initialQuery={searchQuery} />}
      </main>

      {/* Movie Details Modal */}
      {selectedMovieId && (
        <MovieDetails movieId={selectedMovieId} onClose={closeMovieDetails} />
      )}

      {/* TV Series Details Modal */}
      {selectedSeriesId && (
        <SeriesDetails seriesId={selectedSeriesId} onClose={closeSeriesDetails} />
      )}

      {/* Fullscreen Video Player */}
      <Player />

      {/* Resume Playback Prompt Modal */}
      <ResumeModal />

      {/* Toast notifications */}
      <ToastContainer />
    </div>
  );
};

export const App: React.FC = () => {
  return (
    <ToastProvider>
      <LibraryProvider>
        <PlaybackProvider>
          <AppContent />
        </PlaybackProvider>
      </LibraryProvider>
    </ToastProvider>
  );
};

export default App;
