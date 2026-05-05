import type { WindowBusEvent } from "../../app/windowBusTypes";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { formatWindowEventPayload, windowEventMatchesCategory } from "./eventFormatters";

export function EventTable({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <EventTableView
      events={services.eventRows ?? []}
      filter={services.eventFilter ?? "all"}
      onFilterChange={services.setEventFilter ?? (() => undefined)}
      onSearchChange={services.setEventSearch ?? (() => undefined)}
      onSessionFilterChange={services.setEventSessionFilter ?? (() => undefined)}
      onSourceFilterChange={services.setEventSourceFilter ?? (() => undefined)}
      search={services.eventSearch ?? ""}
      sessionFilter={services.eventSessionFilter ?? "all"}
      sourceFilter={services.eventSourceFilter ?? "all"}
      toolbarState={services.toolbarState}
      windowEvents={services.windowEventRows ?? []}
    />
  );
}

function EventTableView({
  events,
  filter,
  onFilterChange,
  onSearchChange,
  onSessionFilterChange,
  onSourceFilterChange,
  search,
  sessionFilter,
  sourceFilter,
  toolbarState,
  windowEvents,
}: {
  events: Array<{ type: string }>;
  filter: string;
  onFilterChange: (filter: string) => void;
  onSearchChange: (search: string) => void;
  onSessionFilterChange: (filter: string) => void;
  onSourceFilterChange: (filter: string) => void;
  search: string;
  sessionFilter: string;
  sourceFilter: string;
  toolbarState?: Record<string, string | boolean>;
  windowEvents: WindowBusEvent[];
}) {
  const eventTypes = Array.from(new Set(windowEvents.map((event) => event.type))).sort();
  const sessions = Array.from(new Set(windowEvents.flatMap((event) => (event.sessionId ? [event.sessionId] : [])))).sort();
  const sources = Array.from(new Set(windowEvents.flatMap((event) => (event.sourceWindow ? [event.sourceWindow] : [])))).sort();
  const normalizedSearch = search.trim().toLowerCase();
  const category = String(toolbarState?.category ?? "all");
  const filteredWindowEvents = windowEvents.filter((event) => {
    if (!windowEventMatchesCategory(event, category)) return false;
    if (filter !== "all" && event.type !== filter) return false;
    if (sessionFilter !== "all" && event.sessionId !== sessionFilter) return false;
    if (sourceFilter !== "all" && event.sourceWindow !== sourceFilter) return false;
    if (!normalizedSearch) return true;
    return `${event.type} ${event.sourceWindow ?? ""} ${event.sessionId ?? ""} ${formatWindowEventPayload(event)}`
      .toLowerCase()
      .includes(normalizedSearch);
  });

  async function copyPayload(event: WindowBusEvent) {
    await navigator.clipboard.writeText(JSON.stringify(event, null, 2));
  }

  return (
    <div className="event-log-panel">
      <div className="event-log-toolbar">
        <input placeholder="Search events..." value={search} onChange={(event) => onSearchChange(event.target.value)} />
        <select value={filter} onChange={(event) => onFilterChange(event.target.value)}>
          <option value="all">All window events</option>
          {eventTypes.map((type) => <option key={type} value={type}>{type}</option>)}
        </select>
        <select value={sessionFilter} onChange={(event) => onSessionFilterChange(event.target.value)}>
          <option value="all">All sessions</option>
          {sessions.map((session) => <option key={session} value={session}>{session}</option>)}
        </select>
        <select value={sourceFilter} onChange={(event) => onSourceFilterChange(event.target.value)}>
          <option value="all">All sources</option>
          {sources.map((source) => <option key={source} value={source}>{source}</option>)}
        </select>
      </div>
      <table className="workspace-table">
        <thead>
          <tr>
            <th>Time</th>
            <th>Type</th>
            <th>Source</th>
            <th>Session</th>
            <th>Summary</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {filteredWindowEvents.map((event) => (
            <tr key={event.eventId}>
              <td>{new Date(event.timestampMs).toLocaleTimeString()}</td>
              <td><code>{event.type}</code></td>
              <td>{event.sourceWindow ?? "app"}</td>
              <td>{event.sessionId ?? ""}</td>
              <td>{formatWindowEventPayload(event)}</td>
              <td>
                <button className="button button-ghost compact-button" type="button" onClick={() => void copyPayload(event)}>
                  Copy
                </button>
              </td>
            </tr>
          ))}
          {events.map((event, index) => (
            <tr key={`${event.type}:${index}`}>
              <td></td>
              <td><code>{event.type}</code></td>
              <td>{index === 0 ? "latest" : "editor"}</td>
              <td></td>
              <td></td>
              <td></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
