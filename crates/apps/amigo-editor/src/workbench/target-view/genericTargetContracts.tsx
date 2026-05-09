import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type {
  EditorTargetKind,
  EditorTargetRef,
  ResolvedEditorTarget,
} from "../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { defineTargetContract } from "./defineTargetContract";
import type {
  AnyTargetContract,
  TargetModelInput,
} from "./targetViewTypes";

type GenericTargetModel = {
  target: ResolvedEditorTarget;
};

const GENERIC_TARGET_KINDS: EditorTargetKind[] = [
  "mod",
  "projectNode",
  "script",
  "uiDocument",
  "uiNode",
  "diagnostic",
  "capability",
  "dependency",
];

export const genericTargetContracts = GENERIC_TARGET_KINDS.map((kind) =>
  defineGenericTargetContract(kind),
);

function defineGenericTargetContract(kind: EditorTargetKind): AnyTargetContract {
  return defineTargetContract({
    id: kind,
    kind,
    buildModel: (input: TargetModelInput<EditorTargetRef>): GenericTargetModel => ({
      target: input.resolved,
    }),
    buildActions: () => ({}),
    capabilities: {},
    renderSlots: ({ context, instance, model, services }) => ({
      "right.top": (
        <GenericTargetPanel
          context={context}
          instance={instance}
          services={services}
          target={model.target}
        />
      ),
    }),
  });
}

function GenericTargetPanel({
  services,
  target,
}: EditorComponentProps<WorkspaceRuntimeServices> & {
  target: ResolvedEditorTarget;
}) {
  return (
    <div className="workbench-generic-target-panel">
      <header className="workbench-generic-target-header">
        <strong>{target.descriptor.label}</strong>
        <small>{target.ref.kind}</small>
      </header>
      <p className="muted workspace-empty">No dedicated target content yet.</p>
      {services.currentEditorTarget?.descriptor.canInspect ? (
        <p className="muted workspace-note">Runtime services are connected.</p>
      ) : null}
    </div>
  );
}
