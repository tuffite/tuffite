# Tuffite BrowserContext services

This directory owns per-profile browser services: `client_hints`, `download`,
`file_chooser`, `javascript_dialog`, `network_context`, `permission`, `shell_api`,
`webauthn` and `window`. Their factories use Chromium's BrowserContext
keyed-service graph.
`devtools` and `context_menu` also live here because they implement browser and
WebContents interactions; they use BrowserContext factories. ContextMenu depends
on DevTools and borrows
application policy through constructor injection. Windows shut down before menus,
and menus shut down before DevTools. The process-wide debugging server uses a
weak ownership lease, separate from the profile service lifetime.
Application-scoped services and the shared typed factory infrastructure live in
[app/services](../../app/services/README.md).

A BrowserContext service can borrow application service interfaces because its
profile is destroyed before the owning Application. `DependsOn()` orders services
only within the same scope; it must not connect the two dependency graphs.
Service implementations can depend on `browser:support` and application service
interfaces, but never on `browser:browser` or `app:app`.

NetworkContextService owns encrypted Cookie persistence and the profile's HTTP
cache. On macOS its Safe Storage key has stable application-identifier lookup
attributes and a `productName + " Safe Storage"` display label. Product renames
update the label without rotating the key. A profile with existing encrypted
cookies can copy the former Chromium password into its application item once;
this may require one legacy Keychain consent. The Chromium item is never changed.
Failed Keychain access does not enable plaintext or known-key fallback.

FileChooserService supplies native HTML file input selection to Window and
Explorer. Each session observes its requesting document, cancels on navigation,
frame destruction or service shutdown, and completes its listener once. Windows
shut down before the chooser and JavaScript dialog services through factory
dependencies. File System Access grants remain separate.

PermissionService currently supports activation-bound sanitized clipboard writes.
Other capabilities remain denied until their backend, consent and revocation
semantics exist. Its diagnostic logs contain permission types and decisions,
without URLs or authentication data.
