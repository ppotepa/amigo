import { RightTargetPanel } from "../layout/RightTargetPanel";
import type { WorkbenchSlotMap } from "./targetViewTypes";

export function WorkbenchSlotContent({ slots }: { slots: WorkbenchSlotMap }) {
  return (
    <RightTargetPanel
      top={slots["right.top"]}
      bottom={slots["right.bottom"]}
    />
  );
}
