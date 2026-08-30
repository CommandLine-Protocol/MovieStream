use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedMediaType {
    Movie,
    Episode {
        season_number: u32,
        episode_number: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFilename {
    pub title_guess: String,
    pub year_guess: Option<u16>,
    pub edition_guess: Option<String>,
    pub resolution_guess: Option<String>,
    pub media_type: ParsedMediaType,
}

lazy_static! {
    static ref TV_EPISODE_REGEX_1: Regex = Regex::new(r"(?i)\bS(\d{1,2})[\.E_x](\d{1,3})\b").unwrap();
    static ref TV_EPISODE_REGEX_2: Regex = Regex::new(r"(?i)\b(\d{1,2})x(\d{1,3})\b").unwrap();
    static ref TV_EPISODE_REGEX_3: Regex = Regex::new(r"(?i)\bseason\s*(\d{1,2})\s*episode\s*(\d{1,3})\b").unwrap();

    static ref YEAR_REGEX: Regex = Regex::new(r"\b((?:19|20)\d{2})\b").unwrap();
    static ref RESOLUTION_REGEX: Regex = Regex::new(r"(?i)\b(2160p|4k|uhd|1080p|1080i|720p|576p|480p)\b").unwrap();
    static ref EDITION_REGEX: Regex = Regex::new(r"(?i)\b(extended|directors\.cut|remastered|unrated|theatrical|criterion|special\.edition|imax)\b").unwrap();
    static ref NOISE_TAGS_REGEX: Regex = Regex::new(
        r"(?i)\b(bluray|blu-ray|web-dl|webrip|web|hdrip|dvdrip|hdtv|x264|x265|hevc|h264|h265|10bit|hdr10\+|hdr10|hdr|sdr|dts-hd\.ma|dts-hd|dts|truehd|atmos|ac3|dd5\.1|aac2\.0|aac5\.1|aac|flac|mp3|proper|repack|rerip|internal|remux)\b"
    ).unwrap();
    static ref CLEANUP_SEPARATORS: Regex = Regex::new(r"[\._\+]+").unwrap();
    static ref MULTI_SPACE: Regex = Regex::new(r"\s+").unwrap();
}

pub struct FilenameParser;

impl FilenameParser {
    pub fn parse(filename_or_stem: &str) -> ParsedFilename {
        // Strip file extension if present
        let stem = if let Some(idx) = filename_or_stem.rfind('.') {
            if idx > 0 && filename_or_stem.len() - idx <= 5 {
                &filename_or_stem[..idx]
            } else {
                filename_or_stem
            }
        } else {
            filename_or_stem
        };

        // Extract resolution if present
        let resolution_guess = RESOLUTION_REGEX.find(stem).map(|m| m.as_str().to_uppercase());

        // Extract edition if present
        let edition_guess = EDITION_REGEX.find(stem).map(|m| m.as_str().replace('.', " ").to_uppercase());

        // Check if TV Episode
        let mut media_type = ParsedMediaType::Movie;
        let mut title_part = stem.to_string();

        if let Some(caps) = TV_EPISODE_REGEX_1.captures(stem) {
            let s: u32 = caps[1].parse().unwrap_or(1);
            let e: u32 = caps[2].parse().unwrap_or(1);
            media_type = ParsedMediaType::Episode { season_number: s, episode_number: e };
            let match_start = caps.get(0).unwrap().start();
            if match_start > 0 {
                title_part = stem[..match_start].to_string();
            }
        } else if let Some(caps) = TV_EPISODE_REGEX_2.captures(stem) {
            let s: u32 = caps[1].parse().unwrap_or(1);
            let e: u32 = caps[2].parse().unwrap_or(1);
            media_type = ParsedMediaType::Episode { season_number: s, episode_number: e };
            let match_start = caps.get(0).unwrap().start();
            if match_start > 0 {
                title_part = stem[..match_start].to_string();
            }
        } else if let Some(caps) = TV_EPISODE_REGEX_3.captures(stem) {
            let s: u32 = caps[1].parse().unwrap_or(1);
            let e: u32 = caps[2].parse().unwrap_or(1);
            media_type = ParsedMediaType::Episode { season_number: s, episode_number: e };
            let match_start = caps.get(0).unwrap().start();
            if match_start > 0 {
                title_part = stem[..match_start].to_string();
            }
        }

        // Look for 4-digit year (1900-2030) if movie or if title contains year
        let mut year_guess: Option<u16> = None;
        let mut all_year_matches: Vec<(u16, usize)> = Vec::new();
        for captures in YEAR_REGEX.captures_iter(&title_part) {
            if let Some(matched_year) = captures.get(1) {
                if let Ok(y) = matched_year.as_str().parse::<u16>() {
                    if (1900..=2099).contains(&y) {
                        let match_start = captures.get(0).unwrap().start();
                        all_year_matches.push((y, match_start));
                    }
                }
            }
        }

        if !all_year_matches.is_empty() {
            let chosen_match = all_year_matches
                .iter()
                .filter(|(y, _)| *y <= 2030)
                .last()
                .or_else(|| all_year_matches.last());

            if let Some(&(y, match_start)) = chosen_match {
                year_guess = Some(y);
                if match_start > 0 && matches!(media_type, ParsedMediaType::Movie) {
                    title_part = title_part[..match_start].to_string();
                }
            }
        }

        // Clean up noise and tokens
        let mut cleaned = title_part;
        cleaned = RESOLUTION_REGEX.replace_all(&cleaned, " ").to_string();
        cleaned = EDITION_REGEX.replace_all(&cleaned, " ").to_string();
        cleaned = NOISE_TAGS_REGEX.replace_all(&cleaned, " ").to_string();

        let cleaned_separators = CLEANUP_SEPARATORS.replace_all(&cleaned, " ");
        let cleaned_noise = NOISE_TAGS_REGEX.replace_all(&cleaned_separators, " ");
        let cleaned_res = RESOLUTION_REGEX.replace_all(&cleaned_noise, " ");
        let no_brackets = cleaned_res.replace(['(', ')', '[', ']', '{', '}', '-'], " ");
        let title_guess = MULTI_SPACE.replace_all(&no_brackets, " ").trim().to_string();

        let final_title = if title_guess.is_empty() {
            stem.to_string()
        } else {
            title_guess
        };

        ParsedFilename {
            title_guess: final_title,
            year_guess,
            edition_guess,
            resolution_guess,
            media_type,
        }
    }
}
