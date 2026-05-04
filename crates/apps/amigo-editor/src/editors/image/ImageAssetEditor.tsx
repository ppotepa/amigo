import { useEffect, useMemo, useState } from "react";
import { AlertTriangle, Image as ImageIcon, Save } from "lucide-react";
import type { EditorProjectFileContentDto, EditorProjectFileDto } from "../../api/dto";
import { writeProjectFile } from "../../api/editorApi";
import { fileSrc } from "../../utils/fileSrc";
import "./image-asset-editor.css";

type ImageAssetDraft = {
  id: string;
  label: string;
  sourceFile: string;
  sourceWidth?: number | null;
  sourceHeight?: number | null;
  usage: string;
  group: string;
  role: string;
  style: string;
  tags: string;
  filter: string;
  wrap: string;
};

export function ImageAssetEditor({
  content,
  file,
  modId,
  onDirtyChange,
  onReveal,
  onSaved,
}: {
  content?: EditorProjectFileContentDto | null;
  file: EditorProjectFileDto | null;
  modId: string;
  onDirtyChange?: (path: string, dirty: boolean) => void;
  onReveal?: () => void;
  onSaved?: () => void;
}) {
  const [draft, setDraft] = useState<ImageAssetDraft | null>(null);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setError(null);
    setDirty(false);
    if (!file || !content?.content) {
      setDraft(null);
      return;
    }
    setDraft(parseImageAssetDescriptor(content.content, file.name));
    onDirtyChange?.(file.relativePath, false);
  }, [content?.content, file?.relativePath]);

  const sourcePreviewPath = useMemo(() => {
    if (!file || !draft?.sourceFile) return null;
    return resolveSourceAbsolutePath(file.path, file.relativePath, draft.sourceFile);
  }, [draft?.sourceFile, file?.path, file?.relativePath]);

  async function saveDraft() {
    if (!file || !draft) return;
    setSaving(true);
    setError(null);
    try {
      const nextContent = serializeImageAssetDescriptor(draft);
      await writeProjectFile(modId, {
        relativePath: file.relativePath,
        content: nextContent,
      });
      setDirty(false);
      onDirtyChange?.(file.relativePath, false);
      onSaved?.();
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
    } finally {
      setSaving(false);
    }
  }

  function update<K extends keyof ImageAssetDraft>(key: K, value: ImageAssetDraft[K]) {
    setDraft((current) => (current ? { ...current, [key]: value } : current));
    if (file) {
      setDirty(true);
      onDirtyChange?.(file.relativePath, true);
    }
  }

  if (!file || !content || !draft) {
    return (
      <div className="image-asset-editor image-asset-editor-state">
        <ImageIcon size={34} />
        <strong>No image asset selected</strong>
        <span>Select a `*.image.yml` descriptor.</span>
      </div>
    );
  }

  return (
    <div className="image-asset-editor">
      <header className="image-asset-toolbar">
        <div className="image-asset-title">
          <span className="dock-icon dock-icon-cyan"><ImageIcon size={14} /></span>
          <strong>{draft.label || draft.id || file.name}</strong>
          <span>{file.relativePath}</span>
          {dirty ? <span className="badge badge-warning">modified</span> : null}
          <span className="badge badge-info">image-2d</span>
        </div>
        <div className="image-asset-actions">
          {onReveal ? (
            <button className="button button-tool" type="button" onClick={onReveal}>
              Reveal
            </button>
          ) : null}
          <button className="button button-tool" type="button" disabled={!dirty || saving} onClick={() => void saveDraft()}>
            <Save size={13} />
            {saving ? "Saving" : "Save"}
          </button>
        </div>
      </header>
      <main className="image-asset-body">
        <section className="image-asset-preview">
          {sourcePreviewPath ? (
            <>
              <div className="image-asset-preview-frame">
                <img alt={draft.label || draft.id} src={fileSrc(sourcePreviewPath)} />
              </div>
              <div className="image-asset-preview-meta">
                <span>{draft.sourceFile}</span>
                <span>{draft.sourceWidth ?? "?"} x {draft.sourceHeight ?? "?"}</span>
              </div>
            </>
          ) : (
            <div className="image-asset-editor-state">
              <AlertTriangle size={34} />
              <strong>No source image</strong>
              <span>Descriptor does not point to a source file.</span>
            </div>
          )}
        </section>
        <aside className="image-asset-inspector">
          <section>
            <h3>Identity</h3>
            <div className="sheet-form-grid">
              <TextField label="Id" value={draft.id} onChange={(value) => update("id", value)} />
              <TextField label="Label" value={draft.label} onChange={(value) => update("label", value)} />
              <TextField label="Source" value={draft.sourceFile} onChange={(value) => update("sourceFile", value)} wide />
            </div>
          </section>
          <section>
            <h3>Metadata</h3>
            <div className="sheet-form-grid">
              <NumberField label="Width" value={draft.sourceWidth ?? 0} onChange={(value) => update("sourceWidth", value || null)} />
              <NumberField label="Height" value={draft.sourceHeight ?? 0} onChange={(value) => update("sourceHeight", value || null)} />
              <TextField label="Usage" value={draft.usage} onChange={(value) => update("usage", value)} />
              <TextField label="Group" value={draft.group} onChange={(value) => update("group", value)} />
              <TextField label="Role" value={draft.role} onChange={(value) => update("role", value)} />
              <TextField label="Style" value={draft.style} onChange={(value) => update("style", value)} />
              <TextField label="Tags" value={draft.tags} onChange={(value) => update("tags", value)} wide />
            </div>
          </section>
          <section>
            <h3>Sampling</h3>
            <div className="sheet-form-grid">
              <TextField label="Filter" value={draft.filter} onChange={(value) => update("filter", value)} />
              <TextField label="Wrap" value={draft.wrap} onChange={(value) => update("wrap", value)} />
            </div>
          </section>
          {error ? (
            <section>
              <h3>Error</h3>
              <div className="sheet-diagnostic diagnostic-error">
                <strong>save_failed</strong>
                <span>{error}</span>
              </div>
            </section>
          ) : null}
        </aside>
      </main>
    </div>
  );
}

function parseImageAssetDescriptor(content: string, fallbackName: string): ImageAssetDraft {
  const get = (pattern: RegExp) => content.match(pattern)?.[1]?.trim() ?? "";
  const getNumber = (pattern: RegExp) => {
    const value = content.match(pattern)?.[1];
    return value ? Number(value) : null;
  };
  const tags = content.match(/tags:\s*\[([^\]]*)\]/)?.[1] ?? "";
  return {
    id: get(/^id:\s*(.+)$/m) || fallbackName.replace(/\.image\.ya?ml$/i, ""),
    label: get(/^label:\s*(.+)$/m),
    sourceFile: get(/^\s*file:\s*(.+)$/m),
    sourceWidth: getNumber(/^\s*width:\s*(\d+)$/m),
    sourceHeight: getNumber(/^\s*height:\s*(\d+)$/m),
    usage: get(/^usage:\s*(.+)$/m),
    group: get(/^group:\s*(.+)$/m),
    role: get(/^role:\s*(.+)$/m),
    style: get(/^style:\s*(.+)$/m),
    tags: tags.split(",").map((item) => item.trim()).filter(Boolean).join(", "),
    filter: get(/^\s*filter:\s*(.+)$/m),
    wrap: get(/^\s*wrap:\s*(.+)$/m),
  };
}

function serializeImageAssetDescriptor(draft: ImageAssetDraft): string {
  const lines = [
    "kind: image-2d",
    "schema_version: 1",
    `id: ${draft.id.trim() || "image"}`,
    `label: ${draft.label.trim() || draft.id.trim() || "Image"}`,
    "",
    "source:",
    `  file: ${draft.sourceFile.trim()}`,
  ];
  if (draft.sourceWidth) lines.push(`  width: ${Math.max(0, Math.floor(draft.sourceWidth))}`);
  if (draft.sourceHeight) lines.push(`  height: ${Math.max(0, Math.floor(draft.sourceHeight))}`);
  lines.push("");
  if (draft.usage.trim()) lines.push(`usage: ${draft.usage.trim()}`);
  if (draft.group.trim()) lines.push(`group: ${draft.group.trim()}`);
  if (draft.role.trim()) lines.push(`role: ${draft.role.trim()}`);
  if (draft.style.trim()) lines.push(`style: ${draft.style.trim()}`);
  if (draft.tags.trim()) lines.push(`tags: [${draft.tags.split(",").map((item) => item.trim()).filter(Boolean).join(", ")}]`);
  lines.push("");
  lines.push("sampling:");
  lines.push(`  filter: ${draft.filter.trim() || "linear"}`);
  lines.push(`  wrap: ${draft.wrap.trim() || "clamp"}`);
  lines.push("");
  return lines.join("\n");
}

function resolveSourceAbsolutePath(descriptorPath: string, descriptorRelativePath: string, sourceFile: string): string {
  const normalizedDescriptorPath = descriptorPath.replace(/\\/g, "/");
  const normalizedDescriptorRelativePath = descriptorRelativePath.replace(/\\/g, "/");
  const normalizedSource = sourceFile.replace(/\\/g, "/").trim();
  const descriptorRootIndex = normalizedDescriptorPath.lastIndexOf(normalizedDescriptorRelativePath);
  const projectRoot = descriptorRootIndex >= 0
    ? normalizedDescriptorPath.slice(0, descriptorRootIndex).replace(/\/$/, "")
    : "";
  if (normalizedSource.startsWith("assets/") && projectRoot) {
    return normalizePathParts(`${projectRoot}/${normalizedSource}`);
  }

  const base = normalizedDescriptorPath.split("/");
  base.pop();
  const parts = normalizedSource.split("/");
  for (const part of parts) {
    if (!part || part === ".") continue;
    if (part === "..") {
      base.pop();
    } else {
      base.push(part);
    }
  }
  return normalizePathParts(base.join("/"));
}

function normalizePathParts(path: string): string {
  const prefix = path.match(/^[A-Za-z]:/)?.[0] ?? "";
  const withoutPrefix = prefix ? path.slice(prefix.length) : path;
  const parts: string[] = [];
  for (const part of withoutPrefix.split("/")) {
    if (!part || part === ".") continue;
    if (part === "..") {
      parts.pop();
    } else {
      parts.push(part);
    }
  }
  return `${prefix}/${parts.join("/")}`;
}

function TextField({
  label,
  onChange,
  value,
  wide,
}: {
  label: string;
  onChange: (value: string) => void;
  value: string;
  wide?: boolean;
}) {
  return (
    <label className={`sheet-field ${wide ? "wide" : ""}`}>
      <span>{label}</span>
      <input value={value} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

function NumberField({
  label,
  onChange,
  value,
}: {
  label: string;
  onChange: (value: number) => void;
  value: number;
}) {
  return (
    <label className="sheet-field">
      <span>{label}</span>
      <input type="number" value={value} onChange={(event) => onChange(Number(event.target.value) || 0)} />
    </label>
  );
}
