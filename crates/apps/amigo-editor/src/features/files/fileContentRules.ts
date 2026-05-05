import type { EditorProjectFileDto } from "../../api/dto";
import { fileExtension } from "./filePathUtils";

export const TEXT_EXTENSIONS = new Set([
  ".toml",
  ".yml",
  ".yaml",
  ".rhai",
  ".json",
  ".md",
  ".txt",
  ".ron",
]);

export function canReadProjectFileContent(file: EditorProjectFileDto): boolean {
  return TEXT_EXTENSIONS.has(fileExtension(file.name));
}
