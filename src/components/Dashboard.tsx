interface DashboardProps {
  today?: string;
  week?: string;
  month?: string;
  year?: string;
}

export function Dashboard({ today = "0.00 km", week = "0.00 km", month = "0.00 km", year = "0.00 km" }: DashboardProps) {
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
