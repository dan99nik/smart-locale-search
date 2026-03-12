import { useState, useCallback, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import type { IndexingEvent, IndexingMode } from "../lib/types";
import {
  reindex,
  cancelIndexing as cancelApi,
  getIndexingMode,
  setIndexingMode as setModeApi,
} from "../lib/tauri";

interface IndexingState {
  indexing: boolean;
  event: IndexingEvent | null;
  mode: IndexingMode;
  error: string | null;
  startIndexing: () => void;
  cancelIndexing: () => void;
  setMode: (mode: IndexingMode) => void;
}

export function useIndexing(): IndexingState {
  const [indexing, setIndexing] = useState(false);
  const [event, setEvent] = useState<IndexingEvent | null>(null);
  const [mode, setMode] = useState<IndexingMode>("priority");
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    getIndexingMode()
      .then((m) => setMode(m as IndexingMode))
      .catch(() => {});
  }, []);

  useEffect(() => {
    let cancelled = false;
    listen<IndexingEvent>("indexing-progress", (e) => {
      if (!cancelled) {
        setEvent(e.payload);
        if (e.payload.phase === "completed") {
          setIndexing(false);
        }
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

  const startIndexing = useCallback(() => {
    if (indexing) return;
    setIndexing(true);
    setError(null);
    setEvent(null);
    reindex().catch((e) => {
      setError(String(e));
      setIndexing(false);
    });
  }, [indexing]);

  const cancelIndexing = useCallback(() => {
    cancelApi().catch(() => {});
  }, []);

  const handleSetMode = useCallback((m: IndexingMode) => {
    setMode(m);
    setModeApi(m).catch(() => {});
  }, []);

  return {
    indexing,
    event,
    mode,
    error,
    startIndexing,
    cancelIndexing,
    setMode: handleSetMode,
  };
}
