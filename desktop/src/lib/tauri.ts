import { invoke } from "@tauri-apps/api/core";
import type { SearchResult, IndexedRoot, IndexStats, ModelInfo } from "./types";

export async function searchFiles(query: string): Promise<SearchResult[]> {
  return invoke("search", { query });
}

export async function addFolder(path: string): Promise<IndexedRoot> {
  return invoke("add_folder", { path });
}

export async function removeFolder(id: number): Promise<void> {
  return invoke("remove_folder", { id });
}

export async function getIndexedRoots(): Promise<IndexedRoot[]> {
  return invoke("get_indexed_roots");
}

export async function getIndexStats(): Promise<IndexStats> {
  return invoke("get_index_stats");
}

export async function reindex(): Promise<string> {
  return invoke("reindex");
}

export async function setRootEnabled(id: number, enabled: boolean): Promise<void> {
  return invoke("set_root_enabled", { id, enabled });
}

export async function getDefaultRoots(): Promise<string[]> {
  return invoke("get_default_roots");
}

export async function initDefaultRoots(): Promise<IndexedRoot[]> {
  return invoke("init_default_roots");
}

export async function openFile(path: string): Promise<void> {
  return invoke("open_file", { path });
}

export async function openFileAtLine(path: string, line: number): Promise<void> {
  return invoke("open_file_at_line", { path, line });
}

export async function revealInFolder(path: string): Promise<void> {
  return invoke("reveal_in_folder", { path });
}

export async function getAppVersion(): Promise<string> {
  return invoke("get_app_version");
}

export async function getModelStatus(): Promise<ModelInfo> {
  return invoke("get_model_status");
}

export async function downloadModel(): Promise<ModelInfo> {
  return invoke("download_model");
}

export async function getAllModelStatuses(): Promise<ModelInfo[]> {
  return invoke("get_all_model_statuses");
}

export async function downloadModelById(modelId: string): Promise<ModelInfo> {
  return invoke("download_model_by_id", { modelId });
}

export async function cancelIndexing(): Promise<void> {
  return invoke("cancel_indexing");
}

export async function setIndexingMode(mode: string): Promise<void> {
  return invoke("set_indexing_mode", { mode });
}

export async function getIndexingMode(): Promise<string> {
  return invoke("get_indexing_mode");
}

export async function getImageThumbnail(path: string): Promise<string> {
  return invoke("get_image_thumbnail", { path });
}

export async function checkOcrAvailable(): Promise<boolean> {
  return invoke("check_ocr_available");
}
