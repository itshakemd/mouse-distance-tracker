import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { EMPTY_SNAPSHOT, type DistanceUnit, type TrackerSnapshot } from "../model/tracker";

export function useTracker() {
  const [snapshot, setSnapshot] = useState<TrackerSnapshot>(EMPTY_SNAPSHOT);

  useEffect(() => {
    let active = true;
    invoke<TrackerSnapshot>("get_snapshot").then((initialSnapshot) => {
      if (active) setSnapshot(initialSnapshot);
    });
    return () => { active = false; };
  }, []);

  const toggleTracking = useCallback(async () => {
    setSnapshot(await invoke<TrackerSnapshot>("toggle_tracking"));
  }, []);

  const changeUnit = useCallback(async (unit: DistanceUnit) => {
    const updated = await invoke<TrackerSnapshot>("update_settings", {
      settings: { ...snapshot.settings, unit },
    });
    setSnapshot(updated);
  }, [snapshot.settings]);

  return { snapshot, toggleTracking, changeUnit };
}
