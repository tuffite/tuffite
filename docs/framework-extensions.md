# Business contexts, protocols, shortcuts and resources

These APIs are MVP interfaces. Context factories replace `set_state`/`state`, and
`bundle.resources` accepts only an alias map. Playground's
`framework.html` exercises the complete native path.

## Business Context factories

Application, Browser, Rust Utility and V8 provider setup builders all expose
`.context(factory)`. The factory receives the corresponding base context and
returns `tuffite::Result<C>`, where `C: Any + Send + Sync`. Registration is lazy:
static metadata collection and child process startup do not execute an
Application or Browser context factory. One factory invocation creates one
context per actual runtime generation; V8 providers create their own context once
per named provider, and Rust utilities construct theirs inside the sandbox.

```rust
use std::sync::Arc;
use tuffite::{Browser, BrowserContext, Result};

struct MyContext {
    base: BrowserContext,
    label: String,
}
struct MyBrowser { context: Arc<MyContext> }

impl Browser for MyBrowser {
    fn setup(builder: tuffite::browser::Builder<Self>)
        -> Result<tuffite::browser::Builder<Self>>
    {
        Ok(builder.context(|base| Ok(MyContext {
            base, label: "workspace".into(),
        })))
    }
    fn new(base: BrowserContext) -> Result<Self> {
        Ok(Self { context: base.downcast::<MyContext>()? })
    }
}
```

The sequence is setup registration, base context creation, factory execution,
installation, backend `new`, then initialization. Factory errors and panics fail
construction before `new`. Use `initialize` for work requiring published native
handles; the context factory runs before the Browser generation handle is bound.
Without a factory, downcasting to the base context type succeeds.

`downcast::<T>()` returns `Result<Arc<T>>`; `try_downcast::<T>()` returns
`Option<Arc<T>>`. They restore only the exact installed type, not an embedded base
field. There are no associated context types or propagated business generics.
The runtime retains the installed value, while all base handles retain only a
weak reference to it. A business context containing its own base therefore does
not form an ownership cycle. A retained business Arc keeps its fields alive after
stop, but its native capabilities still observe shutdown admission.

Profiles obtain the Application context through
`base.application().unwrap().downcast::<MyApplicationContext>()`. Browser-side V8
providers use `base.browser_context::<MyBrowserContext>()` to share explicit
fields from the owning Browser. No Rust Arc or Any value crosses IPC. Rust Utility
processes receive serializable configuration and expose IPC interfaces.

Playground demonstrates all four factory kinds in `application.rs`, `browser.rs`,
`utilities/worker.rs`, and `utilities/parser.rs`.

## Protocol resolution

A protocol registers an internal standard/secure/CORS-enabled scheme. Each route
selects a packaged file, packaged directory, or business resolver:

```rust
use tuffite::configuration::{Configuration, Protocol, Route};

let configuration = Configuration::default().protocol(
    Protocol::new("playground")
        .route(Route::new().origin("playground://main")
            .path("/*").directory("ui"))
        .route(Route::new().origin("playground://main")
            .path("/dialog/open").resolver())
);
```

`.origin`/`.path` append one selector; `.origins`/`.paths` append multiple.
Provider methods select one of `file`, `directory`, or `resolver`. Exact paths
win over terminal `/*` prefixes; longer prefixes win over shorter ones. Equal
priority routes overlapping the same origin are rejected. Directory routes
require a terminal `/*`. Disk paths are relative to the packaged resource root,
so they reference the targets of `bundle.resources`, not source project paths.

```rust
async fn resolve(
    &mut self,
    url: &tuffite::Url,
    context: tuffite::ResolveContext,
) -> tuffite::Result<tuffite::Resolution> {
    use tuffite::{Resolution, Resource};
    match url.path() {
        "/logo.png" => Ok(Resolution::Resource(
            Resource::file(self.logo_path.clone()).mime_type("image/png")
        )),
        "/dialog/open" => {
            if context.initiator() != Some("playground://main") {
                return Ok(Resolution::Discard);
            }
            // Perform an action; publish a business event if appropriate.
            Ok(Resolution::Consumed)
        }
        _ => Ok(Resolution::Pass),
    }
}
```

`ResolveContext` exposes the method, initiator, optional source WindowId/profile,
navigation status, and profile stop signal. Caller metadata never grants
permissions by itself. An absent initiator is not a trusted origin. The MVP
loader accepts GET only, and response CORS/CSP follow Chromium's normal rules.
Action routes must validate the caller before causing effects; they are not an
alternative to ShellAPI authorization.

| Resolution | Fetch/subresource | Navigation |
| --- | --- | --- |
| `Resource(resource)` | Resource bytes/file and status | Resource document |
| `Consumed` | Empty 204 response | Cancel in place |
| `Discard` | Aborted request | Cancel in place |
| `Pass` | Less specific static route, otherwise 404 | Same fallback |

Only the selected business resolver runs. Pass falls back to static routes;
it does not repeatedly invoke the application resolver for broader selectors.
Missing files finish with 404, resolver failure finishes with a network error,
and abandoned loads release their Mojo receiver. No outcome leaves a pending
request indefinitely. Resolution runs on the serialized Browser worker and
is interrupted by Browser stop or a 30-second operation deadline. Business
futures must yield; a blocking poll cannot be forcibly interrupted.

`Resource::bytes(Vec<u8>)` supports binary data up to 8 MiB.
`Resource::file(absolute_path)` reads a trusted business-selected regular file on the
native blocking pool, bounded to 64 MiB. `.mime_type(...)` overrides the default
or extension-derived MIME; `.status(...)` selects a non-redirect 200–599 status.
Invalid MIME headers, oversized bytes and relative file paths reject the result.
CSP, nosniff and no-store are supplied by native loading. Sending a Web event is
an independent business side effect, so it can accompany any resolution result.

Browser implements `resolve(&mut self, url, context)`. URL requests are addressed
to their originating profile runtime; application-owned capabilities are available
through the Browser's parent ApplicationContext. V8 implements
`resolve(&self, url, context)` with the same `Result<Resolution>` contract, while
retaining concurrent provider calls. Its ResolveContext carries `session_id()`,
`deadline()` and the session cancellation signal. V8 accepts Resource or Pass;
Consumed/Discard are errors for module loading. Module resources must be status
200, nonempty UTF-8 ECMAScript, bounded to 512 KiB. Resource files are read in the
Browser resolver worker. Pass uses the existing packaged module fallback.

Scheme declaration does not register an operating-system deep-link association.

## Native shortcuts

ApplicationContext exposes a shared registry, with a manager for each scope:

```rust
let keys = context.shortcuts();
keys.application().register("CommandOrControl+Shift+P").await?;
keys.window(window_id).on_shortcut("Escape", move |event| {
    // Capture explicit shared business dependencies here.
}).await?;
keys.system().register("CommandOrControl+Alt+Space").await?;
```

Strings are parsed to a typed `Shortcut`. Typed registration is also supported:

```rust
use tuffite::{Key, Modifiers, Shortcut};
let shortcut = Shortcut::new(
    Modifiers::COMMAND_OR_CONTROL | Modifiers::SHIFT,
    Key::Character('P'),
)?;
keys.application().register(shortcut).await?;
```

`.register` delivers activations to `Application::shortcut`; `.on_shortcut`
delivers them only to that registration's callback. Both execute on the serialized
application worker. `ShortcutEvent` contains the typed shortcut, scope, optional
activating window, and `ShortcutState::Activated`. Chromium's global listener
provides activation, so the API does not invent key release notifications.

A shortcut/scope pair identifies a registration; no extra string ID is required.
Managers provide async `unregister`, `unregister_all`, and `is_registered`.
Querying registration checks this application's native registry; it cannot tell
whether another application owns an unregistered key.

Window bindings win over application bindings. Application bindings apply while
a Tuffite window is receiving keyboard input. System shortcuts use Chromium's
GlobalAcceleratorListener and may fail due to backend support or conflicts.
Backends requiring external asynchronous registration are currently rejected.
A system binding conflicting with an application's own local binding is rejected.
Window destruction removes its bindings; shutdown removes all registrations and
callbacks. Playground registers system shortcuts only after the user opts in.

## Packaging aliases and exclusions

```json
{
  "bundle": {
    "resources": {
      "ui": ["ui/dist", "!**/*.map", "!**/ignore"],
      "templates": ["business/templates"],
      ".": ["branding/logo.png"]
    }
  }
}
```

The keys select target directories under the package's resource root; `.` selects
that root. Source directories copy their contents; source files keep their
filename. The example produces `ui/index.html`, `templates/...`, and `logo.png`,
without a `ui/dist` source hierarchy. To preserve that hierarchy explicitly, use
`"ui/dist": ["ui/dist"]`.

All positive entries are literal project-relative files or directories.
`!pattern` entries exclude files matching a glob against their source path or
path relative to the source directory. Exclusions apply to the whole alias group,
independent of list order. This is not full gitignore syntax: `!` always excludes
and cannot re-include a file. `**` spans directories; `*` stays within a path segment.

Traversal, absolute aliases/sources, included symlinks, reserved targets, different sources
mapping to the same file, and file/directory target collisions fail before copy.
Excluded directories are pruned before traversal, including any symlinks inside them.
Repeated identical source/target pairs are deduplicated. Arrays at the top level
of `bundle.resources` are rejected.

## Verification

```sh
cargo test -p tuffite-runtime -p tuffite --lib
python3 x.py test --suite native
python3 playground/scripts/smoke.py --package dist/playground --framework-extensions
```

The packaged lab covers context-derived metadata, bytes/image/file resources,
consumption, cancellation, static fallback, Web events and local shortcut routing.
OS global shortcuts remain an opt-in interactive check.

Native URL loading is a per-profile `browser/services/url` service. Application-wide
window and shortcut ownership lives in `app/services/window` and
`app/services/shortcut`; Window forwards key input and lifecycle notifications to
ShortcutService. ShortcutService shuts down before WindowService.
