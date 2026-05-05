import type { EditorComponentProps } from "../../editor-components/componentTypes";
import { ImageAssetEditor } from "../../editors/image/ImageAssetEditor";
import { SheetEditor } from "../../editors/sheet/SheetEditor";
import { TileRulesetEditor } from "../../editors/tile-ruleset/TileRulesetEditor";
import { TilemapEditor } from "../../editors/tilemap/TilemapEditor";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { canReadProjectFileContent } from "./fileContentRules";
import { TextFileWorkspace } from "./TextFileWorkspace";

export function FileWorkspaceHost({
  context,
  instance,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  switch (instance.componentId) {
    case "file.image-asset":
      if (services.details?.id && services.selectedFile && services.selectedFileContent) {
        return (
          <ImageAssetEditor
            content={services.selectedFileContent}
            file={services.selectedFile}
            modId={services.details.id}
            onDirtyChange={services.onFileDirtyChange}
            onReveal={services.onRevealSelectedFile}
            onSaved={services.onProjectTreeRefresh}
          />
        );
      }
      break;
    case "file.texture":
    case "file.raw-image":
    case "file.sprite":
    case "file.atlas":
    case "file.tileset":
      if (
        context.sessionId &&
        instance.resourceUri &&
        (instance.componentId === "file.tileset" ||
          ((instance.componentId === "file.sprite" || instance.componentId === "file.atlas") &&
            services.selectedFile &&
            canReadProjectFileContent(services.selectedFile)))
      ) {
        return (
          <SheetEditor
            resourceUri={instance.resourceUri}
            sessionId={context.sessionId}
            onDirtyChange={services.onFileDirtyChange}
            onSaved={services.onProjectTreeRefresh}
            onReveal={services.onRevealSelectedFile}
          />
        );
      }
      break;
    case "file.tilemap":
      if (context.sessionId && instance.resourceUri) {
        return (
          <TilemapEditor
            resourceUri={instance.resourceUri}
            sessionId={context.sessionId}
            onDirtyChange={services.onFileDirtyChange}
          />
        );
      }
      break;
    case "file.tile-ruleset":
      if (context.sessionId && instance.resourceUri) {
        return (
          <TileRulesetEditor
            resourceUri={instance.resourceUri}
            sessionId={context.sessionId}
            sourceText={services.selectedFileContent?.content}
            onReveal={services.onRevealSelectedFile}
          />
        );
      }
      break;
  }

  return (
    <TextFileWorkspace
      file={services.selectedFile ?? null}
      content={services.selectedFileContent ?? undefined}
      onReveal={services.onRevealSelectedFile}
    />
  );
}
