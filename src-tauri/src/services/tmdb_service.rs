use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMovieResult {
    pub id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub vote_average: Option<f32>,
    pub genre_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMovieDetail {
    pub id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<u32>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub vote_average: Option<f32>,
    pub genres: Vec<String>,
    pub cast: Vec<String>,
    pub director: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbTvResult {
    pub id: i64,
    pub name: String,
    pub original_name: Option<String>,
    pub overview: Option<String>,
    pub first_air_date: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub vote_average: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbTvDetail {
    pub id: i64,
    pub name: String,
    pub original_name: Option<String>,
    pub overview: Option<String>,
    pub first_air_date: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub vote_average: Option<f32>,
    pub genres: Vec<String>,
    pub number_of_seasons: u32,
    pub number_of_episodes: u32,
    pub seasons: Vec<TmdbSeasonSummary>,
    pub cast: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbSeasonSummary {
    pub id: i64,
    pub season_number: u32,
    pub name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub episode_count: u32,
    pub air_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbEpisodeDetail {
    pub id: i64,
    pub episode_number: u32,
    pub season_number: u32,
    pub name: String,
    pub overview: Option<String>,
    pub still_path: Option<String>,
    pub air_date: Option<String>,
    pub runtime: Option<u32>,
    pub vote_average: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbTrendingItem {
    pub id: i64,
    pub media_type: String, // "movie" | "tv"
    pub title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: Option<f32>,
}

pub struct TmdbService {
    client: Client,
    api_key: Option<String>,
}

impl TmdbService {
    pub fn new(api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_default();
        Self { client, api_key }
    }

    fn get_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("TMDB_API_KEY").ok())
            .or_else(|| std::env::var("TMDB_API_TOKEN").ok())
            .filter(|k| !k.trim().is_empty())
    }

    pub async fn search_movies(&self, query: &str, year: Option<u16>) -> AppResult<Vec<TmdbMovieResult>> {
        if let Some(key) = self.get_key() {
            let mut url = format!(
                "https://api.themoviedb.org/3/search/movie?api_key={}&query={}&include_adult=false",
                key,
                urlencoding::encode(query)
            );
            if let Some(y) = year {
                url.push_str(&format!("&primary_release_year={}", y));
            }

            let resp = self.client.get(&url).send().await.map_err(|e| AppError::Metadata(e.to_string()))?;
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct Res { results: Option<Vec<RawMovieResult>> }
                #[derive(Deserialize)]
                struct RawMovieResult {
                    id: i64,
                    title: Option<String>,
                    original_title: Option<String>,
                    overview: Option<String>,
                    release_date: Option<String>,
                    poster_path: Option<String>,
                    backdrop_path: Option<String>,
                    vote_average: Option<f32>,
                    genre_ids: Option<Vec<i64>>,
                }
                if let Ok(data) = resp.json::<Res>().await {
                    let results = data.results.unwrap_or_default().into_iter().map(|m| TmdbMovieResult {
                        id: m.id,
                        title: m.title.unwrap_or_else(|| query.to_string()),
                        original_title: m.original_title,
                        overview: m.overview,
                        release_date: m.release_date,
                        poster_path: m.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                        backdrop_path: m.backdrop_path.map(|b| format!("https://image.tmdb.org/t/p/original{}", b)),
                        vote_average: m.vote_average,
                        genre_ids: m.genre_ids,
                    }).collect();
                    return Ok(results);
                }
            }
        }

        // Zero-config iTunes fallback
        self.search_itunes_movies(query).await
    }

    pub async fn get_movie_details(&self, movie_id: i64) -> AppResult<TmdbMovieDetail> {
        if let Some(key) = self.get_key() {
            let url = format!(
                "https://api.themoviedb.org/3/movie/{}?api_key={}&append_to_response=credits",
                movie_id, key
            );
            let resp = self.client.get(&url).send().await.map_err(|e| AppError::Metadata(e.to_string()))?;
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct RawDetail {
                    id: i64,
                    title: Option<String>,
                    original_title: Option<String>,
                    overview: Option<String>,
                    release_date: Option<String>,
                    runtime: Option<u32>,
                    poster_path: Option<String>,
                    backdrop_path: Option<String>,
                    vote_average: Option<f32>,
                    genres: Option<Vec<RawGenre>>,
                    credits: Option<RawCredits>,
                }
                #[derive(Deserialize)] struct RawGenre { name: String }
                #[derive(Deserialize)] struct RawCredits { cast: Option<Vec<RawCast>>, crew: Option<Vec<RawCrew>> }
                #[derive(Deserialize)] struct RawCast { name: String }
                #[derive(Deserialize)] struct RawCrew { name: String, job: String }

                if let Ok(d) = resp.json::<RawDetail>().await {
                    let director = d.credits.as_ref()
                        .and_then(|c| c.crew.as_ref())
                        .and_then(|crew| crew.iter().find(|m| m.job.eq_ignore_ascii_case("Director")).map(|m| m.name.clone()));
                    let cast = d.credits.as_ref()
                        .and_then(|c| c.cast.as_ref())
                        .map(|c| c.iter().take(8).map(|m| m.name.clone()).collect())
                        .unwrap_or_default();
                    let genres = d.genres.unwrap_or_default().into_iter().map(|g| g.name).collect();

                    return Ok(TmdbMovieDetail {
                        id: d.id,
                        title: d.title.unwrap_or_else(|| "Unknown".to_string()),
                        original_title: d.original_title,
                        overview: d.overview,
                        release_date: d.release_date,
                        runtime: d.runtime,
                        poster_path: d.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                        backdrop_path: d.backdrop_path.map(|b| format!("https://image.tmdb.org/t/p/original{}", b)),
                        vote_average: d.vote_average,
                        genres,
                        cast,
                        director,
                    });
                }
            }
        }

        Err(AppError::NotFound("Movie details not found".to_string()))
    }

    pub async fn search_tv(&self, query: &str, year: Option<u16>) -> AppResult<Vec<TmdbTvResult>> {
        if let Some(key) = self.get_key() {
            let mut url = format!(
                "https://api.themoviedb.org/3/search/tv?api_key={}&query={}&include_adult=false",
                key,
                urlencoding::encode(query)
            );
            if let Some(y) = year {
                url.push_str(&format!("&first_air_date_year={}", y));
            }

            let resp = self.client.get(&url).send().await.map_err(|e| AppError::Metadata(e.to_string()))?;
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct Res { results: Option<Vec<RawTvResult>> }
                #[derive(Deserialize)]
                struct RawTvResult {
                    id: i64,
                    name: Option<String>,
                    original_name: Option<String>,
                    overview: Option<String>,
                    first_air_date: Option<String>,
                    poster_path: Option<String>,
                    backdrop_path: Option<String>,
                    vote_average: Option<f32>,
                }
                if let Ok(data) = resp.json::<Res>().await {
                    let results = data.results.unwrap_or_default().into_iter().map(|s| TmdbTvResult {
                        id: s.id,
                        name: s.name.unwrap_or_else(|| query.to_string()),
                        original_name: s.original_name,
                        overview: s.overview,
                        first_air_date: s.first_air_date,
                        poster_path: s.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                        backdrop_path: s.backdrop_path.map(|b| format!("https://image.tmdb.org/t/p/original{}", b)),
                        vote_average: s.vote_average,
                    }).collect();
                    return Ok(results);
                }
            }
        }

        Ok(vec![])
    }

    pub async fn get_tv_details(&self, series_id: i64) -> AppResult<TmdbTvDetail> {
        if let Some(key) = self.get_key() {
            let url = format!(
                "https://api.themoviedb.org/3/tv/{}?api_key={}&append_to_response=credits",
                series_id, key
            );
            let resp = self.client.get(&url).send().await.map_err(|e| AppError::Metadata(e.to_string()))?;
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct RawTvDetail {
                    id: i64,
                    name: Option<String>,
                    original_name: Option<String>,
                    overview: Option<String>,
                    first_air_date: Option<String>,
                    poster_path: Option<String>,
                    backdrop_path: Option<String>,
                    vote_average: Option<f32>,
                    number_of_seasons: Option<u32>,
                    number_of_episodes: Option<u32>,
                    genres: Option<Vec<RawG>>,
                    seasons: Option<Vec<RawS>>,
                    credits: Option<RawCred>,
                }
                #[derive(Deserialize)] struct RawG { name: String }
                #[derive(Deserialize)] struct RawS {
                    id: i64,
                    season_number: u32,
                    name: Option<String>,
                    overview: Option<String>,
                    poster_path: Option<String>,
                    episode_count: Option<u32>,
                    air_date: Option<String>,
                }
                #[derive(Deserialize)] struct RawCred { cast: Option<Vec<RawC>> }
                #[derive(Deserialize)] struct RawC { name: String }

                if let Ok(d) = resp.json::<RawTvDetail>().await {
                    let cast = d.credits.as_ref()
                        .and_then(|c| c.cast.as_ref())
                        .map(|c| c.iter().take(8).map(|m| m.name.clone()).collect())
                        .unwrap_or_default();
                    let genres = d.genres.unwrap_or_default().into_iter().map(|g| g.name).collect();
                    let seasons = d.seasons.unwrap_or_default().into_iter().map(|s| TmdbSeasonSummary {
                        id: s.id,
                        season_number: s.season_number,
                        name: s.name.unwrap_or_else(|| format!("Season {}", s.season_number)),
                        overview: s.overview,
                        poster_path: s.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                        episode_count: s.episode_count.unwrap_or(0),
                        air_date: s.air_date,
                    }).collect();

                    return Ok(TmdbTvDetail {
                        id: d.id,
                        name: d.name.unwrap_or_else(|| "Unknown Series".to_string()),
                        original_name: d.original_name,
                        overview: d.overview,
                        first_air_date: d.first_air_date,
                        poster_path: d.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                        backdrop_path: d.backdrop_path.map(|b| format!("https://image.tmdb.org/t/p/original{}", b)),
                        vote_average: d.vote_average,
                        genres,
                        number_of_seasons: d.number_of_seasons.unwrap_or(1),
                        number_of_episodes: d.number_of_episodes.unwrap_or(0),
                        seasons,
                        cast,
                    });
                }
            }
        }

        Err(AppError::NotFound("Series details not found".to_string()))
    }

    pub async fn get_tv_episodes(&self, series_id: i64, season_number: u32) -> AppResult<Vec<TmdbEpisodeDetail>> {
        if let Some(key) = self.get_key() {
            let url = format!(
                "https://api.themoviedb.org/3/tv/{}/season/{}?api_key={}",
                series_id, season_number, key
            );
            let resp = self.client.get(&url).send().await.map_err(|e| AppError::Metadata(e.to_string()))?;
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct RawSeasonResp { episodes: Option<Vec<RawEp>> }
                #[derive(Deserialize)]
                struct RawEp {
                    id: i64,
                    episode_number: u32,
                    season_number: u32,
                    name: Option<String>,
                    overview: Option<String>,
                    still_path: Option<String>,
                    air_date: Option<String>,
                    runtime: Option<u32>,
                    vote_average: Option<f32>,
                }
                if let Ok(d) = resp.json::<RawSeasonResp>().await {
                    let episodes = d.episodes.unwrap_or_default().into_iter().map(|e| TmdbEpisodeDetail {
                        id: e.id,
                        episode_number: e.episode_number,
                        season_number: e.season_number,
                        name: e.name.unwrap_or_else(|| format!("Episode {}", e.episode_number)),
                        overview: e.overview,
                        still_path: e.still_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                        air_date: e.air_date,
                        runtime: e.runtime,
                        vote_average: e.vote_average,
                    }).collect();
                    return Ok(episodes);
                }
            }
        }

        Ok(vec![])
    }

    pub async fn get_trending(&self, media_type: &str) -> AppResult<Vec<TmdbTrendingItem>> {
        if let Some(key) = self.get_key() {
            let url = format!(
                "https://api.themoviedb.org/3/trending/{}/week?api_key={}",
                if media_type == "tv" { "tv" } else { "movie" },
                key
            );
            let resp = self.client.get(&url).send().await.map_err(|e| AppError::Metadata(e.to_string()))?;
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct Res { results: Option<Vec<RawItem>> }
                #[derive(Deserialize)]
                struct RawItem {
                    id: i64,
                    media_type: Option<String>,
                    title: Option<String>,
                    name: Option<String>,
                    overview: Option<String>,
                    poster_path: Option<String>,
                    backdrop_path: Option<String>,
                    release_date: Option<String>,
                    first_air_date: Option<String>,
                    vote_average: Option<f32>,
                }
                if let Ok(d) = resp.json::<Res>().await {
                    let results = d.results.unwrap_or_default().into_iter().map(|i| {
                        let t = i.title.or(i.name).unwrap_or_default();
                        let r = i.release_date.or(i.first_air_date);
                        TmdbTrendingItem {
                            id: i.id,
                            media_type: i.media_type.unwrap_or_else(|| media_type.to_string()),
                            title: t,
                            overview: i.overview,
                            poster_path: i.poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                            backdrop_path: i.backdrop_path.map(|b| format!("https://image.tmdb.org/t/p/original{}", b)),
                            release_date: r,
                            vote_average: i.vote_average,
                        }
                    }).collect();
                    return Ok(results);
                }
            }
        }

        Ok(vec![])
    }

    async fn search_itunes_movies(&self, query: &str) -> AppResult<Vec<TmdbMovieResult>> {
        let itunes_url = format!(
            "https://itunes.apple.com/search?media=movie&term={}&limit=10",
            urlencoding::encode(query)
        );
        if let Ok(resp) = self.client.get(&itunes_url).send().await {
            #[derive(Deserialize)] struct ITunesResponse { results: Option<Vec<ITunesMovie>> }
            #[derive(Deserialize)] struct ITunesMovie {
                #[serde(rename = "trackId")] track_id: Option<i64>,
                #[serde(rename = "trackName")] track_name: Option<String>,
                #[serde(rename = "artworkUrl100")] artwork_url_100: Option<String>,
                #[serde(rename = "longDescription")] long_description: Option<String>,
                #[serde(rename = "releaseDate")] release_date: Option<String>,
            }
            if let Ok(data) = resp.json::<ITunesResponse>().await {
                let results = data.results.unwrap_or_default().into_iter().map(|item| {
                    let poster = item.artwork_url_100.map(|u| u.replace("100x100bb.jpg", "1000x1000bb.jpg"));
                    TmdbMovieResult {
                        id: item.track_id.unwrap_or(0),
                        title: item.track_name.unwrap_or_else(|| query.to_string()),
                        original_title: None,
                        overview: item.long_description,
                        release_date: item.release_date,
                        poster_path: poster.clone(),
                        backdrop_path: poster,
                        vote_average: Some(7.5),
                        genre_ids: None,
                    }
                }).collect();
                return Ok(results);
            }
        }
        Ok(vec![])
    }
}
