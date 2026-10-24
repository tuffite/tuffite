// Generated from Framework Mojom contracts.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde")]
#[repr(i32)]
pub enum BaseDirectory {
    #[default]
    Resource = 0,
    Data = 1,
    Cache = 2,
    Temp = 3,
}
impl ::tuffite::__private::mojom::Value for BaseDirectory {
    const WIDTH: usize = 4;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.scalar(at, &(*self as i32))
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        match r.scalar::<i32>(at)? {
            0 => Ok(Self::Resource),
            1 => Ok(Self::Data),
            2 => Ok(Self::Cache),
            3 => Ok(Self::Temp),
            _ => Err(::tuffite::__private::codec::CodecError),
        }
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct DialogOpenDirectoryRequest {
    pub r#options: Option<OpenDirectoryDialogOptions>,
}
impl ::tuffite::__private::mojom::Value for DialogOpenDirectoryRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for DialogOpenDirectoryRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#options, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#options: <Option<
                OpenDirectoryDialogOptions,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct DialogOpenDirectoryResponse {
    pub r#result: Option<String>,
}
impl ::tuffite::__private::mojom::Value for DialogOpenDirectoryResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for DialogOpenDirectoryResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct DialogOpenFileRequest {
    pub r#options: Option<OpenFileDialogOptions>,
}
impl ::tuffite::__private::mojom::Value for DialogOpenFileRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for DialogOpenFileRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#options, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#options: <Option<
                OpenFileDialogOptions,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct DialogOpenFileResponse {
    pub r#result: Option<String>,
}
impl ::tuffite::__private::mojom::Value for DialogOpenFileResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for DialogOpenFileResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsCreateDirRequest {
    pub r#base: BaseDirectory,
    pub r#path: String,
}
impl ::tuffite::__private::mojom::Value for FsCreateDirRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsCreateDirRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#base, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#path, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#base: <BaseDirectory as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#path: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsCreateDirResponse {}
impl ::tuffite::__private::mojom::Value for FsCreateDirResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsCreateDirResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsExistsRequest {
    pub r#base: BaseDirectory,
    pub r#path: String,
}
impl ::tuffite::__private::mojom::Value for FsExistsRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsExistsRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#base, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#path, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#base: <BaseDirectory as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#path: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsExistsResponse {
    pub r#result: bool,
}
impl ::tuffite::__private::mojom::Value for FsExistsResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsExistsResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <bool as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsReadFileRequest {
    pub r#base: BaseDirectory,
    pub r#path: String,
}
impl ::tuffite::__private::mojom::Value for FsReadFileRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsReadFileRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#base, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#path, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#base: <BaseDirectory as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#path: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsReadFileResponse {
    pub r#result: Vec<u8>,
}
impl ::tuffite::__private::mojom::Value for FsReadFileResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsReadFileResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <Vec<
                u8,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsRemoveFileRequest {
    pub r#base: BaseDirectory,
    pub r#path: String,
}
impl ::tuffite::__private::mojom::Value for FsRemoveFileRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsRemoveFileRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#base, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#path, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#base: <BaseDirectory as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#path: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsRemoveFileResponse {}
impl ::tuffite::__private::mojom::Value for FsRemoveFileResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsRemoveFileResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsWriteFileRequest {
    pub r#base: BaseDirectory,
    pub r#path: String,
    pub r#contents: Vec<u8>,
}
impl ::tuffite::__private::mojom::Value for FsWriteFileRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsWriteFileRequest {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#base, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#path, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#contents, w, base + 24, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#base: <BaseDirectory as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#path: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
            r#contents: <Vec<
                u8,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct FsWriteFileResponse {}
impl ::tuffite::__private::mojom::Value for FsWriteFileResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for FsWriteFileResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogDebugRequest {
    pub r#message: String,
}
impl ::tuffite::__private::mojom::Value for LogDebugRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogDebugRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#message, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#message: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogDebugResponse {}
impl ::tuffite::__private::mojom::Value for LogDebugResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogDebugResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogErrorRequest {
    pub r#message: String,
}
impl ::tuffite::__private::mojom::Value for LogErrorRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogErrorRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#message, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#message: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogErrorResponse {}
impl ::tuffite::__private::mojom::Value for LogErrorResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogErrorResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogInfoRequest {
    pub r#message: String,
}
impl ::tuffite::__private::mojom::Value for LogInfoRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogInfoRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#message, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#message: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogInfoResponse {}
impl ::tuffite::__private::mojom::Value for LogInfoResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogInfoResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogTraceRequest {
    pub r#message: String,
}
impl ::tuffite::__private::mojom::Value for LogTraceRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogTraceRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#message, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#message: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogTraceResponse {}
impl ::tuffite::__private::mojom::Value for LogTraceResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogTraceResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogWarnRequest {
    pub r#message: String,
}
impl ::tuffite::__private::mojom::Value for LogWarnRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogWarnRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#message, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#message: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct LogWarnResponse {}
impl ::tuffite::__private::mojom::Value for LogWarnResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for LogWarnResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct NetRequestRequest {
    pub r#url: String,
}
impl ::tuffite::__private::mojom::Value for NetRequestRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for NetRequestRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#url, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#url: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct NetRequestResponse {
    pub r#result: NetworkResponse,
}
impl ::tuffite::__private::mojom::Value for NetRequestResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for NetRequestResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <NetworkResponse as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct NetworkResponse {
    pub r#status: f64,
    pub r#body: Vec<u8>,
}
impl ::tuffite::__private::mojom::Value for NetworkResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for NetworkResponse {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#status, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#body, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#status: <f64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#body: <Vec<
                u8,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct OpenDirectoryDialogOptions {
    pub r#title: Option<String>,
}
impl ::tuffite::__private::mojom::Value for OpenDirectoryDialogOptions {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for OpenDirectoryDialogOptions {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#title, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#title: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct OpenFileDialogOptions {
    pub r#title: Option<String>,
    pub r#default_path: Option<String>,
    pub r#extensions: Option<Vec<Vec<String>>>,
}
impl ::tuffite::__private::mojom::Value for OpenFileDialogOptions {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for OpenFileDialogOptions {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#title, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#default_path, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#extensions, w, base + 24, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#title: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#default_path: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#extensions: <Option<
                Vec<Vec<String>>,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct PathDirectories {
    pub r#resource: String,
    pub r#data: String,
    pub r#cache: String,
    pub r#temp: String,
}
impl ::tuffite::__private::mojom::Value for PathDirectories {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for PathDirectories {
    const SIZE: usize = 40;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#resource, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#data, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#cache, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#temp, w, base + 32, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#resource: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#data: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
            r#cache: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 24,
                0,
            )?,
            r#temp: <String as ::tuffite::__private::mojom::Value>::get(r, base + 32, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct PathDirectoriesRequest {}
impl ::tuffite::__private::mojom::Value for PathDirectoriesRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for PathDirectoriesRequest {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct PathDirectoriesResponse {
    pub r#result: PathDirectories,
}
impl ::tuffite::__private::mojom::Value for PathDirectoriesResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for PathDirectoriesResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <PathDirectories as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct PathResolveRequest {
    pub r#base: BaseDirectory,
    pub r#relative: Option<String>,
}
impl ::tuffite::__private::mojom::Value for PathResolveRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for PathResolveRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#base, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#relative, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#base: <BaseDirectory as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#relative: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct PathResolveResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for PathResolveResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for PathResolveResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingCapabilities {
    pub r#available: bool,
    pub r#protocol: String,
    pub r#video_codecs: Vec<String>,
    pub r#input: bool,
    pub r#clipboard: bool,
    pub r#reconnect: bool,
    pub r#snapshot: bool,
}
impl ::tuffite::__private::mojom::Value for RemotingCapabilities {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingCapabilities {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#available, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#protocol, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#video_codecs, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#input, w, base + 8, 1)?;
        ::tuffite::__private::mojom::Value::put(&self.r#clipboard, w, base + 8, 2)?;
        ::tuffite::__private::mojom::Value::put(&self.r#reconnect, w, base + 8, 3)?;
        ::tuffite::__private::mojom::Value::put(&self.r#snapshot, w, base + 8, 4)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#available: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#protocol: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
            r#video_codecs: <Vec<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
            r#input: <bool as ::tuffite::__private::mojom::Value>::get(r, base + 8, 1)?,
            r#clipboard: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                2,
            )?,
            r#reconnect: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                3,
            )?,
            r#snapshot: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                4,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingCapabilitiesRequest {}
impl ::tuffite::__private::mojom::Value for RemotingCapabilitiesRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingCapabilitiesRequest {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingCapabilitiesResponse {
    pub r#result: RemotingCapabilities,
}
impl ::tuffite::__private::mojom::Value for RemotingCapabilitiesResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingCapabilitiesResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <RemotingCapabilities as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingCaptureRequest {
    pub r#screen_id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for RemotingCaptureRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingCaptureRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.boolean(base + 8, 0, self.r#screen_id.is_some())?;
        if let Some(value) = &self.r#screen_id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#screen_id: if r.boolean(base + 8, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingCaptureResponse {
    pub r#result: RemotingSnapshot,
}
impl ::tuffite::__private::mojom::Value for RemotingCaptureResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingCaptureResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <RemotingSnapshot as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingIceCandidate {
    pub r#mid: String,
    pub r#m_line_index: f64,
    pub r#candidate: String,
}
impl ::tuffite::__private::mojom::Value for RemotingIceCandidate {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingIceCandidate {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#mid, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#m_line_index, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#candidate, w, base + 24, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#mid: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#m_line_index: <f64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
            r#candidate: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 24,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingIceServer {
    pub r#urls: Vec<String>,
    pub r#username: Option<String>,
    pub r#credential: Option<String>,
}
impl ::tuffite::__private::mojom::Value for RemotingIceServer {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingIceServer {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#urls, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#username, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#credential, w, base + 24, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#urls: <Vec<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#username: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#credential: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingInjectInputRequest {
    pub r#event: RemotingInputEvent,
}
impl ::tuffite::__private::mojom::Value for RemotingInjectInputRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingInjectInputRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#event, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#event: <RemotingInputEvent as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingInjectInputResponse {}
impl ::tuffite::__private::mojom::Value for RemotingInjectInputResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingInjectInputResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingInputEvent {
    pub r#kind: String,
    pub r#pressed: Option<bool>,
    pub r#usb_keycode: Option<f64>,
    pub r#text: Option<String>,
    pub r#x: Option<f64>,
    pub r#y: Option<f64>,
    pub r#button: Option<String>,
    pub r#button_down: Option<bool>,
    pub r#wheel_delta_x: Option<f64>,
    pub r#wheel_delta_y: Option<f64>,
    pub r#wheel_ticks_x: Option<f64>,
    pub r#wheel_ticks_y: Option<f64>,
    pub r#clipboard_mime_type: Option<String>,
    pub r#clipboard_data: Option<Vec<u8>>,
}
impl ::tuffite::__private::mojom::Value for RemotingInputEvent {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingInputEvent {
    const SIZE: usize = 112;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#kind, w, base + 8, 0)?;
        w.boolean(base + 16, 0, self.r#pressed.is_some())?;
        if let Some(value) = &self.r#pressed {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 1)?;
        }
        w.boolean(base + 16, 2, self.r#usb_keycode.is_some())?;
        if let Some(value) = &self.r#usb_keycode {
            ::tuffite::__private::mojom::Value::put(value, w, base + 24, 0)?;
        }
        ::tuffite::__private::mojom::Value::put(&self.r#text, w, base + 32, 0)?;
        w.boolean(base + 16, 3, self.r#x.is_some())?;
        if let Some(value) = &self.r#x {
            ::tuffite::__private::mojom::Value::put(value, w, base + 40, 0)?;
        }
        w.boolean(base + 16, 4, self.r#y.is_some())?;
        if let Some(value) = &self.r#y {
            ::tuffite::__private::mojom::Value::put(value, w, base + 48, 0)?;
        }
        ::tuffite::__private::mojom::Value::put(&self.r#button, w, base + 56, 0)?;
        w.boolean(base + 16, 5, self.r#button_down.is_some())?;
        if let Some(value) = &self.r#button_down {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 6)?;
        }
        w.boolean(base + 16, 7, self.r#wheel_delta_x.is_some())?;
        if let Some(value) = &self.r#wheel_delta_x {
            ::tuffite::__private::mojom::Value::put(value, w, base + 64, 0)?;
        }
        w.boolean(base + 17, 0, self.r#wheel_delta_y.is_some())?;
        if let Some(value) = &self.r#wheel_delta_y {
            ::tuffite::__private::mojom::Value::put(value, w, base + 72, 0)?;
        }
        w.boolean(base + 17, 1, self.r#wheel_ticks_x.is_some())?;
        if let Some(value) = &self.r#wheel_ticks_x {
            ::tuffite::__private::mojom::Value::put(value, w, base + 80, 0)?;
        }
        w.boolean(base + 17, 2, self.r#wheel_ticks_y.is_some())?;
        if let Some(value) = &self.r#wheel_ticks_y {
            ::tuffite::__private::mojom::Value::put(value, w, base + 88, 0)?;
        }
        ::tuffite::__private::mojom::Value::put(
            &self.r#clipboard_mime_type,
            w,
            base + 96,
            0,
        )?;
        ::tuffite::__private::mojom::Value::put(
            &self.r#clipboard_data,
            w,
            base + 104,
            0,
        )?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#kind: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#pressed: if r.boolean(base + 16, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 1)?)
            } else {
                None
            },
            r#usb_keycode: if r.boolean(base + 16, 2)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 24, 0)?)
            } else {
                None
            },
            r#text: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 32, 0)?,
            r#x: if r.boolean(base + 16, 3)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 40, 0)?)
            } else {
                None
            },
            r#y: if r.boolean(base + 16, 4)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 48, 0)?)
            } else {
                None
            },
            r#button: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 56, 0)?,
            r#button_down: if r.boolean(base + 16, 5)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 6)?)
            } else {
                None
            },
            r#wheel_delta_x: if r.boolean(base + 16, 7)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 64, 0)?)
            } else {
                None
            },
            r#wheel_delta_y: if r.boolean(base + 17, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 72, 0)?)
            } else {
                None
            },
            r#wheel_ticks_x: if r.boolean(base + 17, 1)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 80, 0)?)
            } else {
                None
            },
            r#wheel_ticks_y: if r.boolean(base + 17, 2)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 88, 0)?)
            } else {
                None
            },
            r#clipboard_mime_type: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 96, 0)?,
            r#clipboard_data: <Option<
                Vec<u8>,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 104, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingProcessSignalRequest {
    pub r#id: i64,
    pub r#signal: RemotingSignal,
}
impl ::tuffite::__private::mojom::Value for RemotingProcessSignalRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingProcessSignalRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#signal, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#signal: <RemotingSignal as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingProcessSignalResponse {}
impl ::tuffite::__private::mojom::Value for RemotingProcessSignalResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingProcessSignalResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSendInputRequest {
    pub r#id: i64,
    pub r#event: RemotingInputEvent,
}
impl ::tuffite::__private::mojom::Value for RemotingSendInputRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingSendInputRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#event, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#event: <RemotingInputEvent as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSendInputResponse {}
impl ::tuffite::__private::mojom::Value for RemotingSendInputResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingSendInputResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSession {
    pub r#id: i64,
    pub r#state: String,
}
impl ::tuffite::__private::mojom::Value for RemotingSession {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingSession {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#state, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#state: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSessionDescription {
    pub r#type: String,
    pub r#sdp: String,
}
impl ::tuffite::__private::mojom::Value for RemotingSessionDescription {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingSessionDescription {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#type, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#sdp, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#type: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#sdp: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSignal {
    pub r#sequence: i64,
    pub r#description: Option<RemotingSessionDescription>,
    pub r#candidates: Vec<RemotingIceCandidate>,
}
impl ::tuffite::__private::mojom::Value for RemotingSignal {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingSignal {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#sequence, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#description, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#candidates, w, base + 24, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#sequence: <i64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#description: <Option<
                RemotingSessionDescription,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#candidates: <Vec<
                RemotingIceCandidate,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSnapshot {
    pub r#width: f64,
    pub r#height: f64,
    pub r#mime_type: String,
    pub r#data: Vec<u8>,
}
impl ::tuffite::__private::mojom::Value for RemotingSnapshot {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingSnapshot {
    const SIZE: usize = 40;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#width, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#height, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#mime_type, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#data, w, base + 32, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#width: <f64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#height: <f64 as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#mime_type: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 24,
                0,
            )?,
            r#data: <Vec<
                u8,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 32, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStartOptions {
    pub r#auth_key: String,
    pub r#ice_servers: Vec<RemotingIceServer>,
    pub r#max_bitrate_kbps: Option<f64>,
    pub r#target_framerate: Option<f64>,
    pub r#screen_id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for RemotingStartOptions {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStartOptions {
    const SIZE: usize = 56;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#auth_key, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#ice_servers, w, base + 16, 0)?;
        w.boolean(base + 24, 0, self.r#max_bitrate_kbps.is_some())?;
        if let Some(value) = &self.r#max_bitrate_kbps {
            ::tuffite::__private::mojom::Value::put(value, w, base + 32, 0)?;
        }
        w.boolean(base + 24, 1, self.r#target_framerate.is_some())?;
        if let Some(value) = &self.r#target_framerate {
            ::tuffite::__private::mojom::Value::put(value, w, base + 40, 0)?;
        }
        w.boolean(base + 24, 2, self.r#screen_id.is_some())?;
        if let Some(value) = &self.r#screen_id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 48, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#auth_key: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#ice_servers: <Vec<
                RemotingIceServer,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#max_bitrate_kbps: if r.boolean(base + 24, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 32, 0)?)
            } else {
                None
            },
            r#target_framerate: if r.boolean(base + 24, 1)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 40, 0)?)
            } else {
                None
            },
            r#screen_id: if r.boolean(base + 24, 2)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 48, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStartRequest {
    pub r#options: RemotingStartOptions,
}
impl ::tuffite::__private::mojom::Value for RemotingStartRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStartRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#options, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#options: <RemotingStartOptions as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStartResponse {
    pub r#result: RemotingSession,
}
impl ::tuffite::__private::mojom::Value for RemotingStartResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStartResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <RemotingSession as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStats {
    pub r#state: String,
    pub r#route: String,
    pub r#transport: String,
    pub r#latency_ms: f64,
    pub r#jitter_ms: f64,
    pub r#available_bandwidth_kbps: f64,
    pub r#packet_loss_percent: f64,
}
impl ::tuffite::__private::mojom::Value for RemotingStats {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStats {
    const SIZE: usize = 64;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#state, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#route, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#transport, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#latency_ms, w, base + 32, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#jitter_ms, w, base + 40, 0)?;
        ::tuffite::__private::mojom::Value::put(
            &self.r#available_bandwidth_kbps,
            w,
            base + 48,
            0,
        )?;
        ::tuffite::__private::mojom::Value::put(
            &self.r#packet_loss_percent,
            w,
            base + 56,
            0,
        )?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#state: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#route: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
            r#transport: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 24,
                0,
            )?,
            r#latency_ms: <f64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 32,
                0,
            )?,
            r#jitter_ms: <f64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 40,
                0,
            )?,
            r#available_bandwidth_kbps: <f64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 48,
                0,
            )?,
            r#packet_loss_percent: <f64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 56,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStatsRequest {
    pub r#id: i64,
}
impl ::tuffite::__private::mojom::Value for RemotingStatsRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStatsRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStatsResponse {
    pub r#result: RemotingStats,
}
impl ::tuffite::__private::mojom::Value for RemotingStatsResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStatsResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <RemotingStats as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStopRequest {
    pub r#id: i64,
}
impl ::tuffite::__private::mojom::Value for RemotingStopRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStopRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStopResponse {}
impl ::tuffite::__private::mojom::Value for RemotingStopResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingStopResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingTakeSignalsRequest {
    pub r#id: i64,
    pub r#after_sequence: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for RemotingTakeSignalsRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingTakeSignalsRequest {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        w.boolean(base + 16, 0, self.r#after_sequence.is_some())?;
        if let Some(value) = &self.r#after_sequence {
            ::tuffite::__private::mojom::Value::put(value, w, base + 24, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#after_sequence: if r.boolean(base + 16, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 24, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingTakeSignalsResponse {
    pub r#result: Vec<RemotingSignal>,
}
impl ::tuffite::__private::mojom::Value for RemotingTakeSignalsResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for RemotingTakeSignalsResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <Vec<
                RemotingSignal,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageClearRequest {}
impl ::tuffite::__private::mojom::Value for StorageClearRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageClearRequest {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageClearResponse {}
impl ::tuffite::__private::mojom::Value for StorageClearResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageClearResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageGetRequest {
    pub r#key: String,
}
impl ::tuffite::__private::mojom::Value for StorageGetRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageGetRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#key, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#key: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageGetResponse {
    pub r#result: Option<Vec<u8>>,
}
impl ::tuffite::__private::mojom::Value for StorageGetResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageGetResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <Option<
                Vec<u8>,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageRemoveRequest {
    pub r#key: String,
}
impl ::tuffite::__private::mojom::Value for StorageRemoveRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageRemoveRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#key, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#key: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageRemoveResponse {}
impl ::tuffite::__private::mojom::Value for StorageRemoveResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageRemoveResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageSetRequest {
    pub r#key: String,
    pub r#value: Vec<u8>,
}
impl ::tuffite::__private::mojom::Value for StorageSetRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageSetRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#key, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#value, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#key: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#value: <Vec<
                u8,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct StorageSetResponse {}
impl ::tuffite::__private::mojom::Value for StorageSetResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for StorageSetResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct UtilityCancelRequest {
    pub r#name: String,
    pub r#request_id: i64,
}
impl ::tuffite::__private::mojom::Value for UtilityCancelRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for UtilityCancelRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#name, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#request_id, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#name: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#request_id: <i64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct UtilityCancelResponse {}
impl ::tuffite::__private::mojom::Value for UtilityCancelResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for UtilityCancelResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct UtilityInvokeRequest {
    pub r#name: String,
    pub r#request_id: i64,
    pub r#payload: Option<Vec<u8>>,
}
impl ::tuffite::__private::mojom::Value for UtilityInvokeRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for UtilityInvokeRequest {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#name, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#request_id, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#payload, w, base + 24, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#name: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#request_id: <i64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
            r#payload: <Option<
                Vec<u8>,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct UtilityInvokeResponse {
    pub r#result: Vec<u8>,
}
impl ::tuffite::__private::mojom::Value for UtilityInvokeResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for UtilityInvokeResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <Vec<
                u8,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowBounds {
    pub r#x: f64,
    pub r#y: f64,
    pub r#width: f64,
    pub r#height: f64,
}
impl ::tuffite::__private::mojom::Value for WindowBounds {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowBounds {
    const SIZE: usize = 40;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#x, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#y, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#width, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#height, w, base + 32, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#x: <f64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#y: <f64 as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#width: <f64 as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
            r#height: <f64 as ::tuffite::__private::mojom::Value>::get(r, base + 32, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowCloseRequest {
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowCloseRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowCloseRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.boolean(base + 8, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: if r.boolean(base + 8, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowCloseResponse {}
impl ::tuffite::__private::mojom::Value for WindowCloseResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowCloseResponse {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowCreateRequest {
    pub r#options: Option<WindowOptions>,
}
impl ::tuffite::__private::mojom::Value for WindowCreateRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowCreateRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#options, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#options: <Option<
                WindowOptions,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowCreateResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowCreateResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowCreateResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowCurrentRequest {}
impl ::tuffite::__private::mojom::Value for WindowCurrentRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowCurrentRequest {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowCurrentResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowCurrentResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowCurrentResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowFocusRequest {
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowFocusRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowFocusRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.boolean(base + 8, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: if r.boolean(base + 8, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowFocusResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowFocusResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowFocusResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowGetBoundsRequest {
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowGetBoundsRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowGetBoundsRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.boolean(base + 8, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: if r.boolean(base + 8, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowGetBoundsResponse {
    pub r#result: WindowBounds,
}
impl ::tuffite::__private::mojom::Value for WindowGetBoundsResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowGetBoundsResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowBounds as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowInfo {
    pub r#id: i64,
    pub r#label: String,
    pub r#title: String,
    pub r#url: String,
    pub r#bounds: WindowBounds,
    pub r#focused: bool,
    pub r#minimized: bool,
    pub r#maximized: bool,
    pub r#fullscreen: bool,
}
impl ::tuffite::__private::mojom::Value for WindowInfo {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowInfo {
    const SIZE: usize = 56;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#label, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#title, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#url, w, base + 32, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#bounds, w, base + 40, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#focused, w, base + 48, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#minimized, w, base + 48, 1)?;
        ::tuffite::__private::mojom::Value::put(&self.r#maximized, w, base + 48, 2)?;
        ::tuffite::__private::mojom::Value::put(&self.r#fullscreen, w, base + 48, 3)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#label: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 16,
                0,
            )?,
            r#title: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 24,
                0,
            )?,
            r#url: <String as ::tuffite::__private::mojom::Value>::get(r, base + 32, 0)?,
            r#bounds: <WindowBounds as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 40,
                0,
            )?,
            r#focused: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 48,
                0,
            )?,
            r#minimized: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 48,
                1,
            )?,
            r#maximized: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 48,
                2,
            )?,
            r#fullscreen: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 48,
                3,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowListRequest {}
impl ::tuffite::__private::mojom::Value for WindowListRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowListRequest {
    const SIZE: usize = 8;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {};
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowListResponse {
    pub r#result: Vec<WindowInfo>,
}
impl ::tuffite::__private::mojom::Value for WindowListResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowListResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <Vec<
                WindowInfo,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowMaximizeRequest {
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowMaximizeRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowMaximizeRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.boolean(base + 8, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: if r.boolean(base + 8, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowMaximizeResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowMaximizeResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowMaximizeResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowMinimizeRequest {
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowMinimizeRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowMinimizeRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.boolean(base + 8, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: if r.boolean(base + 8, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowMinimizeResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowMinimizeResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowMinimizeResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowOptions {
    pub r#label: Option<String>,
    pub r#url: Option<String>,
    pub r#title: Option<String>,
    pub r#bounds: Option<WindowBounds>,
    pub r#focused: Option<bool>,
}
impl ::tuffite::__private::mojom::Value for WindowOptions {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowOptions {
    const SIZE: usize = 48;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#label, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#url, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#title, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#bounds, w, base + 32, 0)?;
        w.boolean(base + 40, 0, self.r#focused.is_some())?;
        if let Some(value) = &self.r#focused {
            ::tuffite::__private::mojom::Value::put(value, w, base + 40, 1)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#label: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#url: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#title: <Option<
                String,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 24, 0)?,
            r#bounds: <Option<
                WindowBounds,
            > as ::tuffite::__private::mojom::Value>::get(r, base + 32, 0)?,
            r#focused: if r.boolean(base + 40, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 40, 1)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowRestoreRequest {
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowRestoreRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowRestoreRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.boolean(base + 8, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: if r.boolean(base + 8, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowRestoreResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowRestoreResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowRestoreResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowSetBoundsRequest {
    pub r#bounds: WindowBounds,
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowSetBoundsRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowSetBoundsRequest {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#bounds, w, base + 8, 0)?;
        w.boolean(base + 16, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 24, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#bounds: <WindowBounds as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#id: if r.boolean(base + 16, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 24, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowSetBoundsResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowSetBoundsResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowSetBoundsResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowSetFullscreenRequest {
    pub r#fullscreen: bool,
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowSetFullscreenRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowSetFullscreenRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#fullscreen, w, base + 8, 0)?;
        w.boolean(base + 8, 1, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 16, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#fullscreen: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#id: if r.boolean(base + 8, 1)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 16, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowSetFullscreenResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowSetFullscreenResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowSetFullscreenResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowSetTitleRequest {
    pub r#title: String,
    pub r#id: Option<i64>,
}
impl ::tuffite::__private::mojom::Value for WindowSetTitleRequest {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowSetTitleRequest {
    const SIZE: usize = 32;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#title, w, base + 8, 0)?;
        w.boolean(base + 16, 0, self.r#id.is_some())?;
        if let Some(value) = &self.r#id {
            ::tuffite::__private::mojom::Value::put(value, w, base + 24, 0)?;
        }
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#title: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#id: if r.boolean(base + 16, 0)? {
                Some(::tuffite::__private::mojom::Value::get(r, base + 24, 0)?)
            } else {
                None
            },
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowSetTitleResponse {
    pub r#result: WindowInfo,
}
impl ::tuffite::__private::mojom::Value for WindowSetTitleResponse {
    const WIDTH: usize = 8;
    const REFERENCE: bool = true;
    fn put(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        at: usize,
        _: u8,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        w.structure(at, self)
    }
    fn get(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        at: usize,
        _: u8,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        r.structure(at)
    }
}
impl ::tuffite::__private::mojom::Struct for WindowSetTitleResponse {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#result, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#result: <WindowInfo as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
pub mod dialog {
    pub type OpenFile = super::DialogOpenFileRequest;
    impl ::tuffite::ShellApiRequest for super::DialogOpenFileRequest {
        type Response = Option<String>;
        const CHANNEL: &'static str = "tuffite.Dialog.OpenFile";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::DialogOpenFileResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type OpenDirectory = super::DialogOpenDirectoryRequest;
    impl ::tuffite::ShellApiRequest for super::DialogOpenDirectoryRequest {
        type Response = Option<String>;
        const CHANNEL: &'static str = "tuffite.Dialog.OpenDirectory";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::DialogOpenDirectoryResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
}
pub mod fs {
    pub type ReadFile = super::FsReadFileRequest;
    impl ::tuffite::ShellApiRequest for super::FsReadFileRequest {
        type Response = Vec<u8>;
        const CHANNEL: &'static str = "tuffite.Fs.ReadFile";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::FsReadFileResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type WriteFile = super::FsWriteFileRequest;
    impl ::tuffite::ShellApiRequest for super::FsWriteFileRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Fs.WriteFile";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::FsWriteFileResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Exists = super::FsExistsRequest;
    impl ::tuffite::ShellApiRequest for super::FsExistsRequest {
        type Response = bool;
        const CHANNEL: &'static str = "tuffite.Fs.Exists";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::FsExistsResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type CreateDir = super::FsCreateDirRequest;
    impl ::tuffite::ShellApiRequest for super::FsCreateDirRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Fs.CreateDir";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::FsCreateDirResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type RemoveFile = super::FsRemoveFileRequest;
    impl ::tuffite::ShellApiRequest for super::FsRemoveFileRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Fs.RemoveFile";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::FsRemoveFileResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
}
pub mod log {
    pub type Trace = super::LogTraceRequest;
    impl ::tuffite::ShellApiRequest for super::LogTraceRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Log.Trace";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::LogTraceResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Debug = super::LogDebugRequest;
    impl ::tuffite::ShellApiRequest for super::LogDebugRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Log.Debug";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::LogDebugResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Info = super::LogInfoRequest;
    impl ::tuffite::ShellApiRequest for super::LogInfoRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Log.Info";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::LogInfoResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Warn = super::LogWarnRequest;
    impl ::tuffite::ShellApiRequest for super::LogWarnRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Log.Warn";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::LogWarnResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Error = super::LogErrorRequest;
    impl ::tuffite::ShellApiRequest for super::LogErrorRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Log.Error";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::LogErrorResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
}
pub mod net {
    use super::NetworkResponse;
    pub type Request = super::NetRequestRequest;
    impl ::tuffite::ShellApiRequest for super::NetRequestRequest {
        type Response = NetworkResponse;
        const CHANNEL: &'static str = "tuffite.Net.Request";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::NetRequestResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
}
pub mod path {
    use super::PathDirectories;
    pub type Directories = super::PathDirectoriesRequest;
    impl ::tuffite::ShellApiRequest for super::PathDirectoriesRequest {
        type Response = PathDirectories;
        const CHANNEL: &'static str = "tuffite.Path.Directories";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::PathDirectoriesResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Resolve = super::PathResolveRequest;
    impl ::tuffite::ShellApiRequest for super::PathResolveRequest {
        type Response = String;
        const CHANNEL: &'static str = "tuffite.Path.Resolve";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::PathResolveResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
}
pub mod remoting {
    use super::{
        RemotingCapabilities, RemotingSession, RemotingSignal, RemotingSnapshot,
        RemotingStats,
    };
    pub type Capabilities = super::RemotingCapabilitiesRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingCapabilitiesRequest {
        type Response = RemotingCapabilities;
        const CHANNEL: &'static str = "tuffite.Remoting.Capabilities";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingCapabilitiesResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Start = super::RemotingStartRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingStartRequest {
        type Response = RemotingSession;
        const CHANNEL: &'static str = "tuffite.Remoting.Start";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingStartResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type ProcessSignal = super::RemotingProcessSignalRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingProcessSignalRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Remoting.ProcessSignal";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingProcessSignalResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type TakeSignals = super::RemotingTakeSignalsRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingTakeSignalsRequest {
        type Response = Vec<RemotingSignal>;
        const CHANNEL: &'static str = "tuffite.Remoting.TakeSignals";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingTakeSignalsResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type SendInput = super::RemotingSendInputRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingSendInputRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Remoting.SendInput";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingSendInputResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Stop = super::RemotingStopRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingStopRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Remoting.Stop";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingStopResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Stats = super::RemotingStatsRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingStatsRequest {
        type Response = RemotingStats;
        const CHANNEL: &'static str = "tuffite.Remoting.Stats";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingStatsResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Capture = super::RemotingCaptureRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingCaptureRequest {
        type Response = RemotingSnapshot;
        const CHANNEL: &'static str = "tuffite.Remoting.Capture";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingCaptureResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type InjectInput = super::RemotingInjectInputRequest;
    impl ::tuffite::ShellApiRequest for super::RemotingInjectInputRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Remoting.InjectInput";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::RemotingInjectInputResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
}
pub mod storage {
    pub type Get = super::StorageGetRequest;
    impl ::tuffite::ShellApiRequest for super::StorageGetRequest {
        type Response = Option<Vec<u8>>;
        const CHANNEL: &'static str = "tuffite.Storage.Get";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::StorageGetResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Set = super::StorageSetRequest;
    impl ::tuffite::ShellApiRequest for super::StorageSetRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Storage.Set";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::StorageSetResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Remove = super::StorageRemoveRequest;
    impl ::tuffite::ShellApiRequest for super::StorageRemoveRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Storage.Remove";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::StorageRemoveResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type Clear = super::StorageClearRequest;
    impl ::tuffite::ShellApiRequest for super::StorageClearRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Storage.Clear";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::StorageClearResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
}
pub mod utility {
    pub type Invoke = super::UtilityInvokeRequest;
    impl ::tuffite::ShellApiRequest for super::UtilityInvokeRequest {
        type Response = Vec<u8>;
        const CHANNEL: &'static str = "tuffite.Utility.Invoke";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::UtilityInvokeResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Cancel = super::UtilityCancelRequest;
    impl ::tuffite::ShellApiRequest for super::UtilityCancelRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Utility.Cancel";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::UtilityCancelResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
}
pub mod window {
    use super::{WindowBounds, WindowInfo};
    pub type Current = super::WindowCurrentRequest;
    impl ::tuffite::ShellApiRequest for super::WindowCurrentRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.Current";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowCurrentResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type List = super::WindowListRequest;
    impl ::tuffite::ShellApiRequest for super::WindowListRequest {
        type Response = Vec<WindowInfo>;
        const CHANNEL: &'static str = "tuffite.Window.List";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowListResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Create = super::WindowCreateRequest;
    impl ::tuffite::ShellApiRequest for super::WindowCreateRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.Create";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowCreateResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Focus = super::WindowFocusRequest;
    impl ::tuffite::ShellApiRequest for super::WindowFocusRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.Focus";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowFocusResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Minimize = super::WindowMinimizeRequest;
    impl ::tuffite::ShellApiRequest for super::WindowMinimizeRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.Minimize";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowMinimizeResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Maximize = super::WindowMaximizeRequest;
    impl ::tuffite::ShellApiRequest for super::WindowMaximizeRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.Maximize";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowMaximizeResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Restore = super::WindowRestoreRequest;
    impl ::tuffite::ShellApiRequest for super::WindowRestoreRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.Restore";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowRestoreResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type SetFullscreen = super::WindowSetFullscreenRequest;
    impl ::tuffite::ShellApiRequest for super::WindowSetFullscreenRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.SetFullscreen";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowSetFullscreenResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type Close = super::WindowCloseRequest;
    impl ::tuffite::ShellApiRequest for super::WindowCloseRequest {
        type Response = ();
        const CHANNEL: &'static str = "tuffite.Window.Close";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowCloseResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            let _ = response;
            Ok(())
        }
    }
    pub type SetTitle = super::WindowSetTitleRequest;
    impl ::tuffite::ShellApiRequest for super::WindowSetTitleRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.SetTitle";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowSetTitleResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type GetBounds = super::WindowGetBoundsRequest;
    impl ::tuffite::ShellApiRequest for super::WindowGetBoundsRequest {
        type Response = WindowBounds;
        const CHANNEL: &'static str = "tuffite.Window.GetBounds";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowGetBoundsResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
    pub type SetBounds = super::WindowSetBoundsRequest;
    impl ::tuffite::ShellApiRequest for super::WindowSetBoundsRequest {
        type Response = WindowInfo;
        const CHANNEL: &'static str = "tuffite.Window.SetBounds";
        fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> {
            ::tuffite::__private::mojom::Struct::encode_mojom(self)
        }
        fn decode(
            bytes: &[u8],
        ) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
            let response = <super::WindowSetBoundsResponse as ::tuffite::__private::mojom::Struct>::decode_mojom(
                bytes,
            )?;
            Ok(response.result)
        }
    }
}
