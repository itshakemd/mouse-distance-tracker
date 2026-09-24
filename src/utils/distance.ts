import type { DistanceUnit, TrackerSettings } from "../model/tracker";

const METERS_PER_INCH = 0.0254;

export function formatDistance(
  pixels: number,
  settings: TrackerSettings,
  unit: DistanceUnit = settings.unit,
): string {
  if (unit === "px") {
    return `${Math.round(pixels).toLocaleString()} px`;
  }

  const meters = (pixels / Math.max(settings.ppi, 1)) * METERS_PER_INCH;
  if (unit === "m") {
    return `${meters.toFixed(meters < 10 ? 2 : 1)} m`;
  }

  return `${(meters / 1000).toFixed(3)} km`;
}

export function distanceValue(formattedDistance: string): string {
  return formattedDistance.replace(/\s(?:px|m|km)$/, "");
}
