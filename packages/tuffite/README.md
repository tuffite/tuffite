# tuffite

JavaScript package for Tuffite applications. The root `tuffite` export combines
typed Framework clients, generated Mojom descriptors and their shared codec.
The handwritten runtime is an internal generator input; no separate runtime
module is published.

This directory is the SDK package in the repository's private npm workspace.
Its `package.json` is the sole definition of the SDK exports. Run
`bun install --frozen-lockfile` at the repository root to install the SDK and
Playground UI workspaces using the shared root `bun.lock`.

A dependency on `git+https://github.com/tuffite/tuffite.git#<commit>` installs
the repository root. Git consumers must resolve this child package's exports;
workspace membership alone does not expose `tuffite/vite` from the root package.

Import Framework APIs from `tuffite`, or from `/tuffite.js` in unbundled native
pages. Application `build.rs` generates one `generated/index.js` and one
`generated/index.d.ts` from `build.mojomSources`. The application module imports
the same Framework SDK at `/tuffite.js`; importing either installs no business
globals. No JavaScript/TypeScript output inventory files are needed.

`tuffite/vite` serves `/tuffite.js` during development and mounts the application's
generated module and static resources. It also resolves those modules for
production bundling. Native packaging injects the version-matched SDK resource.
The plugin also exposes the authenticated per-run readiness probe used by
`tuff dev`; a pre-existing server cannot be mistaken for the privileged dev
page. Static mounts are canonicalized before serving, including symlink and
junction targets.
