import { useEffect, useMemo, useState } from "react";
import type { CSSProperties } from "react";
import "./app-splash.css";

const preferredSplashImageUrl = "/splash-desert-night.png";
const splashDarknessMaskUrl = "/splash-desert-night-darkness-mask.png";
const splashLightmapUrl = "/splash-desert-night-lightmap.png";

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

export function AppSplash({ exiting = false }: { exiting?: boolean }) {
  const [imageAvailable, setImageAvailable] = useState(true);
  const [darknessMaskAvailable, setDarknessMaskAvailable] = useState(true);
  const [lightmapAvailable, setLightmapAvailable] = useState(true);
  const [progress, setProgress] = useState(8);
  const [stepIndex, setStepIndex] = useState(0);
  const [visualTime, setVisualTime] = useState(0);

  useEffect(() => {
    const startedAt = window.performance.now();
    const visualTimer = window.setInterval(() => {
      setVisualTime(window.performance.now() - startedAt);
    }, 80);
    const progressTimer = window.setInterval(() => {
      setProgress((current) => Math.min(96, current + 3 + Math.round(Math.random() * 5)));
    }, 150);
    const stepTimer = window.setInterval(() => {
      setStepIndex((current) => (current + 1) % SPLASH_STEPS.length);
    }, 420);

    return () => {
      window.clearInterval(visualTimer);
      window.clearInterval(progressTimer);
      window.clearInterval(stepTimer);
    };
  }, []);

  const status = useMemo(() => SPLASH_STEPS[stepIndex] ?? SPLASH_STEPS[0], [stepIndex]);
  const revealRamp = Math.max(0, Math.min(1, (visualTime - 450) / 1900));
  const lightRamp = Math.max(0, Math.min(1, (visualTime - 850) / 1550));
  const blackoutOpacity = Math.max(0, 0.98 - revealRamp * 0.98);
  const darknessOpacity = Math.max(0.18, 1 - revealRamp * 0.82);
  const exposureOpacity = lightRamp * 0.58;
  const washOpacity = lightRamp * 0.46;

  return (
    <div
      className={`app-splash ${exiting ? "is-exiting" : ""}`}
      role="status"
      aria-live="polite"
      style={{
        "--splash-blackout-opacity": blackoutOpacity,
        "--splash-exposure-opacity": exposureOpacity,
        "--splash-wash-opacity": washOpacity,
      } as CSSProperties}
    >
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
      {darknessMaskAvailable ? (
        <img
          className="app-splash-art app-splash-art-darkness-mask"
          src={splashDarknessMaskUrl}
          alt=""
          width={800}
          height={800}
          style={{ opacity: darknessOpacity }}
          onError={() => setDarknessMaskAvailable(false)}
        />
      ) : null}
      {lightmapAvailable ? (
        <img
          className="app-splash-art app-splash-art-exposure"
          src={splashLightmapUrl}
          alt=""
          width={800}
          height={800}
          onError={() => setLightmapAvailable(false)}
        />
      ) : null}
      <div className="app-splash-light-wash" />
      <div className="app-splash-start-blackout" />
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
