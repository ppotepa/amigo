import { build, context } from "esbuild";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, "..");
const distDir = join(rootDir, "dist");
const watchMode = process.argv.includes("--watch");

function ensureDir(path) {
  if (!existsSync(path)) mkdirSync(path, { recursive: true });
}

function copyStaticAssets() {
  ensureDir(distDir);
  ensureDir(join(distDir, "assets"));

  const rootIndex = readFileSync(join(rootDir, "index.html"), "utf8")
    .replace("./dist/styles.css", "./styles.css")
    .replace("./dist/app.js", "./app.js");

  writeFileSync(join(distDir, "index.html"), rootIndex);
  cpSync(join(rootDir, "assets", "source-rig.svg"), join(distDir, "assets", "source-rig.svg"));
  cpSync(join(rootDir, "src", "styles.css"), join(distDir, "styles.css"));
}

const sharedConfig = {
  entryPoints: [join(rootDir, "src", "main.ts")],
  outfile: join(distDir, "app.js"),
  bundle: true,
  format: "iife",
  target: "es2020",
  sourcemap: false,
  minify: false,
  logLevel: "info",
};

rmSync(distDir, { recursive: true, force: true });
copyStaticAssets();

if (watchMode) {
  const ctx = await context(sharedConfig);
  await ctx.watch();
  console.log("watching character-lab");
} else {
  await build(sharedConfig);
}
