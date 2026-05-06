import type { ContextWidgetId, ContextWidgetStateMap } from "./contextDockTypes";

export function createDefaultContextWidgetState(
  ids: ContextWidgetId[],
  collapsedIds: ContextWidgetId[] = [],
): ContextWidgetStateMap {
  const collapsed = new Set(collapsedIds);
  return Object.fromEntries(
    ids.map((id) => [
      id,
      {
        id,
        collapsed: collapsed.has(id),
      },
    ]),
  );
}

export function toggleContextWidgetCollapsed(
  state: ContextWidgetStateMap,
  id: ContextWidgetId,
): ContextWidgetStateMap {
  const current = state[id] ?? { id, collapsed: false };
  return {
    ...state,
    [id]: {
      ...current,
      collapsed: !current.collapsed,
    },
  };
}
