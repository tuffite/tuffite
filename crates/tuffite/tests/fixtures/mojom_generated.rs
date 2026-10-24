// Generated from Mojom v0. Do not edit.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct AuxProcessIdRequest {}
impl ::tuffite::__private::mojom::Value for AuxProcessIdRequest {
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
impl ::tuffite::__private::mojom::Struct for AuxProcessIdRequest {
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
pub struct AuxProcessIdResponse {
    pub r#result: u32,
}
impl ::tuffite::__private::mojom::Value for AuxProcessIdResponse {
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
impl ::tuffite::__private::mojom::Struct for AuxProcessIdResponse {
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
            r#result: <u32 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
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
pub struct AuxReadStateRequest {}
impl ::tuffite::__private::mojom::Value for AuxReadStateRequest {
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
impl ::tuffite::__private::mojom::Struct for AuxReadStateRequest {
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
pub struct AuxReadStateResponse {
    pub r#result: u64,
}
impl ::tuffite::__private::mojom::Value for AuxReadStateResponse {
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
impl ::tuffite::__private::mojom::Struct for AuxReadStateResponse {
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
            r#result: <u64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
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
pub struct AuxRoundTripRequest {
    pub r#value: MojomSample,
}
impl ::tuffite::__private::mojom::Value for AuxRoundTripRequest {
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
impl ::tuffite::__private::mojom::Struct for AuxRoundTripRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#value, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#value: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct AuxRoundTripResponse {
    pub r#result: MojomSample,
}
impl ::tuffite::__private::mojom::Value for AuxRoundTripResponse {
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
impl ::tuffite::__private::mojom::Struct for AuxRoundTripResponse {
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
            r#result: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct CommandsCheckJavascriptExtensionRequest {}
impl ::tuffite::__private::mojom::Value for CommandsCheckJavascriptExtensionRequest {
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
impl ::tuffite::__private::mojom::Struct for CommandsCheckJavascriptExtensionRequest {
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
pub struct CommandsCheckJavascriptExtensionResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for CommandsCheckJavascriptExtensionResponse {
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
impl ::tuffite::__private::mojom::Struct for CommandsCheckJavascriptExtensionResponse {
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
pub struct CommandsCheckUtilitySessionRequest {}
impl ::tuffite::__private::mojom::Value for CommandsCheckUtilitySessionRequest {
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
impl ::tuffite::__private::mojom::Struct for CommandsCheckUtilitySessionRequest {
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
pub struct CommandsCheckUtilitySessionResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for CommandsCheckUtilitySessionResponse {
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
impl ::tuffite::__private::mojom::Struct for CommandsCheckUtilitySessionResponse {
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
pub struct CommandsDoSomethingRequest {
    pub r#message: String,
}
impl ::tuffite::__private::mojom::Value for CommandsDoSomethingRequest {
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
impl ::tuffite::__private::mojom::Struct for CommandsDoSomethingRequest {
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
pub struct CommandsDoSomethingResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for CommandsDoSomethingResponse {
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
impl ::tuffite::__private::mojom::Struct for CommandsDoSomethingResponse {
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
pub struct CommandsEchoMojomRequest {
    pub r#value: MojomSample,
}
impl ::tuffite::__private::mojom::Value for CommandsEchoMojomRequest {
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
impl ::tuffite::__private::mojom::Struct for CommandsEchoMojomRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#value, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#value: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct CommandsEchoMojomResponse {
    pub r#result: MojomSample,
}
impl ::tuffite::__private::mojom::Value for CommandsEchoMojomResponse {
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
impl ::tuffite::__private::mojom::Struct for CommandsEchoMojomResponse {
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
            r#result: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct DiagnosticsProcessIdRequest {}
impl ::tuffite::__private::mojom::Value for DiagnosticsProcessIdRequest {
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
impl ::tuffite::__private::mojom::Struct for DiagnosticsProcessIdRequest {
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
pub struct DiagnosticsProcessIdResponse {
    pub r#result: u32,
}
impl ::tuffite::__private::mojom::Value for DiagnosticsProcessIdResponse {
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
impl ::tuffite::__private::mojom::Struct for DiagnosticsProcessIdResponse {
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
            r#result: <u32 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
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
pub struct EchoRestrictedRequest {
    pub r#value: MojomSample,
}
impl ::tuffite::__private::mojom::Value for EchoRestrictedRequest {
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
impl ::tuffite::__private::mojom::Struct for EchoRestrictedRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#value, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#value: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct EchoRestrictedResponse {
    pub r#result: MojomSample,
}
impl ::tuffite::__private::mojom::Value for EchoRestrictedResponse {
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
impl ::tuffite::__private::mojom::Struct for EchoRestrictedResponse {
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
            r#result: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct EchoRoundTripRequest {
    pub r#value: MojomSample,
}
impl ::tuffite::__private::mojom::Value for EchoRoundTripRequest {
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
impl ::tuffite::__private::mojom::Struct for EchoRoundTripRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#value, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#value: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct EchoRoundTripResponse {
    pub r#result: MojomSample,
}
impl ::tuffite::__private::mojom::Value for EchoRoundTripResponse {
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
impl ::tuffite::__private::mojom::Struct for EchoRoundTripResponse {
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
            r#result: <MojomSample as ::tuffite::__private::mojom::Value>::get(
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
pub struct MojomNested {
    pub r#label: String,
    pub r#bytes: Vec<u8>,
}
impl ::tuffite::__private::mojom::Value for MojomNested {
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
impl ::tuffite::__private::mojom::Struct for MojomNested {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#label, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#bytes, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#label: <String as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#bytes: <Vec<
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
pub struct MojomSample {
    pub r#small: u8,
    pub r#large: u64,
    pub r#enabled: bool,
    pub r#visible: bool,
    pub r#medium: u16,
    pub r#signed_value: i32,
    pub r#fraction: f64,
    pub r#nested: MojomNested,
}
impl ::tuffite::__private::mojom::Value for MojomSample {
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
impl ::tuffite::__private::mojom::Struct for MojomSample {
    const SIZE: usize = 40;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#small, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#large, w, base + 16, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#enabled, w, base + 9, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#visible, w, base + 9, 1)?;
        ::tuffite::__private::mojom::Value::put(&self.r#medium, w, base + 10, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#signed_value, w, base + 12, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#fraction, w, base + 24, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#nested, w, base + 32, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#small: <u8 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#large: <u64 as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
            r#enabled: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 9,
                0,
            )?,
            r#visible: <bool as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 9,
                1,
            )?,
            r#medium: <u16 as ::tuffite::__private::mojom::Value>::get(r, base + 10, 0)?,
            r#signed_value: <i32 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 12,
                0,
            )?,
            r#fraction: <f64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 24,
                0,
            )?,
            r#nested: <MojomNested as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 32,
                0,
            )?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[allow(dead_code)]
pub mod aux {
    pub type RoundTrip = super::AuxRoundTripRequest;
    impl super::AuxRoundTripRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Aux.RoundTrip",
            0x6aa0_86be_7215_89f9,
            0xddfa_86bc_b211_4566,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::AuxRoundTripResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::AuxRoundTripResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(
                &mut A,
                super::MojomSample,
            ) -> Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#value))
        }
    }
    pub type ReadState = super::AuxReadStateRequest;
    impl super::AuxReadStateRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Aux.ReadState",
            0x114a_b85e_38e3_986d,
            0xe2c4_c159_cf6e_38dc,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<u64, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::AuxReadStateResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::AuxReadStateResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A) -> Result<u64, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app))
        }
    }
    pub type ProcessId = super::AuxProcessIdRequest;
    impl super::AuxProcessIdRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Aux.ProcessId",
            0x1309_d795_517c_e447,
            0xc418_0e34_f8de_d239,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<u32, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::AuxProcessIdResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::AuxProcessIdResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A) -> Result<u32, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app))
        }
    }
    /// Registers this interface on the Browser backend.
    pub fn registry<B: super::Aux + ::tuffite::Browser>() -> ::tuffite::__private::CommandRegistry<
        B,
    > {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Aux>::__tuffite_dispatch(browser, command),
            <B as super::Aux>::__TUFFITE_COMMANDS,
        )
    }
}
#[allow(dead_code)]
pub mod commands {
    pub type DoSomething = super::CommandsDoSomethingRequest;
    impl super::CommandsDoSomethingRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Commands.DoSomething",
            0x83e8_37af_20d1_16fb,
            0x65a0_5c3a_c7ee_7b1e,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::CommandsDoSomethingResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::CommandsDoSomethingResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A, String) -> Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#message))
        }
    }
    pub type CheckUtilitySession = super::CommandsCheckUtilitySessionRequest;
    impl super::CommandsCheckUtilitySessionRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Commands.CheckUtilitySession",
            0xc61a_a21d_2183_3df3,
            0xff32_7e16_9194_e4d0,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::CommandsCheckUtilitySessionResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::CommandsCheckUtilitySessionResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A) -> Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app))
        }
    }
    pub type CheckJavascriptExtension = super::CommandsCheckJavascriptExtensionRequest;
    impl super::CommandsCheckJavascriptExtensionRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Commands.CheckJavascriptExtension",
            0x0652_7c3a_9967_b4a2,
            0x43d8_9727_7b68_2c0f,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::CommandsCheckJavascriptExtensionResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::CommandsCheckJavascriptExtensionResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A) -> Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app))
        }
    }
    pub type EchoMojom = super::CommandsEchoMojomRequest;
    impl super::CommandsEchoMojomRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Commands.EchoMojom",
            0x1595_5978_00d6_d178,
            0x2ffa_4399_d986_3635,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::CommandsEchoMojomResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::CommandsEchoMojomResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(
                &mut A,
                super::MojomSample,
            ) -> Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#value))
        }
    }
    /// Registers this interface on the Browser backend.
    pub fn registry<B: super::Commands + ::tuffite::Browser>() -> ::tuffite::__private::CommandRegistry<
        B,
    > {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Commands>::__tuffite_dispatch(
                browser,
                command,
            ),
            <B as super::Commands>::__TUFFITE_COMMANDS,
        )
    }
}
#[allow(dead_code)]
pub mod diagnostics {
    pub type ProcessId = super::DiagnosticsProcessIdRequest;
    impl super::DiagnosticsProcessIdRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Diagnostics.ProcessId",
            0xd81a_d5fc_08d6_14b0,
            0xdcbe_258a_aed2_6b7d,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<u32, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::DiagnosticsProcessIdResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::DiagnosticsProcessIdResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A) -> Result<u32, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app))
        }
    }
    /// Registers this interface on the Browser backend.
    pub fn registry<B: super::Diagnostics + ::tuffite::Browser>() -> ::tuffite::__private::CommandRegistry<
        B,
    > {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Diagnostics>::__tuffite_dispatch(
                browser,
                command,
            ),
            <B as super::Diagnostics>::__TUFFITE_COMMANDS,
        )
    }
}
#[allow(dead_code)]
pub mod echo {
    pub type RoundTrip = super::EchoRoundTripRequest;
    impl super::EchoRoundTripRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Echo.RoundTrip",
            0x58af_025b_7b66_ca5e,
            0xddfa_86bc_b211_4566,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::EchoRoundTripResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::EchoRoundTripResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(
                &mut A,
                super::MojomSample,
            ) -> Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#value))
        }
    }
    pub type Restricted = super::EchoRestrictedRequest;
    impl super::EchoRestrictedRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "playground.Echo.Restricted",
            0x9329_762a_25f6_f9f9,
            0x4439_2603_7b7c_f97a,
        );
        pub fn __tuffite_decode(
            command: ::tuffite::Command<'_>,
        ) -> Result<Self, ::tuffite::CommandError> {
            if command.id() != Self::COMMAND.id() {
                return Err(::tuffite::CommandError::UnknownCommand);
            }
            if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                return Err(
                    ::tuffite::CommandError::SchemaMismatch(
                        "Mojom schema mismatch".into(),
                    ),
                );
            }
            <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(
                    command.payload(),
                )
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid Mojom payload".into(),
                ))
        }
        pub fn respond(
            result: Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::EchoRestrictedResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::EchoRestrictedResponse {
                        r#result,
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(
                &mut A,
                super::MojomSample,
            ) -> Result<super::MojomSample, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#value))
        }
    }
    /// Registers this interface on the Browser backend.
    pub fn registry<B: super::Echo + ::tuffite::Browser>() -> ::tuffite::__private::CommandRegistry<
        B,
    > {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Echo>::__tuffite_dispatch(browser, command),
            <B as super::Echo>::__TUFFITE_COMMANDS,
        )
    }
}
pub trait Aux {
    fn r#round_trip(
        &self,
        r#value: MojomSample,
    ) -> Result<MojomSample, ::tuffite::CommandError>;
    fn r#read_state(&self) -> Result<u64, ::tuffite::CommandError>;
    fn r#process_id(&self) -> Result<u32, ::tuffite::CommandError>;
    #[doc(hidden)]
    const __TUFFITE_INTERFACE: &'static str = "playground.Aux";
    #[doc(hidden)]
    const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[
        aux::RoundTrip::COMMAND,
        aux::ReadState::COMMAND,
        aux::ProcessId::COMMAND,
    ];
    #[doc(hidden)]
    #[allow(clippy::too_many_lines)]
    fn __tuffite_dispatch(
        &self,
        command: ::tuffite::Command<'_>,
    ) -> ::tuffite::CommandResult {
        if command.id() == AuxRoundTripRequest::COMMAND.id() {
            let request = AuxRoundTripRequest::__tuffite_decode(command)?;
            let _ = &request;
            return AuxRoundTripRequest::respond(self.r#round_trip(request.r#value));
        }
        if command.id() == AuxReadStateRequest::COMMAND.id() {
            let request = AuxReadStateRequest::__tuffite_decode(command)?;
            let _ = &request;
            return AuxReadStateRequest::respond(self.r#read_state());
        }
        if command.id() == AuxProcessIdRequest::COMMAND.id() {
            let request = AuxProcessIdRequest::__tuffite_decode(command)?;
            let _ = &request;
            return AuxProcessIdRequest::respond(self.r#process_id());
        }
        Err(::tuffite::CommandError::UnknownCommand)
    }
}
pub trait Commands {
    fn r#do_something(
        &self,
        r#message: String,
    ) -> Result<String, ::tuffite::CommandError>;
    fn r#check_utility_session(&self) -> Result<String, ::tuffite::CommandError>;
    fn r#check_javascript_extension(&self) -> Result<String, ::tuffite::CommandError>;
    fn r#echo_mojom(
        &self,
        r#value: MojomSample,
    ) -> Result<MojomSample, ::tuffite::CommandError>;
    #[doc(hidden)]
    const __TUFFITE_INTERFACE: &'static str = "playground.Commands";
    #[doc(hidden)]
    const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[
        commands::DoSomething::COMMAND,
        commands::CheckUtilitySession::COMMAND,
        commands::CheckJavascriptExtension::COMMAND,
        commands::EchoMojom::COMMAND,
    ];
    #[doc(hidden)]
    #[allow(clippy::too_many_lines)]
    fn __tuffite_dispatch(
        &self,
        command: ::tuffite::Command<'_>,
    ) -> ::tuffite::CommandResult {
        if command.id() == CommandsDoSomethingRequest::COMMAND.id() {
            let request = CommandsDoSomethingRequest::__tuffite_decode(command)?;
            let _ = &request;
            return CommandsDoSomethingRequest::respond(
                self.r#do_something(request.r#message),
            );
        }
        if command.id() == CommandsCheckUtilitySessionRequest::COMMAND.id() {
            let request = CommandsCheckUtilitySessionRequest::__tuffite_decode(command)?;
            let _ = &request;
            return CommandsCheckUtilitySessionRequest::respond(
                self.r#check_utility_session(),
            );
        }
        if command.id() == CommandsCheckJavascriptExtensionRequest::COMMAND.id() {
            let request = CommandsCheckJavascriptExtensionRequest::__tuffite_decode(
                command,
            )?;
            let _ = &request;
            return CommandsCheckJavascriptExtensionRequest::respond(
                self.r#check_javascript_extension(),
            );
        }
        if command.id() == CommandsEchoMojomRequest::COMMAND.id() {
            let request = CommandsEchoMojomRequest::__tuffite_decode(command)?;
            let _ = &request;
            return CommandsEchoMojomRequest::respond(self.r#echo_mojom(request.r#value));
        }
        Err(::tuffite::CommandError::UnknownCommand)
    }
}
pub trait Diagnostics {
    fn r#process_id(&self) -> Result<u32, ::tuffite::CommandError>;
    #[doc(hidden)]
    const __TUFFITE_INTERFACE: &'static str = "playground.Diagnostics";
    #[doc(hidden)]
    const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[
        diagnostics::ProcessId::COMMAND,
    ];
    #[doc(hidden)]
    #[allow(clippy::too_many_lines)]
    fn __tuffite_dispatch(
        &self,
        command: ::tuffite::Command<'_>,
    ) -> ::tuffite::CommandResult {
        if command.id() == DiagnosticsProcessIdRequest::COMMAND.id() {
            let request = DiagnosticsProcessIdRequest::__tuffite_decode(command)?;
            let _ = &request;
            return DiagnosticsProcessIdRequest::respond(self.r#process_id());
        }
        Err(::tuffite::CommandError::UnknownCommand)
    }
}
pub trait Echo {
    fn r#round_trip(
        &self,
        r#value: MojomSample,
    ) -> Result<MojomSample, ::tuffite::CommandError>;
    fn r#restricted(
        &self,
        r#value: MojomSample,
    ) -> Result<MojomSample, ::tuffite::CommandError>;
    #[doc(hidden)]
    const __TUFFITE_INTERFACE: &'static str = "playground.Echo";
    #[doc(hidden)]
    const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[
        echo::RoundTrip::COMMAND,
        echo::Restricted::COMMAND,
    ];
    #[doc(hidden)]
    #[allow(clippy::too_many_lines)]
    fn __tuffite_dispatch(
        &self,
        command: ::tuffite::Command<'_>,
    ) -> ::tuffite::CommandResult {
        if command.id() == EchoRoundTripRequest::COMMAND.id() {
            let request = EchoRoundTripRequest::__tuffite_decode(command)?;
            let _ = &request;
            return EchoRoundTripRequest::respond(self.r#round_trip(request.r#value));
        }
        if command.id() == EchoRestrictedRequest::COMMAND.id() {
            let request = EchoRestrictedRequest::__tuffite_decode(command)?;
            let _ = &request;
            return EchoRestrictedRequest::respond(self.r#restricted(request.r#value));
        }
        Err(::tuffite::CommandError::UnknownCommand)
    }
}
