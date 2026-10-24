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
pub struct CommandsStatusRequest {
    pub r#value: Status,
}
impl ::tuffite::__private::mojom::Value for CommandsStatusRequest {
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
impl ::tuffite::__private::mojom::Struct for CommandsStatusRequest {
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
            r#value: <Status as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
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
pub struct CommandsStatusResponse {
    pub r#result: Status,
}
impl ::tuffite::__private::mojom::Value for CommandsStatusResponse {
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
impl ::tuffite::__private::mojom::Struct for CommandsStatusResponse {
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
            r#result: <Status as ::tuffite::__private::mojom::Value>::get(
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
pub struct Status {
    pub r#ready: bool,
}
impl ::tuffite::__private::mojom::Value for Status {
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
impl ::tuffite::__private::mojom::Struct for Status {
    const SIZE: usize = 16;
    fn write(
        &self,
        w: &mut ::tuffite::__private::mojom::Encoder,
        base: usize,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::mojom::Value::put(&self.r#ready, w, base + 8, 0)?;
        let _ = (w, base);
        Ok(())
    }
    fn read(
        r: &mut ::tuffite::__private::mojom::Decoder<'_>,
        base: usize,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = Self {
            r#ready: <bool as ::tuffite::__private::mojom::Value>::get(r, base + 8, 0)?,
        };
        let _ = (r, base);
        Ok(value)
    }
}
#[allow(dead_code)]
pub mod commands {
    pub type Status = super::CommandsStatusRequest;
    impl super::CommandsStatusRequest {
        pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
            "collision.Commands.Status",
            0x0155_62f1_0344_d19c,
            0x36fb_9cab_11c1_c9e5,
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
            result: Result<super::Status, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            let result = result?;
            let _ = &result;
            <super::CommandsStatusResponse as ::tuffite::__private::mojom::Struct>::encode_mojom(
                    &super::CommandsStatusResponse {
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
                super::Status,
            ) -> Result<super::Status, ::tuffite::CommandError>,
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
pub trait Commands {
    fn r#status(&self, r#value: Status) -> Result<Status, ::tuffite::CommandError>;
    #[doc(hidden)]
    const __TUFFITE_INTERFACE: &'static str = "collision.Commands";
    #[doc(hidden)]
    const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[
        commands::Status::COMMAND,
    ];
    #[doc(hidden)]
    #[allow(clippy::too_many_lines)]
    fn __tuffite_dispatch(
        &self,
        command: ::tuffite::Command<'_>,
    ) -> ::tuffite::CommandResult {
        if command.id() == CommandsStatusRequest::COMMAND.id() {
            let request = CommandsStatusRequest::__tuffite_decode(command)?;
            let _ = &request;
            return CommandsStatusRequest::respond(self.r#status(request.r#value));
        }
        Err(::tuffite::CommandError::UnknownCommand)
    }
}
