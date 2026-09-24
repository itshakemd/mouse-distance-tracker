import "./App.css";
import { Dashboard } from "./components/Dashboard";
import { useTracker } from "./hooks/useTracker";

export default function App() {
  const tracker = useTracker();

  return (
    <Dashboard
      snapshot={tracker.snapshot}
      onUnitChange={tracker.changeUnit}
    />
  );
}
