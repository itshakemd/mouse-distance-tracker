interface DashboardProps {
  distance?: string;
  today?: string;
  week?: string;
  month?: string;
  year?: string;
}

export function Dashboard({ distance = "0.00", today = "0.00 km", week = "0.00 km", month = "0.00 km", year = "0.00 km" }: DashboardProps) {
  const stats = [
    ["Today", today],
    ["This Week", week],
    ["This Month", month],
    ["This Year", year],
  ] as const;

  return (
    <main className="app-shell">
      <section className="hero">
        <img className="app-logo" src="/mouse.png" alt="" />
        <p>Distance today</p>
        <div className="distance-value">
          <strong>{distance}</strong>
          <select aria-label="Measurement unit" defaultValue="km">
            <option value="px">px</option>
            <option value="m">m</option>
            <option value="km">km</option>
          </select>
        </div>
      </section>

      <section className="stats" aria-label="Distance statistics">
        {stats.map(([label, value]) => (
          <article key={label}>
            <span>{label}</span>
            <b>{value}</b>
          </article>
        ))}
      </section>
    </main>
  );
}
