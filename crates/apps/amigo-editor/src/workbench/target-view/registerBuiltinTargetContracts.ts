import { assetTargetContract } from "../../features/asset/target";
import { componentTargetContract } from "../../features/component/target";
import { entityTargetContract } from "../../features/entity/target";
import { fileTargetContract } from "../../features/file/target";
import { sceneTargetContract } from "../../features/scene/target";
import { genericTargetContracts } from "./genericTargetContracts";
import { registerTargetContract } from "./targetViewRegistry";

let registered = false;

export function registerBuiltinTargetContracts() {
  if (registered) return;
  registered = true;
  registerTargetContract(sceneTargetContract);
  registerTargetContract(entityTargetContract);
  registerTargetContract(assetTargetContract);
  registerTargetContract(componentTargetContract);
  registerTargetContract(fileTargetContract);
  for (const contract of genericTargetContracts) {
    registerTargetContract(contract);
  }
}

export function resetBuiltinTargetContractsForTests() {
  registered = false;
}
