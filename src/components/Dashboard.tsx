import type { DistanceUnit, TrackerSnapshot } from "../model/tracker";

interface DashboardProps {
  snapshot: TrackerSnapshot;
  onUnitChange?: (unit: DistanceUnit) => void;
}

export function Dashboard({ snapshot, onUnitChange }: DashboardProps) {
  const stats = [
    ["Today", snapshot.todayPixels],
    ["This Week", snapshot.weekPixels],
    ["This Month", snapshot.monthPixels],
    ["This Year", snapshot.yearPixels],
  ] as const;

  return (
    <main className="app-shell">
      <section className="hero">
        <img className="app-logo" src="/mouse.png" alt="" />
        <p>Distance today</p>
        <div className="distance-value">
          <strong>{snapshot.todayPixels}</strong>
          <select
            aria-label="Measurement unit"
            value={snapshot.settings.unit}
            onChange={(e) => onUnitChange?.(e.target.value as DistanceUnit)}
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
            <b>{pixels}</b>
          </article>
        ))}
      </section>
    </main>
  );
}
