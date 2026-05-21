import { build, context } from "esbuild";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, "..");
const distDir = join(rootDir, "dist");
const watchMode = process.argv.includes("--watch");
const devHost = "127.0.0.1";
const devPort = Number(process.env.PORT ?? 4174);

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

const mimeTypes = new Map([
  [".css", "text/css; charset=utf-8"],
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".svg", "image/svg+xml; charset=utf-8"],
]);

function contentType(path) {
  const ext = path.includes(".") ? path.slice(path.lastIndexOf(".")).toLowerCase() : "";
  return mimeTypes.get(ext) ?? "application/octet-stream";
}

function serveDist() {
  let port = devPort;
  const server = createServer((request, response) => {
    const url = new URL(request.url ?? "/", `http://${devHost}:${port}`);
    const pathname = decodeURIComponent(url.pathname);
    const relativePath = pathname === "/" ? "index.html" : pathname.replace(/^\/+/, "");
    const filePath = join(distDir, relativePath);

    if (!filePath.startsWith(distDir)) {
      response.writeHead(403);
      response.end("Forbidden");
      return;
    }

    try {
      const body = readFileSync(filePath);
      response.writeHead(200, { "Content-Type": contentType(filePath) });
      response.end(body);
    } catch {
      response.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
      response.end("Not found");
    }
  });

  server.on("error", error => {
    if (error.code === "EADDRINUSE" && port < devPort + 20) {
      port += 1;
      server.listen(port, devHost);
      return;
    }
    console.error(`could not start dev server on http://${devHost}:${port}/`);
    console.error(error.message);
    process.exit(1);
  });

  server.listen(port, devHost, () => {
    console.log(`character-lab dev server: http://${devHost}:${port}/`);
  });
}

rmSync(distDir, { recursive: true, force: true });
copyStaticAssets();

if (watchMode) {
  const ctx = await context(sharedConfig);
  await ctx.watch();
  serveDist();
  console.log("watching character-lab");
} else {
  await build(sharedConfig);
}
