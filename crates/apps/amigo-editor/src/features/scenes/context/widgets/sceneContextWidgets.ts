import type { WidgetDefinition } from "../../../../workbench/widgets/widgetTypes";
import type { SceneContextModel } from "../sceneContextTypes";
import { SceneComponentsWidget } from "./SceneComponentsWidget";
import { SceneEntitiesWidget } from "./SceneEntitiesWidget";
import { SceneHeaderWidget } from "./SceneHeaderWidget";
import { SceneNavigationWidget } from "./SceneNavigationWidget";

export const SCENE_CONTEXT_WIDGETS = [
  {
    id: "scene.header",
    title: "Scene Header",
    icon: "gauge",
    domain: "scene",
    targetKinds: ["scene"],
    placement: "top",
    order: 10,
    getStatus: (model) => model.header.status,
    getFoldedHint: (model) => model.header.foldedHint,
    render: SceneHeaderWidget,
  },
  {
    id: "scene.navigation",
    title: "Scene Navigation",
    icon: "folder",
    domain: "scene",
    targetKinds: ["scene"],
    placement: "top",
    order: 20,
    getStatus: () => "ok",
    getFoldedHint: (model) => model.navigation.foldedHint,
    render: SceneNavigationWidget,
  },
  {
    id: "scene.components",
    title: "Scene Components",
    icon: "box",
    domain: "scene",
    targetKinds: ["scene"],
    placement: "top",
    order: 30,
    getStatus: (model) => model.components.warningCount ? "warning" : "ok",
    getFoldedHint: (model) => model.components.foldedHint,
    render: SceneComponentsWidget,
  },
  {
    id: "scene.entities",
    title: "Scene Entities",
    icon: "list-tree",
    domain: "scene",
    targetKinds: ["scene"],
    placement: "top",
    order: 40,
    getStatus: (model) => model.entitiesInfo.warningCount ? "warning" : "ok",
    getFoldedHint: (model) => `${model.entitiesInfo.total} entities`,
    render: SceneEntitiesWidget,
  },
] satisfies WidgetDefinition<SceneContextModel>[];
