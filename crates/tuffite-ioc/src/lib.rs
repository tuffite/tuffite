//! Typed service identities and context-aware construction contracts.
use std::any::{Any, TypeId, type_name};
use std::marker::PhantomData;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, String>;

/// Consumer-owned identity, independent of its construction implementation.
pub trait KeyedService: 'static {
    type Value: ?Sized + Send + Sync + 'static;
}
pub struct Service<T: ?Sized>(PhantomData<fn() -> T>);
impl<T: ?Sized + Send + Sync + 'static> KeyedService for Service<T> {
    type Value = T;
}

/// One construction implementation, evaluated once in each successful scope.
pub trait Factory<C>: Send + Sync + 'static {
    /// Consumer identity to which this factory binds; the instance is `Target::Value`.
    type Target: KeyedService;
    /// Whether this binding is constructed during registry initialization.
    fn eager(&self) -> bool {
        false
    }
    fn build(
        &self,
        context: &mut FactoryContext<'_, C>,
    ) -> Result<Arc<<Self::Target as KeyedService>::Value>>;
}

/// Adapts a single-argument construction function to the Factory interface.
pub fn factory<K: KeyedService, C: 'static, F>(build: F) -> impl Factory<C, Target = K>
where
    F: Fn(&mut FactoryContext<'_, C>) -> Result<Arc<K::Value>> + Send + Sync + 'static,
{
    FunctionFactory::<K, C, F> {
        build,
        marker: PhantomData,
    }
}
struct FunctionFactory<K, C, F> {
    build: F,
    marker: PhantomData<fn(&C) -> K>,
}
impl<K: KeyedService, C: 'static, F> Factory<C> for FunctionFactory<K, C, F>
where
    F: Fn(&mut FactoryContext<'_, C>) -> Result<Arc<K::Value>> + Send + Sync + 'static,
{
    type Target = K;
    fn build(&self, context: &mut FactoryContext<'_, C>) -> Result<Arc<K::Value>> {
        (self.build)(context)
    }
}

/// Borrowed construction capability: startup data and dependency resolution.
/// ```compile_fail
/// use tuffite_ioc::FactoryContext;
/// fn retain<C>(context: FactoryContext<'_, C>) -> FactoryContext<'static, C> { context }
/// ```
pub struct FactoryContext<'a, C> {
    context: &'a C,
    resolver: &'a mut dyn bridge::Resolver,
}
impl<'a, C> FactoryContext<'a, C> {
    /// Immutable startup context, independent of the temporary resolver borrow.
    #[must_use]
    pub fn context(&self) -> &'a C {
        self.context
    }
    /// Resolve a registered consumer service in this scope or an ancestor.
    /// # Panics
    /// Propagates factory panics or an internal typed-bridge invariant violation.
    pub fn depends_on<K: KeyedService>(&mut self) -> Result<Arc<K::Value>> {
        let value = self.resolver.resolve(bridge::Request::of::<K>())?;
        Ok(Arc::clone(
            value
                .downcast_ref::<Arc<K::Value>>()
                .expect("typed dependency bridge"),
        ))
    }
}

/// Domain modules can register factories, without owning or resolving a registry.
pub trait Registrar<C> {
    fn register<F: Factory<C>>(&mut self, factory: F) -> Result<&mut Self>;
    /// Declare a scope root. Its dependencies are still resolved by the container.
    fn register_eager<F: Factory<C>>(&mut self, factory: F) -> Result<&mut Self> {
        self.register(EagerFactory(factory))
    }
}
struct EagerFactory<F>(F);
impl<C, F: Factory<C>> Factory<C> for EagerFactory<F> {
    type Target = F::Target;
    fn eager(&self) -> bool {
        true
    }
    fn build(
        &self,
        context: &mut FactoryContext<'_, C>,
    ) -> Result<Arc<<Self::Target as KeyedService>::Value>> {
        self.0.build(context)
    }
}

/// Registry integration only; business modules must not import this bridge.
#[doc(hidden)]
pub mod bridge {
    use super::{Any, Arc, FactoryContext, KeyedService, Result, TypeId, type_name};
    pub type Erased = Arc<dyn Any + Send + Sync>;
    pub struct Request {
        pub id: TypeId,
        pub name: &'static str,
    }
    impl Request {
        #[must_use]
        pub fn of<K: KeyedService>() -> Self {
            Self {
                id: TypeId::of::<K>(),
                name: type_name::<K>(),
            }
        }
    }
    pub trait Resolver {
        fn resolve(&mut self, request: Request) -> Result<Erased>;
    }
    pub fn construction<'a, C>(
        context: &'a C,
        resolver: &'a mut dyn Resolver,
    ) -> FactoryContext<'a, C> {
        FactoryContext { context, resolver }
    }
}

#[cfg(feature = "registry")]
mod registry;
#[cfg(feature = "registry")]
pub use registry::Registry;
