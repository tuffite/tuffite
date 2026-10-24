//! Handle-free, version-zero Mojom struct payloads. No Mojo runtime is required.
//!
//! This is deliberately a data codec, not a Mojo message/endpoint implementation.
use crate::codec::CodecError;

const MAX_BYTES: usize = 16 * 1024 * 1024;

pub trait Struct: Sized {
    const SIZE: usize;
    fn write(&self, encoder: &mut Encoder, base: usize) -> Result<(), CodecError>;
    fn read(decoder: &mut Decoder<'_>, base: usize) -> Result<Self, CodecError>;

    fn encode_mojom(&self) -> Result<Vec<u8>, CodecError> {
        let mut encoder = Encoder { bytes: Vec::new() };
        let base = encoder.allocate(Self::SIZE)?;
        encoder.header(base, Self::SIZE)?;
        self.write(&mut encoder, base)?;
        Ok(encoder.bytes)
    }

    fn decode_mojom(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() > MAX_BYTES {
            return Err(CodecError);
        }
        let mut decoder = Decoder { bytes, claimed: 0 };
        decoder.claim(0, Self::SIZE)?;
        decoder.header(0, Self::SIZE)?;
        let value = Self::read(&mut decoder, 0)?;
        if decoder.claimed != bytes.len() {
            return Err(CodecError);
        }
        Ok(value)
    }
}

pub trait Scalar: Sized {
    const SIZE: usize;
    fn write(&self, bytes: &mut [u8]);
    fn read(bytes: &[u8]) -> Result<Self, CodecError>;
}

macro_rules! scalar {
    ($($ty:ty),*) => {$(
        impl Scalar for $ty {
            const SIZE: usize = size_of::<Self>();
            fn write(&self, bytes: &mut [u8]) { bytes.copy_from_slice(&self.to_le_bytes()); }
            fn read(bytes: &[u8]) -> Result<Self, CodecError> {
                Ok(Self::from_le_bytes(bytes.try_into().map_err(|_| CodecError)?))
            }
        }
    )*};
}
scalar!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

/// One Mojom array element or pointer field.
pub trait Value: Sized {
    const WIDTH: usize;
    const BOOLEAN: bool = false;
    const REFERENCE: bool = false;
    fn put(&self, w: &mut Encoder, at: usize, bit: u8) -> Result<(), CodecError>;
    fn get(r: &mut Decoder<'_>, at: usize, bit: u8) -> Result<Self, CodecError>;
}
macro_rules! values { ($($ty:ty),*) => {$(
    impl Value for $ty {
        const WIDTH: usize = size_of::<Self>();
        fn put(&self, w: &mut Encoder, at: usize, _: u8) -> Result<(), CodecError> { w.scalar(at, self) }
        fn get(r: &mut Decoder<'_>, at: usize, _: u8) -> Result<Self, CodecError> { r.scalar(at) }
    }
)*}; }
values!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
impl Value for bool {
    const WIDTH: usize = 1;
    const BOOLEAN: bool = true;
    fn put(&self, w: &mut Encoder, at: usize, bit: u8) -> Result<(), CodecError> {
        w.boolean(at, bit, *self)
    }
    fn get(r: &mut Decoder<'_>, at: usize, bit: u8) -> Result<Self, CodecError> {
        r.boolean(at, bit)
    }
}
impl Value for String {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(&self, w: &mut Encoder, at: usize, _: u8) -> Result<(), CodecError> {
        w.bytes(at, self.as_bytes())
    }
    fn get(r: &mut Decoder<'_>, at: usize, _: u8) -> Result<Self, CodecError> {
        r.string(at)
    }
}
impl<T: Value> Value for Vec<T> {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(&self, w: &mut Encoder, at: usize, _: u8) -> Result<(), CodecError> {
        w.array(at, self)
    }
    fn get(r: &mut Decoder<'_>, at: usize, _: u8) -> Result<Self, CodecError> {
        r.array(at)
    }
}
impl<T: Value> Value for Option<T> {
    const WIDTH: usize = T::WIDTH;
    const REFERENCE: bool = T::REFERENCE;
    fn put(&self, w: &mut Encoder, at: usize, bit: u8) -> Result<(), CodecError> {
        if !T::REFERENCE {
            return Err(CodecError);
        }
        if let Some(value) = self {
            value.put(w, at, bit)?;
        }
        Ok(())
    }
    fn get(r: &mut Decoder<'_>, at: usize, bit: u8) -> Result<Self, CodecError> {
        if !T::REFERENCE {
            return Err(CodecError);
        }
        if r.scalar::<u64>(at)? == 0 {
            Ok(None)
        } else {
            T::get(r, at, bit).map(Some)
        }
    }
}

fn aligned(size: usize) -> Result<usize, CodecError> {
    size.checked_add(7)
        .map(|value| value & !7)
        .ok_or(CodecError)
}

pub struct Encoder {
    bytes: Vec<u8>,
}

impl Encoder {
    fn allocate(&mut self, size: usize) -> Result<usize, CodecError> {
        let base = self.bytes.len();
        let end = base.checked_add(aligned(size)?).ok_or(CodecError)?;
        if end > MAX_BYTES {
            return Err(CodecError);
        }
        self.bytes.resize(end, 0);
        Ok(base)
    }

    fn header(&mut self, base: usize, size: usize) -> Result<(), CodecError> {
        self.scalar(base, &u32::try_from(size).map_err(|_| CodecError)?)?;
        self.scalar(base + 4, &0_u32)
    }

    pub fn scalar<T: Scalar>(&mut self, offset: usize, value: &T) -> Result<(), CodecError> {
        let end = offset.checked_add(T::SIZE).ok_or(CodecError)?;
        value.write(self.bytes.get_mut(offset..end).ok_or(CodecError)?);
        Ok(())
    }

    pub fn boolean(&mut self, offset: usize, bit: u8, value: bool) -> Result<(), CodecError> {
        let byte = self.bytes.get_mut(offset).ok_or(CodecError)?;
        *byte |= u8::from(value) << bit;
        Ok(())
    }

    pub fn bytes(&mut self, offset: usize, value: &[u8]) -> Result<(), CodecError> {
        let size = value.len().checked_add(8).ok_or(CodecError)?;
        let target = self.allocate(size)?;
        self.scalar(
            offset,
            &u64::try_from(target.checked_sub(offset).ok_or(CodecError)?)
                .map_err(|_| CodecError)?,
        )?;
        self.scalar(target, &u32::try_from(size).map_err(|_| CodecError)?)?;
        self.scalar(
            target + 4,
            &u32::try_from(value.len()).map_err(|_| CodecError)?,
        )?;
        self.bytes[target + 8..target + size].copy_from_slice(value);
        Ok(())
    }

    pub fn array<T: Value>(&mut self, offset: usize, values: &[T]) -> Result<(), CodecError> {
        if values.len() > MAX_BYTES {
            return Err(CodecError);
        }
        let payload = if T::BOOLEAN {
            values.len().div_ceil(8)
        } else {
            values.len().checked_mul(T::WIDTH).ok_or(CodecError)?
        };
        let size = payload.checked_add(8).ok_or(CodecError)?;
        let target = self.allocate(size)?;
        self.scalar(
            offset,
            &u64::try_from(target.checked_sub(offset).ok_or(CodecError)?)
                .map_err(|_| CodecError)?,
        )?;
        self.scalar(target, &u32::try_from(size).map_err(|_| CodecError)?)?;
        self.scalar(
            target + 4,
            &u32::try_from(values.len()).map_err(|_| CodecError)?,
        )?;
        for (index, value) in values.iter().enumerate() {
            let at = target
                + 8
                + if T::BOOLEAN {
                    index / 8
                } else {
                    index * T::WIDTH
                };
            value.put(
                self,
                at,
                if T::BOOLEAN {
                    u8::try_from(index % 8).map_err(|_| CodecError)?
                } else {
                    0
                },
            )?;
        }
        Ok(())
    }

    pub fn structure<T: Struct>(&mut self, offset: usize, value: &T) -> Result<(), CodecError> {
        let target = self.allocate(T::SIZE)?;
        self.scalar(
            offset,
            &u64::try_from(target.checked_sub(offset).ok_or(CodecError)?)
                .map_err(|_| CodecError)?,
        )?;
        self.header(target, T::SIZE)?;
        value.write(self, target)
    }
}

pub struct Decoder<'a> {
    bytes: &'a [u8],
    claimed: usize,
}

impl Decoder<'_> {
    fn claim(&mut self, base: usize, size: usize) -> Result<(), CodecError> {
        let end = base.checked_add(aligned(size)?).ok_or(CodecError)?;
        // A forward, non-overlapping object graph, as required by Mojo validation.
        if !base.is_multiple_of(8) || base < self.claimed || end > self.bytes.len() {
            return Err(CodecError);
        }
        self.claimed = end;
        Ok(())
    }

    fn header(&self, base: usize, size: usize) -> Result<(), CodecError> {
        if self.scalar::<u32>(base)? as usize != size || self.scalar::<u32>(base + 4)? != 0 {
            return Err(CodecError);
        }
        Ok(())
    }

    fn pointer(&self, offset: usize) -> Result<usize, CodecError> {
        let distance = usize::try_from(self.scalar::<u64>(offset)?).map_err(|_| CodecError)?;
        if distance < 8 {
            return Err(CodecError);
        }
        let target = offset.checked_add(distance).ok_or(CodecError)?;
        if target >= self.bytes.len() {
            return Err(CodecError);
        }
        Ok(target)
    }

    pub fn scalar<T: Scalar>(&self, offset: usize) -> Result<T, CodecError> {
        let end = offset.checked_add(T::SIZE).ok_or(CodecError)?;
        T::read(self.bytes.get(offset..end).ok_or(CodecError)?)
    }

    pub fn boolean(&self, offset: usize, bit: u8) -> Result<bool, CodecError> {
        Ok(self.scalar::<u8>(offset)? & (1 << bit) != 0)
    }

    pub fn bytes(&mut self, offset: usize) -> Result<Vec<u8>, CodecError> {
        let target = self.pointer(offset)?;
        let size = self.scalar::<u32>(target)? as usize;
        let length = self.scalar::<u32>(target + 4)? as usize;
        if length.checked_add(8) != Some(size) {
            return Err(CodecError);
        }
        self.claim(target, size)?;
        Ok(self.bytes[target + 8..target + size].to_vec())
    }

    pub fn string(&mut self, offset: usize) -> Result<String, CodecError> {
        String::from_utf8(self.bytes(offset)?).map_err(|_| CodecError)
    }

    pub fn array<T: Value>(&mut self, offset: usize) -> Result<Vec<T>, CodecError> {
        let target = self.pointer(offset)?;
        let size = self.scalar::<u32>(target)? as usize;
        let length = self.scalar::<u32>(target + 4)? as usize;
        if length > MAX_BYTES {
            return Err(CodecError);
        }
        let payload = if T::BOOLEAN {
            length.div_ceil(8)
        } else {
            length.checked_mul(T::WIDTH).ok_or(CodecError)?
        };
        if payload.checked_add(8) != Some(size) {
            return Err(CodecError);
        }
        self.claim(target, size)?;
        (0..length)
            .map(|index| {
                let at = target
                    + 8
                    + if T::BOOLEAN {
                        index / 8
                    } else {
                        index * T::WIDTH
                    };
                T::get(
                    self,
                    at,
                    if T::BOOLEAN {
                        u8::try_from(index % 8).map_err(|_| CodecError)?
                    } else {
                        0
                    },
                )
            })
            .collect()
    }

    pub fn structure<T: Struct>(&mut self, offset: usize) -> Result<T, CodecError> {
        let target = self.pointer(offset)?;
        self.claim(target, T::SIZE)?;
        self.header(target, T::SIZE)?;
        T::read(self, target)
    }
}
