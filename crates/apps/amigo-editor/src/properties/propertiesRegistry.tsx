import { AssetPropertiesPanel } from "./panels/AssetPropertiesPanel";
import { EmptyPropertiesPanel } from "./panels/EmptyPropertiesPanel";
import { EntityPropertiesPanel } from "./panels/EntityPropertiesPanel";
import { ModPropertiesPanel } from "./panels/ModPropertiesPanel";
import { ProjectFilePropertiesPanel } from "./panels/ProjectFilePropertiesPanel";
import { ScenePropertiesPanel } from "./panels/ScenePropertiesPanel";
import { propertiesRenderer } from "./propertiesTypes";
import type {
  AssetSelection,
  EmptySelection,
  EntitySelection,
  ModSelection,
  ProjectFileSelection,
  PropertiesRenderer,
  SceneSelection,
} from "./propertiesTypes";

export const PROPERTY_RENDERERS: PropertiesRenderer[] = [
  propertiesRenderer({
    kind: "asset",
    canRender: (selection): selection is AssetSelection => selection.kind === "asset",
    render: (selection, context) => <AssetPropertiesPanel context={context} selection={selection} />,
  }),
  propertiesRenderer({
    kind: "entity",
    canRender: (selection): selection is EntitySelection => selection.kind === "entity",
    render: (selection) => <EntityPropertiesPanel selection={selection} />,
  }),
  propertiesRenderer({
    kind: "projectFile",
    canRender: (selection): selection is ProjectFileSelection => selection.kind === "projectFile",
    render: (selection) => <ProjectFilePropertiesPanel selection={selection} />,
  }),
  propertiesRenderer({
    kind: "scene",
    canRender: (selection): selection is SceneSelection => selection.kind === "scene",
    render: (selection) => <ScenePropertiesPanel selection={selection} />,
  }),
  propertiesRenderer({
    kind: "mod",
    canRender: (selection): selection is ModSelection => selection.kind === "mod",
    render: (selection) => <ModPropertiesPanel selection={selection} />,
  }),
  propertiesRenderer({
    kind: "empty",
    canRender: (selection): selection is EmptySelection => selection.kind === "empty",
    render: () => <EmptyPropertiesPanel />,
  }),
];
