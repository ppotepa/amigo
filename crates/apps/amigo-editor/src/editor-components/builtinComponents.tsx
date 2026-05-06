import { COMPONENT_DEBUG_SOURCES } from "./builtinComponentDebugSources";
import {
  ASSET_COMPONENTS,
  DIAGNOSTICS_COMPONENTS,
  FILE_COMPONENTS,
  INSPECTOR_COMPONENTS,
  PROJECT_COMPONENTS,
  SCENE_COMPONENTS,
  SYSTEM_COMPONENTS,
  UI_DOCUMENT_COMPONENTS,
} from "./builtin";
import type { EditorComponentDefinition } from "./componentTypes";
import { withComponentDebugSource } from "./componentDefinitionFactory";

const rawBuiltinEditorComponents: EditorComponentDefinition[] = [
  ...PROJECT_COMPONENTS,
  ...ASSET_COMPONENTS,
  ...FILE_COMPONENTS,
  ...SCENE_COMPONENTS,
  ...INSPECTOR_COMPONENTS,
  ...DIAGNOSTICS_COMPONENTS,
  ...SYSTEM_COMPONENTS,
  ...UI_DOCUMENT_COMPONENTS,
];

export const builtinEditorComponents: EditorComponentDefinition[] = rawBuiltinEditorComponents.map((component) =>
  withComponentDebugSource(component, COMPONENT_DEBUG_SOURCES),
);
