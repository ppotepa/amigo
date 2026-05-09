import { useMemo, useState } from "react";
import type { EditorComponentDescriptorDto } from "../../../metadata/editorMetadataTypes";
import { componentDefaultYaml, componentTypeName } from "../../../metadata/editorMetadataTypes";
import {
  canAttachDescriptorToScene,
  filterSceneComponentDescriptors,
  groupSceneComponentDescriptors,
} from "./sceneComponentPickerModel";

export type SceneComponentPickerProps = {
  open: boolean;
  components: EditorComponentDescriptorDto[];
  onClose: () => void;
  onPick: (descriptor: EditorComponentDescriptorDto) => void;
};

export function SceneComponentPicker({
  open,
  components,
  onClose,
  onPick,
}: SceneComponentPickerProps) {
  const [query, setQuery] = useState("");
  const groups = useMemo(
    () => groupSceneComponentDescriptors(filterSceneComponentDescriptors(components, query)),
    [components, query],
  );

  if (!open) return null;

  return (
    <div className="context-picker" role="dialog" aria-label="Add scene component">
      <header className="context-picker-header">
        <strong>Add Scene Component</strong>
        <button type="button" className="context-icon-button" title="Close" onClick={onClose}>
          x
        </button>
      </header>
      <input
        className="context-picker-search"
        type="search"
        value={query}
        placeholder="Filter components"
        onChange={(event) => setQuery(event.currentTarget.value)}
      />
      <div className="context-picker-list">
        {groups.length ? groups.map((group) => (
          <section key={group.id} className="context-picker-group">
            <h4>{group.label}</h4>
            {group.descriptors.map((descriptor) => {
              const attach = canAttachDescriptorToScene(descriptor);
              return (
                <button
                  key={descriptor.kind}
                  type="button"
                  className="context-picker-item"
                  disabled={!attach.allowed}
                  title={attach.reason ?? descriptor.label}
                  onClick={() => onPick(descriptor)}
                >
                  <span>{descriptor.label}</span>
                  <small>{attach.reason ?? componentTypeName(descriptor)}</small>
                  <small className="muted">
                    {componentDefaultYaml(descriptor) ?? `type: ${componentTypeName(descriptor)}`}
                  </small>
                </button>
              );
            })}
          </section>
        )) : (
          <p className="muted workspace-note">No matching components.</p>
        )}
      </div>
    </div>
  );
}
