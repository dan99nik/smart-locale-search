import { useState, useCallback, useEffect } from "react";
import type { IndexStats } from "../lib/types";
import { getIndexStats, reindex as reindexApi } from "../lib/tauri";

export function useSettings() {
  const [stats, setStats] = useState<IndexStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const s = await getIndexStats();
      setStats(s);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const handleReindex = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await reindexApi();
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [refresh]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    stats,
    loading,
    error,
    handleReindex,
    refresh,
  };
}
