import { WidgetFrame } from "./WidgetFrame";
import { GenericPropertiesPanel } from "../../features/metadata/GenericPropertiesPanel";
import type { EditorMetadataCatalogDto } from "../../features/metadata/editorMetadataTypes";
import type { EditorSceneComponentInstanceDto } from "../../api/dto";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import type { PropertiesWidgetModel } from "./widgetTypes";

type PropertiesWidgetProps = {
  model?: PropertiesWidgetModel;
  target?: unknown;
  metadata?: EditorMetadataCatalogDto | null;
  assetRegistry?: unknown;
  services?: WorkspaceRuntimeServices;
  fallbackText?: string;
};

export function PropertiesWidget({
  model,
  target,
  metadata,
  assetRegistry,
  services,
  fallbackText,
}: PropertiesWidgetProps) {
  const title = model?.fallbackText ?? fallbackText ?? "Properties";

  return (
    <WidgetFrame id="properties-widget" title={title} compact>
      {services && metadata ? (
        <GenericPropertiesPanel
          metadata={metadata}
          assetRegistry={assetRegistry as never}
          component={pickComponentTarget(target ?? model?.target) as EditorSceneComponentInstanceDto | null}
        />
      ) : (
        <p className="muted workspace-empty">{model?.fallbackText ?? fallbackText ?? "No properties available for this target."}</p>
      )}
    </WidgetFrame>
  );
}

function pickComponentTarget(target: unknown): EditorSceneComponentInstanceDto | null {
  if (!target || typeof target !== "object") return null;

  const candidate = target as {
    typeName?: unknown;
    type?: unknown;
    componentIndex?: unknown;
    componentType?: unknown;
  };

  if (typeof candidate.typeName === "string" && typeof candidate.componentIndex === "number") {
    return target as EditorSceneComponentInstanceDto;
  }

  if (typeof candidate.type === "string" && candidate.componentIndex === undefined) {
    return null;
  }

  return null;
}
