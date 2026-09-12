import { createReadStream, existsSync, readFileSync } from "node:fs";
import { readFile, realpath, stat } from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(import.meta.url));
const devProbePath = "/.well-known/tuffite-dev";
const devTokenHeader = "x-tuffite-dev-token";

export function tuffite(options = {}) {
  const manifestPath = options.manifest ?? findManifest(options.root ?? process.cwd());
  const applicationRoot = manifestPath
    ? dirname(manifestPath)
    : resolve(options.root ?? process.cwd());
  const manifest = manifestPath
    ? JSON.parse(readFileSync(manifestPath, "utf8"))
    : undefined;
  const mounts = [
    ["/tuffite-runtime.js", resolve(packageRoot, "runtime.js")],
    ["/tuffite-shell-api.js", resolve(packageRoot, "generated/index.js")],
  ];
  const manifestStatic = Object.fromEntries(
    (manifest?.bundle?.resources ?? []).map((root) => [
      `/${root.replaceAll("\\", "/").replace(/^\/+|\/+$/g, "")}/`,
      root,
    ]),
  );
  const staticRoots = Object.entries(options.static ?? manifestStatic).map(([prefix, root]) => [
    prefix.endsWith("/") ? prefix : prefix + "/",
    resolve(applicationRoot, root),
  ]);
  const shellApi = options.shellApi ??
    (manifest?.app?.shellApi
      ? { url: "/shell_api.js", file: "generated/shell_api.js" }
      : undefined);
  const generatedMount = shellApi
    ? [shellApi.url, resolve(applicationRoot, shellApi.file)]
    : undefined;
  const buildModules = new Map([...mounts, ...(generatedMount ? [generatedMount] : [])]);
  const virtualPrefix = "\0tuffite-runtime:";
  return {
    name: "tuffite-dev-runtime",
    buildStart() {
      if (generatedMount && !existsSync(generatedMount[1])) {
        throw new Error(`Missing application Shell API bindings: ${generatedMount[1]}. Build the application with Cargo first so build.rs generates them, then start or build the frontend.`);
      }
    },
    resolveId(id) {
      return buildModules.has(id) ? `${virtualPrefix}${id}` : undefined;
    },
    async load(id) {
      if (!id.startsWith(virtualPrefix)) return undefined;
      const source = buildModules.get(id.slice(virtualPrefix.length));
      return source ? readFile(source, "utf8") : undefined;
    },
    async generateBundle() {
      if (!generatedMount) return;
      this.emitFile({
        type: "asset",
        fileName: generatedMount[0].replace(/^\/+/, ""),
        source: await readFile(generatedMount[1], "utf8"),
      });
    },
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        const pathname = decodeRequestPath(request.url);
        if (pathname === undefined) {
          response.statusCode = 400;
          response.end("Bad request");
          return;
        }
        if (pathname === devProbePath) {
          const token = process.env.TUFFITE_DEV_TOKEN;
          if (!token) return next();
          response.statusCode = 204;
          response.setHeader(devTokenHeader, createHash("sha256").update(token).digest("hex"));
          response.end();
          return;
        }
        const framework = mounts.find(([url]) => url === pathname);
        if (framework) return sendJavaScript(framework[1], response, next);
        const generated = generatedMount && pathname === generatedMount[0]
          ? generatedMount[1]
          : undefined;
        if (generated) return sendJavaScript(generated, response, next);
        for (const [prefix, root] of staticRoots) {
          if (!pathname.startsWith(prefix)) continue;
          const relative = pathname.slice(prefix.length);
          const candidate = resolve(root, relative || "index.html");
          if (candidate !== root && !candidate.startsWith(root + sep)) {
            response.statusCode = 403;
            response.end("Forbidden");
            return;
          }
          return sendStaticFile(candidate, response, next, root);
        }
        next();
      });
    },
  };
}

export function findManifest(start) {
  let directory = resolve(start);
  while (true) {
    const candidate = resolve(directory, "tuffite.json");
    if (existsSync(candidate)) return candidate;
    const parent = dirname(directory);
    if (parent === directory) return undefined;
    directory = parent;
  }
}

async function sendStaticFile(path, response, next, confinedRoot) {
  const contentType = contentTypeForPath(path);
  if (contentType) response.setHeader("Content-Type", contentType);
  return sendFile(path, response, next, confinedRoot);
}

export function contentTypeForPath(path) {
  const lower = path.toLowerCase();
  if (lower.endsWith(".js") || lower.endsWith(".mjs"))
    return "text/javascript; charset=utf-8";
  if (lower.endsWith(".css")) return "text/css; charset=utf-8";
  if (lower.endsWith(".json")) return "application/json; charset=utf-8";
  if (lower.endsWith(".svg")) return "image/svg+xml";
  if (lower.endsWith(".html") || lower.endsWith(".htm"))
    return "text/html; charset=utf-8";
  return undefined;
}

async function sendJavaScript(path, response, next) {
  response.setHeader("Content-Type", "text/javascript; charset=utf-8");
  return sendFile(path, response, next);
}

async function sendFile(path, response, next, confinedRoot) {
  try {
    if (confinedRoot) {
      const canonicalPath = await resolveConfinedFile(confinedRoot, path);
      if (!canonicalPath) {
        response.statusCode = 403;
        response.end("Forbidden");
        return;
      }
      path = canonicalPath;
    }
    if (!(await stat(path)).isFile()) return next();
    createReadStream(path).pipe(response);
  } catch (error) {
    if (error?.code === "ENOENT") return next();
    next(error);
  }
}

export function decodeRequestPath(requestUrl) {
  try {
    return decodeURIComponent(new URL(requestUrl ?? "/", "http://127.0.0.1").pathname);
  } catch {
    return undefined;
  }
}

export async function resolveConfinedFile(root, path) {
  const [canonicalRoot, canonicalPath] = await Promise.all([realpath(root), realpath(path)]);
  return canonicalPath === canonicalRoot || canonicalPath.startsWith(canonicalRoot + sep)
    ? canonicalPath
    : undefined;
}
