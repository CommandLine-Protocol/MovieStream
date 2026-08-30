use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::abstractions::MetadataProvider;
use crate::domain::{MetadataStatus, Movie};
use crate::error::AppResult;

pub struct MetadataResolver {
    provider: Arc<dyn MetadataProvider>,
    cache_dir: PathBuf,
}

impl MetadataResolver {
    pub fn new(provider: Arc<dyn MetadataProvider>, cache_dir: PathBuf) -> Self {
        if !cache_dir.exists() {
            let _ = std::fs::create_dir_all(&cache_dir);
        }
        Self {
            provider,
            cache_dir,
        }
    }

    pub fn resolve(
        &self,
        title_guess: &str,
        year_guess: Option<u16>,
    ) -> Movie {
        self.resolve_with_file_path(title_guess, year_guess, None)
    }

    pub fn resolve_with_file_path(
        &self,
        title_guess: &str,
        year_guess: Option<u16>,
        media_path: Option<&str>,
    ) -> Movie {
        let movie_id = Uuid::new_v4();
        let now = Utc::now();

        // 1. Check for local folder artwork
        let mut local_poster: Option<String> = None;
        let mut local_backdrop: Option<String> = None;

        if let Some(path_str) = media_path {
            let p = std::path::Path::new(path_str);
            if let Some(parent) = p.parent() {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let poster_candidates = [
                    format!("{}.jpg", stem),
                    format!("{}.png", stem),
                    format!("{}-poster.jpg", stem),
                    "poster.jpg".to_string(),
                    "poster.png".to_string(),
                    "cover.jpg".to_string(),
                    "cover.png".to_string(),
                    "folder.jpg".to_string(),
                    "folder.png".to_string(),
                ];
                for cand in &poster_candidates {
                    let cand_path = parent.join(cand);
                    if cand_path.exists() {
                        local_poster = Some(cand_path.to_string_lossy().to_string());
                        break;
                    }
                }

                let backdrop_candidates = [
                    format!("{}-fanart.jpg", stem),
                    format!("{}-backdrop.jpg", stem),
                    "backdrop.jpg".to_string(),
                    "backdrop.png".to_string(),
                    "fanart.jpg".to_string(),
                    "fanart.png".to_string(),
                ];
                for cand in &backdrop_candidates {
                    let cand_path = parent.join(cand);
                    if cand_path.exists() {
                        local_backdrop = Some(cand_path.to_string_lossy().to_string());
                        break;
                    }
                }
            }
        }

        // 2. Attempt metadata lookup
        match self.provider.search(title_guess, year_guess) {
            Ok(candidates) if !candidates.is_empty() => {
                let candidate = &candidates[0];
                match self.provider.fetch_details(&candidate.id) {
                    Ok(details) => {
                        let poster_path = local_poster.or_else(|| {
                            details.poster_url.as_ref().and_then(|url| {
                                self.cache_artwork(&movie_id, "poster", url).ok()
                            })
                        });
                        let backdrop_path = local_backdrop.or_else(|| {
                            details.backdrop_url.as_ref().and_then(|url| {
                                self.cache_artwork(&movie_id, "backdrop", url).ok()
                            })
                        });

                        Movie {
                            id: movie_id,
                            title: details.title,
                            original_title: details.original_title,
                            year: details.year.or(year_guess),
                            description: details.description,
                            poster_path,
                            backdrop_path,
                            genres: details.genres,
                            cast: details.cast,
                            director: details.director,
                            rating: details.rating,
                            metadata_provider_id: Some(details.id),
                            metadata_status: MetadataStatus::AutoMatched,
                            created_at: now,
                            updated_at: now,
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Failed to fetch details for {}: {}", candidate.id, err);
                        let mut m = Self::create_unmatched(movie_id, title_guess, year_guess, now);
                        m.poster_path = local_poster;
                        m.backdrop_path = local_backdrop;
                        m
                    }
                }
            }
            Ok(_) | Err(_) => {
                let mut m = Self::create_unmatched(movie_id, title_guess, year_guess, now);
                m.poster_path = local_poster;
                m.backdrop_path = local_backdrop;
                m
            }
        }
    }

    pub fn resolve_manual(
        &self,
        existing_movie: &Movie,
        provider_id: &str,
    ) -> AppResult<Movie> {
        let details = self.provider.fetch_details(provider_id)?;
        let poster_path = details.poster_url.as_ref().and_then(|url| {
            self.cache_artwork(&existing_movie.id, "poster", url).ok()
        });
        let backdrop_path = details.backdrop_url.as_ref().and_then(|url| {
            self.cache_artwork(&existing_movie.id, "backdrop", url).ok()
        });

        let mut updated = existing_movie.clone();
        updated.title = details.title;
        updated.original_title = details.original_title;
        updated.year = details.year.or(existing_movie.year);
        updated.description = details.description;
        if poster_path.is_some() {
            updated.poster_path = poster_path;
        }
        if backdrop_path.is_some() {
            updated.backdrop_path = backdrop_path;
        }
        updated.genres = details.genres;
        updated.cast = details.cast;
        updated.director = details.director;
        updated.rating = details.rating;
        updated.metadata_provider_id = Some(details.id);
        updated.metadata_status = MetadataStatus::ManuallyMatched;
        updated.updated_at = Utc::now();

        Ok(updated)
    }

    fn create_unmatched(id: Uuid, title_guess: &str, year_guess: Option<u16>, now: chrono::DateTime<Utc>) -> Movie {
        Movie {
            id,
            title: title_guess.to_string(),
            original_title: None,
            year: year_guess,
            description: None,
            poster_path: None,
            backdrop_path: None,
            genres: Vec::new(),
            cast: Vec::new(),
            director: None,
            rating: None,
            metadata_provider_id: None,
            metadata_status: MetadataStatus::Unmatched,
            created_at: now,
            updated_at: now,
        }
    }

    fn cache_artwork(&self, movie_id: &Uuid, kind: &str, url: &str) -> AppResult<String> {
        let image_bytes = self.provider.fetch_image(url)?;
        let file_name = format!("{}_{}.jpg", movie_id, kind);
        let dest_path = self.cache_dir.join(file_name);
        std::fs::write(&dest_path, image_bytes)?;
        Ok(dest_path.to_string_lossy().to_string())
    }
}
