# Application, profiles and window content

One native `ApplicationContext` corresponds to one Rust `ApplicationContext`
and one `Application` backend. `ProfileManager` is an application keyed service.
It owns a persistent catalog and creates a native `BrowserContext`, Rust
`BrowserContext`, Rust `Browser` backend and utility session scope for each
opened profile. Concurrent opens of the same profile share initialization.

```rust,ignore
impl tuffite::Application for App {
    fn new(context: tuffite::ApplicationContext) -> tuffite::Result<Self> {
        Ok(Self { context })
    }
}

impl tuffite::Browser for ProfileBackend {
    fn new(context: tuffite::BrowserContext) -> tuffite::Result<Self> {
        Ok(Self { context })
    }
}

#[tuffite::main]
fn main() -> tuffite::Embedding<App> {
    tuffite::Embedding::new().browser::<ProfileBackend>()
}
```

Both backends have `new`, asynchronous `initialize` and asynchronous `shutdown`.
Constructors assemble dependencies; initialization performs only necessary work.
Application initialization precedes the automatic opening of the default
profile. Browser initialization precedes releasing business commands and
creating that profile's initial content. An initialization failure cleans up the
affected runtime; a failed later profile open leaves other profiles running.
Application shutdown follows all Browser shutdowns. Cleanup uses the existing
`ShutdownContext` deadline and cancellation contract. UI-dependent calls reject
once shutdown admission closes; Chromium executor cleanup remains available.

`ApplicationContext::state` and `BrowserContext::state` are separate typed maps.
Profile code explicitly obtains shared services through
`BrowserContext::application()`. The optional return also permits standalone contexts in low-level tests. Utilities keep
static application-wide definitions but independent per-runtime sessions and
V8 providers. Browser events reach only that BrowserContext's subscribers;
Application events deliberately broadcast across the application's contexts.

## Profile identity and persistence

`ProfileId` is a stable, lowercase ASCII ID (`a-z`, `0-9`, `_`, `-`, at most
128 bytes). `BrowserContextHandle` identifies one loaded generation. Reopening
a closed profile creates a new generation; stale handles cannot address it.
Rust context clones do not own native objects and dropping them does not close
a profile. New control-plane clients reject after their runtime stops.

The catalog is stored at `<user-data-dir>/profiles/catalog.json` and protected
by a catalog lock. Additional profiles have separate `profile-<id>` directories beneath that root.
Without `--user-data-dir`, the root is the application's resolved data directory
with `profile` appended. The default profile retains the existing user-data
directory and its browsing databases. No database copy or migration is needed
on the cold startup path. The network storage root and scoped utility service
are bound before BrowserContext services are constructed.

Listing or creating catalog entries does not construct a BrowserContext or
launch a renderer. Cold startup opens only `default`; other contexts and all
their business services remain lazy. This reduces startup work; it is not a
promise of a particular startup latency or an in-memory process snapshot.

## Compose windows with content

`WindowService` is an application keyed service. A native window retains its
identity while its content changes. A WebContents keeps the BrowserContext
chosen at creation throughout its lifetime. Replacing content never mutates a
WebContents' profile or a BrowserContext's parent ApplicationContext.

```rust,ignore
let browser = app.profiles().open(&profile_id).await?;
let prepared = browser.views().create(url).await?;
let window_id = app.windows().create(tuffite::WindowOptions::default()).await?;
let previous = app.windows().set_content(window_id, prepared).await?;
// Keep the previous view for a warm return, or release it explicitly.
if let Some(previous) = previous { app.windows().close_view(previous).await?; }
```

Creating a detached view waits for a successful primary navigation, with a
30-second deadline. Commit to a window is a separate operation; failed
preparation leaves existing content displayed. `set_content` consumes the
prepared view handle and returns an optional detached handle for the old content (`None` for an empty window). Empty windows have no BrowserContext.
A view cannot be attached to multiple windows. Cancelling a pending creation
future releases its resulting detached view even if completion races with
cancellation. Dropping a Rust view handle
does not destroy the content. The application releases abandoned prepared
views explicitly, including when it cancels a switch.

Applications coordinate unsaved state, concurrent switch requests and the
decision to keep or close old contexts. There is no `Window::switch_profile`
or process-wide active profile. `ProfileManager::close` rejects while a window
or detached view still references the context. Browser shutdown then stops
its utilities and Rust work before destroying the native context and storage
partitions. Application exit releases every view and window before contexts.

## Interface registration and command routing

`Application` and `Browser` are ordinary lifecycle traits. The main attribute
infers the backend from the returned Builder; no backend registration attribute
or Definition trait is required. Generated `api::interfaces::Name` markers bind
to the corresponding `api::Name` implementation at compile time. Repeated
Application and Browser `.interfaces` groups append registrations.
Each group also accepts `commands(generate_handler![...])` for explicit handlers.
Duplicate command IDs, channels and backend/utility ownership fail registration.
The complete registry and command-to-scope map are frozen once per process.
Dispatch borrows these tables without rebuilding interface metadata.


An interface belongs to exactly one backend scope. Registration rejects
collisions between Application, Browser and Utility definitions. A renderer's
actual frame selects its BrowserContext. Static command metadata selects the
application backend or that context's backend; unknown Browser methods do not
fall back to application handlers. Existing manifest origin and method grants
still apply. Native profile/view operations are a trusted Rust capability;
applications expose only the frontend orchestration methods they choose.
Builtin renderer interfaces are composed by each BrowserContext's ShellApiService.
Its router receives a RendererRequestContext containing a real weak document,
connection token and request ID. It owns document admission, origin/method grants,
cancellation, subscriptions and profile utility routing. There is no application
DispatchService. The ABI Host adapts explicitly scoped native capabilities
(profiles/windows, isolated worlds, utilities and Remoting) directly to services;
it never fabricates a renderer caller or invokes builtin renderer routes.
Application interface requests route through the parent ABI runtime; browser
interface requests stay in their original context. Both directions reject
unknown or wrong-scope commands; cancellation follows the operation to its owner.

Renderer window listing, control and Explorer window-message targeting remain
within the caller's BrowserContext. Application window handles provide explicit
cross-profile orchestration, including creating and closing empty windows.

The Playground profile lab at `ui/profiles.html` demonstrates shared application
state, independent Browser state and storage, and keeping one native window
while replacing its content. Run the packaged regression with:

```sh
python3 playground/scripts/smoke.py --package dist/playground --profiles
```

The Framework smoke suite includes this test. The previous
`#[tuffite::main(browser = Backend)]` form remains available with
`tuffite::app::BrowserBuilder<Backend>`; it preserves single-context lifecycle
and `Browser::setup`; its `dispatch` hook is specific to that legacy builder.
The new builder dispatches only explicitly registered interfaces/commands and
never invokes an unregistered fallback. New applications use `app::Builder<Application>` and
`Builder::browser_policy` for trusted startup policy. An existing associated
Browser's static `setup` hook is also honored once per application during
migration; it does not run for each profile instance. Builder denials supplement
that hook and the manifest.
