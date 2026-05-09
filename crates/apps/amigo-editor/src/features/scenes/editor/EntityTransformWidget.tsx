import { useEffect, useState, type ReactNode } from "react";
import { Move3D, RotateCcw } from "lucide-react";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorModeSessionDto,
  EditorSceneObjectDto,
  EditorTransform2Dto,
} from "../../../api/dto";
import { WidgetFrame } from "../../../workbench/widgets/WidgetFrame";
import { sceneContextIcon } from "../context/sceneContextIcons";

export function EntityTransformWidget({
  editorModeSession,
  object,
  sceneId,
  onApplyCommand,
}: {
  sceneId: string;
  editorModeSession?: EditorModeSessionDto | null;
  object: EditorSceneObjectDto | null;
  onApplyCommand?: (command: EditorCommandDto) => Promise<EditorCommandResultDto | null>;
}) {
  const [draft, setDraft] = useState<EditorTransform2Dto | null>(object?.transform2 ?? null);

  useEffect(() => {
    setDraft(object?.transform2 ?? null);
  }, [object?.entityId, object?.transform2]);

  if (!object) {
    return (
      <WidgetFrame
        id="entity-transform"
        title="Transform"
        icon={<Move3D size={14} />}
        badge="none"
        badgeTone="muted"
        defaultCollapsed
      >
        <p className="muted workspace-note">Select an entity to edit its transform.</p>
      </WidgetFrame>
    );
  }

  if (!draft) {
    return (
      <WidgetFrame
        id="entity-transform"
        title="Transform"
        icon={<Move3D size={14} />}
        badge="unsupported"
        badgeTone="warning"
      >
        <p className="muted workspace-note">Selected entity has no editable 2D transform.</p>
      </WidgetFrame>
    );
  }

  if (editorModeSession) {
    return (
      <WidgetFrame
        id="entity-transform"
        title="Transform"
        icon={sceneContextIcon("entity-motion")}
        badge="session"
        badgeTone={editorModeSession.dirty ? "warning" : "info"}
      >
        <p className="muted workspace-note">
          Transform edits are owned by the active editor mode session. Use the canvas tools and
          session save/discard controls instead of direct YAML commands.
        </p>
      </WidgetFrame>
    );
  }

  const editableObject = object;

  function updateField(field: keyof EditorTransform2Dto, value: number) {
    setDraft((current) => current ? { ...current, [field]: value } : current);
  }

  async function commit() {
    if (!draft) return;
    const result = await onApplyCommand?.({
      type: "SetEntityTransform2D",
      sceneId,
      entityId: editableObject.entityId,
      transform: draft,
    });
    if (result?.ok && result.snapshot) {
      const nextObject = result.snapshot.objects.find((candidate) =>
        candidate.entityId === editableObject.entityId
      );
      setDraft(nextObject?.transform2 ?? draft);
    }
  }

  function reset() {
    setDraft({
      x: 0,
      y: 0,
      rotation: 0,
      scaleX: 1,
      scaleY: 1,
      zIndex: draft?.zIndex,
    });
  }

  return (
    <WidgetFrame
      id="entity-transform"
      title="Transform"
      icon={sceneContextIcon("entity-motion")}
      badge="2D"
      badgeTone="info"
    >
      <div className="entity-transform-widget">
        <FieldRow label="Position">
          <NumberField label="X" value={draft.x} onChange={(value) => updateField("x", value)} />
          <NumberField label="Y" value={draft.y} onChange={(value) => updateField("y", value)} />
        </FieldRow>
        <FieldRow label="Rotation">
          <NumberField label="deg" value={radiansToDegrees(draft.rotation)} onChange={(value) => updateField("rotation", degreesToRadians(value))} />
        </FieldRow>
        <FieldRow label="Scale">
          <NumberField label="X" value={draft.scaleX} step={0.1} onChange={(value) => updateField("scaleX", value)} />
          <NumberField label="Y" value={draft.scaleY} step={0.1} onChange={(value) => updateField("scaleY", value)} />
        </FieldRow>
        <div className="entity-transform-actions">
          <button className="button button-tool" type="button" onClick={reset}>
            <RotateCcw size={13} />
            Reset
          </button>
          <button className="button button-primary" type="button" onClick={() => void commit()}>
            Apply
          </button>
        </div>
      </div>
    </WidgetFrame>
  );
}

function FieldRow({
  children,
  label,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <label className="entity-transform-row">
      <span>{label}</span>
      <div>{children}</div>
    </label>
  );
}

function NumberField({
  label,
  onChange,
  step = 1,
  value,
}: {
  label: string;
  value: number;
  step?: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="entity-transform-number">
      <span>{label}</span>
      <input
        type="number"
        value={Number.isFinite(value) ? roundForInput(value) : 0}
        step={step}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function radiansToDegrees(value: number): number {
  return value * 180 / Math.PI;
}

function degreesToRadians(value: number): number {
  return value * Math.PI / 180;
}

function roundForInput(value: number): number {
  return Math.round(value * 1000) / 1000;
}
