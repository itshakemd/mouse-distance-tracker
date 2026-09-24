import { Pause, Play } from "lucide-react";
import type { DistanceUnit, TrackerSnapshot } from "../model/tracker";
import { distanceValue, formatDistance } from "../utils/distance";

interface DashboardProps {
  snapshot: TrackerSnapshot;
  onToggleTracking: () => Promise<void>;
  onUnitChange: (unit: DistanceUnit) => Promise<void>;
}

export function Dashboard({
  snapshot,
  onToggleTracking,
  onUnitChange,
}: DashboardProps) {
  const stats = [
    ["Today", snapshot.todayPixels],
    ["This Week", snapshot.weekPixels],
    ["This Month", snapshot.monthPixels],
    ["This Year", snapshot.yearPixels],
  ] as const;

  const today = formatDistance(snapshot.todayPixels, snapshot.settings);

  return (
    <main className="app-shell">
      <section className="hero">
        <img className="app-logo" src="/mouse.png" alt="" />
        <p>Distance today</p>
        <div className="distance-value">
          <strong>{distanceValue(today)}</strong>
          <select
            aria-label="Measurement unit"
            value={snapshot.settings.unit}
            onChange={(event) =>
              void onUnitChange(event.target.value as DistanceUnit)
            }
          >
            <option value="px">px</option>
            <option value="m">m</option>
            <option value="km">km</option>
          </select>
        </div>
      </section>

      <section className="stats" aria-label="Distance statistics">
        {stats.map(([label, pixels]) => (
          <article key={label}>
            <span>{label}</span>
            <b>{formatDistance(pixels, snapshot.settings)}</b>
          </article>
        ))}
      </section>

      <section className="controls">
        <button
          className="primary"
          type="button"
          onClick={() => void onToggleTracking()}
        >
          {snapshot.paused ? <Play aria-hidden="true" /> : <Pause aria-hidden="true" />}
          {snapshot.paused ? "Resume tracking" : "Pause tracking"}
        </button>
      </section>
    </main>
  );
}
