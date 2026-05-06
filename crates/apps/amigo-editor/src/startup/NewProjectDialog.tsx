import { useMemo, useState } from "react";
import {
  AlertTriangle,
  Box,
  CheckCircle2,
  FileText,
  Grid3X3,
  Layers3,
  Lock,
  Sparkles,
  X,
} from "lucide-react";
import type { CreateModProjectTypeDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";

const PROJECT_ID_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

export function NewProjectDialog({ onClose }: { onClose: () => void }) {
  const { createModProject, openSelectedMod, state } = useEditorStore();
  const defaultProjectId = useMemo(() => nextAvailableProjectId(state.mods.map((mod) => mod.id)), [state.mods]);
  const [projectType, setProjectType] = useState<CreateModProjectTypeDto>("2d");
  const [projectName, setProjectName] = useState(defaultProjectId);
  const [projectId, setProjectId] = useState(defaultProjectId);
  const [projectIdEdited, setProjectIdEdited] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const initialSceneId = "start";
  const location = `mods/${projectId || "new-project"}`;

  const createdFiles = useMemo(
    () => [
      `${location}/mod.toml`,
      `${location}/scenes/${initialSceneId}/scene.yml`,
      `${location}/scenes/${initialSceneId}/scene.rhai`,
      `${location}/fonts/debug-ui/font.yml`,
    ],
    [initialSceneId, location],
  );

  const projectExists = state.mods.some((mod) => mod.id.toLowerCase() === projectId.trim().toLowerCase());
  const validation = validateProjectDraft(projectName, projectId, projectType, projectExists);
  const validationError = validation.error;
  const canCreate = !busy && !validationError;

  function handleProjectNameChanged(value: string) {
    setProjectName(value);
    if (!projectIdEdited) {
      setProjectId(slugProjectId(value));
    }
  }

  async function handleCreate() {
    if (!canCreate) return;
    setBusy(true);
    setError(null);
    try {
      await createModProject({
        projectType,
        projectName: projectName.trim(),
        projectId: projectId.trim(),
      });
      await openSelectedMod();
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="new-project-backdrop" onMouseDown={busy ? undefined : onClose}>
      <section
        className="new-project-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="new-project-title"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <header className="new-project-header">
          <div>
            <h2 id="new-project-title">New Project</h2>
            <p>Create a new Amigo mod project in /mods</p>
          </div>
          <button
            className="new-project-close-button"
            type="button"
            aria-label="Close New Project dialog"
            onClick={onClose}
            disabled={busy}
          >
            <X size={17} />
          </button>
        </header>
        <div className="new-project-body">
          <section className="new-project-section">
            <div className="new-project-section-title">Project type</div>
            <div className="project-type-grid" role="radiogroup" aria-label="Project type">
              <ProjectTypeButton
                type={{ id: "2d", label: "2D", description: "Top-down, UI, sprites, text", icon: "grid" }}
                selected={projectType === "2d"}
                onSelect={() => setProjectType("2d")}
              />
              <ProjectTypeButton
                type={{ id: "2_5d", label: "2.5D", description: "Isometric / layered 2D", badge: "Coming soon", disabled: true, icon: "layers" }}
                selected={projectType === "2_5d"}
                onSelect={() => setProjectType("2_5d")}
              />
              <ProjectTypeButton
                type={{ id: "3d", label: "3D", description: "3D scene project", badge: "Coming soon", disabled: true, icon: "box" }}
                selected={projectType === "3d"}
                onSelect={() => setProjectType("3d")}
              />
            </div>
          </section>

          <section className="new-project-form" aria-label="Project settings">
            <label className="new-project-field">
              <span>Project name</span>
              <input
                className={validation.nameInvalid ? "is-invalid" : ""}
                aria-invalid={validation.nameInvalid}
                value={projectName}
                placeholder="new-project"
                disabled={busy}
                onChange={(event) => handleProjectNameChanged(event.target.value)}
              />
            </label>

            <label className="new-project-field">
              <span>Project id</span>
              <input
                className={validation.idInvalid ? "is-invalid" : ""}
                aria-invalid={validation.idInvalid}
                value={projectId}
                placeholder="they-are-rotten"
                disabled={busy}
                onChange={(event) => {
                  setProjectIdEdited(true);
                  setProjectId(event.target.value.toLowerCase());
                }}
              />
              <small>Auto-generated from project name. You can edit it before creation.</small>
            </label>

            <label className="new-project-field">
              <span>Location</span>
              <input value={location} readOnly disabled />
              <small>Projects are created in the current mods folder.</small>
            </label>

            <label className="new-project-field">
              <span>Initial scene</span>
              <input value={initialSceneId} readOnly disabled />
            </label>
          </section>

          <section className="new-project-files" aria-label="Generated files preview">
            <div className="new-project-section-title">This will create:</div>
            <div className="new-project-file-list">
              {createdFiles.map((file) => (
                <div key={file} className="new-project-file-row">
                  <FileText size={14} />
                  <code>{file}</code>
                </div>
              ))}
            </div>
          </section>

          {validationError || error ? (
            <div className="new-project-diagnostic" role="alert">
              <AlertTriangle size={16} />
              <span>{error ?? validationError}</span>
            </div>
          ) : null}
        </div>
        <footer className="new-project-footer">
          <button className="button button-ghost" type="button" disabled={busy} onClick={onClose}>Cancel</button>
          <button className="button button-primary" type="button" disabled={!canCreate} onClick={() => void handleCreate()}>
            {busy ? <span className="spinner spinner-inline" /> : <Sparkles size={16} />}
            {busy ? "Creating..." : "Create Project"}
          </button>
        </footer>
      </section>
    </div>
  );
}

function ProjectTypeButton({
  type,
  selected,
  onSelect,
}: {
  type: {
    id: CreateModProjectTypeDto;
    label: string;
    description: string;
    badge?: string;
    disabled?: boolean;
    icon: "grid" | "layers" | "box";
  };
  selected: boolean;
  onSelect: () => void;
}) {
  const Icon = iconForProjectType(type.icon);
  return (
    <button className={`project-type-card ${selected ? "selected" : ""}`} type="button" role="radio" aria-checked={selected} disabled={type.disabled} onClick={onSelect}>
      <span className="project-type-lock" aria-hidden="true">
        {type.disabled ? <Lock size={14} /> : selected ? <CheckCircle2 size={16} /> : null}
      </span>
      <span className="project-type-icon"><Icon size={38} strokeWidth={1.7} /></span>
      <strong>{type.label}</strong>
      <small>{type.description}</small>
      {type.badge ? <span className="project-type-badge">{type.badge}</span> : null}
    </button>
  );
}

function iconForProjectType(icon: "grid" | "layers" | "box") {
  switch (icon) {
    case "layers":
      return Layers3;
    case "box":
      return Box;
    case "grid":
    default:
      return Grid3X3;
  }
}

function validateProjectDraft(
  projectName: string,
  projectId: string,
  projectType: CreateModProjectTypeDto,
  projectExists: boolean,
): { error: string | null; nameInvalid: boolean; idInvalid: boolean } {
  if (!projectName.trim()) {
    return { error: "Project name is required.", nameInvalid: true, idInvalid: false };
  }
  if (!PROJECT_ID_PATTERN.test(projectId.trim())) {
    return {
      error: "Project id must use lowercase letters, numbers, and single dashes between words.",
      nameInvalid: false,
      idInvalid: true,
    };
  }
  if (projectExists) {
    return { error: "Project id already exists. Choose a different id.", nameInvalid: false, idInvalid: true };
  }
  if (projectType !== "2d") {
    return { error: "Only the 2D template is available right now.", nameInvalid: false, idInvalid: false };
  }
  return { error: null, nameInvalid: false, idInvalid: false };
}

function slugProjectId(value: string): string {
  return value
    .trim()
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[łŁ]/g, "l")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .replace(/-{2,}/g, "-");
}

function nextAvailableProjectId(existingIds: string[]): string {
  const taken = new Set(existingIds.map((id) => id.toLowerCase()));
  if (!taken.has("new-project")) {
    return "new-project";
  }
  let n = 2;
  while (taken.has(`new-project-${n}`)) {
    n += 1;
  }
  return `new-project-${n}`;
}
