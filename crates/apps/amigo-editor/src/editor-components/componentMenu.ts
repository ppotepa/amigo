import { EDITOR_COMPONENTS } from "./componentRegistry";
import type { EditorComponentCategory, EditorComponentDefinition } from "./componentTypes";

export interface ComponentMenuGroup {
  category: EditorComponentCategory;
  components: EditorComponentDefinition[];
}

export function componentMenuGroups(): ComponentMenuGroup[] {
  const groups = new Map<EditorComponentCategory, EditorComponentDefinition[]>();
  for (const component of EDITOR_COMPONENTS) {
    const components = groups.get(component.category) ?? [];
    components.push(component);
    groups.set(component.category, components);
  }

  return Array.from(groups.entries()).map(([category, components]) => ({
    category,
    components: components.sort((left, right) => left.title.localeCompare(right.title)),
  }));
}

export function componentsForWindowMenu(): EditorComponentDefinition[] {
  return EDITOR_COMPONENTS.filter((component) => component.canDock || component.canOpenInWindow).sort((left, right) =>
    left.title.localeCompare(right.title),
  );
}
