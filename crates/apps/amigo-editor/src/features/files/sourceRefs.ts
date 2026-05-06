export type SourceFileKind =
  | "yaml"
  | "script"
  | "raw"
  | "folder";

export type SourceFileRef = {
  kind: SourceFileKind;
  label: string;
  path: string;
  title?: string;
};

export function sourceFileName(path: string | null | undefined): string {
  const value = path ?? "";
  const parts = value.split(/[\\/]/).filter(Boolean);
  return parts.length ? parts[parts.length - 1] : value;
}
