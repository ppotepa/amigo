import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import type { ScenePreviewDto } from "../api/dto";

function toImageSrc(url: string): string {
  if (url.startsWith("data:") || url.startsWith("http://") || url.startsWith("https://")) {
    return url;
  }
  return convertFileSrc(url);
}

export function previewImageSrc(url?: string | null): string | null {
  return url ? toImageSrc(url) : null;
}

export function EngineSlideshowPreview({ preview, playing }: { preview: ScenePreviewDto; playing: boolean }) {
  const [frameIndex, setFrameIndex] = useState(0);

  const frameUrls = useMemo(() => preview.frameUrls.map(toImageSrc), [preview.frameUrls]);

  useEffect(() => {
    setFrameIndex(0);
  }, [preview.sourceHash, preview.sceneId]);

  useEffect(() => {
    if (!playing || frameUrls.length <= 1) {
      return;
    }

    const delay = Math.max(1, Math.round(1000 / preview.fps));
    const handle = window.setInterval(() => {
      setFrameIndex((current) => (current + 1) % frameUrls.length);
    }, delay);

    return () => window.clearInterval(handle);
  }, [playing, frameUrls.length, preview.fps]);

  if (frameUrls.length === 0) {
    return (
      <div className="preview-canvas preview-empty">
        <strong>No slideshow frames</strong>
        <span>Regenerate scene preview.</span>
      </div>
    );
  }

  return (
    <div className="preview-canvas preview-ready">
      <img src={frameUrls[frameIndex]} alt={`Scene preview frame ${frameIndex + 1}`} draggable={false} />
    </div>
  );
}
