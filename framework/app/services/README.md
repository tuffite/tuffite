# Tuffite application service architecture

This directory contains application-scoped services owned by Tuffite. They run
in the browser process but are shared by every BrowserContext owned by the
application. The service
architecture reuses Chromium's keyed-service infrastructure for factory
registration, dependency ordering, lazy construction, test replacement, and
two-phase shutdown. Tuffite adds only an application-specific context,
dependency manager, and factory adapter.

## GN targets

Each application service directory owns one `source_set`: `manifest`, `asset`,
`utility` and `remoting`. Public lists
expose service APIs and factory entry points; implementation headers stay
private. BrowserContext services live separately in `framework/browser/services`, together
with browser-facing DevTools and context menu services. They use the BrowserContext dependency graph.
Both sets can depend on the application context and typed factory targets here,
without depending on `app:app` or the browser composition target.

Shared application infrastructure has separate targets:

- `app:context` exposes typed service lookup and application inputs.
- `services:factory` exposes the typed keyed-service factory adapter.
- `services:manager` owns registration and Chromium lifecycle coordination.
- `app:app` implements context construction and registers concrete factories.

`app:app` and `services:factory` depend privately on `services:manager`; public context headers
contain no manager operations. The manager's `visibility` permits only those
infrastructure targets and `app:app`. Service implementation
and handler targets can include the context but cannot include the manager header
or add a direct manager dependency. Concrete factory headers remain public so
other factories can declare `DependsOn()` without accessing the manager.

The private factory base declaration is shared by the manager implementation and
the typed factory template. Its constructor registers the type key out of line,
and context lookup delegates to the manager out of line. This keeps all service
storage and shutdown ordering in Chromium's existing keyed-service machinery.

`//tuffite/framework/browser:support` owns BrowserContext declarations and
WebContents role tagging, below the services and Shell API handlers. Shell API
commands in `browser/services/shell_api/commands` depend on manifest, utility, remoting, and window
services; the
`browser/services/shell_api` service then depends on those handlers. Keeping this
ordering avoids a cycle through the browser composition root.

Depending on a service target also links its implementation dependencies, even
when the caller only includes an interface header. `public` limits header
access; it does not create an independently linkable interface library.

## Goals

- Consumers program against service interfaces rather than implementations.
- One browser process may host more than one Tuffite `MainDelegate`.
- Application services are isolated from services belonging to another
  application.
- Factories are registered at one explicit composition root.
- Service instances may be created at different startup phases without
  changing the factory graph after startup begins.
- Dependencies determine construction and destruction order.
- BrowserContext and document services continue to use Chromium's native
  lifecycle mechanisms.

This is not intended to be a general-purpose runtime DI container. In
particular, service registration is static for the lifetime of the process.

## Scope model

Use the Chromium mechanism matching the lifetime of the service:

| Scope | Mechanism |
| --- | --- |
| Application | `ApplicationDependencyManager` and `ApplicationKeyedServiceFactory` |
| BrowserContext | `BrowserContextDependencyManager` and `BrowserContextKeyedServiceFactory` |
| WebContents | explicit ownership or `WebContentsUserData` |
| Document/frame | `DocumentService` |
| Utility process | Mojo remote owned by an application service |

Application and BrowserContext services are separate dependency graphs. A
`DependsOn()` edge must not cross between those graphs. Their ordering is
instead guaranteed by ownership: application services are created before any
of their BrowserContexts and destroyed after all of them.

`permission/PermissionService` is a BrowserContext service implementing
Chromium's `PermissionControllerDelegate`. Standard clipboard writes receive
a one-request grant only for an active, focused, visible document with fresh
user activation and a trustworthy origin. Clipboard reads, unsanitized writes,
worker requests and other permission types are denied until their own policies
and consent UI are implemented. It stores no persistent grants.

`client_hints/ClientHintsService` implements Chromium's
`ClientHintsControllerDelegate` through Chromium's
`InMemoryClientHintsControllerDelegate` per BrowserContext. It uses the same truthful
embedder User-Agent metadata as JavaScript, tracks network quality, and retains
`Accept-CH` preferences by trustworthy origin for the profile's lifetime.
Chromium generates the headers and applies its secure-context, feature and
Permissions Policy checks; higher entropy hints are not enabled globally.
Preferences are not persisted across application restarts.

## Application service context

`ApplicationContext` identifies one application scope. It is a plain
Tuffite type and does not derive from `SimpleFactoryKey`, because
`SimpleFactoryKey` always registers itself with Chromium's global
`SimpleDependencyManager`.

It owns no services. Chromium's keyed-service factories retain service
instances; the context only carries stable application inputs and exposes the
typed lookup boundary:

```cpp
class ApplicationContext final {
 public:
  template <typename Service>
  Service* GetService(bool create = true) {
    return static_cast<Service*>(
        GetServiceByType(internal::ApplicationServiceTypeKey<Service>(), create));
  }

  AbiHostBridge& bridge() const;

 private:
  KeyedService* GetServiceByType(const void* service_type, bool create);

  raw_ref<AbiHostBridge> bridge_;
};
```

Consumers include this context and the requested service interface. They do not
include the concrete `XxxServiceFactory`. `MainDelegate` owns its context,
and every Tuffite BrowserContext for that application retains a non-owning
reference. There is no process-global active application.

## Application dependency manager

`ApplicationDependencyManager` is the process-lifetime composition
registry and lifecycle facade over Chromium's `DependencyManager`. Its
`Get()` address is stable for the process lifetime.

Each typed factory registers one interface type key with the manager. The
registry maps an interface type to its factory only; it does not own service
instances or implement another shutdown graph. Lookup delegates immediately to
the registered Chromium factory, whose existing context map owns the service.

```cpp
class ApplicationDependencyManager final : public DependencyManager {
 public:
  static ApplicationDependencyManager* Get();

  void CreateServices(ApplicationContext*);
  void CreateServicesForTest(ApplicationContext*);
  void DestroyServices(ApplicationContext*);
};
```

Context lifecycle continues to use `MarkContextLive()`,
`CreateContextServices()`, and `DestroyContextServices()`. Tuffite must
not add a second service-instance map or shutdown algorithm.

## Application keyed-service factory

`ApplicationKeyedServiceFactory<Service>` adapts Chromium's supported
`KeyedServiceFactory` specialization to `ApplicationContext`.
This Chromium revision does not link arbitrary
`KeyedServiceTemplatedFactory<Interface>` instantiations, so the adapter
keeps the opaque context cast in one place.

Its constructor also registers `Service`'s type key with
`ApplicationDependencyManager`. The untyped lookup entry is private to the
manager; normal callers use `ApplicationContext::GetService<T>()`.

`ApplicationDependency` is the thin type-erased base required for that private
manager-to-factory call: Chromium keeps `GetServiceForContext()` protected, so
the manager cannot store unrelated typed factories directly. It owns no service
and adds no lifecycle behavior.

```cpp
template <typename Service>
class ApplicationKeyedServiceFactory
    : public ApplicationDependency {
 public:
  using TestingFactory =
      base::OnceCallback<std::unique_ptr<Service>(
          ApplicationContext&)>;

  void SetTestingFactoryForApplication(
      ApplicationContext* context,
      TestingFactory factory);

 protected:
  explicit ApplicationKeyedServiceFactory(const char* name)
      : ApplicationDependency(
            name, internal::ApplicationServiceTypeKey<Service>()) {}

  virtual std::unique_ptr<Service>
  BuildServiceInstanceForApplication(
      ApplicationContext&) const = 0;
};
```

Concrete factories remain explicit graph nodes. They construct implementations
and expose the service interface through the context:

```cpp
class UtilityServiceFactory final
    : public ApplicationKeyedServiceFactory<UtilityService> {
 public:
  static UtilityServiceFactory* Get();

 private:
  friend class base::NoDestructor<UtilityServiceFactory>;
  UtilityServiceFactory();

  std::unique_ptr<UtilityService>
  BuildServiceInstanceForApplication(
      ApplicationContext&) const override;
};

UtilityService* utility =
    application.GetService<UtilityService>();
```

## Dependencies

Concrete application factories use Chromium's inherited `DependsOn()`
directly.

```cpp
UtilityServiceFactory::UtilityServiceFactory()
    : ApplicationKeyedServiceFactory<UtilityService>("TuffiteUtility") {
  DependsOn(ManifestServiceFactory::Get());
}
```

All application factories must inherit `ApplicationKeyedServiceFactory`, whose
constructor fixes the manager to `ApplicationDependencyManager::Get()`.
This convention is important because Chromium's `DependsOn()` checks the
factory type but does not prove that both factories belong to the same manager.

Dependency direction should follow lifetime and construction requirements. If
`UtilityService` needs `ManifestService`, Utility depends on Manifest:

```text
construction: Policy -> Utility
shutdown:     Utility -> Policy
```

`DependsOn()` declares lifecycle ordering; it does not return a service instance.
Resolve required service interfaces with `ApplicationContext::GetService<T>()`
inside the factory's construction method and inject them into the implementation.
The implementation should retain those explicit interface dependencies rather than
repeatedly looking them up through the application context. Other factories may
include public factory headers to declare `DependsOn()`; ordinary service consumers
include only the interface and public types. Cross-scope dependencies use factory
injection and ownership ordering, not `DependsOn()`.

Services must release asynchronous work and borrowed dependencies from
`KeyedService::Shutdown()`. Destructors must not look up other keyed services.

## Central factory registration

All application factories are registered explicitly in the `ApplicationContext`
constructor. It instantiates the manifest, asset, remoting, and utility factories,
then freezes registration through Chromium's dependency manager. Registration
runs before `MainDelegate` creates keyed services. No separate registration entry
point is required.

The required order is:

```text
Construct ApplicationContext (register factories and freeze registration)
ApplicationDependencyManager::CreateServices(context)
Create BrowserContexts
...
Destroy all BrowserContexts
ApplicationDependencyManager::DestroyServices(context)
Destroy ApplicationContext
```

Factories are process-lifetime singletons. Services are per-context instances,
so the same factory naturally maps Application A and Application B to distinct
service objects.

## Startup phases

Factory registration and service instantiation are separate operations. All
factories are built at the composition root before the first application
context becomes live. Services may then be resolved lazily from their owning
context:

```cpp
void CreateEarlyApplicationServices(ApplicationContext& application) {
  application.GetService<ManifestService>();
}

void CreateBrowserReadyApplicationServices(
    ApplicationContext& application) {
  application.GetService<UtilityService>();
}
```

Map these calls to `BrowserMainParts` phases according to actual runtime
requirements. Stop asynchronous services while their required browser threads
still exist; final destruction remains dependency ordered.

## Resolving from ShellAPI and ABI boundaries

ShellAPI receives a `RenderFrameHost`. Resolve the Tuffite BrowserContext
through the centralized framework conversion and then use its application
context:

```text
RenderFrameHost
  -> BrowserContext::GetFrom()
  -> ApplicationContext
  -> GetService<UtilityService>()
  -> UtilityService interface
```

```cpp
auto& browser = BrowserContext::GetFrom(frame);
auto& application = browser.application_context();
auto* utility = application.GetService<UtilityService>();
CHECK(utility);
utility->InvokeUtilityService(/* ... */);
```

`BrowserContext::GetFrom(content::BrowserContext&)` and
`BrowserContext::GetFrom(RenderFrameHost&)` are the only Tuffite
BrowserContext downcast helpers. Call sites must not repeat
`static_cast<BrowserContext&>`.

The ABI bridge has no renderer sender. It retains its owning
`ApplicationContext` and resolves the same interfaces from that
context. It never consults a process-global active key.

Caller identity remains separate from service scope. Utility operations retain
the document token so identical request IDs, cancellation, and teardown cannot
cross documents:

```text
(application context, document token, utility name, request id)
```

## Cross-scope injection

A BrowserContext service may borrow an application service because all
BrowserContexts are destroyed before their owning application context. Resolve
the dependency while building the BrowserContext service and inject only
interfaces:

```cpp
std::unique_ptr<KeyedService>
ShellApiServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  auto& browser = BrowserContext::GetFrom(*context);
  auto& application = browser.application_context();

  return std::make_unique<ShellApiServiceImpl>(
      application.bridge(),
      *application.GetService<ManifestService>(),
      *application.GetService<UtilityService>());
}
```

Do not express cross-scope relationships with `DependsOn()`; the
application and BrowserContext factories belong to separate dependency graphs.
The ownership hierarchy supplies their lifetime ordering.

## Reducing factory boilerplate

`ApplicationKeyedServiceFactory<Service>` centralizes:

- opaque-context conversion;
- service-interface type registration;
- selection of `ApplicationDependencyManager`;
- Chromium's `SIMPLE` factory mode;
- Chromium service storage and testing-factory reuse.

A concrete factory retains only its fixed singleton, dependency edges, and
construction function. Callers never need that concrete type.

Tuffite uses this singleton naming rule:

- `Get()`: the returned address is fixed for the process lifetime, such as
  a `base::NoDestructor` dependency manager or keyed-service factory;
- `GetInstance()`: the current instance can change or be null at runtime,
  typically through a mutable global instance pointer.

Chromium-owned classes retain their upstream method names. For example,
`BrowserContextDependencyManager::GetInstance()` remains unchanged.

Keep concrete factories explicit and instantiate them from
the `ApplicationContext` constructor. Avoid macros and static
auto-registration because graph membership and initialization order must remain
searchable.

## Testing

Reuse `SetTestingFactoryForApplication()` and Chromium's testing-factory storage rather than adding a
second override registry. Tests should be able to install an implementation for
a specific application context before first use.

At minimum, cover:

- two live Applications receive distinct instances from the same factory;
- destruction of one Application does not affect services of another;
- dependency shutdown and destruction occur in reverse dependency order;
- factories cannot be registered after the first application context is live;
- ShellAPI requests resolve through the sender's BrowserContext;
- identical utility request IDs from two documents do not collide;
- destroying a document cancels only that document's operations;
- ABI callbacks resolve against their owning Application during posted work.

## Implemented migration sequence

1. Add `ApplicationContext`, `ApplicationDependencyManager`, and
   `ApplicationKeyedServiceFactory`.
2. Register concrete factories in the `ApplicationContext` constructor, before
   creating keyed services, and then freeze factory registration.
3. Move `ManifestServiceFactory` and `UtilityServiceFactory` to the new
   manager and typed context.
4. Store an `ApplicationContext&` in each Tuffite `BrowserContext`.
5. Inject application service interfaces into BrowserContext services.
6. Resolve ShellAPI utility calls from their sender and preserve document
   identity through invocation and cancellation.
7. Resolve ABI utility calls from the owning `MainDelegate`.
8. Remove `ApplicationServiceKey::ActiveKey()` and then remove the old
   `SimpleDependencyManager` integration.

This migration preserves Chromium's factory and dependency-graph semantics
while making application identity explicit and keeping Tuffite-specific
infrastructure small.

## Browser and app responsibilities

Application services own manifest configuration and grants, asset factory creation,
utility processes and remoting. BrowserContext services own
WebAuthn, DevTools, context menus, downloads, windows (including configured startup windows),
and ShellAPI routing.
Download services detach their manager and item observers before profile teardown.

`MainDelegate`, `BrowserMainParts`, and `ContentBrowserClient` remain the
composition and Chromium callback adapters. They resolve service interfaces and
coordinate startup/shutdown phases. `BrowserContext` owns storage partitions and
profile inputs; concrete feature behavior belongs in services.

Per-request URL loaders, navigation throttles, authentication delegates/dialogs,
DevTools frontend controllers, and context menus retain their Chromium request or
WebContents lifetimes. These helpers implement the owning service's behavior and
are not independent keyed services. Native window backends remain platform adapters.

Chromium has one remote debugging server per process. Each profile DevToolsService holds a
weak ownership lease for that server; another profile cannot replace it or
stop it during shutdown. Manager delegates consult their service through a weak
reference, so no closed profile is retained after shutdown. The frontend controller
remains scoped to its frontend WebContents, with weak references to its inspected
contents. Production startup opens no endpoint without an explicit debugging port.

ContextMenuService is BrowserContext-scoped and depends explicitly on the
DevToolsService factory. ManifestService is borrowed through injection; it
belongs to a separate graph and outlives all BrowserContexts. ContentBrowserClient
injects it into the thin `browser/web_contents_view_delegate.*` adapter, which
owns a service-created ContextMenuSession for its WebContents and forwards show
and dismiss callbacks. The session, menu model and MenuRunner implementations
are private to `context_menu/`. They receive service interfaces through
constructor injection and never resolve the service context. Menu visibility
is evaluated by ContextMenuService using ManifestService and browser state, inspection uses DevToolsService, and native editing and
navigation commands target the session's WebContents. Coordinate conversion and
menu presentation also belong to the session. Menu state is not shared across
views. Browser/profile teardown destroys the sessions before application
services are destroyed; factory dependencies order ContextMenuService shutdown before DevToolsService,
and DevToolsService before WindowService. ManifestService outlives
the entire profile graph.

## Network and native document dialogs

`NetworkContextService` is BrowserContext-scoped. It configures each storage
partition's cache and encrypted Cookie database and owns the OSCrypt providers
and Mojo encryption receivers. Partitions shut down before this service is
destroyed. Live BrowserContext profile roots supply the sandbox's network storage
allowlist; temporary partitions never receive disk paths. OS key access failure
does not enable a plaintext fallback.

`JavaScriptDialogService` is also BrowserContext-scoped and owns one active native
dialog per top-level WebContents. Window forwards Chromium's dialog hook to the
service; its keyed factory depends on the dialog factory so windows shut down
first. Dialogs display the requesting origin, cancel on navigation/destruction,
and complete their callback once. Explorer keeps its existing application event
bridge and permission policy rather than receiving these native dialogs.

`FileChooserService` is BrowserContext-scoped and is shared by Window and
Explorer. Window forwards file selection and directory enumeration to the
service; Explorer forwards those callbacks through its owning document delegate.
Each selection session observes the requesting frame and cancels when the
document navigates, the frame disappears, or the profile shuts down. Native
selection and recursive directory enumeration complete their listener once.
WindowService depends on the chooser factory so windows shut down first.
Selection grants remain Chromium-owned; this service does not grant general
File System Access permissions.

The current `en-US,en` language preference is shared by network headers, Window
renderer preferences and workers. Explorer copies its owning document's language
preference when creating the inner WebContents.

`ApplicationContext` exposes the ABI host and typed
service lookup. Its constructor registers the concrete factories; `MainDelegate`
creates and destroys service instances through the dependency manager. GN keeps
the public header in `app:context` and constructor implementation in `app:app`
to avoid a circular dependency between context and concrete factories.

## Manifest ownership and browser policy

`ManifestService` is eagerly registered with the application graph, but does not
load files or assemble startup policy. Before service creation, MainDelegate runs:

1. `app::ManifestLoader::Load(package_root)` parses and validates the package into
   an owned `ManifestDefinition`.
2. `app::ConfigurationAssembler::Build(definition, environment, options)` merges
   authenticated launch context and typed host options into `EffectiveConfiguration`.
3. `ManifestServiceFactory::Create(configuration)` creates the read-only service.

`StartupEnvironment` owns the development URL. `BrowserOptions` owns host security
restrictions. Host denials are validated and merged before development-origin
aliases are applied, so the alias inherits both manifest and host restrictions.
Host restrictions cannot introduce ShellAPI or Explorer grants. Neither input is
retained by reference; the effective snapshot owns all compiled values. Mutable
schema data stays in a private header, and only the assembler can construct an
effective snapshot. The service retains it as const storage with no mutator.

The original validated package bytes remain the Rust runtime configuration;
effective startup window URLs and origin policy are queried separately. Utility
configuration bytes are compiled once by the loader, not reparsed by the service.
Scheme and network bootstrap reads belong to ManifestLoader and still run before
custom scheme registration and the application service graph exist.

MainDelegate transfers the unique service to ApplicationContext as a one-time
bootstrap input. The factory consumes that input when the service graph is created.

Consumers query application information, startup windows, network limits,
origin grants, paths and runtime serialization through explicit service APIs.
Asset and Utility factories declare DependsOn(ManifestServiceFactory). Resource
loaders and utility module resolvers retain only copyable AssetConfiguration
snapshots, which survive service shutdown without retaining permissions or a
service reference.

Browser services resolve ManifestService through their ApplicationContext and
receive it through constructor injection. The application graph outlives every
profile graph. The BrowserContext keyed-service graph owns ExplorerService,
which combines
manifest grants with frame and WebContents roles. Navigation throttles are
browser request objects and retain only the application scheme. Menu sessions
query manifest grants through weak pointers and evaluate their own runtime menu
visibility. ShellApiService evaluates frame authorization and command routing.
ExplorerService also creates navigation throttles and stops policy admission
at shutdown. WindowService depends on its factory so window WebContents are
closed before their provider is destroyed. Application services outlive the
profile graph. There is no ApplicationPolicyService or separate manifest model.

Application projections use `app_config.h` and the `tuffite::app` namespace.
ManifestService returns `app::AppInfo` through `app_info()`, startup windows and
network limits through specific queries, and `asset::AssetConfiguration` resource
snapshots. Resource configuration is owned by the asset module's configuration
target, which is separate from AssetService to keep dependencies acyclic.
