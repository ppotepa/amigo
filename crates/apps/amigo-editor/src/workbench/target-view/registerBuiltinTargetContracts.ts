import { entityTargetContract } from "../../features/entity/target";
import { sceneTargetContract } from "../../features/scene/target";
import { genericTargetContracts } from "./genericTargetContracts";
import { registerTargetContract } from "./targetViewRegistry";

let registered = false;

export function registerBuiltinTargetContracts() {
  if (registered) return;
  registered = true;
  registerTargetContract(sceneTargetContract);
  registerTargetContract(entityTargetContract);
  for (const contract of genericTargetContracts) {
    registerTargetContract(contract);
  }
}

export function resetBuiltinTargetContractsForTests() {
  registered = false;
}
