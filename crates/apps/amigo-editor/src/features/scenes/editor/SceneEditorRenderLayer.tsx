import type { EditorFrameDto } from "../../../api/dto";
import { EditorViewportFrameHost } from "./viewport/EditorViewportFrameHost";

type Props = {
  frame?: EditorFrameDto | null;
};

export function SceneEditorRenderLayer({ frame }: Props) {
  return <EditorViewportFrameHost frame={frame} />;
}
