import { useState, useEffect, useCallback, useRef } from "react";
import type { DriveInfo } from "../lib/types";
import { listDrives } from "../lib/tauri";

export function useDrives() {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const isFirstLoad = useRef(true);

  const refresh = useCallback(async () => {
    if (isFirstLoad.current) {
      setLoading(true);
    }
    try {
      const result = await listDrives();
      setDrives(result);
      setError(null);
    } catch (e) {
      // Only show error on first load, don't flash errors on auto-refresh
      if (isFirstLoad.current) {
        setError(String(e));
      }
    } finally {
      if (isFirstLoad.current) {
        setLoading(false);
        isFirstLoad.current = false;
      }
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 5000);
    return () => clearInterval(interval);
  }, [refresh]);

  const manualRefresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listDrives();
      setDrives(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  return { drives, loading, error, refresh: manualRefresh };
}
