import type { ResolvedEditorTarget } from "../../editor-targets";
import { allTargetContracts } from "./targetViewRegistry";

export function resolveTargetContract(target: ResolvedEditorTarget) {
  return allTargetContracts().find((contract) => contract.kind === target.ref.kind) ?? null;
}
