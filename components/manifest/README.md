# Manifest component

Owns portable application manifest models, resource routing/access rules, and
process/document snapshot transports. Application-specific loading, startup
resolution, frozen `EffectiveManifest` ownership and `ManifestService` lifecycle
belong to `framework/app/services/manifest`. Component sources and targets do not
depend on Framework services or name their implementation classes.

- `common:configuration`: the actual `Manifest` model and application projections.
- `common:assets`: declarative resource routes, scheme matching and CSP rules.
- `common:access`: normalization and matching of manifest access rules.
- `bootstrap`: bounded process snapshot codec and read-only shared-memory
  ownership, available before Chromium scheme registration. The loader reads
  minimal package declarations before the full configuration can be assembled.
- `common:mojom`: minimal document exposure snapshot. Renderer consumers do not
  link the manifest loader or receive full manifests and grant lists.
- `browser/content` and `renderer/blink`: document snapshot delivery and reading
  through existing PolicyContainer creation/commit messages. Framework supplies
  the browser callback that combines manifest grants with document origin,
  WebContents role and Explorer policy.

Shared in-memory resource values live in Framework's `common/resource.h`, outside
the manifest schema. The browser URL service owns resource resolver callbacks.

## File naming

Use the full `configuration` spelling for configuration values and snapshots.
File names identify their role rather than using an umbrella configuration header:

- `application_configuration.h`, `asset_configuration.h` and `manifest.h`
  define application projections, resource routing and the actual manifest model.
- Framework's `app/services/manifest/manifest_loader.*` loads, validates and
  resolves startup options in one call, returning `EffectiveManifest`. The
  service owns the frozen result; no independent assembler class is needed.
- `bootstrap/process_configuration.*` implements the process snapshot transport.
- `browser/content/document_configuration_provider.*` supplies document exposure;
  `renderer/blink/document_configuration_reader.*` reads that snapshot.
- Tests use the corresponding production file name plus `_unittest.cc`; the
  Content conversion test remains under `renderer/content` for its narrow private
  dependency exception. Chromium integration lists retain `tuffite_in_chromium.gni`.
