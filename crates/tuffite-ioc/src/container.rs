//! Context-local service ownership with frozen configuration and scoped imports.
use crate::bridge::{self, Erased, Request, Resolver};
use crate::{Factory, FactoryContext, KeyedService, Result, Service, factory};
use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::sync::Arc;

type Provider<C> = Box<dyn Fn(&mut FactoryContext<'_, C>) -> Result<Erased> + Send + Sync>;

pub struct Registry<'a, C> {
    parent: Option<&'a mut dyn Resolver>,
    context: Arc<C>,
    providers: HashMap<TypeId, Provider<C>>,
}
impl<C: Send + Sync + 'static> Registry<'_, C> {
    #[must_use]
    pub fn new(context: Arc<C>) -> Self {
        let mut registry = Self {
            parent: None,
            context: Arc::clone(&context),
            providers: HashMap::new(),
        };
        registry.providers.insert(
            TypeId::of::<Service<C>>(),
            Box::new(move |_| Ok(Arc::new(Arc::clone(&context)))),
        );
        registry
    }
    /// Share an explicitly selected instance from the frozen parent.
    pub fn import<K: KeyedService>(&mut self) -> Result<&mut Self> {
        let parent = self
            .parent
            .as_mut()
            .ok_or("root registry has no parent to import from")?;
        let value = parent.resolve(Request::of::<K>())?;
        let value = Arc::clone(
            value
                .downcast_ref::<Arc<K::Value>>()
                .ok_or("invalid parent service identity")?,
        );
        self.register(factory::<K, C, _>(move |_| Ok(Arc::clone(&value))))?;
        Ok(self)
    }
    /// Register exactly one factory per service identity; duplicates are errors.
    pub fn register<F: Factory<C>>(&mut self, factory: F) -> Result<&mut Self> {
        let id = TypeId::of::<F::Target>();
        if self.providers.contains_key(&id) {
            return Err(format!(
                "Duplicate provider for {}",
                type_name::<F::Target>()
            ));
        }
        self.providers.insert(
            id,
            Box::new(move |context| {
                factory
                    .build(context)
                    .map(|value| Arc::new(value) as Erased)
            }),
        );
        Ok(self)
    }
    pub fn inject<T: ?Sized + Send + Sync + 'static>(
        &mut self,
        value: Arc<T>,
    ) -> Result<&mut Self> {
        self.register(factory::<Service<T>, C, _>(move |_| Ok(Arc::clone(&value))))
    }
    #[must_use]
    pub fn freeze(self) -> Container<C> {
        Container {
            context: self.context,
            providers: self.providers,
            instances: HashMap::new(),
            stack: Vec::new(),
            order: Vec::new(),
        }
    }
}
impl<C: Send + Sync + 'static> crate::Registrar<C> for Registry<'_, C> {
    fn register<F: Factory<C>>(&mut self, factory: F) -> Result<&mut Self> {
        Self::register(self, factory)
    }
}

/// Frozen bindings with a context-local instance cache. New scopes remain allowed.
/// ```compile_fail
/// use std::sync::Arc;
/// use tuffite_ioc::container::Registry;
/// let mut frozen = Registry::new(Arc::new(())).freeze();
/// frozen.inject(Arc::new(42_u32));
/// ```
pub struct Container<C> {
    context: Arc<C>,
    providers: HashMap<TypeId, Provider<C>>,
    instances: HashMap<TypeId, Erased>,
    stack: Vec<(TypeId, &'static str)>,
    order: Vec<TypeId>,
}

impl<C: Send + Sync + 'static> Container<C> {
    /// Start an independent registry with this frozen parent.
    pub fn scope<S: Send + Sync + 'static>(&mut self, context: S) -> Registry<'_, S> {
        let mut registry = Registry::new(Arc::new(context));
        registry.parent = Some(self);
        registry
    }
    /// Invocation or application startup context; available to the composition root.
    #[must_use]
    pub fn context(&self) -> &C {
        &self.context
    }
    /// Reads an existing instance without starting work during shutdown.
    /// # Panics
    /// Panics on internal cache corruption; safe registration preserves types.
    #[must_use]
    pub fn cached<K: KeyedService>(&self) -> Option<Arc<K::Value>> {
        self.instances.get(&TypeId::of::<K>()).map(|value| {
            Arc::clone(
                value
                    .downcast_ref::<Arc<K::Value>>()
                    .expect("typed service cache"),
            )
        })
    }
    /// Constructs once per successful scope; errors remain retryable.
    /// # Panics
    /// Propagates factory panics after restoring resolution state.
    pub fn get<K: KeyedService>(&mut self) -> Result<Arc<K::Value>> {
        let value = self.resolve(Request::of::<K>())?;
        Ok(Arc::clone(
            value
                .downcast_ref::<Arc<K::Value>>()
                .expect("typed service cache"),
        ))
    }
}
impl<C: Send + Sync + 'static> Resolver for Container<C> {
    fn resolve(&mut self, request: Request) -> Result<Erased> {
        let id = request.id;
        if self.stack.iter().any(|(active, _)| *active == id) {
            let mut path: Vec<_> = self.stack.iter().map(|(_, name)| *name).collect();
            path.push(request.name);
            return Err(format!("Circular dependency: {}", path.join(" -> ")));
        }
        if let Some(instance) = self.instances.get(&id) {
            return Ok(Arc::clone(instance));
        }
        let provider = self
            .providers
            .remove(&id)
            .ok_or_else(|| format!("No provider registered for {}", request.name))?;
        self.stack.push((id, request.name));
        let context = Arc::clone(&self.context);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            provider(&mut bridge::construction(&context, self))
        }));
        self.providers.insert(id, provider);
        self.stack.pop();
        let result = match result {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        let instance = result.map_err(|error| format!("{}: {error}", request.name))?;
        self.instances.insert(id, Arc::clone(&instance));
        self.order.push(id);
        Ok(instance)
    }
}
impl<C> Drop for Container<C> {
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
        }))
        .freeze();
        let mut registry = parent.scope(BuildContext {
            target: "desktop".to_owned(),
        });
        registry.import::<Service<AppContext>>().unwrap();
        registry
            .register(factory::<Service<String>, _, _>(|deps| {
                let context: &BuildContext = deps.context();
                let app = deps.depends_on::<Service<AppContext>>()?;
                Ok(Arc::new(format!("{}:{}", app.name, context.target)))
            }))
            .unwrap();
        let mut child = registry.freeze();
        assert_eq!(child.context().target, "desktop");
        assert_eq!(parent.context().name, "app");
        assert_eq!(&*child.get::<Service<String>>().unwrap(), "app:desktop");
    }

    #[test]
    fn scopes_import_shared_instances_and_isolate_local_bindings() {
        let shared = Arc::new(String::from("root"));
        let mut registry = Registry::new(Arc::new(()));
        registry.inject(Arc::clone(&shared)).unwrap();
        registry.inject(Arc::new(99_u64)).unwrap();
        let mut parent = registry.freeze();
        let mut registry = parent.scope(7_u32);
        registry.import::<Service<String>>().unwrap();
        registry
            .register(factory::<Service<usize>, _, _>(|deps| {
                let context: &u32 = deps.context();
                assert_eq!(&*deps.depends_on::<Service<String>>()?, "root");
                Ok(Arc::new(*context as usize))
            }))
            .unwrap();
        let mut first = registry.freeze();
        assert_eq!(*first.get::<Service<usize>>().unwrap(), 7);
        assert!(Arc::ptr_eq(
            &shared,
            &first.get::<Service<String>>().unwrap()
        ));
        assert!(first.get::<Service<u64>>().is_err());
        let mut registry = parent.scope(8_u32);
        registry.inject(Arc::new(String::from("local"))).unwrap();
        let mut second = registry.freeze();
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
        let mut first = first.freeze();
        let mut second = second.freeze();
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
        assert_eq!(*registry.freeze().get::<Service<u32>>().unwrap(), 1);
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
        let mut scope = registry.freeze();
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
        let mut scope = registry.freeze();
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
        let mut scope = registry.freeze();
        assert!(
            scope
                .get::<Service<u32>>()
                .unwrap_err()
                .contains("Circular dependency")
        );
        assert!(scope.stack.is_empty());
        let mut registry = Registry::new(Arc::new(()));
        registry
            .register(factory::<Service<u32>, _, _>(|_| panic!("factory failure")))
            .unwrap();
        let mut scope = registry.freeze();
        for _ in 0..2 {
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || scope.get::<Service<u32>>()
                ))
                .is_err()
            );
            assert!(scope.stack.is_empty());
        }
        assert!(
            scope
                .get::<Service<u64>>()
                .unwrap_err()
                .contains("No provider")
        );
    }
}
