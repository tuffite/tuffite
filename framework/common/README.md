# Shared Framework modules

This directory holds process-neutral helpers and contracts shared across process
roles. Consumers depend on each module's GN target; there is no umbrella common
target and no application service implementation here.

| Module | Target | Responsibility |
| --- | --- | --- |
| switches.h | :switches | Command-line contract shared by bootstrap, browser and renderer |
| languages.h | :languages | Ordered language preference shared by browser networking and renderer preferences |
| access | :access | Scope rule normalization/matching and root-relative filesystem component checks |
| utility | utility:mojom and utility:service_name | Browser/utility-process IPC and shared service identity; generated paths stay stable |

Application configuration belongs to app/services/manifest/app_config.h under
`tuffite::app`, exposed by the manifest target. AppInfo, AppWindow and NetworkLimits
are application configuration projections rather than generic common types.
Resource routing and copyable snapshots belong to app/services/asset under
`tuffite::asset`, in its configuration target. Separating that target from the
AssetService implementation lets ManifestService consume resource configuration
without a service dependency cycle.

ContentClient is an embedder adapter owned by MainDelegate in app/content_client.
The development token environment key is private bootstrap state in MainDelegate.
Neither belongs in the shared modules.

`access.h` groups shared access helpers under `tuffite::access`. Origin, URL and
file-access rule matching is pure; HasNoSymlinkComponents queries the filesystem
and runs on a sequence that permits blocking I/O. ManifestService owns grant
resolution and admission decisions; file operations remain in their handlers.
