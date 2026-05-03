import { useEffect, useRef, useState } from "react";
import type React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";

export type DockTabDescriptor = {
  id: string;
  title: string;
  icon: React.ReactNode;
};

export function DockAreaHost({
  activeTab,
  children,
  className,
  onSelect,
  tabs,
}: {
  activeTab: string;
  children: React.ReactNode;
  className: string;
  onSelect: (tabId: string) => void;
  tabs: DockTabDescriptor[];
}) {
  const stripRef = useRef<HTMLDivElement | null>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);

  const refreshScrollState = () => {
    const strip = stripRef.current;
    if (!strip) return;
    setCanScrollLeft(strip.scrollLeft > 1);
    setCanScrollRight(strip.scrollLeft + strip.clientWidth < strip.scrollWidth - 1);
  };

  useEffect(() => {
    refreshScrollState();
    const strip = stripRef.current;
    if (!strip) return;
    strip.addEventListener("scroll", refreshScrollState);
    window.addEventListener("resize", refreshScrollState);
    return () => {
      strip.removeEventListener("scroll", refreshScrollState);
      window.removeEventListener("resize", refreshScrollState);
    };
  }, [tabs.length]);

  const scrollTabs = (direction: -1 | 1) => {
    stripRef.current?.scrollBy({ left: direction * 160, behavior: "smooth" });
  };

  return (
    <section className={`workspace-dock dock-area-host ${className}`}>
      <div
        className={`workspace-dock-tabs-shell ${canScrollLeft ? "has-left-arrow" : ""} ${
          canScrollRight ? "has-right-arrow" : ""
        }`}
      >
        {canScrollLeft ? (
          <button
            className="dock-tab-scroll-button left"
            type="button"
            aria-label="Scroll tabs left"
            onClick={() => scrollTabs(-1)}
          >
            <ChevronLeft size={13} />
          </button>
        ) : null}
        <div className="workspace-dock-tabs-scroll" ref={stripRef}>
          <div className="workspace-dock-tabs">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                type="button"
                className={`workspace-dock-tab ${activeTab === tab.id ? "active" : ""}`}
                onClick={() => onSelect(tab.id)}
                title={tab.title}
              >
                {tab.icon}
                <span>{tab.title}</span>
              </button>
            ))}
          </div>
        </div>
        {canScrollRight ? (
          <button
            className="dock-tab-scroll-button right"
            type="button"
            aria-label="Scroll tabs right"
            onClick={() => scrollTabs(1)}
          >
            <ChevronRight size={13} />
          </button>
        ) : null}
      </div>
      <div className="workspace-dock-body">{children}</div>
    </section>
  );
}
