// App — composes the instrument cluster inside the full-viewport,
// non-scrolling CSS grid (layout defined in styles.css, ported from mockup.html).

import { useClusterSnapshot } from "./state/clusterStore";
import { Hud } from "./components/Hud";
import { GaugeCluster } from "./components/GaugeCluster";
import { SessionLanes } from "./components/SessionLanes";
import { Trace } from "./components/Trace";
import { Feed } from "./components/Feed";
import { Odometers } from "./components/Odometers";

function App() {
  const snapshot = useClusterSnapshot();
  const busySessionCount = snapshot.sessions.filter((s) => s.status === "busy").length;

  return (
    <div className="cluster">
      <Hud
        host={snapshot.host}
        busySessionCount={busySessionCount}
        totalSessionCount={snapshot.sessions.length}
        telltales={snapshot.telltales}
      />
      <GaugeCluster gauges={snapshot.gauges} />
      <SessionLanes sessions={snapshot.sessions} />
      <Trace trace={snapshot.trace} />
      <Feed recentEvents={snapshot.recentEvents} />
      <Odometers odometers={snapshot.odometers} />
    </div>
  );
}

export default App;
