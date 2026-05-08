import type { EditorComponentProps } from "../../editor-components/componentTypes";
import {
  FileAtlasComponent,
  FileImageAssetComponent,
  FileRawImageComponent,
  FileSpriteComponent,
  FileTilemapComponent,
  FileTileRulesetComponent,
  FileTilesetComponent,
  FileTextureComponent,
} from "../../editor-components/componentRegistry";
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
  switch (instance.component) {
    case FileImageAssetComponent:
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
    case FileTextureComponent:
    case FileRawImageComponent:
    case FileSpriteComponent:
    case FileAtlasComponent:
    case FileTilesetComponent:
      if (
        context.sessionId &&
        instance.resourceUri &&
        (instance.component === FileTilesetComponent ||
          ((instance.component === FileSpriteComponent || instance.component === FileAtlasComponent) &&
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
    case FileTilemapComponent:
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
    case FileTileRulesetComponent:
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
