import { useEffect, useMemo, useState } from "react";
import "./app-splash.css";

const preferredSplashImageUrl = "/splash-desert-night.png";
const splashLightOverlayUrl = "/splash-desert-night-inv.png";

const SPLASH_STEPS = [
  {
    title: "Initializing engine...",
    detail: "Loading renderer, assets, and scripting modules",
  },
  {
    title: "Loading renderer backend...",
    detail: "Preparing 2D and 3D runtime systems",
  },
  {
    title: "Parsing YAML scenes...",
    detail: "Scanning available mods and scene manifests",
  },
  {
    title: "Preparing scripting runtime...",
    detail: "Compiling script bindings and engine contracts",
  },
  {
    title: "Building render graph...",
    detail: "Starting Amigo runtime",
  },
];

export function AppSplash() {
  const [imageAvailable, setImageAvailable] = useState(true);
  const [overlayAvailable, setOverlayAvailable] = useState(true);
  const [progress, setProgress] = useState(8);
  const [stepIndex, setStepIndex] = useState(0);

  useEffect(() => {
    const progressTimer = window.setInterval(() => {
      setProgress((current) => Math.min(96, current + 7 + Math.round(Math.random() * 8)));
    }, 120);
    const stepTimer = window.setInterval(() => {
      setStepIndex((current) => (current + 1) % SPLASH_STEPS.length);
    }, 260);

    return () => {
      window.clearInterval(progressTimer);
      window.clearInterval(stepTimer);
    };
  }, []);

  const status = useMemo(() => SPLASH_STEPS[stepIndex] ?? SPLASH_STEPS[0], [stepIndex]);

  return (
    <div className="app-splash" role="status" aria-live="polite">
      {imageAvailable ? (
        <img
          className="app-splash-art"
          src={preferredSplashImageUrl}
          alt=""
          width={800}
          height={800}
          onError={() => setImageAvailable(false)}
        />
      ) : null}
      {overlayAvailable ? (
        <img
          className="app-splash-art app-splash-art-lights"
          src={splashLightOverlayUrl}
          alt=""
          width={800}
          height={800}
          style={{ opacity: 1 - progress / 100 }}
          onError={() => setOverlayAvailable(false)}
        />
      ) : null}
      <div className="app-splash-vignette" />
      <section className="app-splash-copy">
        <h1>AMIGO</h1>
        <strong>2D / 3D ENGINE</strong>
        <span>v0.1.0-alpha</span>
        <div className="app-splash-status">
          <p><i aria-hidden="true" />{status.title}</p>
          <small>{status.detail}</small>
        </div>
      </section>
      <div className="app-splash-loading">
        <strong>{progress}%</strong>
        <div className="app-splash-track">
          <span style={{ width: `${progress}%` }} />
        </div>
      </div>
    </div>
  );
}
