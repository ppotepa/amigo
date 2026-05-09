import type { TabSpec } from "./layoutTypes";

export function TabHost({
  activeTabId,
  className = "",
  onTabChange,
  tabs,
}: {
  tabs: TabSpec[];
  activeTabId?: string;
  onTabChange?: (tabId: string) => void;
  className?: string;
}) {
  const activeId = activeTabId ?? tabs[0]?.id ?? "";

  return (
    <nav className={`workbench-tab-host ${className}`} role="tablist">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          aria-selected={tab.id === activeId}
          className={`workbench-tab ${tab.id === activeId ? "active" : ""}`}
          onClick={() => onTabChange?.(tab.id)}
          title={tab.title}
        >
          {tab.icon ? <span className="workbench-tab-icon">{tab.icon}</span> : null}
          <span>{tab.title}</span>
        </button>
      ))}
    </nav>
  );
}
