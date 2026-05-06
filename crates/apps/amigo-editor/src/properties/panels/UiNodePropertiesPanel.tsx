import { useEffect, useMemo, useState } from "react";
import { MousePointer2, RotateCcw, Save } from "lucide-react";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorUiNodeEditablePropertyDto,
  EditorUiNodePropertyValueDto,
} from "../../api/dto";
import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { PropertiesContext, UiNodeSelection } from "../propertiesTypes";

type UiNodeDraft = {
  id: string;
  text: string;
  styleClass: string;
  visible: boolean;
  enabled: boolean;
  left: string;
  top: string;
  width: string;
  height: string;
  fontSize: string;
  color: string;
  background: string;
  borderColor: string;
  borderWidth: string;
  borderRadius: string;
  padding: string;
  gap: string;
};

type UiNodeDiff = {
  propertyPath: EditorUiNodeEditablePropertyDto;
  value: EditorUiNodePropertyValueDto;
};

export function UiNodePropertiesPanel({
  context,
  selection,
}: {
  context: PropertiesContext;
  selection: UiNodeSelection;
}) {
  const { node, entity, nodeRef, scene } = selection;
  const [draft, setDraft] = useState<UiNodeDraft>(() => draftFromSelection(selection));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setDraft(draftFromSelection(selection));
    setError(null);
  }, [selection]);

  const dirtyFields = useMemo(() => diffDraft(selection, draft), [selection, draft]);
  const dirty = dirtyFields.length > 0;
  const editable = Boolean(context.onApplyEditorCommand && scene?.id);

  async function commit() {
    if (!context.onApplyEditorCommand || !scene?.id || dirtyFields.length === 0) {
      return;
    }

    setBusy(true);
    setError(null);

    try {
      for (const field of dirtyFields) {
        const command: EditorCommandDto = {
          type: "SetUiNodeProperty",
          sceneId: scene.id,
          entityId: nodeRef.entityId,
          componentIndex: nodeRef.componentIndex,
          nodePath: nodeRef.nodePath,
          propertyPath: field.propertyPath,
          value: field.value,
        };

        const result: EditorCommandResultDto | null = await context.onApplyEditorCommand(command);
        if (result && !result.ok) {
          throw new Error(result.message ?? result.diagnostics.map((diagnostic) => diagnostic.message).join("\n"));
        }
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  function reset() {
    setDraft(draftFromSelection(selection));
    setError(null);
  }

  return (
    <>
      <KeyValueSection
        title="UI Node"
        rows={[
          { label: "ID", value: node.id },
          { label: "Label", value: node.label },
          { label: "Type", value: node.kind },
          { label: "Path", value: node.path, title: node.path },
          { label: "Scene", value: scene?.label ?? "none", title: scene?.id },
          { label: "Entity", value: entity.name, title: entity.id },
          { label: "Component", value: `UiDocument #${nodeRef.componentIndex}` },
        ]}
      />

      <section className="workspace-section">
        <h3>Editable Content</h3>
        <Field label="ID">
          <input className="property-input" value={draft.id} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, id: event.target.value }))} />
        </Field>
        <Field label="Text">
          <input className="property-input" value={draft.text} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, text: event.target.value }))} />
        </Field>
        <Field label="Style class">
          <input className="property-input" value={draft.styleClass} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, styleClass: event.target.value }))} />
        </Field>
        <Field label="Visible">
          <input type="checkbox" checked={draft.visible} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, visible: event.target.checked }))} />
        </Field>
        <Field label="Enabled">
          <input type="checkbox" checked={draft.enabled} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, enabled: event.target.checked }))} />
        </Field>
      </section>

      <section className="workspace-section">
        <h3>Style</h3>
        <Field label="Left"><NumberInput value={draft.left} disabled={!editable || busy} onChange={(left) => setDraft((current) => ({ ...current, left }))} /></Field>
        <Field label="Top"><NumberInput value={draft.top} disabled={!editable || busy} onChange={(top) => setDraft((current) => ({ ...current, top }))} /></Field>
        <Field label="Width"><NumberInput value={draft.width} disabled={!editable || busy} onChange={(width) => setDraft((current) => ({ ...current, width }))} /></Field>
        <Field label="Height"><NumberInput value={draft.height} disabled={!editable || busy} onChange={(height) => setDraft((current) => ({ ...current, height }))} /></Field>
        <Field label="Font size"><NumberInput value={draft.fontSize} disabled={!editable || busy} onChange={(fontSize) => setDraft((current) => ({ ...current, fontSize }))} /></Field>
        <Field label="Color">
          <input className="property-input property-input-mono" value={draft.color} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, color: event.target.value }))} />
        </Field>
        <Field label="Background">
          <input className="property-input property-input-mono" value={draft.background} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, background: event.target.value }))} />
        </Field>
        <Field label="Border color">
          <input className="property-input property-input-mono" value={draft.borderColor} disabled={!editable || busy} onChange={(event) => setDraft((current) => ({ ...current, borderColor: event.target.value }))} />
        </Field>
        <Field label="Border width"><NumberInput value={draft.borderWidth} disabled={!editable || busy} onChange={(borderWidth) => setDraft((current) => ({ ...current, borderWidth }))} /></Field>
        <Field label="Border radius"><NumberInput value={draft.borderRadius} disabled={!editable || busy} onChange={(borderRadius) => setDraft((current) => ({ ...current, borderRadius }))} /></Field>
        <Field label="Padding"><NumberInput value={draft.padding} disabled={!editable || busy} onChange={(padding) => setDraft((current) => ({ ...current, padding }))} /></Field>
        <Field label="Gap"><NumberInput value={draft.gap} disabled={!editable || busy} onChange={(gap) => setDraft((current) => ({ ...current, gap }))} /></Field>
      </section>

      <KeyValueSection
        title="Action"
        rows={[{ label: "on_click", value: node.actionEvent ?? "none", title: node.actionEvent ?? undefined }]}
      />

      {error ? (
        <section className="workspace-section">
          <p className="muted workspace-note warning-text">{error}</p>
        </section>
      ) : null}

      <section className="workspace-section">
        <h3>Editor</h3>
        <div className="workspace-row">
          <span className="dock-icon dock-icon-blue"><MousePointer2 size={14} /></span>
          <span>
            <strong>Canvas selection</strong>
            <small>Layout editing comes later. This step edits node fields through Inspector commands.</small>
          </span>
          <em className="badge badge-muted">{editable ? "editable" : "read-only"}</em>
        </div>
        <div className="property-actions">
          <button className="button button-ghost" type="button" disabled={!dirty || busy} onClick={reset}>
            <RotateCcw size={14} /> Reset
          </button>
          <button className="button button-primary" type="button" disabled={!dirty || busy || !editable} onClick={() => void commit()}>
            <Save size={14} /> {busy ? "Applying..." : "Apply"}
          </button>
        </div>
      </section>
    </>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="property-field">
      <span>{label}</span>
      {children}
    </label>
  );
}

function NumberInput({ disabled, onChange, value }: { disabled: boolean; onChange: (value: string) => void; value: string }) {
  return <input className="property-input property-input-number" type="number" value={value} disabled={disabled} onChange={(event) => onChange(event.target.value)} />;
}

function draftFromSelection(selection: UiNodeSelection): UiNodeDraft {
  const style = selection.node.style ?? {};
  return {
    id: selection.node.id ?? "",
    text: selection.node.text ?? "",
    styleClass: selection.node.styleClass ?? "",
    visible: selection.node.visible,
    enabled: selection.node.enabled,
    left: numberDraft(style.left),
    top: numberDraft(style.top),
    width: numberDraft(style.width),
    height: numberDraft(style.height),
    fontSize: numberDraft(style.fontSize),
    color: style.color ?? "",
    background: style.background ?? "",
    borderColor: style.borderColor ?? "",
    borderWidth: numberDraft(style.borderWidth),
    borderRadius: numberDraft(style.borderRadius),
    padding: numberDraft(style.padding),
    gap: numberDraft(style.gap),
  };
}

function numberDraft(value: number | null | undefined): string {
  return typeof value === "number" && Number.isFinite(value) ? String(value) : "";
}

function diffDraft(selection: UiNodeSelection, draft: UiNodeDraft): UiNodeDiff[] {
  const original = draftFromSelection(selection);
  const diffs: UiNodeDiff[] = [];
  pushStringDiff(diffs, original.id, draft.id, "id");
  pushStringDiff(diffs, original.text, draft.text, "text");
  pushStringDiff(diffs, original.styleClass, draft.styleClass, "style_class");
  pushBoolDiff(diffs, original.visible, draft.visible, "visible");
  pushBoolDiff(diffs, original.enabled, draft.enabled, "enabled");
  pushNumberDiff(diffs, original.left, draft.left, "style.left");
  pushNumberDiff(diffs, original.top, draft.top, "style.top");
  pushNumberDiff(diffs, original.width, draft.width, "style.width");
  pushNumberDiff(diffs, original.height, draft.height, "style.height");
  pushNumberDiff(diffs, original.fontSize, draft.fontSize, "style.font_size");
  pushStringDiff(diffs, original.color, draft.color, "style.color");
  pushStringDiff(diffs, original.background, draft.background, "style.background");
  pushStringDiff(diffs, original.borderColor, draft.borderColor, "style.border_color");
  pushNumberDiff(diffs, original.borderWidth, draft.borderWidth, "style.border_width");
  pushNumberDiff(diffs, original.borderRadius, draft.borderRadius, "style.border_radius");
  pushNumberDiff(diffs, original.padding, draft.padding, "style.padding");
  pushNumberDiff(diffs, original.gap, draft.gap, "style.gap");
  return diffs;
}

function pushStringDiff(diffs: UiNodeDiff[], before: string, after: string, propertyPath: EditorUiNodeEditablePropertyDto) {
  if (before === after) return;
  diffs.push({ propertyPath, value: after.trim() ? { kind: "string", value: after } : { kind: "null" } });
}

function pushNumberDiff(diffs: UiNodeDiff[], before: string, after: string, propertyPath: EditorUiNodeEditablePropertyDto) {
  if (before === after) return;
  if (!after.trim()) {
    diffs.push({ propertyPath, value: { kind: "null" } });
    return;
  }
  const parsed = Number(after);
  if (Number.isFinite(parsed)) {
    diffs.push({ propertyPath, value: { kind: "number", value: parsed } });
  }
}

function pushBoolDiff(diffs: UiNodeDiff[], before: boolean, after: boolean, propertyPath: EditorUiNodeEditablePropertyDto) {
  if (before !== after) {
    diffs.push({ propertyPath, value: { kind: "bool", value: after } });
  }
}
