import { useState, useCallback, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { TestConfig, TestProgress, TestResult } from "../lib/types";
import { startTest, stopTest } from "../lib/tauri";

export function useTest() {
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<TestProgress | null>(null);
  const [result, setResult] = useState<TestResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlistenProgress = listen<TestProgress>("test-progress", (event) => {
      setProgress(event.payload);
      if (event.payload.error) {
        setError(event.payload.error);
      }
    });

    const unlistenComplete = listen<TestResult>("test-complete", (event) => {
      setResult(event.payload);
      setRunning(false);
      setProgress(null);
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  const start = useCallback(async (config: TestConfig) => {
    setRunning(true);
    setResult(null);
    setError(null);
    setProgress(null);
    try {
      await startTest(config);
    } catch (e) {
      setError(String(e));
      setRunning(false);
    }
  }, []);

  const stop = useCallback(async () => {
    try {
      await stopTest();
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return { running, progress, result, error, start, stop };
}
