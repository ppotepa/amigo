import { Box } from "lucide-react";
import { editorComponentById, iconForEditorComponent } from "../editor-components/componentRegistry";
import type { EditorComponentInstance } from "../editor-components/componentTypes";

export function componentTabs(instances: EditorComponentInstance[]) {
  return instances.map((instance) => {
    const definition = editorComponentById(instance.componentId);
    return {
      id: instance.instanceId,
      title: instance.titleOverride ?? definition?.title ?? instance.componentId,
      icon: definition ? iconForEditorComponent(definition.icon, 13) : <Box size={13} />,
    };
  });
}
