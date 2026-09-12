# tuffite

JavaScript package for Tuffite applications. Its `tuffite/runtime` export is
used by generated ShellAPI facades and implements the binary request envelope,
deterministic codecs, and immutable Proxy trees.

Applications normally do not import the runtime directly. The generation phase
of `tuff build`, `package`, and `dev` creates the typed facade, and packaging injects this module at the
framework-owned `/tuffite-runtime.js` URL together with the generated built-in
facade at `/tuffite-shell-api.js`. The npm export exists for tooling, bundler
integration, and direct low-level tests. No additional Tuffite npm packages are
part of the public surface.

`tuffite/vite` exports a development-only Vite plugin. It serves the framework
runtime at `/tuffite-runtime.js` and `/tuffite-shell-api.js`, can mount an
application generated facade and static resources, and is active only for the
Vite development server. This lets a packaged native application load a
loopback Vite page with HMR without copying framework scripts into source code.
The plugin also exposes the authenticated per-run readiness probe used by
`tuff dev`; a pre-existing server cannot be mistaken for the privileged dev
page. Static mounts are canonicalized before serving, including symlink and
junction targets.
