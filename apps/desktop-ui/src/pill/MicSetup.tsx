// Mic setup: explicit permission request, device picker, live level meter
// (the plan's onboarding "mic pick with live meter"). getUserMedia is what
// reliably triggers the browser's permission prompt — SpeechRecognition alone
// is silently denied in embedded contexts. The chosen track is handed to
// recognition where the browser supports it (Chrome 136+); otherwise the
// system default mic is used and we say so.

import { useCallback, useEffect, useRef, useState } from "react";

export function MicSetup({
  onReady,
  onCancel,
}: {
  onReady: (stream: MediaStream) => void;
  onCancel: () => void;
}) {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [deviceId, setDeviceId] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [level, setLevel] = useState(0);
  const audioRef = useRef<{ ctx: AudioContext; raf: number } | null>(null);

  const acquire = useCallback(async (id?: string) => {
    setError(null);
    try {
      const s = await navigator.mediaDevices.getUserMedia({
        audio: id ? { deviceId: { exact: id } } : true,
      });
      setStream((old) => {
        old?.getTracks().forEach((t) => t.stop());
        return s;
      });
      // Labels are only populated after permission is granted.
      const all = await navigator.mediaDevices.enumerateDevices();
      setDevices(all.filter((d) => d.kind === "audioinput"));
      const settings = s.getAudioTracks()[0]?.getSettings();
      if (settings?.deviceId) setDeviceId(settings.deviceId);
    } catch (e) {
      const name = e instanceof DOMException ? e.name : "";
      if (name === "NotAllowedError" || name === "SecurityError") {
        setError(
          window.self !== window.top
            ? "The microphone is blocked in this embedded demo — the sandbox never shows the permission prompt. Open the standalone app link (or run it locally) for live voice."
            : "Microphone access was denied. Allow the mic for this site in the address bar, then try again."
        );
      } else if (name === "NotFoundError") {
        setError("No microphone found. Plug one in and try again.");
      } else {
        setError(`Couldn't open the microphone: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
  }, []);

  useEffect(() => {
    void acquire();
    return () => {
      // The chosen stream is handed off via onReady; only stop on abandon.
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Live level meter on the current stream.
  useEffect(() => {
    if (!stream) return;
    const ctx = new AudioContext();
    const analyser = ctx.createAnalyser();
    analyser.fftSize = 512;
    ctx.createMediaStreamSource(stream).connect(analyser);
    const buf = new Uint8Array(analyser.fftSize);
    let raf = 0;
    const loop = () => {
      analyser.getByteTimeDomainData(buf);
      let sum = 0;
      for (const v of buf) {
        const c = (v - 128) / 128;
        sum += c * c;
      }
      setLevel(Math.min(1, Math.sqrt(sum / buf.length) * 4));
      raf = requestAnimationFrame(loop);
    };
    loop();
    audioRef.current = { ctx, raf };
    return () => {
      cancelAnimationFrame(raf);
      void ctx.close();
    };
  }, [stream]);

  const cancel = () => {
    stream?.getTracks().forEach((t) => t.stop());
    onCancel();
  };

  return (
    <div className="micsetup" data-testid="mic-setup">
      <h2>Microphone check</h2>
      {error ? (
        <>
          <p className="micsetup__error">{error}</p>
          <div className="script__actions">
            <button className="btn" onClick={() => void acquire()}>
              Try again
            </button>
            <button className="btn btn--quiet" onClick={cancel}>
              Back
            </button>
          </div>
        </>
      ) : !stream ? (
        <p className="micsetup__hint">Waiting for microphone permission — check the browser prompt…</p>
      ) : (
        <>
          <label className="micsetup__label">
            Microphone
            <select
              className="micsetup__select"
              value={deviceId}
              data-testid="mic-select"
              onChange={(e) => void acquire(e.target.value)}
            >
              {devices.map((d, i) => (
                <option key={d.deviceId || i} value={d.deviceId}>
                  {d.label || `Microphone ${i + 1}`}
                </option>
              ))}
            </select>
          </label>
          <div className="micsetup__meter" aria-label="microphone level">
            <div className="micsetup__meter-fill" style={{ width: `${Math.round(level * 100)}%` }} />
          </div>
          <p className="micsetup__hint">
            Say something — the bar should move. Then start, and read the script at your own pace.
          </p>
          <div className="script__actions">
            <button className="btn btn--primary" data-testid="mic-begin" onClick={() => onReady(stream)}>
              Start prompting
            </button>
            <button className="btn btn--quiet" onClick={cancel}>
              Back
            </button>
          </div>
        </>
      )}
    </div>
  );
}
