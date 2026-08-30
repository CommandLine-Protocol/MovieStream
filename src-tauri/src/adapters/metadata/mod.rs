use serde::Deserialize;
use std::time::Duration;

use crate::abstractions::{MetadataCandidate, MetadataProvider, MovieMetadata};
use crate::error::{AppError, AppResult};

pub struct OpenMovieMetadataProvider {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl OpenMovieMetadataProvider {
    pub fn new(api_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();
        Self { client, api_key }
    }
}

impl Default for OpenMovieMetadataProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

#[derive(Deserialize)]
struct TMDbSearchResponse {
    results: Option<Vec<TMDbMovieResult>>,
}

#[derive(Deserialize)]
struct TMDbMovieResult {
    id: i64,
    title: Option<String>,
    original_title: Option<String>,
    release_date: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    vote_average: Option<f32>,
}

#[derive(Deserialize)]
struct TMDbMovieDetail {
    id: i64,
    title: Option<String>,
    original_title: Option<String>,
    release_date: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    vote_average: Option<f32>,
    genres: Option<Vec<TMDbGenre>>,
    credits: Option<TMDbCredits>,
}

#[derive(Deserialize)]
struct TMDbGenre {
    name: String,
}

#[derive(Deserialize)]
struct TMDbCredits {
    cast: Option<Vec<TMDbCastMember>>,
    crew: Option<Vec<TMDbCrewMember>>,
}

#[derive(Deserialize)]
struct TMDbCastMember {
    name: String,
}

#[derive(Deserialize)]
struct TMDbCrewMember {
    name: String,
    job: String,
}

impl MetadataProvider for OpenMovieMetadataProvider {
    fn name(&self) -> &str {
        "tmdb"
    }

    fn search(&self, title_guess: &str, year_guess: Option<u16>) -> AppResult<Vec<MetadataCandidate>> {
        let env_key = std::env::var("TMDB_API_KEY").ok();
        let api_key = self.api_key.as_ref().or(env_key.as_ref()).filter(|k| !k.is_empty());

        if let Some(k) = api_key {
            // Use TMDB API if key is provided
            let mut url = format!(
                "https://api.themoviedb.org/3/search/movie?api_key={}&query={}",
                k,
                urlencoding::encode(title_guess)
            );
            if let Some(year) = year_guess {
                url.push_str(&format!("&primary_release_year={}", year));
            }

            let resp = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.client.get(&url).send().await
                })
            }).map_err(|e| AppError::Metadata(format!("Network request failed: {}", e)))?;

            if resp.status().is_success() {
                if let Ok(search_res) = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        resp.json::<TMDbSearchResponse>().await
                    })
                }) {
                    let candidates: Vec<MetadataCandidate> = search_res.results.unwrap_or_default().into_iter().map(|item| {
                        let year = item.release_date.as_ref().and_then(|d| {
                            d.split('-').next().and_then(|y| y.parse::<u16>().ok())
                        });
                        let poster_url = item.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p));
                        let backdrop_url = item.backdrop_path.map(|b| format!("https://image.tmdb.org/t/p/original{}", b));

                        MetadataCandidate {
                            id: format!("tmdb-{}", item.id),
                            title: item.title.unwrap_or_else(|| title_guess.to_string()),
                            original_title: item.original_title,
                            year,
                            overview: item.overview,
                            poster_url,
                            backdrop_url,
                            rating: item.vote_average,
                        }
                    }).collect();

                    if !candidates.is_empty() {
                        return Ok(candidates);
                    }
                }
            }
        }

        // Zero-config fallback: Free iTunes Search API (no API key required)
        let itunes_url = format!(
            "https://itunes.apple.com/search?media=movie&term={}&limit=5",
            urlencoding::encode(title_guess)
        );

        let itunes_resp = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.client.get(&itunes_url).send().await
            })
        });

        if let Ok(resp) = itunes_resp {
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct ITunesResponse {
                    results: Option<Vec<ITunesMovie>>,
                }
                #[allow(dead_code)]
                #[derive(Deserialize)]
                struct ITunesMovie {
                    #[serde(rename = "trackId")]
                    track_id: Option<i64>,
                    #[serde(rename = "trackName")]
                    track_name: Option<String>,
                    #[serde(rename = "releaseDate")]
                    release_date: Option<String>,
                    #[serde(rename = "longDescription")]
                    long_description: Option<String>,
                    #[serde(rename = "shortDescription")]
                    short_description: Option<String>,
                    #[serde(rename = "artworkUrl100")]
                    artwork_url_100: Option<String>,
                    #[serde(rename = "primaryGenreName")]
                    primary_genre_name: Option<String>,
                    #[serde(rename = "artistName")]
                    artist_name: Option<String>,
                }

                if let Ok(parsed) = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        resp.json::<ITunesResponse>().await
                    })
                }) {
                    let candidates: Vec<MetadataCandidate> = parsed
                        .results
                        .unwrap_or_default()
                        .into_iter()
                        .map(|m| {
                            let y = m.release_date.as_ref().and_then(|d| {
                                d.split('-').next().and_then(|s| s.parse::<u16>().ok())
                            });
                            // Upgrade 100x100 thumbnail to 1000x1000 high-res poster
                            let poster_url = m.artwork_url_100.as_ref().map(|u| {
                                u.replace("100x100bb.jpg", "1000x1000bb.jpg")
                                 .replace("100x100bb.png", "1000x1000bb.png")
                            });
                            let backdrop_url = poster_url.clone();
                            let overview = m.long_description.or(m.short_description);

                            MetadataCandidate {
                                id: format!("itunes-{}", m.track_id.unwrap_or(0)),
                                title: m.track_name.unwrap_or_else(|| title_guess.to_string()),
                                original_title: None,
                                year: y.or(year_guess),
                                overview,
                                poster_url,
                                backdrop_url,
                                rating: Some(8.0),
                            }
                        })
                        .collect();

                    if !candidates.is_empty() {
                        return Ok(candidates);
                    }
                }
            }
        }

        // Resilient Offline Fallback
        Ok(vec![MetadataCandidate {
            id: format!("local-{}", title_guess.to_lowercase().replace(' ', "-")),
            title: title_guess.to_string(),
            original_title: None,
            year: year_guess,
            overview: Some(format!("Movie entry for {}", title_guess)),
            poster_url: None,
            backdrop_url: None,
            rating: None,
        }])
    }

    fn fetch_details(&self, provider_id: &str) -> AppResult<MovieMetadata> {
        if provider_id.starts_with("local-") {
            let title = provider_id.trim_start_matches("local-").replace('-', " ");
            return Ok(MovieMetadata {
                id: provider_id.to_string(),
                title,
                original_title: None,
                year: None,
                description: None,
                genres: Vec::new(),
                cast: Vec::new(),
                director: None,
                rating: None,
                poster_url: None,
                backdrop_url: None,
            });
        }

        if let Some(itunes_id) = provider_id.strip_prefix("itunes-") {
            let lookup_url = format!("https://itunes.apple.com/lookup?id={}", itunes_id);
            let resp = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.client.get(&lookup_url).send().await
                })
            }).map_err(|e| AppError::Metadata(format!("iTunes request failed: {}", e)))?;

            #[derive(Deserialize)]
            struct ITunesLookupResponse {
                results: Option<Vec<ITunesLookupItem>>,
            }
            #[derive(Deserialize)]
            struct ITunesLookupItem {
                #[serde(rename = "trackName")]
                track_name: Option<String>,
                #[serde(rename = "releaseDate")]
                release_date: Option<String>,
                #[serde(rename = "longDescription")]
                long_description: Option<String>,
                #[serde(rename = "shortDescription")]
                short_description: Option<String>,
                #[serde(rename = "artworkUrl100")]
                artwork_url_100: Option<String>,
                #[serde(rename = "primaryGenreName")]
                primary_genre_name: Option<String>,
                #[serde(rename = "artistName")]
                artist_name: Option<String>,
            }

            if let Ok(lookup_res) = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    resp.json::<ITunesLookupResponse>().await
                })
            }) {
                if let Some(item) = lookup_res.results.and_then(|mut r| if !r.is_empty() { Some(r.remove(0)) } else { None }) {
                    let year = item.release_date.as_ref().and_then(|d| {
                        d.split('-').next().and_then(|s| s.parse::<u16>().ok())
                    });
                    let poster_url = item.artwork_url_100.as_ref().map(|u| {
                        u.replace("100x100bb.jpg", "1000x1000bb.jpg")
                         .replace("100x100bb.png", "1000x1000bb.png")
                    });
                    let backdrop_url = poster_url.clone();
                    let genres = item.primary_genre_name.map(|g| vec![g]).unwrap_or_default();
                    let director = item.artist_name;

                    return Ok(MovieMetadata {
                        id: provider_id.to_string(),
                        title: item.track_name.unwrap_or_else(|| "Unknown Movie".to_string()),
                        original_title: None,
                        year,
                        description: item.long_description.or(item.short_description),
                        genres,
                        cast: Vec::new(),
                        director,
                        rating: Some(8.0),
                        poster_url,
                        backdrop_url,
                    });
                }
            }
        }

        let raw_tmdb_id = provider_id.trim_start_matches("tmdb-");
        let env_key = std::env::var("TMDB_API_KEY").ok();
        let api_key = self.api_key.as_ref().or(env_key.as_ref()).filter(|k| !k.is_empty());

        let api_key = match api_key {
            Some(k) => k.clone(),
            _ => return Err(AppError::Metadata("API key not configured".to_string())),
        };

        let url = format!(
            "https://api.themoviedb.org/3/movie/{}?api_key={}&append_to_response=credits",
            raw_tmdb_id, api_key
        );

        let resp = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.client.get(&url).send().await
            })
        }).map_err(|e| AppError::Metadata(format!("Network request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::Metadata(format!("HTTP error: {}", resp.status())));
        }

        let detail: TMDbMovieDetail = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                resp.json().await
            })
        }).map_err(|e| AppError::Metadata(format!("Failed to parse JSON: {}", e)))?;

        let year = detail.release_date.as_ref().and_then(|d| {
            d.split('-').next().and_then(|y| y.parse::<u16>().ok())
        });
        let genres = detail.genres.unwrap_or_default().into_iter().map(|g| g.name).collect();
        let cast = detail.credits.as_ref().and_then(|c| c.cast.as_ref()).map(|c| {
            c.iter().take(5).map(|m| m.name.clone()).collect()
        }).unwrap_or_default();
        let director = detail.credits.as_ref().and_then(|c| c.crew.as_ref()).and_then(|crew| {
            crew.iter().find(|m| m.job == "Director").map(|m| m.name.clone())
        });

        let poster_url = detail.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p));
        let backdrop_url = detail.backdrop_path.map(|b| format!("https://image.tmdb.org/t/p/original{}", b));

        Ok(MovieMetadata {
            id: detail.id.to_string(),
            title: detail.title.unwrap_or_default(),
            original_title: detail.original_title,
            year,
            description: detail.overview,
            genres,
            cast,
            director,
            rating: detail.vote_average,
            poster_url,
            backdrop_url,
        })
    }

    fn fetch_image(&self, url: &str) -> AppResult<Vec<u8>> {
        let resp = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.client.get(url).send().await
            })
        }).map_err(|e| AppError::Metadata(format!("Image download failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::Metadata(format!("Image HTTP status: {}", resp.status())));
        }

        let bytes = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                resp.bytes().await
            })
        }).map_err(|e| AppError::Metadata(format!("Failed to read image bytes: {}", e)))?;

        Ok(bytes.to_vec())
    }
}

pub struct MockMetadataProvider {
    candidates: Vec<MetadataCandidate>,
    details: std::collections::HashMap<String, MovieMetadata>,
}

impl MockMetadataProvider {
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            details: std::collections::HashMap::new(),
        }
    }

    pub fn with_mock_movie(mut self, metadata: MovieMetadata) -> Self {
        let candidate = MetadataCandidate {
            id: metadata.id.clone(),
            title: metadata.title.clone(),
            original_title: metadata.original_title.clone(),
            year: metadata.year,
            overview: metadata.description.clone(),
            poster_url: metadata.poster_url.clone(),
            backdrop_url: metadata.backdrop_url.clone(),
            rating: metadata.rating,
        };
        self.candidates.push(candidate);
        self.details.insert(metadata.id.clone(), metadata);
        self
    }
}

impl Default for MockMetadataProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataProvider for MockMetadataProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn search(&self, title_guess: &str, year_guess: Option<u16>) -> AppResult<Vec<MetadataCandidate>> {
        let title_lower = title_guess.to_lowercase();
        let matches: Vec<MetadataCandidate> = self
            .candidates
            .iter()
            .filter(|c| {
                let matches_title = c.title.to_lowercase().contains(&title_lower);
                let matches_year = match (year_guess, c.year) {
                    (Some(y1), Some(y2)) => (y1 as i32 - y2 as i32).abs() <= 1,
                    _ => true,
                };
                matches_title && matches_year
            })
            .cloned()
            .collect();

        if matches.is_empty() {
            // Return generated fallback
            Ok(vec![MetadataCandidate {
                id: format!("mock-{}", title_guess.to_lowercase().replace(' ', "-")),
                title: title_guess.to_string(),
                original_title: None,
                year: year_guess,
                overview: Some(format!("Synopsis for {}", title_guess)),
                poster_url: None,
                backdrop_url: None,
                rating: Some(8.5),
            }])
        } else {
            Ok(matches)
        }
    }

    fn fetch_details(&self, provider_id: &str) -> AppResult<MovieMetadata> {
        if let Some(detail) = self.details.get(provider_id) {
            Ok(detail.clone())
        } else {
            Ok(MovieMetadata {
                id: provider_id.to_string(),
                title: provider_id.replace('-', " "),
                original_title: None,
                year: Some(2024),
                description: Some("Detailed overview from mock provider".to_string()),
                genres: vec!["Sci-Fi".to_string(), "Action".to_string()],
                cast: vec!["Actor One".to_string(), "Actor Two".to_string()],
                director: Some("Famous Director".to_string()),
                rating: Some(8.8),
                poster_url: None,
                backdrop_url: None,
            })
        }
    }

    fn fetch_image(&self, _url: &str) -> AppResult<Vec<u8>> {
        // Return dummy 1x1 PNG bytes
        Ok(vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ])
    }
}
