import type React from "react";
import type { AssetVisualTone } from "../../assets/assetVisualRegistry";

export type FolderViewStatus = "valid" | "warning" | "error" | "missing" | "muted";

export type FolderViewDensity = "compact" | "comfortable";

export type FolderViewThumbnailMode = "contain" | "cover" | "pixel";

export type FolderViewAction = {
  id: string;
  label: string;
  onRun: () => void;
};

export type FolderViewItem = {
  id: string;
  title: string;
  subtitle?: string;
  thumbnailSrc?: string;
  icon?: React.ReactNode;
  status?: FolderViewStatus;
  tone?: AssetVisualTone;
  selected?: boolean;
  kind?: string;
  onOpen: () => void;
  actions?: FolderViewAction[];
};

export type FolderViewGroup = {
  id: string;
  title: string;
  subtitle?: string;
  icon?: React.ReactNode;
  items: FolderViewItem[];
};
