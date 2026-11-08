# Shared Framework modules

This directory holds process-neutral helpers and contracts shared across process
roles. Consumers depend on each module's GN target; there is no umbrella common
target and no application service implementation here.

| Module      | Target                                 | Responsibility                                                                       |
| ----------- | -------------------------------------- | ------------------------------------------------------------------------------------ |
| switches.h  | :switches                              | Command-line contract shared by bootstrap, browser and renderer                      |
| languages.h | :languages                             | Ordered language preference shared by browser networking and renderer preferences    |
| resource.h  | :resource                              | Shared in-memory resource value, without resolver callbacks                          |
| utility     | utility:mojom and utility:service_name | Browser/utility-process IPC and shared service identity; generated paths stay stable |

Application declarations, resource routing, configuration snapshots, and access
rule normalization/matching belong to `components/manifest`. ManifestService
retains application service lifetime and authorization queries. URLService owns
resource resolver callbacks; `resource.h` carries only the shared resource value.

ContentClient is an embedder adapter owned by MainDelegate in app/content_client.
The development token environment key is private bootstrap state in MainDelegate.
Neither belongs in the shared modules.
