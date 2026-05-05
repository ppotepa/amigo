import type {
  EditorLiveSceneSessionDto,
  EditorSceneSnapshotDto,
} from "../../../../api/dto";

export type LiveEditorState = {
  session: EditorLiveSceneSessionDto | null;
  snapshot: EditorSceneSnapshotDto | null;
  opening: boolean;
  saving: boolean;
  error: string | null;
};

export const EMPTY_LIVE_EDITOR_STATE: LiveEditorState = {
  session: null,
  snapshot: null,
  opening: false,
  saving: false,
  error: null,
};
