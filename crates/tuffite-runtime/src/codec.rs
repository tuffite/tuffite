//! Deterministic primitives used by generated ShellAPI codecs.

const MAX_COLLECTION_LEN: usize = 16 * 1024 * 1024;
/// Maximum bytes any single decode session may consume from length-prefixed
/// fields. Prevents nested collections from combining small per-field caps
/// into a runaway total allocation.
const DEFAULT_MAX_TOTAL_ALLOC: usize = 32 * 1024 * 1024;
/// Maximum elements across all length-prefixed collection reads in one
/// session. Independent of the per-collection cap so a deeply nested payload
/// cannot exhaust CPU by walking many small collections.
const DEFAULT_MAX_TOTAL_ELEMENTS: u64 = 4 * 1024 * 1024;
/// Maximum nesting depth callers may enter when decoding.
const DEFAULT_MAX_DEPTH: u32 = 128;

/// Session-wide decode budget shared by all length-prefixed reads on one
/// [`Decoder`]. Every allocation-bearing read consumes budget so the total
/// cost of decoding one payload is bounded even for pathological nested
/// structures.
#[derive(Clone, Copy, Debug)]
pub struct DecodeBudget {
    /// Total bytes allocated so far by length-prefixed reads.
    bytes_used: usize,
    /// Total elements observed across collection reads.
    elements_used: u64,
    /// Current nesting depth.
    depth_used: u32,
    max_bytes: usize,
    max_elements: u64,
    max_depth: u32,
    max_collection_len: usize,
}

impl DecodeBudget {
    /// Returns the default session budget.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bytes_used: 0,
            elements_used: 0,
            depth_used: 0,
            max_bytes: DEFAULT_MAX_TOTAL_ALLOC,
            max_elements: DEFAULT_MAX_TOTAL_ELEMENTS,
            max_depth: DEFAULT_MAX_DEPTH,
            max_collection_len: MAX_COLLECTION_LEN,
        }
    }

    fn charge_bytes(&mut self, additional: usize) -> Result<(), CodecError> {
        let new_total = self.bytes_used.checked_add(additional).ok_or(CodecError)?;
        if new_total > self.max_bytes {
            return Err(CodecError);
        }
        self.bytes_used = new_total;
        Ok(())
    }

    fn charge_elements(&mut self, additional: u64) -> Result<(), CodecError> {
        let new_total = self
            .elements_used
            .checked_add(additional)
            .ok_or(CodecError)?;
        if new_total > self.max_elements {
            return Err(CodecError);
        }
        self.elements_used = new_total;
        Ok(())
    }

    /// Charges one level of nesting and returns a guard that releases it on
    /// drop. Depth is checked BEFORE incrementing so a decoder cannot
    /// silently exceed its cap.
    pub fn enter_depth(&mut self) -> Result<DepthGuard<'_>, CodecError> {
        if self.depth_used >= self.max_depth {
            return Err(CodecError);
        }
        self.depth_used += 1;
        Ok(DepthGuard { budget: self })
    }
}

impl Default for DecodeBudget {
    fn default() -> Self {
        Self::new()
    }
}

/// Automatically decrements the current depth on scope exit.
pub struct DepthGuard<'a> {
    budget: &'a mut DecodeBudget,
}

impl Drop for DepthGuard<'_> {
    fn drop(&mut self) {
        self.budget.depth_used = self.budget.depth_used.saturating_sub(1);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid binary ShellAPI payload")]
pub struct CodecError;

/// Owned binary value corresponding to TypeScript `Uint8Array`.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Bytes(pub Vec<u8>);

/// UTC milliseconds since the Unix epoch, corresponding to TypeScript `Date`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub i64);

pub trait Encode {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError>;
}

pub trait Decode: Sized {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError>;
}

#[derive(Default)]
pub struct Encoder {
    bytes: Vec<u8>,
}

impl Encode for () {
    fn encode(&self, _encoder: &mut Encoder) -> Result<(), CodecError> {
        Ok(())
    }
}

impl Decode for () {
    fn decode(_decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
        Ok(())
    }
}

impl Encode for bool {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
        encoder.write_u8(u8::from(*self));
        Ok(())
    }
}

impl Decode for bool {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
        match decoder.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CodecError),
        }
    }
}

macro_rules! integer_codec {
    ($type:ty, $write:ident, $read:ident) => {
        impl Encode for $type {
            fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
                encoder.$write(*self);
                Ok(())
            }
        }
        impl Decode for $type {
            fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
                decoder.$read()
            }
        }
    };
}

integer_codec!(u32, write_u32, read_u32);
integer_codec!(i64, write_i64, read_i64);
integer_codec!(f64, write_f64, read_f64);

impl Encode for String {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
        encoder.write_string(self)
    }
}

impl Decode for String {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
        decoder.read_string()
    }
}

impl Encode for Bytes {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
        encoder.write_bytes(&self.0)
    }
}

impl Decode for Bytes {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
        decoder.read_bytes().map(Self)
    }
}

impl Encode for Timestamp {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
        self.0.encode(encoder)
    }
}

impl Decode for Timestamp {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
        i64::decode(decoder).map(Self)
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
        encoder.write_u32(u32::try_from(self.len()).map_err(|_| CodecError)?);
        for value in self {
            value.encode(encoder)?;
        }
        Ok(())
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
        let len = decoder.read_len()?;
        (0..len).map(|_| T::decode(decoder)).collect()
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
        if let Some(value) = self {
            encoder.write_u8(1);
            value.encode(encoder)
        } else {
            encoder.write_u8(0);
            Ok(())
        }
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, CodecError> {
        match decoder.read_u8()? {
            0 => Ok(None),
            1 => T::decode(decoder).map(Some),
            _ => Err(CodecError),
        }
    }
}

impl Encoder {
    #[must_use]
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_f64(&mut self, value: f64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_bytes(&mut self, value: &[u8]) -> Result<(), CodecError> {
        let len = u32::try_from(value.len()).map_err(|_| CodecError)?;
        self.write_u32(len);
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    pub fn write_string(&mut self, value: &str) -> Result<(), CodecError> {
        self.write_bytes(value.as_bytes())
    }

    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

pub struct Decoder<'a> {
    bytes: &'a [u8],
    position: usize,
    budget: DecodeBudget,
}

impl<'a> Decoder<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            budget: DecodeBudget::new(),
        }
    }

    /// Constructs a decoder with a caller-supplied budget. Used by tests and
    /// specialized entrypoints (e.g. utility service requests) that want to
    /// tune allocation ceilings.
    #[must_use]
    pub const fn with_budget(bytes: &'a [u8], budget: DecodeBudget) -> Self {
        Self {
            bytes,
            position: 0,
            budget,
        }
    }

    /// Returns a mutable reference to the shared decode budget. Callers of
    /// generated codecs use this to enter/leave nesting levels.
    pub fn budget(&mut self) -> &mut DecodeBudget {
        &mut self.budget
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], CodecError> {
        let end = self.position.checked_add(len).ok_or(CodecError)?;
        let value = self.bytes.get(self.position..end).ok_or(CodecError)?;
        self.position = end;
        Ok(value)
    }

    pub fn read_u8(&mut self) -> Result<u8, CodecError> {
        Ok(self.take(1)?[0])
    }

    pub fn read_u32(&mut self) -> Result<u32, CodecError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| CodecError)?,
        ))
    }

    pub fn read_i64(&mut self) -> Result<i64, CodecError> {
        Ok(i64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| CodecError)?,
        ))
    }

    pub fn read_f64(&mut self) -> Result<f64, CodecError> {
        Ok(f64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| CodecError)?,
        ))
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, CodecError> {
        let len = usize::try_from(self.read_u32()?).map_err(|_| CodecError)?;
        if len > self.budget.max_collection_len {
            return Err(CodecError);
        }
        self.budget.charge_bytes(len)?;
        // Bytes are copied in bulk, not decoded element by element. Keep image
        // payloads within the byte budget without consuming the vector-item cap.
        Ok(self.take(len)?.to_vec())
    }

    pub fn read_string(&mut self) -> Result<String, CodecError> {
        String::from_utf8(self.read_bytes()?).map_err(|_| CodecError)
    }

    pub fn read_len(&mut self) -> Result<usize, CodecError> {
        let len = usize::try_from(self.read_u32()?).map_err(|_| CodecError)?;
        if len > self.budget.max_collection_len {
            return Err(CodecError);
        }
        // Element count contributes to the session-wide element budget so a
        // payload with many separate collections cannot outrun the cap by
        // splitting itself into small pieces.
        self.budget.charge_elements(len as u64)?;
        Ok(len)
    }

    pub fn finish(self) -> Result<(), CodecError> {
        (self.position == self.bytes.len())
            .then_some(())
            .ok_or(CodecError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_round_trip_and_trailing_data_is_rejected() {
        let mut encoder = Encoder::new();
        encoder.write_u8(1);
        encoder.write_u32(42);
        encoder.write_i64(-9);
        encoder.write_f64(1.5);
        encoder.write_string("hello").unwrap();
        let bytes = encoder.finish();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(decoder.read_u8(), Ok(1));
        assert_eq!(decoder.read_u32(), Ok(42));
        assert_eq!(decoder.read_i64(), Ok(-9));
        assert_eq!(decoder.read_f64(), Ok(1.5));
        assert_eq!(decoder.read_string(), Ok("hello".to_owned()));
        assert_eq!(decoder.finish(), Ok(()));

        let mut trailing = Decoder::new(&[0, 1]);
        assert_eq!(trailing.read_u8(), Ok(0));
        assert_eq!(trailing.finish(), Err(CodecError));
    }

    #[test]
    fn budget_caps_total_allocations_across_reads() {
        // 2 MiB max total, per-collection cap left at default.
        let mut budget = DecodeBudget::new();
        budget.max_bytes = 2 * 1024 * 1024;
        // Two 1 MiB reads succeed, the third fails because the session cap
        // would be exceeded even though each individual field is within
        // MAX_COLLECTION_LEN.
        let one_mib = vec![0_u8; 1024 * 1024];
        let mut encoder = Encoder::new();
        encoder.write_bytes(&one_mib).unwrap();
        encoder.write_bytes(&one_mib).unwrap();
        encoder.write_bytes(&one_mib).unwrap();
        let bytes = encoder.finish();
        let mut decoder = Decoder::with_budget(&bytes, budget);
        assert!(decoder.read_bytes().is_ok());
        assert!(decoder.read_bytes().is_ok());
        assert_eq!(decoder.read_bytes(), Err(CodecError));
    }

    #[test]
    fn bulk_binary_payload_uses_byte_budget_not_vector_item_budget() {
        let data = vec![1; 5 * 1024 * 1024];
        let mut encoder = Encoder::new();
        encoder.write_bytes(&data).unwrap();
        let wire = encoder.finish();
        let mut decoder = Decoder::new(&wire);
        assert_eq!(decoder.read_bytes().unwrap(), data);
        assert_eq!(decoder.finish(), Ok(()));
    }

    #[test]
    fn budget_enforces_depth_limit_via_guard() {
        // Depth guards hold a mutable borrow of the budget while alive so
        // callers naturally cannot nest deeper than the configured limit.
        // Sequential entries without holding a prior guard succeed.
        let mut budget = DecodeBudget::new();
        budget.max_depth = 2;
        let mut decoder = Decoder::with_budget(&[], budget);
        {
            let g1 = decoder.budget().enter_depth().expect("depth 1");
            drop(g1);
        }
        {
            let g2 = decoder
                .budget()
                .enter_depth()
                .expect("depth 2 (after drop)");
            drop(g2);
        }
        // Two nested guards fill the cap so a third fails.
        // Simulate manually by peeking the internal state.
        // Bump depth to the cap directly and confirm the next enter fails.
        decoder.budget().depth_used = decoder.budget().max_depth;
        assert!(decoder.budget().enter_depth().is_err());
    }
}
