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
pub struct BrowserDiagnosticsIdentityRequest {}
impl ::tuffite::__private::mojom::Value for BrowserDiagnosticsIdentityRequest {
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
impl ::tuffite::__private::mojom::Struct for BrowserDiagnosticsIdentityRequest {
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
pub struct BrowserDiagnosticsIdentityResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for BrowserDiagnosticsIdentityResponse {
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
impl ::tuffite::__private::mojom::Struct for BrowserDiagnosticsIdentityResponse {
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
pub struct BrowserDiagnosticsIncrementRequest {}
impl ::tuffite::__private::mojom::Value for BrowserDiagnosticsIncrementRequest {
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
impl ::tuffite::__private::mojom::Struct for BrowserDiagnosticsIncrementRequest {
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
pub struct BrowserDiagnosticsIncrementResponse {
    pub r#result: u32,
}
impl ::tuffite::__private::mojom::Value for BrowserDiagnosticsIncrementResponse {
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
impl ::tuffite::__private::mojom::Struct for BrowserDiagnosticsIncrementResponse {
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
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct ProfilesApplicationStateRequest {}
impl ::tuffite::__private::mojom::Value for ProfilesApplicationStateRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesApplicationStateRequest {
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
pub struct ProfilesApplicationStateResponse {
    pub r#result: u32,
}
impl ::tuffite::__private::mojom::Value for ProfilesApplicationStateResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesApplicationStateResponse {
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
pub struct ProfilesAttachRequest {
    pub r#window_id: i64,
    pub r#view: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesAttachRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesAttachRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#window_id, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#view, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#window_id: <i64 as ::tuffite::__private::mojom::Value>::get(
                r,
                base + 8,
                0,
            )?,
            r#view: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
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
pub struct ProfilesAttachResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesAttachResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesAttachResponse {
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
pub struct ProfilesCloseRequest {
    pub r#context: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesCloseRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCloseRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#context, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#context: <String as ::tuffite::__private::mojom::Value>::get(
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
pub struct ProfilesCloseResponse {}
impl ::tuffite::__private::mojom::Value for ProfilesCloseResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCloseResponse {
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
pub struct ProfilesCloseWindowRequest {
    pub r#window_id: i64,
}
impl ::tuffite::__private::mojom::Value for ProfilesCloseWindowRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCloseWindowRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#window_id, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#window_id: <i64 as ::tuffite::__private::mojom::Value>::get(
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
pub struct ProfilesCloseWindowResponse {}
impl ::tuffite::__private::mojom::Value for ProfilesCloseWindowResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCloseWindowResponse {
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
pub struct ProfilesCreateRequest {
    pub r#id: String,
    pub r#name: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesCreateRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCreateRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#name, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#name: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
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
pub struct ProfilesCreateResponse {}
impl ::tuffite::__private::mojom::Value for ProfilesCreateResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCreateResponse {
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
pub struct ProfilesCreateWindowRequest {}
impl ::tuffite::__private::mojom::Value for ProfilesCreateWindowRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCreateWindowRequest {
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
pub struct ProfilesCreateWindowResponse {
    pub r#result: i64,
}
impl ::tuffite::__private::mojom::Value for ProfilesCreateWindowResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesCreateWindowResponse {
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
            r#result: <i64 as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
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
pub struct ProfilesListRequest {}
impl ::tuffite::__private::mojom::Value for ProfilesListRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesListRequest {
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
pub struct ProfilesListResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesListResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesListResponse {
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
pub struct ProfilesPrepareRequest {
    pub r#id: String,
    pub r#url: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesPrepareRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesPrepareRequest {
    const SIZE: usize = 24;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#id, w, base + 8, 0)?;
        ::tuffite::__private::mojom::Value::put(&self.r#url, w, base + 16, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#id: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
            r#url: <String as ::tuffite::__private::mojom::Value>::get(r, base + 16, 0)?,
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
pub struct ProfilesPrepareResponse {
    pub r#result: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesPrepareResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesPrepareResponse {
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
pub struct ProfilesReleaseViewRequest {
    pub r#view: String,
}
impl ::tuffite::__private::mojom::Value for ProfilesReleaseViewRequest {
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
impl ::tuffite::__private::mojom::Struct for ProfilesReleaseViewRequest {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#view, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#view: <String as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
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
pub struct ProfilesReleaseViewResponse {}
impl ::tuffite::__private::mojom::Value for ProfilesReleaseViewResponse {
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
impl ::tuffite::__private::mojom::Struct for ProfilesReleaseViewResponse {
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
#[allow(dead_code, non_snake_case)]
pub mod aux {
    pub type RoundTrip = super::AuxRoundTripRequest;
    impl super::AuxRoundTripRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Aux.RoundTrip",
                0x6aa0_86be_7215_89f9,
                0xddfa_86bc_b211_4566,
            )
            .with_ordinal(0u32);
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
            )
            .with_ordinal(1u32);
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
            )
            .with_ordinal(2u32);
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
    /// Registers this interface on an Application or Browser backend.
    pub fn registry<B: super::Aux>() -> ::tuffite::__private::CommandRegistry<B> {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Aux>::__tuffite_dispatch(browser, command),
            <B as super::Aux>::__TUFFITE_COMMANDS,
        )
    }
}
#[allow(dead_code, non_snake_case)]
pub mod browserDiagnostics {
    pub type Identity = super::BrowserDiagnosticsIdentityRequest;
    impl super::BrowserDiagnosticsIdentityRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.BrowserDiagnostics.Identity",
                0x58f4_c53e_02c8_f05d,
                0x9ad8_9e2d_1099_976c,
            )
            .with_ordinal(0u32);
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
            <super::BrowserDiagnosticsIdentityResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::BrowserDiagnosticsIdentityResponse {
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
    pub type Increment = super::BrowserDiagnosticsIncrementRequest;
    impl super::BrowserDiagnosticsIncrementRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.BrowserDiagnostics.Increment",
                0x902e_376b_c585_b70d,
                0xa8f9_759d_8049_db8e,
            )
            .with_ordinal(1u32);
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
            <super::BrowserDiagnosticsIncrementResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::BrowserDiagnosticsIncrementResponse {
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
    /// Registers this interface on an Application or Browser backend.
    pub fn registry<B: super::BrowserDiagnostics>() -> ::tuffite::__private::CommandRegistry<
        B,
    > {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::BrowserDiagnostics>::__tuffite_dispatch(
                browser,
                command,
            ),
            <B as super::BrowserDiagnostics>::__TUFFITE_COMMANDS,
        )
    }
}
#[allow(dead_code, non_snake_case)]
pub mod commands {
    pub type DoSomething = super::CommandsDoSomethingRequest;
    impl super::CommandsDoSomethingRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Commands.DoSomething",
                0x83e8_37af_20d1_16fb,
                0x65a0_5c3a_c7ee_7b1e,
            )
            .with_ordinal(0u32);
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
            )
            .with_ordinal(1u32);
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
            )
            .with_ordinal(2u32);
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
            )
            .with_ordinal(3u32);
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
    /// Registers this interface on an Application or Browser backend.
    pub fn registry<B: super::Commands>() -> ::tuffite::__private::CommandRegistry<B> {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Commands>::__tuffite_dispatch(
                browser,
                command,
            ),
            <B as super::Commands>::__TUFFITE_COMMANDS,
        )
    }
}
#[allow(dead_code, non_snake_case)]
pub mod diagnostics {
    pub type ProcessId = super::DiagnosticsProcessIdRequest;
    impl super::DiagnosticsProcessIdRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Diagnostics.ProcessId",
                0xd81a_d5fc_08d6_14b0,
                0xdcbe_258a_aed2_6b7d,
            )
            .with_ordinal(0u32);
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
    /// Registers this interface on an Application or Browser backend.
    pub fn registry<B: super::Diagnostics>() -> ::tuffite::__private::CommandRegistry<
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
#[allow(dead_code, non_snake_case)]
pub mod echo {
    pub type RoundTrip = super::EchoRoundTripRequest;
    impl super::EchoRoundTripRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Echo.RoundTrip",
                0x58af_025b_7b66_ca5e,
                0xddfa_86bc_b211_4566,
            )
            .with_ordinal(0u32);
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
            )
            .with_ordinal(1u32);
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
    /// Registers this interface on an Application or Browser backend.
    pub fn registry<B: super::Echo>() -> ::tuffite::__private::CommandRegistry<B> {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Echo>::__tuffite_dispatch(browser, command),
            <B as super::Echo>::__TUFFITE_COMMANDS,
        )
    }
}
#[allow(dead_code, non_snake_case)]
pub mod profiles {
    pub type List = super::ProfilesListRequest;
    impl super::ProfilesListRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.List",
                0x6fef_7683_4bd4_15be,
                0x9ad8_9e2d_1099_976c,
            )
            .with_ordinal(0u32);
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
            <super::ProfilesListResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesListResponse {
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
    pub type Create = super::ProfilesCreateRequest;
    impl super::ProfilesCreateRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.Create",
                0xb691_e427_65a2_ab02,
                0x24ab_bf55_09a2_6ced,
            )
            .with_ordinal(1u32);
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
            result: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            result?;
            <super::ProfilesCreateResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesCreateResponse {},
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A, String, String) -> Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#id, request.r#name))
        }
    }
    pub type Prepare = super::ProfilesPrepareRequest;
    impl super::ProfilesPrepareRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.Prepare",
                0x37cc_2b2e_a0c4_226e,
                0x21f4_8f42_7f68_33a5,
            )
            .with_ordinal(2u32);
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
            <super::ProfilesPrepareResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesPrepareResponse {
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
                String,
                String,
            ) -> Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#id, request.r#url))
        }
    }
    pub type Attach = super::ProfilesAttachRequest;
    impl super::ProfilesAttachRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.Attach",
                0x93d6_a32b_8242_9b9b,
                0x8e17_797e_b4ce_ef74,
            )
            .with_ordinal(3u32);
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
            <super::ProfilesAttachResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesAttachResponse {
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
            handler: fn(&mut A, i64, String) -> Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#window_id, request.r#view))
        }
    }
    pub type ReleaseView = super::ProfilesReleaseViewRequest;
    impl super::ProfilesReleaseViewRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.ReleaseView",
                0x635c_d6d2_c39d_37e7,
                0x4324_574e_baf4_51d5,
            )
            .with_ordinal(4u32);
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
            result: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            result?;
            <super::ProfilesReleaseViewResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesReleaseViewResponse {
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#view))
        }
    }
    pub type Close = super::ProfilesCloseRequest;
    impl super::ProfilesCloseRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.Close",
                0x0c4c_b572_c1ae_a249,
                0x5404_6f99_3e6d_d968,
            )
            .with_ordinal(5u32);
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
            result: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            result?;
            <super::ProfilesCloseResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesCloseResponse {},
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#context))
        }
    }
    pub type ApplicationState = super::ProfilesApplicationStateRequest;
    impl super::ProfilesApplicationStateRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.ApplicationState",
                0x216b_d68c_91e2_d7b1,
                0x3593_5a08_b94a_d2b6,
            )
            .with_ordinal(6u32);
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
            <super::ProfilesApplicationStateResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesApplicationStateResponse {
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
    pub type CreateWindow = super::ProfilesCreateWindowRequest;
    impl super::ProfilesCreateWindowRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.CreateWindow",
                0x71e7_29b5_b1af_ad66,
                0x3b5d_acf6_c469_b9a8,
            )
            .with_ordinal(7u32);
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
            result: Result<i64, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            <super::ProfilesCreateWindowResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesCreateWindowResponse {
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
            handler: fn(&mut A) -> Result<i64, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app))
        }
    }
    pub type CloseWindow = super::ProfilesCloseWindowRequest;
    impl super::ProfilesCloseWindowRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
                "playground.Profiles.CloseWindow",
                0x9949_081e_dfe9_825b,
                0xf299_1096_defc_8c85,
            )
            .with_ordinal(8u32);
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
            result: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            result?;
            <super::ProfilesCloseWindowResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::ProfilesCloseWindowResponse {
                    },
                )
                .map_err(|_| ::tuffite::CommandError::Failed(
                    "Mojom encode failed".into(),
                ))
        }
        pub fn __tuffite_dispatch<A>(
            command: ::tuffite::Command<'_>,
            app: &mut A,
            handler: fn(&mut A, i64) -> Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let request = Self::__tuffite_decode(command)?;
            let _ = &request;
            Self::respond(handler(app, request.r#window_id))
        }
    }
    /// Registers this interface on an Application or Browser backend.
    pub fn registry<B: super::Profiles>() -> ::tuffite::__private::CommandRegistry<B> {
        ::tuffite::__private::CommandRegistry::new(
            |browser, command| <B as super::Profiles>::__tuffite_dispatch(
                browser,
                command,
            ),
            <B as super::Profiles>::__TUFFITE_COMMANDS,
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
pub trait BrowserDiagnostics {
    fn r#identity(&self) -> Result<String, ::tuffite::CommandError>;
    fn r#increment(&self) -> Result<u32, ::tuffite::CommandError>;
    #[doc(hidden)]
    const __TUFFITE_INTERFACE: &'static str = "playground.BrowserDiagnostics";
    #[doc(hidden)]
    const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[
        browserDiagnostics::Identity::COMMAND,
        browserDiagnostics::Increment::COMMAND,
    ];
    #[doc(hidden)]
    #[allow(clippy::too_many_lines)]
    fn __tuffite_dispatch(
        &self,
        command: ::tuffite::Command<'_>,
    ) -> ::tuffite::CommandResult {
        if command.id() == BrowserDiagnosticsIdentityRequest::COMMAND.id() {
            let request = BrowserDiagnosticsIdentityRequest::__tuffite_decode(command)?;
            let _ = &request;
            return BrowserDiagnosticsIdentityRequest::respond(self.r#identity());
        }
        if command.id() == BrowserDiagnosticsIncrementRequest::COMMAND.id() {
            let request = BrowserDiagnosticsIncrementRequest::__tuffite_decode(command)?;
            let _ = &request;
            return BrowserDiagnosticsIncrementRequest::respond(self.r#increment());
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
pub trait Profiles {
    fn r#list(&self) -> Result<String, ::tuffite::CommandError>;
    fn r#create(
        &self,
        r#id: String,
        r#name: String,
    ) -> Result<(), ::tuffite::CommandError>;
    fn r#prepare(
        &self,
        r#id: String,
        r#url: String,
    ) -> Result<String, ::tuffite::CommandError>;
    fn r#attach(
        &self,
        r#window_id: i64,
        r#view: String,
    ) -> Result<String, ::tuffite::CommandError>;
    fn r#release_view(&self, r#view: String) -> Result<(), ::tuffite::CommandError>;
    fn r#close(&self, r#context: String) -> Result<(), ::tuffite::CommandError>;
    fn r#application_state(&self) -> Result<u32, ::tuffite::CommandError>;
    fn r#create_window(&self) -> Result<i64, ::tuffite::CommandError>;
    fn r#close_window(&self, r#window_id: i64) -> Result<(), ::tuffite::CommandError>;
    #[doc(hidden)]
    const __TUFFITE_INTERFACE: &'static str = "playground.Profiles";
    #[doc(hidden)]
    const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[
        profiles::List::COMMAND,
        profiles::Create::COMMAND,
        profiles::Prepare::COMMAND,
        profiles::Attach::COMMAND,
        profiles::ReleaseView::COMMAND,
        profiles::Close::COMMAND,
        profiles::ApplicationState::COMMAND,
        profiles::CreateWindow::COMMAND,
        profiles::CloseWindow::COMMAND,
    ];
    #[doc(hidden)]
    #[allow(clippy::too_many_lines)]
    fn __tuffite_dispatch(
        &self,
        command: ::tuffite::Command<'_>,
    ) -> ::tuffite::CommandResult {
        if command.id() == ProfilesListRequest::COMMAND.id() {
            let request = ProfilesListRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesListRequest::respond(self.r#list());
        }
        if command.id() == ProfilesCreateRequest::COMMAND.id() {
            let request = ProfilesCreateRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesCreateRequest::respond(
                self.r#create(request.r#id, request.r#name),
            );
        }
        if command.id() == ProfilesPrepareRequest::COMMAND.id() {
            let request = ProfilesPrepareRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesPrepareRequest::respond(
                self.r#prepare(request.r#id, request.r#url),
            );
        }
        if command.id() == ProfilesAttachRequest::COMMAND.id() {
            let request = ProfilesAttachRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesAttachRequest::respond(
                self.r#attach(request.r#window_id, request.r#view),
            );
        }
        if command.id() == ProfilesReleaseViewRequest::COMMAND.id() {
            let request = ProfilesReleaseViewRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesReleaseViewRequest::respond(
                self.r#release_view(request.r#view),
            );
        }
        if command.id() == ProfilesCloseRequest::COMMAND.id() {
            let request = ProfilesCloseRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesCloseRequest::respond(self.r#close(request.r#context));
        }
        if command.id() == ProfilesApplicationStateRequest::COMMAND.id() {
            let request = ProfilesApplicationStateRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesApplicationStateRequest::respond(self.r#application_state());
        }
        if command.id() == ProfilesCreateWindowRequest::COMMAND.id() {
            let request = ProfilesCreateWindowRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesCreateWindowRequest::respond(self.r#create_window());
        }
        if command.id() == ProfilesCloseWindowRequest::COMMAND.id() {
            let request = ProfilesCloseWindowRequest::__tuffite_decode(command)?;
            let _ = &request;
            return ProfilesCloseWindowRequest::respond(
                self.r#close_window(request.r#window_id),
            );
        }
        Err(::tuffite::CommandError::UnknownCommand)
    }
}
#[allow(dead_code)]
pub mod interfaces {
    /// Type-level interface registration for application and browser scopes.
    pub struct Aux;
    impl<T: super::Aux> ::tuffite::utility::InterfaceFor<T> for Aux {
        fn interface() -> ::tuffite::utility::Interface<T> {
            ::tuffite::utility::Interface::new(
                <T as super::Aux>::__TUFFITE_INTERFACE,
                <T as super::Aux>::__tuffite_dispatch,
                <T as super::Aux>::__TUFFITE_COMMANDS,
            )
        }
    }
    impl<T: super::Aux> ::tuffite::app::InterfaceFor<T> for Aux {
        fn registry() -> ::tuffite::__private::CommandRegistry<T> {
            super::aux::registry::<T>()
        }
    }
    /// Type-level interface registration for application and browser scopes.
    pub struct BrowserDiagnostics;
    impl<T: super::BrowserDiagnostics> ::tuffite::utility::InterfaceFor<T>
    for BrowserDiagnostics {
        fn interface() -> ::tuffite::utility::Interface<T> {
            ::tuffite::utility::Interface::new(
                <T as super::BrowserDiagnostics>::__TUFFITE_INTERFACE,
                <T as super::BrowserDiagnostics>::__tuffite_dispatch,
                <T as super::BrowserDiagnostics>::__TUFFITE_COMMANDS,
            )
        }
    }
    impl<T: super::BrowserDiagnostics> ::tuffite::app::InterfaceFor<T>
    for BrowserDiagnostics {
        fn registry() -> ::tuffite::__private::CommandRegistry<T> {
            super::browserDiagnostics::registry::<T>()
        }
    }
    /// Type-level interface registration for application and browser scopes.
    pub struct Commands;
    impl<T: super::Commands> ::tuffite::utility::InterfaceFor<T> for Commands {
        fn interface() -> ::tuffite::utility::Interface<T> {
            ::tuffite::utility::Interface::new(
                <T as super::Commands>::__TUFFITE_INTERFACE,
                <T as super::Commands>::__tuffite_dispatch,
                <T as super::Commands>::__TUFFITE_COMMANDS,
            )
        }
    }
    impl<T: super::Commands> ::tuffite::app::InterfaceFor<T> for Commands {
        fn registry() -> ::tuffite::__private::CommandRegistry<T> {
            super::commands::registry::<T>()
        }
    }
    /// Type-level interface registration for application and browser scopes.
    pub struct Diagnostics;
    impl<T: super::Diagnostics> ::tuffite::utility::InterfaceFor<T> for Diagnostics {
        fn interface() -> ::tuffite::utility::Interface<T> {
            ::tuffite::utility::Interface::new(
                <T as super::Diagnostics>::__TUFFITE_INTERFACE,
                <T as super::Diagnostics>::__tuffite_dispatch,
                <T as super::Diagnostics>::__TUFFITE_COMMANDS,
            )
        }
    }
    impl<T: super::Diagnostics> ::tuffite::app::InterfaceFor<T> for Diagnostics {
        fn registry() -> ::tuffite::__private::CommandRegistry<T> {
            super::diagnostics::registry::<T>()
        }
    }
    /// Type-level interface registration for application and browser scopes.
    pub struct Echo;
    impl<T: super::Echo> ::tuffite::utility::InterfaceFor<T> for Echo {
        fn interface() -> ::tuffite::utility::Interface<T> {
            ::tuffite::utility::Interface::new(
                <T as super::Echo>::__TUFFITE_INTERFACE,
                <T as super::Echo>::__tuffite_dispatch,
                <T as super::Echo>::__TUFFITE_COMMANDS,
            )
        }
    }
    impl<T: super::Echo> ::tuffite::app::InterfaceFor<T> for Echo {
        fn registry() -> ::tuffite::__private::CommandRegistry<T> {
            super::echo::registry::<T>()
        }
    }
    /// Type-level interface registration for application and browser scopes.
    pub struct Profiles;
    impl<T: super::Profiles> ::tuffite::utility::InterfaceFor<T> for Profiles {
        fn interface() -> ::tuffite::utility::Interface<T> {
            ::tuffite::utility::Interface::new(
                <T as super::Profiles>::__TUFFITE_INTERFACE,
                <T as super::Profiles>::__tuffite_dispatch,
                <T as super::Profiles>::__TUFFITE_COMMANDS,
            )
        }
    }
    impl<T: super::Profiles> ::tuffite::app::InterfaceFor<T> for Profiles {
        fn registry() -> ::tuffite::__private::CommandRegistry<T> {
            super::profiles::registry::<T>()
        }
    }
}
