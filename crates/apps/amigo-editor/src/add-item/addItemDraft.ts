import { ADD_ITEM_DEFINITIONS } from "./addItemCatalog";
import type { AddItemFormValue, AddItemKind, AddItemScope } from "./addItemTypes";
import { nextAvailableSlugId, normalizeSlugId } from "../ui/validation/slugId";

export function initialTargetFolder(kind: AddItemKind, scope?: AddItemScope): string {
  if (scope?.kind === "project-folder") return scope.path;
  const definition = ADD_ITEM_DEFINITIONS.find((entry) => entry.kind === kind);
  return definition?.defaultTargetPath ?? "";
}

export function defaultLabelForKind(kind: AddItemKind): string {
  if (kind === "scene") return "New Scene";
  if (kind === "ui-theme") return "New UI Theme";
  if (kind === "ui-document") return "New UI Document";
  if (kind === "ui-main-menu") return "Main Menu";
  if (kind === "ui-hud") return "New HUD";
  if (kind === "ui-dialog") return "New Dialog";
  if (kind === "ui-component") return "New UI Component";
  if (kind === "font") return "New Font";
  if (kind === "script") return "New Script";
  return "New Item";
}

export function defaultItemIdForKind(kind: AddItemKind): string {
  if (kind === "scene") return "new-scene";
  if (kind === "ui-theme") return "new-theme";
  if (kind === "ui-document") return "new-ui";
  if (kind === "ui-main-menu") return "main-menu";
  if (kind === "ui-hud") return "new-hud";
  if (kind === "ui-dialog") return "new-dialog";
  if (kind === "ui-component") return "new-ui-component";
  if (kind === "font") return "new-font";
  if (kind === "script") return "new-script";
  return "new-item";
}

export function initialValues(
  kind: AddItemKind,
  scope?: AddItemScope,
  prefillRawFilePath?: string,
): AddItemFormValue {
  return {
    itemId: defaultItemIdForKind(kind),
    label: defaultLabelForKind(kind),
    targetFolder: initialTargetFolder(kind, scope),
    createScript: kind === "scene",
    launcherVisible: false,
    sourceFilePath: prefillRawFilePath ?? "",
    descriptorKind: "image",
    importOptions: {
      tileWidth: 32,
      tileHeight: 32,
      columns: 1,
      rows: 1,
      tileCount: 1,
    },
  };
}

export function resolveInitialItemId(
  kind: AddItemKind,
  values: AddItemFormValue,
  isItemIdTaken?: (payload: {
    kind: AddItemKind;
    itemId: string;
    targetFolder: string;
    descriptorKind: "image" | "sprite" | "tileset";
  }) => boolean,
): string {
  if (!isItemIdTaken) return defaultItemIdForKind(kind);
  return nextAvailableSlugId(defaultItemIdForKind(kind), (candidate) => (
    isItemIdTaken({
      kind,
      itemId: candidate,
      targetFolder: values.targetFolder.trim(),
      descriptorKind: values.descriptorKind,
    })
  ));
}

function joinPath(base: string, tail: string): string {
  if (!base.trim()) return tail;
  return `${base.replace(/\/+$/, "")}/${tail.replace(/^\/+/, "")}`;
}

function sourceFileName(sourcePath: string): string {
  const normalized = sourcePath.split("\\").join("/");
  const last = normalized.split("/").pop()?.trim();
  return last || "source.file";
}

export function createdPathsPreview(kind: AddItemKind, values: AddItemFormValue): string[] {
  const id = normalizeSlugId(values.itemId);
  if (!id) return [];

  if (kind === "scene") {
    const root = joinPath(values.targetFolder || "scenes", id);
    const files = [`${root}/scene.yml`];
    if (values.createScript) files.push(`${root}/scene.rhai`);
    return files;
  }

  if (kind === "script") {
    return [joinPath(values.targetFolder || "scripts", `${id}.rhai`)];
  }

  if (kind === "font") {
    return [joinPath(values.targetFolder || "fonts", `${id}/font.yml`)];
  }

  if (kind === "ui-theme") {
    return [joinPath(values.targetFolder || "ui/themes", `${id}.yml`)];
  }

  if (kind === "ui-document") {
    return [joinPath(values.targetFolder || "ui/documents", `${id}.yml`)];
  }

  if (kind === "ui-main-menu") {
    return [joinPath(values.targetFolder || "ui/menus", `${id}.yml`)];
  }

  if (kind === "ui-hud") {
    return [joinPath(values.targetFolder || "ui/hud", `${id}.yml`)];
  }

  if (kind === "ui-dialog") {
    return [joinPath(values.targetFolder || "ui/dialogs", `${id}.yml`)];
  }

  if (kind === "ui-component") {
    return [joinPath(values.targetFolder || "ui/components", `${id}.yml`)];
  }

  if (kind === "raw-source") {
    return [joinPath(values.targetFolder || "raw", sourceFileName(values.sourceFilePath))];
  }

  if (kind === "folder") {
    return [`${joinPath(values.targetFolder, id)}/`];
  }

  if (kind === "image") {
    return [`descriptor:${values.descriptorKind}:${id}`];
  }

  return [];
}
