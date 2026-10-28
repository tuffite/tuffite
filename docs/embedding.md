# Native embedding and startup

```rust
#[tuffite::main]
fn main() -> tuffite::Embedding<App> {
    tuffite::Embedding::new().browser::<Profile>().v8_utility::<Parser>()
}
```

`Embedding` chooses implementations and generates stable native ABI callbacks.
The macro emits `tuffite_get_application` directly, with a cached registration
factory and panic isolation. It never constructs a business backend or runs setup.
Exactly one Browser implementation is required; duplicate Utility names/interfaces
are rejected. No independent `Application::Browser` association exists.

Application setup is called once per process, with native argv before ContentMain:

```rust
impl tuffite::Application for App {
    fn setup(mut builder: tuffite::app::Builder<Self>)
        -> tuffite::Result<tuffite::app::Builder<Self>>
    {
        builder.argv_mut().retain(|arg| arg != "--business-debug");
        Ok(builder.configuration(configuration())
            .interfaces(|registry| registry.register::<api::Profiles>()))
    }
    // new / initialize / shutdown
}
impl tuffite::Browser for Profile {
    fn setup(builder: tuffite::browser::Builder<Self>)
        -> tuffite::Result<tuffite::browser::Builder<Self>>
    {
        Ok(builder.interfaces(|registry| registry.register::<api::Settings>()))
    }
    // new / initialize / shutdown
}
```

Builders collect definitions, while private framework finalization creates/binds
contexts at the required native stage. Setup is synchronous and cannot open
windows, profiles or sessions. `new(context)` constructs state; Application/Browser
`initialize` executes work after native binding. Utility uses asynchronous `new`
and is Ready only after it succeeds.

Browser setup is first called without a profile to freeze command ownership and
native policy. Per-profile setup receives `builder.profile_id()` before `new`;
changing descriptors or denials relative to the static pass rejects that profile.
This preserves lazy profile construction and deterministic dispatch ownership.

`Startup::default()` snapshots process information. In app setup,
`builder.startup()` contains the native input; `builder.argv_mut()` explicitly
submits edits. Framework validates executable, role and argument limits. Utility
and V8 builders can inspect the resulting startup; Utility setup additionally
sees `builder.service_name()`, but cannot rewrite Chromium arguments at that late stage.

Utility startup is process setup -> UtilityMain -> Mojo Start -> service setup ->
UtilityContext -> async new -> Ready -> calls/sessions. Application and Browser
contexts are never constructed in that child. V8Context belongs to the Browser
resource provider, not the child's JS isolate.

Source JSON describes tools and files; runtime Rust describes schemes, URLs and
policy. List every referenced asset/isolated-world file in `bundle.resources`.
No runtime declaration is imported by build.rs or extracted by executing a target.

General lifecycle methods return `tuffite::Result<T>` with `tuffite::Error`.
Standard IO/JSON/URL failures and Framework domain errors convert with `?`;
the general error preserves their typed cause through `std::error::Error::source`.
Domain methods retain their matchable enums, such as `CommandError` and
`isolated::Error`. `Result<T, DomainError>` can explicitly select that error type.
