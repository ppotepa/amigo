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

  return <EditorImageFrame frame={frame} />;
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
