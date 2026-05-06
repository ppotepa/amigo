import { Bug } from "lucide-react";
import { createContext, useContext, useEffect, useState } from "react";
import type { PropsWithChildren, ReactNode } from "react";
import "./debug-source.css";

const DEBUG_SOURCE_STORAGE_KEY = "amigo-editor:show-debug-sources";
const DebugSourceContext = createContext<boolean | undefined>(undefined);

export function debugSourceEnabledByDefault(): boolean {
  return import.meta.env.DEV;
}

export function readDebugSourcePreference(): boolean {
  if (!debugSourceEnabledByDefault()) return false;
  try {
    const value = window.localStorage.getItem(DEBUG_SOURCE_STORAGE_KEY);
    if (value === null) return true;
    return value === "true";
  } catch {
    return true;
  }
}

export function writeDebugSourcePreference(value: boolean) {
  try {
    window.localStorage.setItem(DEBUG_SOURCE_STORAGE_KEY, String(value));
  } catch {
    // Ignore storage write failures and keep the in-memory state.
  }
}

export function useDebugSourceToggle() {
  const [showDebugSources, setShowDebugSources] = useState<boolean>(() => readDebugSourcePreference());

  useEffect(() => {
    writeDebugSourcePreference(showDebugSources);
  }, [showDebugSources]);

  useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key !== DEBUG_SOURCE_STORAGE_KEY) return;
      setShowDebugSources(readDebugSourcePreference());
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, []);

  return {
    debugMode: debugSourceEnabledByDefault(),
    showDebugSources,
    setShowDebugSources,
  };
}

export function DebugSourceProvider({
  children,
  value,
}: PropsWithChildren<{ value: boolean }>) {
  return (
    <DebugSourceContext.Provider value={value}>
      {children}
    </DebugSourceContext.Provider>
  );
}

export function useDebugSourceEnabled(): boolean {
  const value = useContext(DebugSourceContext);
  return value ?? readDebugSourcePreference();
}

export function DebugSourceToggleButton({
  showDebugSources,
  onToggle,
}: {
  showDebugSources: boolean;
  onToggle: () => void;
}) {
  if (!debugSourceEnabledByDefault()) return null;
  return (
    <button
      className={`titlebar-action-button titlebar-toggle-button ${showDebugSources ? "pressed" : ""}`}
      type="button"
      aria-pressed={showDebugSources}
      title={showDebugSources ? "Hide component source labels" : "Show component source labels"}
      onClick={onToggle}
    >
      <Bug size={15} />
      Source
    </button>
  );
}

export function DebugSourceOverlay({
  children,
  enabled,
  source,
  className,
  contentClassName,
}: PropsWithChildren<{
  enabled: boolean;
  source: string;
  className?: string;
  contentClassName?: string;
}>) {
  const overlay = enabled && source ? (
    <div className="workspace-component-debug-overlay">
      <code>{source}</code>
    </div>
  ) : null;

  const title = enabled && source ? "Ctrl+Shift+Click to copy source path" : undefined;

  return (
    <section
      className={className ?? "workspace-component-shell"}
      title={title}
      onClickCapture={(event) => {
        if (!enabled || !source) return;
        if (!event.ctrlKey || !event.shiftKey) return;
        event.preventDefault();
        event.stopPropagation();
        void navigator.clipboard.writeText(source);
      }}
    >
      <div className={contentClassName ?? "workspace-component-body"}>
        {children}
      </div>
      {overlay}
    </section>
  );
}

export function maybeWrapWithDebugOverlay(node: ReactNode, enabled: boolean, source: string) {
  return (
    <DebugSourceOverlay enabled={enabled} source={source}>
      {node}
    </DebugSourceOverlay>
  );
}
