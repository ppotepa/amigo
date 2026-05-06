import type { EditorFrameDto } from "../../../../api/dto";

type Props = {
  frame?: EditorFrameDto | null;
};

export function EditorViewportFrameHost({ frame }: Props) {
  if (!frame) {
    return (
      <div className="scene-editor-render-layer scene-editor-render-placeholder">
        <span>No engine frame yet</span>
      </div>
    );
  }

  switch (frame.transport) {
    case "image-url":
      return <EditorImageFrame frame={frame} />;
    case "stream":
      return <EditorStreamFrame frame={frame} />;
    case "native-surface":
      return <EditorNativeSurfaceFrame frame={frame} />;
  }
}

function EditorImageFrame({ frame }: { frame: EditorFrameDto }) {
  if (!frame.imageUrl) {
    return (
      <div className="scene-editor-render-layer scene-editor-render-placeholder">
        <strong>Editor frame missing image URL</strong>
        <span>Image-url transport is active, but backend returned no image.</span>
      </div>
    );
  }

  return (
    <div className="scene-editor-render-layer scene-editor-render-layer-image">
      <img src={frame.imageUrl} alt="Engine editor viewport" draggable={false} />
    </div>
  );
}

function EditorStreamFrame({ frame }: { frame: EditorFrameDto }) {
  return <div className="scene-editor-render-layer" data-stream-id={frame.streamId ?? ""} />;
}

function EditorNativeSurfaceFrame({ frame }: { frame: EditorFrameDto }) {
  return <div className="scene-editor-render-layer" data-surface-id={frame.surfaceId ?? ""} />;
}
