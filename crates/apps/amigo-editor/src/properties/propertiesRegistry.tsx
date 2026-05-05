import { AssetPropertiesPanel } from "./panels/AssetPropertiesPanel";
import { EmptyPropertiesPanel } from "./panels/EmptyPropertiesPanel";
import { EntityPropertiesPanel } from "./panels/EntityPropertiesPanel";
import { ModPropertiesPanel } from "./panels/ModPropertiesPanel";
import { ProjectFilePropertiesPanel } from "./panels/ProjectFilePropertiesPanel";
import { ScenePropertiesPanel } from "./panels/ScenePropertiesPanel";
import { propertyPanel } from "./propertiesTypes";
import type { PropertiesRenderer } from "./propertiesTypes";

export const PROPERTY_RENDERERS: PropertiesRenderer[] = [
  propertyPanel("asset", (selection, context) => (
    <AssetPropertiesPanel context={context} selection={selection} />
  )),
  propertyPanel("entity", (selection) => <EntityPropertiesPanel selection={selection} />),
  propertyPanel("projectFile", (selection) => (
    <ProjectFilePropertiesPanel selection={selection} />
  )),
  propertyPanel("scene", (selection) => <ScenePropertiesPanel selection={selection} />),
  propertyPanel("mod", (selection) => <ModPropertiesPanel selection={selection} />),
  propertyPanel("empty", () => <EmptyPropertiesPanel />),
];
