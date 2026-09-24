import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import {
  EMPTY_SNAPSHOT,
  type DistanceUnit,
  type TrackerSnapshot,
} from "../model/tracker";

const UPDATE_EVENT = "tracker-update";

export function useTracker() {
  const [snapshot, setSnapshot] = useState<TrackerSnapshot>(EMPTY_SNAPSHOT);

  useEffect(() => {
    let active = true;

    invoke<TrackerSnapshot>("get_snapshot").then((initialSnapshot) => {
      if (active) setSnapshot(initialSnapshot);
    });

    const unlisten = listen<TrackerSnapshot>(UPDATE_EVENT, (event) => {
      setSnapshot(event.payload);
    });

    return () => {
      active = false;
      void unlisten.then((stopListening) => stopListening());
    };
  }, []);

  const toggleTracking = useCallback(async () => {
    setSnapshot(await invoke<TrackerSnapshot>("toggle_tracking"));
  }, []);

  const changeUnit = useCallback(
    async (unit: DistanceUnit) => {
      const updated = await invoke<TrackerSnapshot>("update_settings", {
        settings: { ...snapshot.settings, unit },
      });
      setSnapshot(updated);
    },
    [snapshot.settings],
  );

  return { snapshot, toggleTracking, changeUnit };
}
