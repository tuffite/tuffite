//! One immutable, type-erased business context per runtime generation.
use crate::{Error, Result};
use std::{
    any::Any,
    sync::{Arc, OnceLock, Weak},
};
pub(crate) type Value = Arc<dyn Any + Send + Sync>;
#[derive(Clone, Default)]
pub(crate) struct Slot(Arc<OnceLock<Weak<dyn Any + Send + Sync>>>);
impl std::fmt::Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContextSlot")
    }
}
impl Slot {
    pub(crate) fn install(&self, value: &Value) -> Result<()> {
        self.0
            .set(Arc::downgrade(value))
            .map_err(|_| Error::new("business context already installed"))
    }
    pub(crate) fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.0.get()?.upgrade()?.downcast().ok()
    }
    pub(crate) fn require<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        self.get().ok_or_else(|| {
            Error::new(format!(
                "business context is unavailable or has a different type: {}",
                std::any::type_name::<T>()
            ))
        })
    }
}
pub(crate) struct Factory<Base>(Arc<dyn Fn(Base) -> Result<Value> + Send + Sync>);
impl<Base> Clone for Factory<Base> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<Base: Any + Send + Sync> Default for Factory<Base> {
    fn default() -> Self {
        Self::new(|base| Ok(base))
    }
}
impl<Base> Factory<Base> {
    pub(crate) fn new<C: Any + Send + Sync>(
        factory: impl Fn(Base) -> Result<C> + Send + Sync + 'static,
    ) -> Self {
        Self(Arc::new(move |base| {
            factory(base).map(|value| Arc::new(value) as Value)
        }))
    }
    pub(crate) fn erased(factory: impl Fn(Base) -> Result<Value> + Send + Sync + 'static) -> Self {
        Self(Arc::new(factory))
    }
    pub(crate) fn build(&self, base: Base) -> Result<Value> {
        (self.0)(base)
    }
}

impl<Base> std::fmt::Debug for Factory<Base> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContextFactory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn factory_is_lazy_and_base_slot_does_not_retain_its_owner() {
        struct Business {
            base: Slot,
            drops: Arc<std::sync::atomic::AtomicUsize>,
        }
        impl Drop for Business {
            fn drop(&mut self) {
                self.drops.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
        let builds = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let drops = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let factory = Factory::new({
            let builds = builds.clone();
            let drops = drops.clone();
            move |base| {
                builds.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(Business {
                    base,
                    drops: drops.clone(),
                })
            }
        });
        assert_eq!(builds.load(std::sync::atomic::Ordering::SeqCst), 0);
        let slot = Slot::default();
        let owner = factory.build(slot.clone()).unwrap();
        slot.install(&owner).unwrap();
        assert!(slot.get::<String>().is_none());
        assert!(slot.install(&owner).is_err());
        let business = slot.require::<Business>().unwrap();
        assert!(Arc::ptr_eq(&business, &business.base.require().unwrap()));
        drop(owner);
        assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 0);
        drop(business);
        assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(slot.require::<Business>().is_err());
    }
}
