//! Generation-aware opaque handles used by Runtime.

use std::fmt;
use std::marker::PhantomData;

const SLOT_BITS: u32 = 32;
const GENERATION_BITS: u32 = 24;
const GENERATION_SHIFT: u32 = SLOT_BITS;
const KIND_SHIFT: u32 = SLOT_BITS + GENERATION_BITS;
const SLOT_MASK: u64 = u32::MAX as u64;
const GENERATION_MASK: u64 = (1_u64 << GENERATION_BITS) - 1;
const MAX_SLOT: usize = u32::MAX as usize;

/// Stable object categories encoded into [`RawHandle`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum HandleKind {
    /// An invalid or uninitialized handle.
    Invalid = 0,
    /// A Tuffite Runtime instance.
    Runtime = 1,
    /// Sandboxed application utility service instance.
    UtilityService = 6,
    /// Long-lived session inside a sandboxed Utility service.
    UtilitySession = 7,
    /// A platform window.
    Window = 2,
    /// A Chromium `WebContents`.
    WebContents = 3,
    /// A supervised Runtime task.
    Task = 4,
    /// A brokered network request.
    NetworkRequest = 5,
}

impl HandleKind {
    /// Converts the stable wire value into a known handle kind.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Invalid),
            1 => Some(Self::Runtime),
            2 => Some(Self::Window),
            3 => Some(Self::WebContents),
            4 => Some(Self::Task),
            5 => Some(Self::NetworkRequest),
            6 => Some(Self::UtilityService),
            7 => Some(Self::UtilitySession),
            _ => None,
        }
    }
}

/// Errors returned while constructing or resolving a handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum HandleError {
    /// Zero is reserved for an invalid handle.
    #[error("invalid handle")]
    Invalid,
    /// The encoded kind is unknown or does not match the requested type.
    #[error("handle kind mismatch")]
    WrongKind,
    /// The slot is absent.
    #[error("handle slot is missing")]
    Missing,
    /// The slot exists but has since been reused with another generation.
    #[error("handle generation is stale")]
    Stale,
    /// The table cannot represent another slot.
    #[error("handle table is exhausted")]
    Exhausted,
}

/// A C-ABI-safe, pointer-free object identity.
///
/// Layout from most to least significant bits: kind (8), generation (24),
/// slot (32). Generation zero is never issued. Raw value zero is invalid.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct RawHandle(u64);

impl RawHandle {
    /// The invalid sentinel.
    pub const INVALID: Self = Self(0);

    /// Constructs a handle from validated parts.
    pub const fn new(kind: HandleKind, slot: u32, generation: u32) -> Result<Self, HandleError> {
        if matches!(kind, HandleKind::Invalid) || generation == 0 {
            return Err(HandleError::Invalid);
        }
        if generation as u64 > GENERATION_MASK {
            return Err(HandleError::Invalid);
        }
        Ok(Self(
            (((kind as u8) as u64) << KIND_SHIFT)
                | ((generation as u64) << GENERATION_SHIFT)
                | slot as u64,
        ))
    }

    /// Parses an untrusted wire value.
    pub const fn from_u64(value: u64) -> Result<Self, HandleError> {
        let handle = Self(value);
        if value == 0 || handle.generation() == 0 {
            return Err(HandleError::Invalid);
        }
        match handle.kind() {
            Some(HandleKind::Invalid) | None => Err(HandleError::WrongKind),
            Some(_) => Ok(handle),
        }
    }

    /// Returns the stable wire representation.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the encoded kind when it is known.
    #[must_use]
    pub const fn kind(self) -> Option<HandleKind> {
        HandleKind::from_u8((self.0 >> KIND_SHIFT) as u8)
    }

    /// Returns the zero-based slot.
    #[must_use]
    pub const fn slot(self) -> u32 {
        (self.0 & SLOT_MASK) as u32
    }

    /// Returns the nonzero generation.
    #[must_use]
    pub const fn generation(self) -> u32 {
        ((self.0 >> GENERATION_SHIFT) & GENERATION_MASK) as u32
    }
}

impl fmt::Debug for RawHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawHandle")
            .field("kind", &self.kind())
            .field("slot", &self.slot())
            .field("generation", &self.generation())
            .finish()
    }
}

/// Associates a typed Rust handle with one stable wire kind.
pub trait HandleType {
    /// Kind encoded into the raw handle.
    const KIND: HandleKind;
}

/// A typed wrapper over a pointer-free [`RawHandle`].
pub struct Handle<T: HandleType> {
    raw: RawHandle,
    marker: PhantomData<fn() -> T>,
}

impl<T: HandleType> Handle<T> {
    /// Validates and wraps an untrusted raw handle.
    pub fn from_raw(raw: RawHandle) -> Result<Self, HandleError> {
        if raw.kind() != Some(T::KIND) {
            return Err(HandleError::WrongKind);
        }
        Ok(Self {
            raw,
            marker: PhantomData,
        })
    }

    /// Returns the raw wire identity.
    #[must_use]
    pub const fn raw(self) -> RawHandle {
        self.raw
    }
}

impl<T: HandleType> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: HandleType> Copy for Handle<T> {}

impl<T: HandleType> fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(formatter)
    }
}

impl<T: HandleType> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T: HandleType> Eq for Handle<T> {}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// A slot table that rejects stale and wrong-kind handles.
pub struct HandleTable<T, K: HandleType> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    marker: PhantomData<fn() -> K>,
}

impl<T, K: HandleType> HandleTable<T, K> {
    /// Creates an empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Inserts a value and returns its typed identity.
    pub fn insert(&mut self, value: T) -> Result<Handle<K>, HandleError> {
        let slot = if let Some(slot) = self.free.pop() {
            slot
        } else {
            if self.slots.len() > MAX_SLOT {
                return Err(HandleError::Exhausted);
            }
            let slot = u32::try_from(self.slots.len()).map_err(|_| HandleError::Exhausted)?;
            self.slots.push(Slot {
                generation: 1,
                value: None,
            });
            slot
        };

        let entry = &mut self.slots[slot as usize];
        debug_assert!(entry.value.is_none());
        entry.value = Some(value);
        let raw = RawHandle::new(K::KIND, slot, entry.generation)?;
        Handle::from_raw(raw)
    }

    /// Resolves an immutable reference.
    pub fn get(&self, handle: Handle<K>) -> Result<&T, HandleError> {
        self.resolve(handle)
            .and_then(|slot| slot.value.as_ref().ok_or(HandleError::Missing))
    }

    /// Resolves a mutable reference.
    pub fn get_mut(&mut self, handle: Handle<K>) -> Result<&mut T, HandleError> {
        self.resolve_mut(handle)
            .and_then(|slot| slot.value.as_mut().ok_or(HandleError::Missing))
    }

    /// Removes a value and invalidates all copies of its handle.
    pub fn remove(&mut self, handle: Handle<K>) -> Result<T, HandleError> {
        let slot_index = handle.raw().slot();
        let entry = self.resolve_mut(handle)?;
        let value = entry.value.take().ok_or(HandleError::Missing)?;
        entry.generation = next_generation(entry.generation);
        self.free.push(slot_index);
        Ok(value)
    }

    /// Removes all live values and invalidates their handles.
    pub fn drain(&mut self) -> Vec<T> {
        self.free.clear();
        let mut values = Vec::new();
        for (index, slot) in (0_u32..).zip(self.slots.iter_mut()) {
            if let Some(value) = slot.value.take() {
                values.push(value);
                slot.generation = next_generation(slot.generation);
            }
            self.free.push(index);
        }
        values
    }

    /// Returns the currently live handles without exposing stored values.
    pub fn handles(&self) -> impl Iterator<Item = Handle<K>> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.value.is_some())
            .filter_map(|(index, entry)| {
                let slot = u32::try_from(index).ok()?;
                RawHandle::new(K::KIND, slot, entry.generation).ok()
            })
            .filter_map(|raw| Handle::from_raw(raw).ok())
    }

    fn resolve(&self, handle: Handle<K>) -> Result<&Slot<T>, HandleError> {
        let entry = self
            .slots
            .get(handle.raw().slot() as usize)
            .ok_or(HandleError::Missing)?;
        if entry.generation != handle.raw().generation() {
            return Err(HandleError::Stale);
        }
        Ok(entry)
    }

    fn resolve_mut(&mut self, handle: Handle<K>) -> Result<&mut Slot<T>, HandleError> {
        let entry = self
            .slots
            .get_mut(handle.raw().slot() as usize)
            .ok_or(HandleError::Missing)?;
        if entry.generation != handle.raw().generation() {
            return Err(HandleError::Stale);
        }
        Ok(entry)
    }
}

impl<T, K: HandleType> Default for HandleTable<T, K> {
    fn default() -> Self {
        Self::new()
    }
}

const fn next_generation(current: u32) -> u32 {
    let next = (current as u64 + 1) & GENERATION_MASK;
    if next == 0 { 1 } else { next as u32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum WindowTag {}

    impl HandleType for WindowTag {
        const KIND: HandleKind = HandleKind::Window;
    }

    #[test]
    fn raw_handle_round_trips() {
        let handle = RawHandle::new(HandleKind::Window, 42, 7).unwrap();
        assert_eq!(handle.kind(), Some(HandleKind::Window));
        assert_eq!(handle.slot(), 42);
        assert_eq!(handle.generation(), 7);
        assert_eq!(RawHandle::from_u64(handle.as_u64()), Ok(handle));
    }

    #[test]
    fn table_rejects_a_stale_generation() {
        let mut table = HandleTable::<String, WindowTag>::new();
        let first = table.insert("first".to_owned()).unwrap();
        assert_eq!(table.remove(first).unwrap(), "first");

        let second = table.insert("second".to_owned()).unwrap();
        assert_eq!(first.raw().slot(), second.raw().slot());
        assert_ne!(first.raw().generation(), second.raw().generation());
        assert_eq!(table.get(first), Err(HandleError::Stale));
        assert_eq!(table.get(second).unwrap(), "second");
    }

    #[test]
    fn typed_handle_rejects_the_wrong_kind() {
        let raw = RawHandle::new(HandleKind::Task, 0, 1).unwrap();
        assert_eq!(
            Handle::<WindowTag>::from_raw(raw),
            Err(HandleError::WrongKind)
        );
    }

    #[test]
    fn drain_invalidates_live_handles() {
        let mut table = HandleTable::<u32, WindowTag>::new();
        let handle = table.insert(7).unwrap();
        assert_eq!(table.drain(), vec![7]);
        assert_eq!(table.get(handle), Err(HandleError::Stale));
    }
}
