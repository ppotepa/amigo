import { Box } from "lucide-react";
import { iconForEditorComponent } from "../editor-components/componentRegistry";
import type { EditorComponentInstance } from "../editor-components/componentTypes";
import { toneForComponentDomain } from "../theme/semanticColorRegistry";

export function componentTabs(instances: EditorComponentInstance[]) {
  return instances.map((instance) => {
    const definition = instance.component;
    const icon = instance.iconOverride ?? definition.icon;
    return {
      id: instance.instanceId,
      title: instance.titleOverride ?? definition?.title ?? instance.componentId,
      icon: definition ? iconForEditorComponent(icon, 13, toneForComponentDomain(definition.domain)) : <Box size={13} />,
    };
  });
}
