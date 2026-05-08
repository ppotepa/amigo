import { execFileSync } from node:child_process;

const forbidden = [
  String.raw`component\.typeName\s*===\s*[']`,
