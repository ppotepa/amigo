import type { AddItemKindDto, CreateAssetImportOptionsDto } from "../api/dto";

export type AddItemKind = AddItemKindDto;

export type AssetCategoryScope =
  | "scenes"
  | "fonts"
  | "raw"
  | "spritesheets"
  | "tilemaps"
  | "audio"
  | "scripts";

export type AddItemScope =
  | { kind: "project-root" }
  | { kind: "asset-category"; category: AssetCategoryScope }
  | { kind: "project-folder"; path: string }
  | { kind: "asset"; assetKey: string };

export type AddItemMode = "catalog" | "direct";

export interface AddItemDialogRequest {
  mode: AddItemMode;
  scope?: AddItemScope;
  itemKind?: AddItemKind;
  prefillRawFilePath?: string;
  prefillDescriptorKind?: "image" | "sprite" | "tileset";
}

export interface AddItemFormValue {
  itemId: string;
  label: string;
  targetFolder: string;
  createScript: boolean;
  launcherVisible: boolean;
  sourceFilePath: string;
  descriptorKind: "image" | "sprite" | "tileset";
  importOptions: CreateAssetImportOptionsDto;
}
