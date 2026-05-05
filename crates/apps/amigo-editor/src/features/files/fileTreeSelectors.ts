import type { EditorProjectFileDto } from "../../api/dto";
import { canReadProjectFileContent } from "./fileContentRules";

export type FileDiagnosticLike = {
  level: "info" | "warning" | "error";
  code: string;
  message: string;
  path?: string | null;
};

export function flattenProjectFiles(root: EditorProjectFileDto): EditorProjectFileDto[] {
  return root.children.flatMap((child) => [child, ...flattenProjectFiles(child)]).filter((file) => !file.isDir);
}

export function normalizePath(path: string): string {
  return path.replace(/\\/g, "/");
}

export function findProjectFile(root: EditorProjectFileDto, relativePath: string): EditorProjectFileDto | null {
  if (root.relativePath === relativePath) {
    return root;
  }

  for (const child of root.children) {
    const match = findProjectFile(child, relativePath);
    if (match) {
      return match;
    }
  }

  return null;
}

export function fileDiagnosticsFor(
  file: EditorProjectFileDto,
  content?: { diagnostics: FileDiagnosticLike[] },
): FileDiagnosticLike[] {
  const diagnostics = [...(content?.diagnostics ?? [])];
  if (file.kind === "unknown") {
    diagnostics.push({
      level: "warning",
      code: "unknown_project_file",
      message: `File type for ${file.relativePath} is not recognized by the editor yet.`,
      path: file.relativePath,
    });
  }
  if (canReadProjectFileContent(file) && !content) {
    diagnostics.push({
      level: "info",
      code: "text_preview_pending",
      message: `Text preview for ${file.relativePath} is not loaded yet.`,
      path: file.relativePath,
    });
  }
  return diagnostics;
}
