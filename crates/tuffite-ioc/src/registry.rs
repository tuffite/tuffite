//! Frozen registries with context-local ownership and inherited parent services.
use crate::bridge::{self, Erased, Request, Resolver};
use crate::{Factory, FactoryContext, KeyedService, Result, Service, factory};
use std::any::{TypeId, type_name};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

type Provider<C> = Box<dyn Fn(&mut FactoryContext<'_, C>) -> Result<Erased> + Send + Sync>;

/// Construction bindings, startup context and service ownership for one scope.
/// Freeze before resolving services. Child registries inherit missing bindings
/// from their parent; inherited factories always use the parent's context.
pub struct Registry<C> {
    inner: Arc<RegistryCore<C>>,
}
impl<C: Send + Sync + 'static> Registry<C> {
    #[must_use]
    pub fn new(context: Arc<C>) -> Self {
        Self::with_parent(context, None)
    }
    fn with_parent(context: Arc<C>, parent: Option<Arc<dyn ParentRegistry>>) -> Self {
        let startup = Arc::clone(&context);
        let mut providers: HashMap<TypeId, Provider<C>> = HashMap::new();
        providers.insert(
            TypeId::of::<Service<C>>(),
            Box::new(move |_| Ok(Arc::new(Arc::clone(&startup)))),
        );
        Self {
            inner: Arc::new(RegistryCore {
                context,
                state: Mutex::new(State {
                    frozen: false,
                    eager: Vec::new(),
                    providers,
                    instances: HashMap::new(),
                    stack: Vec::new(),
                    order: Vec::new(),
                }),
                parent,
            }),
        }
    }
    /// Register exactly one factory per local identity; frozen bindings are immutable.
    pub fn register<F: Factory<C>>(&mut self, factory: F) -> Result<&mut Self> {
        {
            let mut state = self.inner.state();
            if state.frozen {
                return Err("Registry is frozen; registration is prohibited".to_owned());
            }
            let id = TypeId::of::<F::Target>();
            if state.providers.contains_key(&id) {
                return Err(format!(
                    "Duplicate provider for {}",
                    type_name::<F::Target>()
                ));
            }
            if factory.eager() {
                state.eager.push(Request::of::<F::Target>());
            }
            state.providers.insert(
                id,
                Box::new(move |context| {
                    factory
                        .build(context)
                        .map(|value| Arc::new(value) as Erased)
                }),
            );
        }
        Ok(self)
    }
    pub fn inject<T: ?Sized + Send + Sync + 'static>(
        &mut self,
        value: Arc<T>,
    ) -> Result<&mut Self> {
        self.register(factory::<Service<T>, C, _>(move |_| Ok(Arc::clone(&value))))
    }
    /// Prohibit further registration while retaining resolution and scope creation.
    pub fn freeze(&mut self) {
        self.inner.state().frozen = true;
    }
    /// Construct declared eager roots in registration order after freezing.
    /// Dependencies precede consumers regardless of registration order. On failure,
    /// dropping this registry releases every successfully constructed instance.
    /// Repeated calls reuse cached instances; failed factories remain retryable.
    pub fn initialize(&self) -> Result<()> {
        let requests = {
            let state = self.inner.state();
            if !state.frozen {
                return Err("Registry must be frozen before initialization".to_owned());
            }
            state
                .eager
                .iter()
                .map(|request| Request {
                    id: request.id,
                    name: request.name,
                })
                .collect::<Vec<_>>()
        };
        for request in requests {
            self.inner.resolve(request)?;
        }
        Ok(())
    }
    /// Create an independent registry retaining its parent and its own startup context.
    /// A local binding shadows the parent's identity without replacing its instance.
    #[must_use]
    pub fn scope<S: Send + Sync + 'static>(&self, context: S) -> Registry<S> {
        Registry::with_parent(
            Arc::new(context),
            Some(Arc::clone(&self.inner) as Arc<dyn ParentRegistry>),
        )
    }
    #[must_use]
    pub fn context(&self) -> &C {
        &self.inner.context
    }
    /// Read an existing local or inherited instance without constructing services.
    /// A registered but unconstructed local service masks its parent's cache.
    /// # Panics
    /// Panics if called reentrantly from this registry's factory, or on cache corruption.
    #[must_use]
    pub fn cached<K: KeyedService>(&self) -> Option<Arc<K::Value>> {
        self.inner.cached(Request::of::<K>()).map(|value| {
            Arc::clone(
                value
                    .downcast_ref::<Arc<K::Value>>()
                    .expect("typed service cache"),
            )
        })
    }
    /// Construct once in the registry owning the binding; errors remain retryable.
    /// Resolution requires every visited registry to be frozen.
    /// # Panics
    /// Propagates factory panics after restoring resolution state.
    pub fn get<K: KeyedService>(&self) -> Result<Arc<K::Value>> {
        let value = self.inner.resolve(Request::of::<K>())?;
        Ok(Arc::clone(
            value
                .downcast_ref::<Arc<K::Value>>()
                .expect("typed service cache"),
        ))
    }
}
impl<C: Send + Sync + 'static> crate::Registrar<C> for Registry<C> {
    fn register<F: Factory<C>>(&mut self, factory: F) -> Result<&mut Self> {
        Self::register(self, factory)
    }
}

trait ParentRegistry: Send + Sync {
    fn resolve(&self, request: Request) -> Result<Erased>;
    fn cached(&self, request: Request) -> Option<Erased>;
}
struct RegistryCore<C> {
    state: Mutex<State<C>>,
    context: Arc<C>,
    // Release parent ownership after local providers and cached instances.
    parent: Option<Arc<dyn ParentRegistry>>,
}
struct State<C> {
    frozen: bool,
    eager: Vec<Request>,
    providers: HashMap<TypeId, Provider<C>>,
    instances: HashMap<TypeId, Erased>,
    stack: Vec<(TypeId, &'static str)>,
    order: Vec<TypeId>,
}
impl<C> RegistryCore<C> {
    fn state(&self) -> MutexGuard<'_, State<C>> {
        // Factory panics restore the state before unwinding; poisoning is recoverable.
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn identity(&self) -> usize {
        std::ptr::from_ref(self) as usize
    }
}

thread_local! { static ACTIVE: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) }; }
struct ActiveRegistry;
impl ActiveRegistry {
    fn enter(identity: usize, request: &Request) -> Result<Self> {
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            if active.contains(&identity) {
                return Err(format!("Circular dependency: reentrant registry resolution for {}; use FactoryContext::depends_on", request.name));
            }
            active.push(identity);
            Ok(Self)
        })
    }
}
impl Drop for ActiveRegistry {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.borrow_mut().pop();
        });
    }
}
impl<C: Send + Sync + 'static> ParentRegistry for RegistryCore<C> {
    fn resolve(&self, request: Request) -> Result<Erased> {
        let _active = ActiveRegistry::enter(self.identity(), &request)?;
        let mut state = self.state();
        if !state.frozen {
            return Err("Registry must be frozen before resolving services".to_owned());
        }
        Resolution {
            core: self,
            state: &mut state,
        }
        .resolve(request)
    }
    fn cached(&self, request: Request) -> Option<Erased> {
        assert!(
            !ACTIVE.with(|active| active.borrow().contains(&self.identity())),
            "cached cannot be called reentrantly from a registry factory"
        );
        let state = self.state();
        if let Some(instance) = state.instances.get(&request.id) {
            return Some(Arc::clone(instance));
        }
        if state.providers.contains_key(&request.id) {
            return None;
        }
        drop(state);
        self.parent
            .as_ref()
            .and_then(|parent| parent.cached(request))
    }
}

struct Resolution<'a, C> {
    core: &'a RegistryCore<C>,
    state: &'a mut State<C>,
}
impl<C: Send + Sync + 'static> Resolver for Resolution<'_, C> {
    fn resolve(&mut self, request: Request) -> Result<Erased> {
        let id = request.id;
        if self.state.stack.iter().any(|(active, _)| *active == id) {
            let mut path: Vec<_> = self.state.stack.iter().map(|(_, name)| *name).collect();
            path.push(request.name);
            return Err(format!("Circular dependency: {}", path.join(" -> ")));
        }
        if let Some(instance) = self.state.instances.get(&id) {
            return Ok(Arc::clone(instance));
        }
        let Some(provider) = self.state.providers.remove(&id) else {
            return match &self.core.parent {
                Some(parent) => parent.resolve(request),
                None => Err(format!("No provider registered for {}", request.name)),
            };
        };
        self.state.stack.push((id, request.name));
        let context = Arc::clone(&self.core.context);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            provider(&mut bridge::construction(&context, self))
        }));
        self.state.providers.insert(id, provider);
        self.state.stack.pop();
        let result = match result {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        let instance = result.map_err(|error| format!("{}: {error}", request.name))?;
        self.state.instances.insert(id, Arc::clone(&instance));
        self.state.order.push(id);
        Ok(instance)
    }
}
impl<C> Drop for State<C> {
    fn drop(&mut self) {
        self.providers.clear();
        for id in self.order.drain(..).rev() {
            self.instances.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Registrar as _;

    #[test]
    fn eager_roots_resolve_dependencies_and_unwind_failed_startup() {
        struct Probe(&'static str, Arc<Mutex<Vec<&'static str>>>);
        impl Drop for Probe {
            fn drop(&mut self) {
                self.1.lock().unwrap().push(self.0);
            }
        }
        enum Dependency {}
        enum Root {}
        impl KeyedService for Dependency {
            type Value = Probe;
        }
        impl KeyedService for Root {
            type Value = Probe;
        }
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut registry = Registry::new(Arc::new(()));
        let observed = Arc::clone(&events);
        registry
            .register_eager(factory::<Root, _, _>(move |deps| {
                deps.depends_on::<Dependency>()?;
                Ok(Arc::new(Probe("root", Arc::clone(&observed))))
            }))
            .unwrap();
        let observed = Arc::clone(&events);
        registry
            .register(factory::<Dependency, _, _>(move |_| {
                Ok(Arc::new(Probe("dependency", Arc::clone(&observed))))
            }))
            .unwrap();
        registry
            .register_eager(factory::<Service<u32>, _, _>(|_| {
                Err("startup failed".into())
            }))
            .unwrap();
        registry
            .register(factory::<Service<bool>, _, _>(|_| {
                panic!("lazy binding must stay lazy")
            }))
            .unwrap();
        assert!(registry.initialize().is_err());
        assert!(registry.cached::<Root>().is_none());
        registry.freeze();
        assert!(
            registry
                .initialize()
                .unwrap_err()
                .contains("startup failed")
        );
        let root = registry.get::<Root>().unwrap();
        assert!(registry.initialize().is_err());
        assert!(Arc::ptr_eq(&root, &registry.get::<Root>().unwrap()));
        drop(root);
        drop(registry);
        assert_eq!(*events.lock().unwrap(), ["root", "dependency"]);
    }

    #[test]
    fn initialization_is_idempotent_and_local_to_each_scope() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let mut registry = Registry::new(Arc::new(()));
        registry
            .register_eager(factory::<Service<usize>, _, _>(move |_| {
                Ok(Arc::new(observed.fetch_add(1, Ordering::SeqCst)))
            }))
            .unwrap();
        registry.freeze();
        let mut child = registry.scope(());
        child.freeze();
        child.initialize().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        registry.initialize().unwrap();
        registry.initialize().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    struct NamedFactory;

    #[test]
    fn scope_and_parent_keep_distinct_typed_contexts() {
        struct AppContext {
            name: String,
        }
        struct BuildContext {
            target: String,
        }
        let mut parent = Registry::new(Arc::new(AppContext {
            name: "app".to_owned(),
        }));
        parent.freeze();
        let mut registry = parent.scope(BuildContext {
            target: "desktop".to_owned(),
        });
        registry
            .register(factory::<Service<String>, _, _>(|deps| {
                let context: &BuildContext = deps.context();
                let app = deps.depends_on::<Service<AppContext>>()?;
                Ok(Arc::new(format!("{}:{}", app.name, context.target)))
            }))
            .unwrap();
        registry.freeze();
        let child = registry;
        assert_eq!(child.context().target, "desktop");
        assert_eq!(parent.context().name, "app");
        assert_eq!(&*child.get::<Service<String>>().unwrap(), "app:desktop");
    }

    #[test]
    fn scopes_inherit_shared_instances_and_isolate_local_bindings() {
        let shared = Arc::new(String::from("root"));
        let mut registry = Registry::new(Arc::new(()));
        registry.inject(Arc::clone(&shared)).unwrap();
        registry.inject(Arc::new(99_u64)).unwrap();
        registry.freeze();
        let parent = registry;
        let mut registry = parent.scope(7_u32);
        registry
            .register(factory::<Service<usize>, _, _>(|deps| {
                let context: &u32 = deps.context();
                assert_eq!(&*deps.depends_on::<Service<String>>()?, "root");
                Ok(Arc::new(*context as usize))
            }))
            .unwrap();
        registry.freeze();
        let first = registry;
        assert_eq!(*first.get::<Service<usize>>().unwrap(), 7);
        assert!(Arc::ptr_eq(
            &shared,
            &first.get::<Service<String>>().unwrap()
        ));
        assert_eq!(*first.get::<Service<u64>>().unwrap(), 99);
        let mut registry = parent.scope(8_u32);
        registry.inject(Arc::new(String::from("local"))).unwrap();
        registry.freeze();
        let second = registry;
        assert_eq!(&*second.get::<Service<String>>().unwrap(), "local");
        assert_eq!(&*parent.get::<Service<String>>().unwrap(), "root");
        assert_eq!(&*first.get::<Service<String>>().unwrap(), "root");
    }
    struct TestContext(String);
    impl Factory<TestContext> for NamedFactory {
        type Target = Service<String>;
        fn build(&self, build: &mut FactoryContext<'_, TestContext>) -> Result<Arc<String>> {
            Ok(Arc::new(build.context().0.clone()))
        }
    }
    #[test]
    fn contexts_have_independent_cached_instances() {
        let mut first = Registry::new(Arc::new(TestContext(String::from("first"))));
        first.register(NamedFactory).unwrap();
        assert!(first.register(NamedFactory).is_err());
        let mut second = Registry::new(Arc::new(TestContext(String::from("second"))));
        second.register(NamedFactory).unwrap();
        first.freeze();
        second.freeze();
        let a = first.get::<Service<String>>().unwrap();
        assert_eq!(&*a, "first");
        assert_eq!(&*second.get::<Service<String>>().unwrap(), "second");
        assert!(Arc::ptr_eq(&a, &first.get::<Service<String>>().unwrap()));
        assert!(!Arc::ptr_eq(&a, &second.get::<Service<String>>().unwrap()));
    }

    #[test]
    fn duplicate_registration_and_injection_are_errors() {
        let mut registry = Registry::new(Arc::new(()));
        registry.inject(Arc::new(1_u32)).unwrap();
        assert!(registry.inject(Arc::new(2_u32)).is_err());
        assert!(
            registry
                .register(factory::<Service<u32>, _, _>(|_| Ok(Arc::new(3))))
                .is_err()
        );
        registry.freeze();
        assert_eq!(*registry.get::<Service<u32>>().unwrap(), 1);
    }

    #[test]
    fn failed_factories_retry_and_successes_are_cached() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let mut registry = Registry::new(Arc::new(()));
        registry
            .register(factory::<Service<u32>, _, _>(move |_| {
                if observed.fetch_add(1, Ordering::SeqCst) == 0 {
                    Err("retry".to_owned())
                } else {
                    Ok(Arc::new(42))
                }
            }))
            .unwrap();
        registry.freeze();
        let scope = registry;
        assert!(scope.cached::<Service<u32>>().is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(scope.get::<Service<u32>>().is_err());
        let value = scope.get::<Service<u32>>().unwrap();
        assert!(Arc::ptr_eq(&value, &scope.get::<Service<u32>>().unwrap()));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn cached_consumers_release_before_dependencies() {
        use std::sync::Mutex;
        struct Probe(&'static str, Arc<Mutex<Vec<&'static str>>>);
        impl Drop for Probe {
            fn drop(&mut self) {
                self.1.lock().unwrap().push(self.0);
            }
        }
        enum Leaf {}
        enum Consumer {}
        impl KeyedService for Leaf {
            type Value = Probe;
        }
        impl KeyedService for Consumer {
            type Value = Probe;
        }
        let order = Arc::new(Mutex::new(Vec::new()));
        let leaf_order = Arc::clone(&order);
        let consumer_order = Arc::clone(&order);
        let mut registry = Registry::new(Arc::new(()));
        registry
            .register(factory::<Leaf, _, _>(move |_| {
                Ok(Arc::new(Probe("leaf", Arc::clone(&leaf_order))))
            }))
            .unwrap();
        registry
            .register(factory::<Consumer, _, _>(move |deps| {
                deps.depends_on::<Leaf>()?;
                Ok(Arc::new(Probe("consumer", Arc::clone(&consumer_order))))
            }))
            .unwrap();
        registry.freeze();
        let scope = registry;
        scope.get::<Consumer>().unwrap();
        drop(scope);
        assert_eq!(*order.lock().unwrap(), ["consumer", "leaf"]);
    }
    #[test]
    fn cycles_errors_and_panics_leave_resolution_retryable() {
        let mut registry = Registry::new(Arc::new(()));
        registry
            .register(factory::<Service<u32>, _, _>(|deps| {
                deps.depends_on::<Service<u32>>()
            }))
            .unwrap();
        registry.freeze();
        let scope = registry;
        assert!(
            scope
                .get::<Service<u32>>()
                .unwrap_err()
                .contains("Circular dependency")
        );
        assert!(scope.inner.state().stack.is_empty());
        let mut registry = Registry::new(Arc::new(()));
        registry
            .register(factory::<Service<u32>, _, _>(|_| panic!("factory failure")))
            .unwrap();
        registry.freeze();
        let scope = registry;
        for _ in 0..2 {
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || scope.get::<Service<u32>>()
                ))
                .is_err()
            );
            assert!(scope.inner.state().stack.is_empty());
        }
        assert!(
            scope
                .get::<Service<u64>>()
                .unwrap_err()
                .contains("No provider")
        );
    }

    #[test]
    fn freeze_prohibits_registration_and_resolution_requires_freeze() {
        let mut parent = Registry::new(Arc::new(()));
        parent.inject(Arc::new(7_u32)).unwrap();
        assert!(parent.get::<Service<u32>>().unwrap_err().contains("frozen"));
        let mut child = parent.scope("child");
        child.freeze();
        assert!(child.get::<Service<u32>>().unwrap_err().contains("frozen"));
        parent.freeze();
        parent.freeze();
        assert!(parent.inject(Arc::new(8_u64)).is_err());
        assert!(
            parent
                .register(factory::<Service<u64>, (), _>(|_| Ok(Arc::new(8))))
                .is_err()
        );
        assert!(child.cached::<Service<u32>>().is_none());
        assert_eq!(*child.get::<Service<u32>>().unwrap(), 7);
        assert!(Arc::ptr_eq(
            &child.cached::<Service<u32>>().unwrap(),
            &parent.get::<Service<u32>>().unwrap()
        ));
        let mut nested = child.scope(5_u16);
        nested.freeze();
        assert_eq!(*nested.get::<Service<u32>>().unwrap(), 7);
    }

    #[test]
    fn inherited_factories_use_parent_context_and_parent_dependencies() {
        struct ParentContext(&'static str);
        struct ChildContext(&'static str);
        let mut parent = Registry::new(Arc::new(ParentContext("parent")));
        parent
            .inject(Arc::new(String::from("parent-port")))
            .unwrap();
        parent
            .register(factory::<Service<Vec<String>>, ParentContext, _>(|build| {
                Ok(Arc::new(vec![
                    build.context().0.to_owned(),
                    (*build.depends_on::<Service<String>>()?).clone(),
                ]))
            }))
            .unwrap();
        parent.freeze();
        let mut child = parent.scope(ChildContext("child"));
        child.inject(Arc::new(String::from("child-port"))).unwrap();
        child.freeze();
        assert_eq!(child.context().0, "child");
        let value = child.get::<Service<Vec<String>>>().unwrap();
        assert_eq!(&*value, &["parent", "parent-port"]);
        assert!(Arc::ptr_eq(
            &value,
            &parent.get::<Service<Vec<String>>>().unwrap()
        ));
        assert_eq!(&*child.get::<Service<String>>().unwrap(), "child-port");
    }

    #[test]
    fn local_binding_masks_parent_cache_and_errors_do_not_fall_back() {
        let mut parent = Registry::new(Arc::new(()));
        parent.inject(Arc::new(42_u32)).unwrap();
        parent.freeze();
        parent.get::<Service<u32>>().unwrap();
        let mut child = parent.scope("local");
        child
            .register(factory::<Service<u32>, _, _>(|_| {
                Err("local failure".to_owned())
            }))
            .unwrap();
        child.freeze();
        assert!(child.cached::<Service<u32>>().is_none());
        assert!(
            child
                .get::<Service<u32>>()
                .unwrap_err()
                .contains("local failure")
        );
        assert_eq!(*parent.get::<Service<u32>>().unwrap(), 42);
    }

    #[test]
    fn siblings_share_lazy_parent_cache_across_threads() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let mut parent = Registry::new(Arc::new(()));
        parent
            .register(factory::<Service<u32>, _, _>(move |_| {
                observed.fetch_add(1, Ordering::SeqCst);
                Ok(Arc::new(42))
            }))
            .unwrap();
        parent.freeze();
        let barrier = Arc::new(std::sync::Barrier::new(8));
        let threads: Vec<_> = (0..8)
            .map(|index| {
                let mut child = parent.scope(index);
                child.freeze();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    child.get::<Service<u32>>().unwrap()
                })
            })
            .collect();
        let values: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(values.iter().all(|value| Arc::ptr_eq(value, &values[0])));
    }

    #[test]
    fn child_retains_parent_ownership_and_drops_local_services_first() {
        struct Probe(&'static str, Arc<Mutex<Vec<&'static str>>>);
        impl Drop for Probe {
            fn drop(&mut self) {
                self.1.lock().unwrap().push(self.0);
            }
        }
        enum ParentProbe {}
        enum ChildProbe {}
        impl KeyedService for ParentProbe {
            type Value = Probe;
        }
        impl KeyedService for ChildProbe {
            type Value = Probe;
        }
        let order = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&order);
        let mut parent = Registry::new(Arc::new(()));
        parent
            .register(factory::<ParentProbe, _, _>(move |_| {
                Ok(Arc::new(Probe("parent", Arc::clone(&observed))))
            }))
            .unwrap();
        parent.freeze();
        let mut child = parent.scope("child");
        let observed = Arc::clone(&order);
        child
            .register(factory::<ChildProbe, _, _>(move |build| {
                build.depends_on::<ParentProbe>()?;
                Ok(Arc::new(Probe("child", Arc::clone(&observed))))
            }))
            .unwrap();
        child.freeze();
        drop(parent);
        child.get::<ChildProbe>().unwrap();
        assert!(order.lock().unwrap().is_empty());
        drop(child);
        assert_eq!(*order.lock().unwrap(), ["child", "parent"]);
    }

    #[test]
    fn external_reentry_reports_a_cycle_without_deadlocking() {
        let holder = Arc::new(std::sync::OnceLock::<std::sync::Weak<Registry<()>>>::new());
        let observed = Arc::clone(&holder);
        let mut registry = Registry::new(Arc::new(()));
        registry
            .register(factory::<Service<u32>, _, _>(move |_| {
                observed
                    .get()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .get::<Service<u32>>()
            }))
            .unwrap();
        registry.freeze();
        let registry = Arc::new(registry);
        holder.set(Arc::downgrade(&registry)).unwrap();
        for _ in 0..2 {
            assert!(
                registry
                    .get::<Service<u32>>()
                    .unwrap_err()
                    .contains("Circular dependency")
            );
        }
    }
}
