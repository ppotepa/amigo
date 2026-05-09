import type { AnyTargetContract } from "./targetViewTypes";

const TARGET_CONTRACTS: AnyTargetContract[] = [];

export function registerTargetContract(contract: AnyTargetContract) {
  const existingIndex = TARGET_CONTRACTS.findIndex((candidate) => candidate.id === contract.id);
  if (existingIndex >= 0) {
    TARGET_CONTRACTS[existingIndex] = contract;
    return;
  }

  TARGET_CONTRACTS.push(contract);
}

export function allTargetContracts() {
  return [...TARGET_CONTRACTS];
}

export function clearTargetContractsForTests() {
  TARGET_CONTRACTS.length = 0;
}
