import type { ReactNode } from "react";
import type {
  EditorComponentContext,
  EditorComponentInstance,
} from "../../editor-components/componentTypes";
import type {
  EditorTargetKind,
  EditorTargetRef,
  ResolvedEditorTarget,
} from "../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import type { SplitSlotContent } from "../layout/layoutTypes";

export type WorkbenchSlotId =
  | "left"
  | "center"
  | "right.top"
  | "right.bottom"
  | "bottom";

export type WorkbenchSlotContent = SplitSlotContent | ReactNode;

export type WorkbenchSlotMap = Partial<Record<WorkbenchSlotId, WorkbenchSlotContent>>;

export type TargetCapabilities = Record<string, boolean | string | number | null>;

export type TargetActionContract = {
  id: string;
  label: string;
  enabled?: boolean;
  visible?: boolean;
  run: () => void | Promise<void>;
};

export type TargetModelInput<TRef extends EditorTargetRef> = {
  context: EditorComponentContext;
  instance: EditorComponentInstance;
  ref: TRef;
  resolved: ResolvedEditorTarget;
  services: WorkspaceRuntimeServices;
};

export type TargetRenderInput<TModel, TActions> = {
  context: EditorComponentContext;
  instance: EditorComponentInstance;
  model: TModel;
  actions: TActions;
  services: WorkspaceRuntimeServices;
};

export type TargetContract<
  TKind extends EditorTargetKind,
  TRef extends EditorTargetRef,
  TModel,
  TSlots extends WorkbenchSlotMap,
  TActions,
  TCapabilities extends TargetCapabilities,
> = {
  id: string;
  kind: TKind;
  buildModel: (input: TargetModelInput<TRef>) => TModel;
  buildActions: (input: TargetModelInput<TRef>, model: TModel) => TActions;
  capabilities: TCapabilities;
  renderSlots: (input: TargetRenderInput<TModel, TActions>) => TSlots;
};

export type AnyTargetContract = TargetContract<any, any, any, any, any, any>;
