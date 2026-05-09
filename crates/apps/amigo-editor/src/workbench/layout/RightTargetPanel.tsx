import type { WorkbenchSlotContent } from "../target-view/targetViewTypes";
import { SlotHost } from "./SlotHost";
import { SplitPane } from "./SplitPane";

export function RightTargetPanel({
  bottom,
  top,
}: {
  top?: WorkbenchSlotContent;
  bottom?: WorkbenchSlotContent;
}) {
  return (
    <div className="right-target-panel">
      <SplitPane
        top={top ? <SlotHost>{top}</SlotHost> : null}
        bottom={bottom ? <SlotHost>{bottom}</SlotHost> : null}
      />
    </div>
  );
}
