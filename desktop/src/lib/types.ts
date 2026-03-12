export interface SearchResult {
  file_id: number;
  path: string;
  filename: string;
  extension: string | null;
  size: number;
  modified_time: string;
  snippet: string | null;
  score: number;
  match_type: string;
  line_start: number | null;
  line_end: number | null;
  symbol_name: string | null;
}

export interface IndexedRoot {
  id: number;
  path: string;
  added_time: string;
  last_scan: string | null;
  enabled: boolean;
}

export interface IndexStats {
  total_files: number;
  total_chunks: number;
  total_vectors: number;
  total_roots: number;
  db_size_bytes: number;
}

export type ModelStatusValue =
  | "not_installed"
  | "downloading"
  | "installed"
  | "failed";

export interface ModelFileInfo {
  filename: string;
  expected_size: number | null;
  actual_size: number | null;
  present: boolean;
}

export interface ModelInfo {
  id: string;
  display_name: string;
  version: string;
  status: ModelStatusValue;
  model_path: string;
  files: ModelFileInfo[];
}

export interface DownloadProgress {
  file: string;
  file_index: number;
  file_count: number;
  downloaded_bytes: number;
  total_bytes: number | null;
  percent: number | null;
}

export type IndexingMode = "priority" | "idle";

export interface IndexingEvent {
  phase: "scanning" | "indexing" | "completed";
  processed: number;
  total: number;
  percent: number;
  current_file: string;
  status: string;
  skipped: number;
  indexed: number;
  errors: number;
}
