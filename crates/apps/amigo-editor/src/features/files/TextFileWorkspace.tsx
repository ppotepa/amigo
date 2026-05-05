import { AlertTriangle, FileCode2, FolderOpen } from "lucide-react";
import type { EditorProjectFileDto } from "../../api/dto";
import type { FileWorkspaceDescriptor } from "./fileWorkspaceTypes";
import { canReadProjectFileContent } from "./fileContentRules";
import { resolveFileWorkspaceDescriptor, workspaceDescriptorLanguage } from "./fileWorkspaceRules";
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
  descriptor?: FileWorkspaceDescriptor;
  onReveal?: () => void;
}) {
  if (!file) {
    return <p className="muted workspace-empty">No file selected.</p>;
  }

  const resolved = descriptor ?? resolveFileWorkspaceDescriptor(file);
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
