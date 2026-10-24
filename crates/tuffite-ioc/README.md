# Tuffite IoC

One dependency-free crate provides typed construction contracts and an optional
context-local `Registry`. Default features are empty; enable `registry` only at
application, tool, or test composition roots.

`Service<T>` implements `KeyedService`: consumers depend on that identity and its
interface, independently of the factory implementation. `Factory<C>::build`
receives one borrowed `FactoryContext<C>` exposing `context()` and `depends_on()`.
Named factories and the `factory` function adapter implement the same interface.

```rust
use std::sync::Arc;
use tuffite_ioc::{FactoryContext, Factory, Registry, Result, Service};

struct AppContext { greeting: String }
struct GreetingFactory;
impl Factory<AppContext> for GreetingFactory {
    type Target = Service<String>;
    fn build(&self, build: &mut FactoryContext<'_, AppContext>) -> Result<Arc<String>> {
        Ok(Arc::new(build.context().greeting.clone()))
    }
}

let mut registry = Registry::new(Arc::new(AppContext { greeting: "hello".into() }));
registry.register(GreetingFactory)?;
registry.inject(Arc::new(42_u32))?;
registry.freeze();
assert_eq!(&*registry.get::<Service<String>>()?, "hello");

let mut child = registry.scope(7_u16);
child.freeze();
assert_eq!(*child.get::<Service<u32>>()?, 42);
# Ok::<(), String>(())
```

- `register(factory)` binds construction; duplicate local identities return errors.
- `register_eager(factory)` declares a startup root beside its domain provider.
- `initialize()` resolves local eager roots after freezing, with dependencies first.
- `inject(Arc<T>)` binds an existing instance with the `Service<T>` identity.
- `freeze()` freezes the same registry in place; later registration returns an error.
- `get<Key>` constructs and caches successful instances in the owning registry.
- `cached<Key>` reads only existing local or inherited instances, including at shutdown.
- `context()` exposes startup context to the composition root.
- `scope(context)` creates a child registry with its own context and local cache.

Freeze before resolving services. Creating a scope does not resolve parent services;
any parent visited during resolution must also be frozen. A child first checks its
own bindings, then its parent. Inherited factories always use the parent's context
and dependencies, and siblings share the same parent instance. A local binding
shadows a parent identity without replacing it; a failing local factory never
falls back to the parent. A local unconstructed binding masks the parent's cache.
The parent and child context types can differ. Children retain parent ownership,
so parents may be dropped by callers while their scopes remain alive. Scopes may
create further scopes. There is no separate Container or explicit import step.

Each registry owns an independent local instance cache, including when roots share
one context Arc. Resolution never implicitly registers factories. Failures and
panics restore construction state so resolution can be retried. Cycles report the
dependency path. Shared resolution is serialized; concurrent sibling requests
construct a parent service once. Factories must resolve dependencies through
`FactoryContext::depends_on`, not reenter a registry through captured handles.
Cached ownership releases in reverse construction order, with child ownership
released before its retained parent. External Arcs retain their own lifetimes.
Resource owners start in their constructors and stop/join in `Drop`. Dropping the
registry also unwinds partially initialized scopes; no application shutdown list is
needed. External Arcs can extend resource lifetimes beyond the registry. Fallible
business operations that require caller-visible errors remain explicit methods.

The CLI uses Clap Options as each subcommand context. Concrete services such as
`BuildService` and `DevService` register their constructors through the `factory`
adapter, capture dependencies and execute with `run()`.
Business provider modules accept `&mut impl Registrar<C>` and must not import
`Registry` or the hidden bridge. Cargo feature unification can expose registry
code to a workspace, so architecture checks also enforce this boundary.

The public `tuffite` crate reexports contracts with feature `ioc`, and `Registry`
plus `browser_registry` with `ioc-registry`. `browser_registry` accepts a context
value implementing `AsRef<BrowserContext>`, retains the complete application
context and seeds native capabilities. Plain `BrowserContext` implements `AsRef`
too. The CLI uses its own invocation context and the same registry implementation.
