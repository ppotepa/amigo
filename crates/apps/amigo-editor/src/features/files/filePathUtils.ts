export function normalizePath(path: string): string {
  return path.replace(/\\/g, "/");
}

export function fileExtension(fileName: string): string {
  const index = fileName.lastIndexOf(".");
  return index >= 0 ? fileName.slice(index).toLowerCase() : "";
}
