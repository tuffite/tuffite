import { createReadStream, existsSync, readFileSync, statSync } from "node:fs";
import { readFile, realpath, stat } from "node:fs/promises";
import { createHash } from "node:crypto";
import { basename, dirname, resolve, sep } from "node:path";
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
    ["/tuffite.js", resolve(packageRoot, "generated/index.js")],
  ];
  const staticRoots = options.static
    ? Object.entries(options.static).map(([prefix, root]) => ({
        prefix: prefix.endsWith("/") ? prefix : prefix + "/",
        root: resolve(applicationRoot, root), file: false, excludes: [], source: root,
      }))
    : resourceMounts(manifest?.bundle?.resources ?? {}, applicationRoot);
  const shellApi = options.shellApi ??
    (manifest?.build?.mojomSources?.length
      ? { url: "/shell_api.js", file: "generated/index.js" }
      : undefined);
  const generatedMount = shellApi
    ? [shellApi.url, resolve(applicationRoot, shellApi.file)]
    : undefined;
  const generatedMounts = generatedMount ? [generatedMount] : [];
  const buildModules = new Map([...mounts, ...generatedMounts]);
  const virtualPrefix = "\0tuffite-runtime:";
  return {
    name: "tuffite-dev-runtime",
    buildStart() {
      if (generatedMount && !existsSync(generatedMount[1])) {
        throw new Error(`Missing application Shell API bindings: ${generatedMount[1]}. Build the application with Cargo first so build.rs generates them, then start or build the frontend.`);
      }
    },
    resolveId(id, importer) {
      if (id.startsWith(".") && importer?.startsWith(virtualPrefix)) {
        const pathname = new URL(id, `http://tuffite.local${importer.slice(virtualPrefix.length)}`).pathname;
        if (buildModules.has(pathname)) return `${virtualPrefix}${pathname}`;
      }
      return buildModules.has(id) ? `${virtualPrefix}${id}` : undefined;
    },
    async load(id) {
      if (!id.startsWith(virtualPrefix)) return undefined;
      const source = buildModules.get(id.slice(virtualPrefix.length));
      return source ? readFile(source, "utf8") : undefined;
    },
    async generateBundle() {
      for (const [url, source] of generatedMounts) {
        this.emitFile({
          type: "asset",
          fileName: url.replace(/^\/+/, ""),
          source: await readFile(source, "utf8"),
        });
      }
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
        const generated = generatedMounts.find(([url]) => pathname === url);
        if (generated) return sendJavaScript(generated[1], response, next);
        for (const mount of staticRoots) {
          const { prefix, root } = mount;
          if (mount.file ? pathname !== prefix : !pathname.startsWith(prefix)) continue;
          const child = mount.file ? "" : pathname.slice(prefix.length) || "index.html";
          const candidate = mount.file ? root : resolve(root, child);
          if (resourceExcluded(mount.excludes, [mount.source + (child ? "/" + child : ""), child])) continue;
          if (candidate !== root && !candidate.startsWith(root + sep)) {
            response.statusCode = 403;
            response.end("Forbidden");
            return;
          }
          if (!existsSync(candidate)) continue;
          return sendStaticFile(candidate, response, next, mount.file ? dirname(root) : root);
        }
        next();
      });
    },
  };
}

// Mirror packaging aliases when publishing resources through the development server.
export function resourceMounts(resources, applicationRoot) {
  if (!resources || typeof resources !== "object" || Array.isArray(resources)) throw new TypeError("bundle.resources must be an alias object");
  const mounts = [];
  for (const [alias, sources] of Object.entries(resources)) {
    if (alias !== "." && !safeResourcePath(alias)) throw new TypeError("unsafe resource alias");
    if (!Array.isArray(sources) || sources.some(source => typeof source !== "string" || !source)) throw new TypeError("resource sources must be nonempty strings");
    const excludes = sources.filter(source => source.startsWith("!")).map(source => source.slice(1));
    if (excludes.some(pattern => !pattern)) throw new TypeError("empty resource exclusion");
    for (const source of sources.filter(source => !source.startsWith("!"))) {
      if (!safeResourcePath(source)) throw new TypeError("unsafe resource source");
      const root = resolve(applicationRoot, source);
      const file = existsSync(root) && statSync(root).isFile();
      const prefix = (alias === "." ? "/" : "/" + alias.replace(/\/$/, "") + "/") + (file ? basename(root) : "");
      mounts.push({ prefix, root, file, excludes, source });
    }
  }
  return mounts;
}
function safeResourcePath(path) {
  return !path.includes("\\") && !path.includes(":") && !path.startsWith("/") && path.split("/").every(part => part && part !== "." && part !== "..");
}
export function resourceExcluded(patterns, paths) {
  const expressions = patterns.map(globExpression);
  return paths.some(path => {
    const segments = path.split("/");
    return segments.some((_, index) => expressions.some(expression => expression.test(segments.slice(0, index + 1).join("/"))));
  });
}
function globExpression(pattern) {
  let expression = "^";
  const segments = pattern.split("/");
  for (const [index, segment] of segments.entries()) {
    if (segment === "**") {
      expression += index === segments.length - 1 ? ".*" : "(?:[^/]+/)*";
      continue;
    }
    for (let cursor = 0; cursor < segment.length; cursor++) {
      const character = segment[cursor];
      if (character === "*") expression += "[^/]*";
      else if (character === "?") expression += "[^/]";
      else if (character === "[") {
        const end = segment.indexOf("]", cursor + 1);
        if (end < 0) throw new TypeError("invalid resource exclusion");
        expression += "[" + segment.slice(cursor + 1, end).replace(/^!/, "^") + "]";
        cursor = end;
      } else expression += character.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }
    if (index !== segments.length - 1) expression += "/";
  }
  return new RegExp(expression + "$");
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
