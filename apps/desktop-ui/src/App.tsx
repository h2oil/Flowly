import { Pill } from "./pill/Pill";

// Browser demo shell. In the Tauri build the Pill renders alone on a
// transparent always-on-top window; this page stands in for the desktop and
// marks where the webcam sits.
export default function App() {
  return (
    <div className="desktop">
      <div className="camera-hint">webcam</div>
      <Pill />
      <p className="demo-note">
        Click the pill to run the demo take: it reads the launch script, takes a thinking pause
        (<code>PAUSED</code>), wanders off-script (<code>LOST</code> — text holds still, amber),
        re-anchors, then retakes the numbers sentence (jump back). Hover the pill while it runs
        for controls. The cursor feed is scripted; the Windows build drives the same UI from the
        live alignment engine.
      </p>
    </div>
  );
}
