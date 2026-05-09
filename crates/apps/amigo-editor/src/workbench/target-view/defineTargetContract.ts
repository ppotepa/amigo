import type {
  EditorTargetKind,
  EditorTargetRef,
} from "../../editor-targets";
import type {
  TargetCapabilities,
  TargetContract,
  WorkbenchSlotMap,
} from "./targetViewTypes";

export function defineTargetContract<
  TKind extends EditorTargetKind,
  TRef extends EditorTargetRef,
  TModel,
  TSlots extends WorkbenchSlotMap,
  TActions,
  TCapabilities extends TargetCapabilities,
>(
  contract: TargetContract<TKind, TRef, TModel, TSlots, TActions, TCapabilities>,
) {
  return contract;
}
