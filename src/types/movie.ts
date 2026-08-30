export type MetadataStatus = "unmatched" | "auto_matched" | "manually_matched" | "failed";

export type MovieSort =
  | "title_asc"
  | "title_desc"
  | "year_asc"
  | "year_desc"
  | "date_added_asc"
  | "date_added_desc"
  | "recently_watched"
  | "rating_desc";

export interface MovieFilter {
  genre?: string;
  year?: number;
  watched?: boolean;
  in_watchlist?: boolean;
  source_id?: string;
  min_rating?: number;
  is_available?: boolean;
}

export interface Movie {
  id: string;
  title: string;
  original_title: string | null;
  year: number | null;
  description: string | null;
  poster_path: string | null;
  backdrop_path: string | null;
  genres: string[];
  cast: string[];
  director: string | null;
  rating: number | null;
  metadata_provider_id: string | null;
  metadata_status: MetadataStatus;
  created_at: string;
  updated_at: string;
}

export interface MovieWithMedia {
  movie: Movie;
  media: import("./media").Media[];
}
