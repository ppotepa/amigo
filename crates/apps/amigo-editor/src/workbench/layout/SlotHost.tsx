import { isSplitSlotContent } from "./layoutTypes";
import { TabHost } from "./TabHost";
import type { WorkbenchSlotContent } from "../target-view/targetViewTypes";

export function SlotHost({
  children,
  className = "",
}: {
  children?: WorkbenchSlotContent;
  className?: string;
}) {
  if (!children) return <section className={`workbench-slot-host ${className}`} />;

  if (isSplitSlotContent(children)) {
    const activeTabId = children.activeTabId ?? children.tabs[0]?.id ?? "";
    const activeTab = children.tabs.find((tab) => tab.id === activeTabId) ?? children.tabs[0];

    return (
      <section className={`workbench-slot-host ${className}`}>
        {children.title || children.subtitle ? (
          <header className="workbench-slot-header">
            {children.title ? <strong>{children.title}</strong> : null}
            {children.subtitle ? <small>{children.subtitle}</small> : null}
          </header>
        ) : null}
        <TabHost
          tabs={children.tabs}
          activeTabId={activeTabId}
          onTabChange={children.onTabChange}
        />
        <div className="workbench-slot-content">{activeTab?.content}</div>
      </section>
    );
  }

  return <section className={`workbench-slot-host ${className}`}>{children}</section>;
}
