export type SourceStatus =
  | "available"
  | "unavailable"
  | "scanning"
  | "indexing"
  | "inaccessible"
  | "disconnected";

export interface LibrarySource {
  id: string;
  path: string;
  name: string;
  status: SourceStatus;
  last_scanned_at: string | null;
  created_at: string;
}

export interface ScanProgressPayload {
  source_id: string;
  files_discovered: number;
  movies_identified: number;
  phase: "scanning" | "analyzing" | "matching" | "indexing" | "completed" | "error";
}
