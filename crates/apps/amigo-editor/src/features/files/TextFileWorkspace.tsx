import { AlertTriangle, FileCode2, FolderOpen } from "lucide-react";
import type { EditorProjectFileDto } from "../../api/dto";
import type { FileWorkspaceDescriptor, WorkspaceFileKind, WorkspaceOpenMode, WorkspaceShape } from "./fileWorkspaceTypes";
import { canReadProjectFileContent } from "./fileContentRules";
import { RawImageWorkspace } from "./RawImageWorkspace";

function formatBytes(sizeBytes: number): string {
  if (sizeBytes < 1024) {
    return `${sizeBytes} B`;
  }
  if (sizeBytes < 1024 * 1024) {
    return `${(sizeBytes / 1024).toFixed(1)} KB`;
  }
  return `${(sizeBytes / (1024 * 1024)).toFixed(1)} MB`;
}


type TextFileWorkspaceDescriptor = Omit<FileWorkspaceDescriptor, "component">;

function fallbackDescriptor(file: EditorProjectFileDto): TextFileWorkspaceDescriptor {
  const fileKind: WorkspaceFileKind = file.kind === "unknownBinary" ? "unknown_binary" : "unknown_text";
  const shape: WorkspaceShape = fileKind === "unknown_binary" ? "unsupported" : "text-editor";
  const openMode: WorkspaceOpenMode = fileKind === "unknown_binary" ? "unsupported" : "editor";
  return {
    fileKind,
    shape,
    openMode,
    title: fileKind === "unknown_binary" ? "Unsupported" : "Text",
    iconText: fileKind === "unknown_binary" ? "Bin" : "Txt",
    editable: fileKind !== "unknown_binary",
  };
}

function workspaceDescriptorLanguage(
  descriptor: TextFileWorkspaceDescriptor,
  content?: { language?: string },
): string {
  if (content?.language) return content.language;

  switch (descriptor.fileKind) {
    case "manifest":
      return "toml";
    case "scene_document":
    case "script_package":
    case "image_asset":
    case "tilemap":
    case "tileset":
    case "tile_ruleset":
    case "atlas":
    case "ui_document":
    case "config":
      return "yaml";
    case "script":
    case "scene_script":
      return "rhai";
    default:
      return "text";
  }
}
function isImageFile(file: EditorProjectFileDto): boolean {
  return file.kind === "texture" || file.kind === "spritesheet" || file.kind === "rawImage" || /\.(png|jpe?g|webp)$/i.test(file.name);
}

export function TextFileWorkspace({
  file,
  content,
  descriptor,
  onReveal,
}: {
  file: EditorProjectFileDto | null;
  content?: { content: string; language: string } | null;
  descriptor?: TextFileWorkspaceDescriptor;
  onReveal?: () => void;
}) {
  if (!file) {
    return <p className="muted workspace-empty">No file selected.</p>;
  }

  const resolved = descriptor ?? fallbackDescriptor(file);
  const effectiveLanguage = workspaceDescriptorLanguage(resolved, content ?? undefined);
  const metadata = (
    <div className="file-metadata-strip">
      <span>{resolved.fileKind}</span>
      <span>{resolved.shape}</span>
      <span>{resolved.editable ? "editable" : "readonly"}</span>
      <span>{formatBytes(file.sizeBytes)}</span>
      <span>{file.path}</span>
    </div>
  );

  return (
    <div className="file-workbench">
      <div className="scene-workbench-toolbar">
        <div className="scene-heading">
          <span className="dock-icon dock-icon-cyan">{resolved.iconText}</span>
          <strong>{file.name}</strong>
          <span>{file.relativePath}</span>
          <span className="badge badge-info">{resolved.title}</span>
        </div>
        <div className="scene-heading-actions">
          {onReveal ? (
            <button className="button button-tool" type="button" onClick={onReveal}>
              <FolderOpen size={14} />
              Reveal
            </button>
          ) : null}
        </div>
      </div>

      {resolved.shape === "preview-plus-inspector" ? (
        <>
          <div className="file-preview-stage">
            {isImageFile(file) ? (
              <RawImageWorkspace file={file} />
            ) : (
              <div className="file-preview-empty">
                <FileCode2 size={40} />
                <strong>{resolved.title}</strong>
                <span>{file.relativePath}</span>
              </div>
            )}
          </div>
          {metadata}
        </>
      ) : resolved.shape === "canvas-editor" ? (
        <>
          <div className="file-preview-stage file-domain-placeholder">
            <div className="file-preview-empty">
              <strong>{resolved.title} Workspace</strong>
              <span>Groundwork is ready. Domain editor surface plugs in here.</span>
            </div>
            {content?.content ? (
              <pre className="file-code-preview file-code-preview-overlay" data-language={effectiveLanguage}>
                <code>{content.content}</code>
              </pre>
            ) : null}
          </div>
          {metadata}
        </>
      ) : resolved.shape === "form-plus-source" ? (
        <>
          <div className="file-form-source-layout">
            <section className="file-form-summary">
              <strong>{resolved.title}</strong>
              <span>{resolved.fileKind}</span>
              <span>{resolved.openMode}</span>
              <span>{resolved.editable ? "Will support structured editing" : "Read-only surface"}</span>
            </section>
            <div className="file-preview-stage">
              {content?.content ? (
                <pre className="file-code-preview" data-language={effectiveLanguage}>
                  <code>{content.content}</code>
                </pre>
              ) : (
                <div className="file-preview-empty">
                  <FileCode2 size={40} />
                  <strong>{resolved.title}</strong>
                  <span>{canReadProjectFileContent(file) ? "Loading structured source..." : file.relativePath}</span>
                </div>
              )}
            </div>
          </div>
          {metadata}
        </>
      ) : resolved.shape === "text-editor" ? (
        <>
          <div className="file-preview-stage">
            {content?.content ? (
              <pre className="file-code-preview" data-language={effectiveLanguage}>
                <code>{content.content}</code>
              </pre>
            ) : (
              <div className="file-preview-empty">
                <FileCode2 size={40} />
                <strong>{resolved.title}</strong>
                <span>{canReadProjectFileContent(file) ? "Loading text preview..." : file.relativePath}</span>
              </div>
            )}
          </div>
          {metadata}
        </>
      ) : (
        <>
          <div className="file-preview-stage">
            <div className="file-preview-empty">
              <AlertTriangle size={40} />
              <strong>{resolved.title}</strong>
              <span>{file.relativePath}</span>
            </div>
          </div>
          {metadata}
        </>
      )}
    </div>
  );
}
