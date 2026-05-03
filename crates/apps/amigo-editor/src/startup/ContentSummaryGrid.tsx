import type { EditorContentSummaryDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";

const SUMMARY_FIELDS: Array<[keyof EditorContentSummaryDto, string]> = [
  ["scenes", "Scenes"],
  ["sceneYaml", "Scene YAML"],
  ["scripts", "Scripts"],
  ["textures", "Textures"],
  ["spritesheets", "Spritesheets"],
  ["audio", "Audio"],
  ["fonts", "Fonts"],
  ["tilemaps", "Tilemaps"],
  ["tilesets", "Tilesets"],
  ["packages", "Packages"],
  ["unknownFiles", "Unknown"],
];

export function ContentSummaryGrid({ summary }: { summary: EditorContentSummaryDto }) {
  const { state, setContentFilter } = useEditorStore();

  return (
    <div className="content-breakdown-list">
      {SUMMARY_FIELDS.map(([key, label]) => {
        const active = state.contentFilter === key;
        return (
          <button key={key} type="button" className={`content-breakdown-row interactive ${active ? "active" : ""}`} onClick={() => setContentFilter(active ? null : key)}>
            <span>{label}</span>
            <strong>{summary[key]}</strong>
          </button>
        );
      })}
    </div>
  );
}
