import { useState, useCallback, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ModelInfo, DownloadProgress, ModelStatusValue } from "../lib/types";
import {
  getModelStatus,
  downloadModel as downloadModelApi,
  getAllModelStatuses,
  downloadModelById,
} from "../lib/tauri";

interface ModelManagerState {
  model: ModelInfo | null;
  status: ModelStatusValue;
  progress: DownloadProgress | null;
  error: string | null;
  loading: boolean;
  allModels: ModelInfo[];
  downloadingModelId: string | null;
  refresh: () => Promise<void>;
  startDownload: () => Promise<void>;
  startDownloadById: (modelId: string) => Promise<void>;
}

export function useModelManager(): ModelManagerState {
  const [model, setModel] = useState<ModelInfo | null>(null);
  const [status, setStatus] = useState<ModelStatusValue>("not_installed");
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [allModels, setAllModels] = useState<ModelInfo[]>([]);
  const [downloadingModelId, setDownloadingModelId] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const refresh = useCallback(async () => {
    try {
      const info = await getModelStatus();
      setModel(info);
      setStatus(info.status);
      if (info.status === "installed") {
        setError(null);
      }
      const models = await getAllModelStatuses();
      setAllModels(models);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const startDownload = useCallback(async () => {
    if (loading) return;

    setLoading(true);
    setError(null);
    setStatus("downloading");
    setProgress(null);
    setDownloadingModelId("multilingual-e5-small");

    try {
      const info = await downloadModelApi();
      setModel(info);
      setStatus(info.status);
      setProgress(null);
      await refresh();
    } catch (e) {
      setStatus("failed");
      setError(String(e));
    } finally {
      setLoading(false);
      setDownloadingModelId(null);
    }
  }, [loading, refresh]);

  const startDownloadById = useCallback(async (modelId: string) => {
    if (loading) return;

    setLoading(true);
    setError(null);
    setProgress(null);
    setDownloadingModelId(modelId);

    try {
      await downloadModelById(modelId);
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
      setDownloadingModelId(null);
    }
  }, [loading, refresh]);

  useEffect(() => {
    let cancelled = false;

    listen<DownloadProgress>("model-download-progress", (event) => {
      if (!cancelled) {
        setProgress(event.payload);
      }
    }).then((unlisten) => {
      if (cancelled) {
        unlisten();
      } else {
        unlistenRef.current = unlisten;
      }
    });

    return () => {
      cancelled = true;
      unlistenRef.current?.();
      unlistenRef.current = null;
    };
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    model, status, progress, error, loading,
    allModels, downloadingModelId,
    refresh, startDownload, startDownloadById,
  };
}
