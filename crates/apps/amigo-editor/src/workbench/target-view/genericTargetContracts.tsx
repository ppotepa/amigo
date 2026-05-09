import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type {
  EditorTargetKind,
  EditorTargetRef,
  ResolvedEditorTarget,
} from "../../editor-targets";
import { TargetContextContent } from "../../features/target-context/TargetContextContent";
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
  "projectFile",
  "script",
  "asset",
  "component",
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
  context,
  instance,
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
      <TargetContextContent
        context={context}
        instance={instance}
        services={services}
      />
    </div>
  );
}
