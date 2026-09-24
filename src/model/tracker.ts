export type DistanceUnit = "px" | "m" | "km";

export interface TrackerSettings {
  unit: DistanceUnit;
  ppi: number;
}

export interface TrackerSnapshot {
  sessionPixels: number;
  todayPixels: number;
  weekPixels: number;
  monthPixels: number;
  yearPixels: number;
  allTimePixels: number;
  paused: boolean;
  settings: TrackerSettings;
}

export const EMPTY_SNAPSHOT: TrackerSnapshot = {
  sessionPixels: 0,
  todayPixels: 0,
  weekPixels: 0,
  monthPixels: 0,
  yearPixels: 0,
  allTimePixels: 0,
  paused: false,
  settings: { unit: "km", ppi: 96 },
};
