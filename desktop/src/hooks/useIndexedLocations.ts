import { useState, useCallback, useEffect } from "react";
import type { IndexedRoot } from "../lib/types";
import {
  getIndexedRoots,
  initDefaultRoots,
  setRootEnabled,
  addFolder,
  removeFolder,
} from "../lib/tauri";

export function useIndexedLocations() {
  const [roots, setRoots] = useState<IndexedRoot[]>([]);
  const [loading, setLoading] = useState(false);
  const [toggling, setToggling] = useState<number | null>(null);

  const refresh = useCallback(async () => {
    try {
      const r = await getIndexedRoots();
      setRoots(r);
    } catch (e) {
      console.error("Failed to refresh roots:", e);
    }
  }, []);

  const initialize = useCallback(async () => {
    setLoading(true);
    try {
      const r = await initDefaultRoots();
      setRoots(r);
    } catch (e) {
      console.error("Failed to initialize default roots:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const toggleRoot = useCallback(
    async (id: number, enabled: boolean) => {
      setToggling(id);
      try {
        await setRootEnabled(id, enabled);
        setRoots((prev) =>
          prev.map((r) => (r.id === id ? { ...r, enabled } : r))
        );
      } catch (e) {
        console.error("Failed to toggle root:", e);
      } finally {
        setToggling(null);
      }
    },
    []
  );

  const handleAddFolder = useCallback(
    async (path: string) => {
      setLoading(true);
      try {
        await addFolder(path);
        await refresh();
      } catch (e) {
        console.error("Failed to add folder:", e);
      } finally {
        setLoading(false);
      }
    },
    [refresh]
  );

  const handleRemoveFolder = useCallback(
    async (id: number) => {
      setLoading(true);
      try {
        await removeFolder(id);
        await refresh();
      } catch (e) {
        console.error("Failed to remove folder:", e);
      } finally {
        setLoading(false);
      }
    },
    [refresh]
  );

  useEffect(() => {
    initialize();
  }, [initialize]);

  return {
    roots,
    loading,
    toggling,
    refresh,
    toggleRoot,
    handleAddFolder,
    handleRemoveFolder,
  };
}
