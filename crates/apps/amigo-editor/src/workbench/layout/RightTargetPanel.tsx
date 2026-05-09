import type { WorkbenchSlotContent } from "../target-view/targetViewTypes";
import { SlotHost } from "./SlotHost";
import { SplitPane } from "./SplitPane";

export function RightTargetPanel({
  bottom,
  top,
  title = "Target",
  subtitle,
}: {
  top?: WorkbenchSlotContent;
  bottom?: WorkbenchSlotContent;
  title?: string;
  subtitle?: string;
}) {
  return (
    <section className="workbench-right-target-panel">
      <header className="workbench-right-target-header">
        <strong>{title}</strong>
        {subtitle ? <small>{subtitle}</small> : null}
      </header>
      <SplitPane
        top={top ? <SlotHost>{top}</SlotHost> : null}
        bottom={bottom ? <SlotHost>{bottom}</SlotHost> : null}
      />
    </section>
  );
}
