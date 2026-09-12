// Generated from ../../framework/shell_api/common.d.ts, ../../framework/shell_api/dialog.d.ts, ../../framework/shell_api/path.d.ts, ../../framework/shell_api/remoting.d.ts, ../../framework/shell_api/fs.d.ts, ../../framework/shell_api/storage.d.ts, ../../framework/shell_api/log.d.ts, ../../framework/shell_api/network.d.ts, ../../framework/shell_api/utility.d.ts, ../../framework/shell_api/window.d.ts. Do not edit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaseDirectory {
    Resource,
    Data,
    Cache,
    Temp,
}
impl ::tuffite::__private::serde::Serialize for BaseDirectory {
    fn serialize<S: ::tuffite::__private::serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value: String = match self {
            Self::Resource => String::from("resource"),
            Self::Data => String::from("data"),
            Self::Cache => String::from("cache"),
            Self::Temp => String::from("temp"),
        };
        ::tuffite::__private::serde::Serialize::serialize(&value, serializer)
    }
}
impl<'de> ::tuffite::__private::serde::Deserialize<'de> for BaseDirectory {
    fn deserialize<D: ::tuffite::__private::serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let value = <String as ::tuffite::__private::serde::Deserialize>::deserialize(
            deserializer,
        )?;
        match value {
            value if value == String::from("resource") => Ok(Self::Resource),
            value if value == String::from("data") => Ok(Self::Data),
            value if value == String::from("cache") => Ok(Self::Cache),
            value if value == String::from("temp") => Ok(Self::Temp),
            _ => {
                Err(
                    ::tuffite::__private::serde::de::Error::custom(
                        "invalid Shell API enum value",
                    ),
                )
            }
        }
    }
}
impl ::tuffite::__private::codec::Encode for BaseDirectory {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let value: String = match self {
            Self::Resource => String::from("resource"),
            Self::Data => String::from("data"),
            Self::Cache => String::from("cache"),
            Self::Temp => String::from("temp"),
        };
        ::tuffite::__private::codec::Encode::encode(&value, encoder)
    }
}
impl ::tuffite::__private::codec::Decode for BaseDirectory {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        let value = <String as ::tuffite::__private::codec::Decode>::decode(decoder)?;
        match value {
            value if value == String::from("resource") => Ok(Self::Resource),
            value if value == String::from("data") => Ok(Self::Data),
            value if value == String::from("cache") => Ok(Self::Cache),
            value if value == String::from("temp") => Ok(Self::Temp),
            _ => Err(::tuffite::__private::codec::CodecError),
        }
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct NetworkResponse {
    pub status: f64,
    pub body: ::tuffite::Bytes,
}
impl ::tuffite::__private::codec::Encode for NetworkResponse {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.status, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.body, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for NetworkResponse {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            status: ::tuffite::__private::codec::Decode::decode(decoder)?,
            body: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct OpenDirectoryDialogOptions {
    pub title: Option<String>,
}
impl ::tuffite::__private::codec::Encode for OpenDirectoryDialogOptions {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.title, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for OpenDirectoryDialogOptions {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            title: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct OpenFileDialogOptions {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub extensions: Option<Vec<Vec<String>>>,
}
impl ::tuffite::__private::codec::Encode for OpenFileDialogOptions {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.title, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.default_path, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.extensions, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for OpenFileDialogOptions {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            title: ::tuffite::__private::codec::Decode::decode(decoder)?,
            default_path: ::tuffite::__private::codec::Decode::decode(decoder)?,
            extensions: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct PathDirectories {
    pub resource: String,
    pub data: String,
    pub cache: String,
    pub temp: String,
}
impl ::tuffite::__private::codec::Encode for PathDirectories {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.resource, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.data, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.cache, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.temp, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for PathDirectories {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            resource: ::tuffite::__private::codec::Decode::decode(decoder)?,
            data: ::tuffite::__private::codec::Decode::decode(decoder)?,
            cache: ::tuffite::__private::codec::Decode::decode(decoder)?,
            temp: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingCapabilities {
    pub available: bool,
    pub protocol: String,
    pub video_codecs: Vec<String>,
    pub input: bool,
    pub clipboard: bool,
    pub reconnect: bool,
    pub snapshot: bool,
}
impl ::tuffite::__private::codec::Encode for RemotingCapabilities {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.available, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.protocol, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.video_codecs, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.input, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.clipboard, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.reconnect, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.snapshot, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingCapabilities {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            available: ::tuffite::__private::codec::Decode::decode(decoder)?,
            protocol: ::tuffite::__private::codec::Decode::decode(decoder)?,
            video_codecs: ::tuffite::__private::codec::Decode::decode(decoder)?,
            input: ::tuffite::__private::codec::Decode::decode(decoder)?,
            clipboard: ::tuffite::__private::codec::Decode::decode(decoder)?,
            reconnect: ::tuffite::__private::codec::Decode::decode(decoder)?,
            snapshot: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingIceCandidate {
    pub mid: String,
    pub m_line_index: f64,
    pub candidate: String,
}
impl ::tuffite::__private::codec::Encode for RemotingIceCandidate {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.mid, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.m_line_index, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.candidate, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingIceCandidate {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            mid: ::tuffite::__private::codec::Decode::decode(decoder)?,
            m_line_index: ::tuffite::__private::codec::Decode::decode(decoder)?,
            candidate: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingIceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}
impl ::tuffite::__private::codec::Encode for RemotingIceServer {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.urls, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.username, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.credential, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingIceServer {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            urls: ::tuffite::__private::codec::Decode::decode(decoder)?,
            username: ::tuffite::__private::codec::Decode::decode(decoder)?,
            credential: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingInputEvent {
    pub kind: String,
    pub pressed: Option<bool>,
    pub usb_keycode: Option<f64>,
    pub text: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub button: Option<String>,
    pub button_down: Option<bool>,
    pub wheel_delta_x: Option<f64>,
    pub wheel_delta_y: Option<f64>,
    pub wheel_ticks_x: Option<f64>,
    pub wheel_ticks_y: Option<f64>,
    pub clipboard_mime_type: Option<String>,
    pub clipboard_data: Option<::tuffite::Bytes>,
}
impl ::tuffite::__private::codec::Encode for RemotingInputEvent {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.kind, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.pressed, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.usb_keycode, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.text, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.x, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.y, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.button, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.button_down, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.wheel_delta_x, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.wheel_delta_y, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.wheel_ticks_x, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.wheel_ticks_y, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.clipboard_mime_type, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.clipboard_data, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingInputEvent {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            kind: ::tuffite::__private::codec::Decode::decode(decoder)?,
            pressed: ::tuffite::__private::codec::Decode::decode(decoder)?,
            usb_keycode: ::tuffite::__private::codec::Decode::decode(decoder)?,
            text: ::tuffite::__private::codec::Decode::decode(decoder)?,
            x: ::tuffite::__private::codec::Decode::decode(decoder)?,
            y: ::tuffite::__private::codec::Decode::decode(decoder)?,
            button: ::tuffite::__private::codec::Decode::decode(decoder)?,
            button_down: ::tuffite::__private::codec::Decode::decode(decoder)?,
            wheel_delta_x: ::tuffite::__private::codec::Decode::decode(decoder)?,
            wheel_delta_y: ::tuffite::__private::codec::Decode::decode(decoder)?,
            wheel_ticks_x: ::tuffite::__private::codec::Decode::decode(decoder)?,
            wheel_ticks_y: ::tuffite::__private::codec::Decode::decode(decoder)?,
            clipboard_mime_type: ::tuffite::__private::codec::Decode::decode(decoder)?,
            clipboard_data: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSession {
    pub id: i64,
    pub state: String,
}
impl ::tuffite::__private::codec::Encode for RemotingSession {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.state, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingSession {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            id: ::tuffite::__private::codec::Decode::decode(decoder)?,
            state: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSessionDescription {
    pub r#type: String,
    pub sdp: String,
}
impl ::tuffite::__private::codec::Encode for RemotingSessionDescription {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.r#type, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.sdp, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingSessionDescription {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            r#type: ::tuffite::__private::codec::Decode::decode(decoder)?,
            sdp: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSignal {
    pub sequence: i64,
    pub description: Option<RemotingSessionDescription>,
    pub candidates: Vec<RemotingIceCandidate>,
}
impl ::tuffite::__private::codec::Encode for RemotingSignal {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.sequence, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.description, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.candidates, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingSignal {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            sequence: ::tuffite::__private::codec::Decode::decode(decoder)?,
            description: ::tuffite::__private::codec::Decode::decode(decoder)?,
            candidates: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingSnapshot {
    pub width: f64,
    pub height: f64,
    pub mime_type: String,
    pub data: ::tuffite::Bytes,
}
impl ::tuffite::__private::codec::Encode for RemotingSnapshot {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.width, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.height, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.mime_type, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.data, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingSnapshot {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            width: ::tuffite::__private::codec::Decode::decode(decoder)?,
            height: ::tuffite::__private::codec::Decode::decode(decoder)?,
            mime_type: ::tuffite::__private::codec::Decode::decode(decoder)?,
            data: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStartOptions {
    pub auth_key: String,
    pub ice_servers: Vec<RemotingIceServer>,
    pub max_bitrate_kbps: Option<f64>,
    pub target_framerate: Option<f64>,
    pub screen_id: Option<i64>,
}
impl ::tuffite::__private::codec::Encode for RemotingStartOptions {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.auth_key, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.ice_servers, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.max_bitrate_kbps, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.target_framerate, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.screen_id, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingStartOptions {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            auth_key: ::tuffite::__private::codec::Decode::decode(decoder)?,
            ice_servers: ::tuffite::__private::codec::Decode::decode(decoder)?,
            max_bitrate_kbps: ::tuffite::__private::codec::Decode::decode(decoder)?,
            target_framerate: ::tuffite::__private::codec::Decode::decode(decoder)?,
            screen_id: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct RemotingStats {
    pub state: String,
    pub route: String,
    pub transport: String,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub available_bandwidth_kbps: f64,
    pub packet_loss_percent: f64,
}
impl ::tuffite::__private::codec::Encode for RemotingStats {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.state, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.route, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.transport, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.latency_ms, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.jitter_ms, encoder)?;
        ::tuffite::__private::codec::Encode::encode(
            &self.available_bandwidth_kbps,
            encoder,
        )?;
        ::tuffite::__private::codec::Encode::encode(&self.packet_loss_percent, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for RemotingStats {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            state: ::tuffite::__private::codec::Decode::decode(decoder)?,
            route: ::tuffite::__private::codec::Decode::decode(decoder)?,
            transport: ::tuffite::__private::codec::Decode::decode(decoder)?,
            latency_ms: ::tuffite::__private::codec::Decode::decode(decoder)?,
            jitter_ms: ::tuffite::__private::codec::Decode::decode(decoder)?,
            available_bandwidth_kbps: ::tuffite::__private::codec::Decode::decode(
                decoder,
            )?,
            packet_loss_percent: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl ::tuffite::__private::codec::Encode for WindowBounds {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.x, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.y, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.width, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.height, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for WindowBounds {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            x: ::tuffite::__private::codec::Decode::decode(decoder)?,
            y: ::tuffite::__private::codec::Decode::decode(decoder)?,
            width: ::tuffite::__private::codec::Decode::decode(decoder)?,
            height: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
pub type WindowId = i64;
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowInfo {
    pub id: WindowId,
    pub label: String,
    pub title: String,
    pub url: String,
    pub bounds: WindowBounds,
    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,
}
impl ::tuffite::__private::codec::Encode for WindowInfo {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.label, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.title, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.url, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.bounds, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.focused, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.minimized, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.maximized, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.fullscreen, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for WindowInfo {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            id: ::tuffite::__private::codec::Decode::decode(decoder)?,
            label: ::tuffite::__private::codec::Decode::decode(decoder)?,
            title: ::tuffite::__private::codec::Decode::decode(decoder)?,
            url: ::tuffite::__private::codec::Decode::decode(decoder)?,
            bounds: ::tuffite::__private::codec::Decode::decode(decoder)?,
            focused: ::tuffite::__private::codec::Decode::decode(decoder)?,
            minimized: ::tuffite::__private::codec::Decode::decode(decoder)?,
            maximized: ::tuffite::__private::codec::Decode::decode(decoder)?,
            fullscreen: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::tuffite::__private::serde::Serialize,
    ::tuffite::__private::serde::Deserialize
)]
#[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
pub struct WindowOptions {
    pub label: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub bounds: Option<WindowBounds>,
    pub focused: Option<bool>,
}
impl ::tuffite::__private::codec::Encode for WindowOptions {
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        ::tuffite::__private::codec::Encode::encode(&self.label, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.url, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.title, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.bounds, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.focused, encoder)?;
        Ok(())
    }
}
impl ::tuffite::__private::codec::Decode for WindowOptions {
    fn decode(
        decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
    ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
        Ok(Self {
            label: ::tuffite::__private::codec::Decode::decode(decoder)?,
            url: ::tuffite::__private::codec::Decode::decode(decoder)?,
            title: ::tuffite::__private::codec::Decode::decode(decoder)?,
            bounds: ::tuffite::__private::codec::Decode::decode(decoder)?,
            focused: ::tuffite::__private::codec::Decode::decode(decoder)?,
        })
    }
}
pub mod dialog {
    #[derive(Clone, Debug, PartialEq)]
    pub struct OpenFile {
        pub options: Option<super::OpenFileDialogOptions>,
    }
    impl OpenFile {
        pub fn respond(
            response: Result<Option<String>, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct OpenDirectory {
        pub options: Option<super::OpenDirectoryDialogOptions>,
    }
    impl OpenDirectory {
        pub fn respond(
            response: Result<Option<String>, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod path {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Directories {}
    impl Directories {
        pub fn respond(
            response: Result<super::PathDirectories, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Resolve {
        pub base: super::BaseDirectory,
        pub relative: Option<String>,
    }
    impl Resolve {
        pub fn respond(
            response: Result<String, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod remoting {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Capabilities {}
    impl Capabilities {
        pub fn respond(
            response: Result<super::RemotingCapabilities, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Start {
        pub options: super::RemotingStartOptions,
    }
    impl Start {
        pub fn respond(
            response: Result<super::RemotingSession, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct ProcessSignal {
        pub id: i64,
        pub signal: super::RemotingSignal,
    }
    impl ProcessSignal {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct TakeSignals {
        pub id: i64,
        pub after_sequence: Option<i64>,
    }
    impl TakeSignals {
        pub fn respond(
            response: Result<Vec<super::RemotingSignal>, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct SendInput {
        pub id: i64,
        pub event: super::RemotingInputEvent,
    }
    impl SendInput {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Stop {
        pub id: i64,
    }
    impl Stop {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Stats {
        pub id: i64,
    }
    impl Stats {
        pub fn respond(
            response: Result<super::RemotingStats, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Capture {
        pub screen_id: Option<i64>,
    }
    impl Capture {
        pub fn respond(
            response: Result<super::RemotingSnapshot, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct InjectInput {
        pub event: super::RemotingInputEvent,
    }
    impl InjectInput {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod fs {
    #[derive(Clone, Debug, PartialEq)]
    pub struct ReadFile {
        pub base: super::BaseDirectory,
        pub path: String,
    }
    impl ReadFile {
        pub fn respond(
            response: Result<::tuffite::Bytes, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct WriteFile {
        pub base: super::BaseDirectory,
        pub path: String,
        pub contents: ::tuffite::Bytes,
    }
    impl WriteFile {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Exists {
        pub base: super::BaseDirectory,
        pub path: String,
    }
    impl Exists {
        pub fn respond(
            response: Result<bool, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct CreateDir {
        pub base: super::BaseDirectory,
        pub path: String,
    }
    impl CreateDir {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct RemoveFile {
        pub base: super::BaseDirectory,
        pub path: String,
    }
    impl RemoveFile {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod storage {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Get {
        pub key: String,
    }
    impl Get {
        pub fn respond(
            response: Result<Option<::tuffite::Bytes>, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Set {
        pub key: String,
        pub value: ::tuffite::Bytes,
    }
    impl Set {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Remove {
        pub key: String,
    }
    impl Remove {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Clear {}
    impl Clear {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod log {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Trace {
        pub message: String,
    }
    impl Trace {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Debug {
        pub message: String,
    }
    impl Debug {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Info {
        pub message: String,
    }
    impl Info {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Warn {
        pub message: String,
    }
    impl Warn {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod net {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Request {
        pub url: String,
    }
    impl Request {
        pub fn respond(
            response: Result<super::NetworkResponse, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod utility {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Invoke {
        pub name: String,
        pub request_id: i64,
        pub payload: Option<::tuffite::Bytes>,
    }
    impl Invoke {
        pub fn respond(
            response: Result<::tuffite::Bytes, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Cancel {
        pub name: String,
        pub request_id: i64,
    }
    impl Cancel {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
pub mod window {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Current {}
    impl Current {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct List {}
    impl List {
        pub fn respond(
            response: Result<Vec<super::WindowInfo>, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Create {
        pub options: Option<super::WindowOptions>,
    }
    impl Create {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Focus {
        pub id: Option<super::WindowId>,
    }
    impl Focus {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Minimize {
        pub id: Option<super::WindowId>,
    }
    impl Minimize {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Maximize {
        pub id: Option<super::WindowId>,
    }
    impl Maximize {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Restore {
        pub id: Option<super::WindowId>,
    }
    impl Restore {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct SetFullscreen {
        pub fullscreen: bool,
        pub id: Option<super::WindowId>,
    }
    impl SetFullscreen {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Close {
        pub id: Option<super::WindowId>,
    }
    impl Close {
        pub fn respond(
            response: Result<(), ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct SetTitle {
        pub title: String,
        pub id: Option<super::WindowId>,
    }
    impl SetTitle {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct GetBounds {
        pub id: Option<super::WindowId>,
    }
    impl GetBounds {
        pub fn respond(
            response: Result<super::WindowBounds, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct SetBounds {
        pub bounds: super::WindowBounds,
        pub id: Option<super::WindowId>,
    }
    impl SetBounds {
        pub fn respond(
            response: Result<super::WindowInfo, ::tuffite::CommandError>,
        ) -> ::tuffite::CommandResult {
            use ::tuffite::__private::codec::Encode as _;
            let response = response?;
            let mut encoder = ::tuffite::__private::codec::Encoder::new();
            response
                .encode(&mut encoder)
                .map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
            Ok(encoder.finish())
        }
    }
}
impl ::tuffite::ShellApiRequest for dialog::OpenFile {
    type Response = Option<String>;
    const CHANNEL: &'static str = "tuffite.dialog.openFile";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.options, encoder)?;
        Ok(())
    }
}
impl dialog::OpenFile {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.dialog.openFile",
        0xf6c3_3f30_40f2_fe25u64,
        0x3608_5a8d_453f_a7f9u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let options: Option<OpenFileDialogOptions> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { options })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<OpenFileDialogOptions>,
        ) -> Result<Option<String>, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.options))
    }
}
impl ::tuffite::ShellApiRequest for dialog::OpenDirectory {
    type Response = Option<String>;
    const CHANNEL: &'static str = "tuffite.dialog.openDirectory";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.options, encoder)?;
        Ok(())
    }
}
impl dialog::OpenDirectory {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.dialog.openDirectory",
        0xb48f_1748_2022_9a1du64,
        0x466e_2f72_d286_69fau64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let options: Option<OpenDirectoryDialogOptions> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { options })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<OpenDirectoryDialogOptions>,
        ) -> Result<Option<String>, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.options))
    }
}
impl ::tuffite::ShellApiRequest for path::Directories {
    type Response = PathDirectories;
    const CHANNEL: &'static str = "tuffite.path.directories";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        Ok(())
    }
}
impl path::Directories {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.path.directories",
        0xdc47_b521_6a77_1dc7u64,
        0xb013_83ce_4e5b_a27au64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        if !command.payload().is_empty() {
            return Err(
                ::tuffite::CommandError::InvalidArgument(
                    "trailing ShellAPI request data".to_owned(),
                ),
            );
        }
        Ok(Self {})
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A) -> Result<PathDirectories, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        Self::__tuffite_decode(command)?;
        Self::respond(handler(application))
    }
}
impl ::tuffite::ShellApiRequest for path::Resolve {
    type Response = String;
    const CHANNEL: &'static str = "tuffite.path.resolve";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.base, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.relative, encoder)?;
        Ok(())
    }
}
impl path::Resolve {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.path.resolve",
        0xc97a_0a14_2d4f_8077u64,
        0x0170_c444_04ed_c31cu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let base: BaseDirectory = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let relative: Option<String> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { base, relative })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            BaseDirectory,
            Option<String>,
        ) -> Result<String, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.base, request.relative))
    }
}
impl ::tuffite::ShellApiRequest for remoting::Capabilities {
    type Response = RemotingCapabilities;
    const CHANNEL: &'static str = "tuffite.remoting.capabilities";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        Ok(())
    }
}
impl remoting::Capabilities {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.capabilities",
        0xd95b_4f76_c83c_2718u64,
        0x2669_08a3_eb5b_2aa5u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        if !command.payload().is_empty() {
            return Err(
                ::tuffite::CommandError::InvalidArgument(
                    "trailing ShellAPI request data".to_owned(),
                ),
            );
        }
        Ok(Self {})
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A) -> Result<RemotingCapabilities, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        Self::__tuffite_decode(command)?;
        Self::respond(handler(application))
    }
}
impl ::tuffite::ShellApiRequest for remoting::Start {
    type Response = RemotingSession;
    const CHANNEL: &'static str = "tuffite.remoting.start";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.options, encoder)?;
        Ok(())
    }
}
impl remoting::Start {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.start",
        0xef17_8b75_5bff_5be8u64,
        0x351d_30bb_9a8e_0c61u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let options: RemotingStartOptions = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { options })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            RemotingStartOptions,
        ) -> Result<RemotingSession, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.options))
    }
}
impl ::tuffite::ShellApiRequest for remoting::ProcessSignal {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.remoting.processSignal";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.signal, encoder)?;
        Ok(())
    }
}
impl remoting::ProcessSignal {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.processSignal",
        0x9259_0b17_22c0_f00cu64,
        0x3259_6946_f161_45cfu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: i64 = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let signal: RemotingSignal = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id, signal })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, i64, RemotingSignal) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id, request.signal))
    }
}
impl ::tuffite::ShellApiRequest for remoting::TakeSignals {
    type Response = Vec<RemotingSignal>;
    const CHANNEL: &'static str = "tuffite.remoting.takeSignals";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.after_sequence, encoder)?;
        Ok(())
    }
}
impl remoting::TakeSignals {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.takeSignals",
        0xcfd0_7b61_ea21_251bu64,
        0x3583_8aef_6986_98a1u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: i64 = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let after_sequence: Option<i64> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id, after_sequence })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            i64,
            Option<i64>,
        ) -> Result<Vec<RemotingSignal>, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id, request.after_sequence))
    }
}
impl ::tuffite::ShellApiRequest for remoting::SendInput {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.remoting.sendInput";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.event, encoder)?;
        Ok(())
    }
}
impl remoting::SendInput {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.sendInput",
        0x8f01_760c_8fbd_e013u64,
        0x9684_7a08_83ac_9883u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: i64 = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let event: RemotingInputEvent = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id, event })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            i64,
            RemotingInputEvent,
        ) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id, request.event))
    }
}
impl ::tuffite::ShellApiRequest for remoting::Stop {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.remoting.stop";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl remoting::Stop {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.stop",
        0x8db3_cd25_bc28_e427u64,
        0x3289_69ae_8937_4051u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: i64 = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, i64) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for remoting::Stats {
    type Response = RemotingStats;
    const CHANNEL: &'static str = "tuffite.remoting.stats";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl remoting::Stats {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.stats",
        0xf501_1f12_12ad_4546u64,
        0xe361_2f19_69a8_6fa4u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: i64 = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, i64) -> Result<RemotingStats, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for remoting::Capture {
    type Response = RemotingSnapshot;
    const CHANNEL: &'static str = "tuffite.remoting.capture";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.screen_id, encoder)?;
        Ok(())
    }
}
impl remoting::Capture {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.capture",
        0x155d_7c65_1339_b2b9u64,
        0xbd9b_d620_30b9_dc25u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let screen_id: Option<i64> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { screen_id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<i64>,
        ) -> Result<RemotingSnapshot, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.screen_id))
    }
}
impl ::tuffite::ShellApiRequest for remoting::InjectInput {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.remoting.injectInput";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.event, encoder)?;
        Ok(())
    }
}
impl remoting::InjectInput {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.remoting.injectInput",
        0x4d93_0f12_59de_1d5fu64,
        0x3e2c_b856_ef0f_0c8eu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let event: RemotingInputEvent = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { event })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, RemotingInputEvent) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.event))
    }
}
impl ::tuffite::ShellApiRequest for fs::ReadFile {
    type Response = ::tuffite::Bytes;
    const CHANNEL: &'static str = "tuffite.fs.readFile";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.base, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.path, encoder)?;
        Ok(())
    }
}
impl fs::ReadFile {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.fs.readFile",
        0xa26f_3393_8cdc_7294u64,
        0xd411_76c0_7662_7b9bu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let base: BaseDirectory = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let path: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { base, path })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            BaseDirectory,
            String,
        ) -> Result<::tuffite::Bytes, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.base, request.path))
    }
}
impl ::tuffite::ShellApiRequest for fs::WriteFile {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.fs.writeFile";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.base, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.path, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.contents, encoder)?;
        Ok(())
    }
}
impl fs::WriteFile {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.fs.writeFile",
        0x2f93_310d_ba55_c933u64,
        0xe4ba_90d2_2c7b_5a79u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let base: BaseDirectory = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let path: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let contents: ::tuffite::Bytes = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { base, path, contents })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            BaseDirectory,
            String,
            ::tuffite::Bytes,
        ) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.base, request.path, request.contents))
    }
}
impl ::tuffite::ShellApiRequest for fs::Exists {
    type Response = bool;
    const CHANNEL: &'static str = "tuffite.fs.exists";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.base, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.path, encoder)?;
        Ok(())
    }
}
impl fs::Exists {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.fs.exists",
        0xfebb_6a4e_2e5b_af29u64,
        0xc294_caaa_f32f_6c6fu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let base: BaseDirectory = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let path: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { base, path })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            BaseDirectory,
            String,
        ) -> Result<bool, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.base, request.path))
    }
}
impl ::tuffite::ShellApiRequest for fs::CreateDir {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.fs.createDir";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.base, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.path, encoder)?;
        Ok(())
    }
}
impl fs::CreateDir {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.fs.createDir",
        0x920d_3f68_cfee_5a2bu64,
        0x3f9f_56ad_94d4_9c0cu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let base: BaseDirectory = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let path: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { base, path })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, BaseDirectory, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.base, request.path))
    }
}
impl ::tuffite::ShellApiRequest for fs::RemoveFile {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.fs.removeFile";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.base, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.path, encoder)?;
        Ok(())
    }
}
impl fs::RemoveFile {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.fs.removeFile",
        0xefb0_bcaf_7707_86cau64,
        0x3f9f_56ad_94d4_9c0cu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let base: BaseDirectory = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let path: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { base, path })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, BaseDirectory, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.base, request.path))
    }
}
impl ::tuffite::ShellApiRequest for storage::Get {
    type Response = Option<::tuffite::Bytes>;
    const CHANNEL: &'static str = "tuffite.storage.get";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.key, encoder)?;
        Ok(())
    }
}
impl storage::Get {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.storage.get",
        0xb840_92a6_e6ca_f9a6u64,
        0x71ea_bd90_27ef_4903u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let key: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { key })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            String,
        ) -> Result<Option<::tuffite::Bytes>, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.key))
    }
}
impl ::tuffite::ShellApiRequest for storage::Set {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.storage.set";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.key, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.value, encoder)?;
        Ok(())
    }
}
impl storage::Set {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.storage.set",
        0x3ed2_e38b_c15b_bfb8u64,
        0x9f36_d4f5_ebbe_c33eu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let key: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let value: ::tuffite::Bytes = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { key, value })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            String,
            ::tuffite::Bytes,
        ) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.key, request.value))
    }
}
impl ::tuffite::ShellApiRequest for storage::Remove {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.storage.remove";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.key, encoder)?;
        Ok(())
    }
}
impl storage::Remove {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.storage.remove",
        0x3b36_008e_53d2_0f70u64,
        0x40aa_8f27_e6e1_fd27u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let key: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { key })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.key))
    }
}
impl ::tuffite::ShellApiRequest for storage::Clear {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.storage.clear";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        Ok(())
    }
}
impl storage::Clear {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.storage.clear",
        0x20da_5788_1305_2815u64,
        0xd3c2_b7fb_02d6_33e2u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        if !command.payload().is_empty() {
            return Err(
                ::tuffite::CommandError::InvalidArgument(
                    "trailing ShellAPI request data".to_owned(),
                ),
            );
        }
        Ok(Self {})
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        Self::__tuffite_decode(command)?;
        Self::respond(handler(application))
    }
}
impl ::tuffite::ShellApiRequest for log::Trace {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.log.trace";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.message, encoder)?;
        Ok(())
    }
}
impl log::Trace {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.log.trace",
        0x4328_4b4e_0543_9064u64,
        0x40aa_8f27_e6e1_fd27u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let message: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { message })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.message))
    }
}
impl ::tuffite::ShellApiRequest for log::Debug {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.log.debug";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.message, encoder)?;
        Ok(())
    }
}
impl log::Debug {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.log.debug",
        0xbe82_2657_209b_1e90u64,
        0x40aa_8f27_e6e1_fd27u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let message: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { message })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.message))
    }
}
impl ::tuffite::ShellApiRequest for log::Info {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.log.info";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.message, encoder)?;
        Ok(())
    }
}
impl log::Info {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.log.info",
        0x4160_d54e_6426_a92cu64,
        0x40aa_8f27_e6e1_fd27u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let message: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { message })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.message))
    }
}
impl ::tuffite::ShellApiRequest for log::Warn {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.log.warn";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.message, encoder)?;
        Ok(())
    }
}
impl log::Warn {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.log.warn",
        0xde6e_140a_5418_6f3eu64,
        0x40aa_8f27_e6e1_fd27u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let message: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { message })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.message))
    }
}
impl ::tuffite::ShellApiRequest for log::Error {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.log.error";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.message, encoder)?;
        Ok(())
    }
}
impl log::Error {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.log.error",
        0xe2dc_7147_6939_9f0fu64,
        0x40aa_8f27_e6e1_fd27u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let message: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { message })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.message))
    }
}
impl ::tuffite::ShellApiRequest for net::Request {
    type Response = NetworkResponse;
    const CHANNEL: &'static str = "tuffite.net.request";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.url, encoder)?;
        Ok(())
    }
}
impl net::Request {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.net.request",
        0x2a0a_1e40_0c90_3a7du64,
        0x353b_4a12_fc82_fd19u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let url: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { url })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String) -> Result<NetworkResponse, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.url))
    }
}
impl ::tuffite::ShellApiRequest for utility::Invoke {
    type Response = ::tuffite::Bytes;
    const CHANNEL: &'static str = "tuffite.utility.invoke";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.name, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.request_id, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.payload, encoder)?;
        Ok(())
    }
}
impl utility::Invoke {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.utility.invoke",
        0x41c7_d64a_8b6e_d93au64,
        0xb528_7876_8ef6_9fb7u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let name: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let request_id: i64 = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let payload: Option<::tuffite::Bytes> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { name, request_id, payload })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            String,
            i64,
            Option<::tuffite::Bytes>,
        ) -> Result<::tuffite::Bytes, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(
            handler(application, request.name, request.request_id, request.payload),
        )
    }
}
impl ::tuffite::ShellApiRequest for utility::Cancel {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.utility.cancel";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.name, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.request_id, encoder)?;
        Ok(())
    }
}
impl utility::Cancel {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.utility.cancel",
        0xff42_61ad_e322_8f69u64,
        0x665d_1fc0_58af_d7beu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let name: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let request_id: i64 = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { name, request_id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, String, i64) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.name, request.request_id))
    }
}
impl ::tuffite::ShellApiRequest for window::Current {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.current";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        Ok(())
    }
}
impl window::Current {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.current",
        0xa775_1ce3_5090_f0adu64,
        0xbe8a_74fb_9eaf_bf49u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        if !command.payload().is_empty() {
            return Err(
                ::tuffite::CommandError::InvalidArgument(
                    "trailing ShellAPI request data".to_owned(),
                ),
            );
        }
        Ok(Self {})
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        Self::__tuffite_decode(command)?;
        Self::respond(handler(application))
    }
}
impl ::tuffite::ShellApiRequest for window::List {
    type Response = Vec<WindowInfo>;
    const CHANNEL: &'static str = "tuffite.window.list";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        Ok(())
    }
}
impl window::List {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.list",
        0xec4d_3013_18c9_bd74u64,
        0xb9f2_e933_315b_14f4u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        if !command.payload().is_empty() {
            return Err(
                ::tuffite::CommandError::InvalidArgument(
                    "trailing ShellAPI request data".to_owned(),
                ),
            );
        }
        Ok(Self {})
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A) -> Result<Vec<WindowInfo>, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        Self::__tuffite_decode(command)?;
        Self::respond(handler(application))
    }
}
impl ::tuffite::ShellApiRequest for window::Create {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.create";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.options, encoder)?;
        Ok(())
    }
}
impl window::Create {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.create",
        0x2780_4c86_ebf9_7cf3u64,
        0x9910_f247_5b83_26c4u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let options: Option<WindowOptions> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { options })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<WindowOptions>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.options))
    }
}
impl ::tuffite::ShellApiRequest for window::Focus {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.focus";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::Focus {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.focus",
        0x7328_589a_4597_f73eu64,
        0x1896_b9c0_d603_79f2u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<WindowId>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::Minimize {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.minimize";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::Minimize {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.minimize",
        0x7103_a11c_2116_72d9u64,
        0x1896_b9c0_d603_79f2u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<WindowId>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::Maximize {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.maximize";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::Maximize {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.maximize",
        0x9ccd_2de1_2523_e120u64,
        0x1896_b9c0_d603_79f2u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<WindowId>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::Restore {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.restore";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::Restore {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.restore",
        0x83ee_03e8_39ae_389bu64,
        0x1896_b9c0_d603_79f2u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<WindowId>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::SetFullscreen {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.setFullscreen";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.fullscreen, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::SetFullscreen {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.setFullscreen",
        0x8930_6e3b_b335_392au64,
        0xd87b_48da_1a9a_fa04u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let fullscreen: bool = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { fullscreen, id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            bool,
            Option<WindowId>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.fullscreen, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::Close {
    type Response = ();
    const CHANNEL: &'static str = "tuffite.window.close";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::Close {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.close",
        0xbef2_0904_2f19_30efu64,
        0x8b2a_a9b1_8608_5f5bu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(&mut A, Option<WindowId>) -> Result<(), ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::SetTitle {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.setTitle";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.title, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::SetTitle {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.setTitle",
        0x316f_0207_55c4_79cau64,
        0x0457_3d00_d14e_1335u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let title: String = ::tuffite::__private::codec::Decode::decode(&mut decoder)
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { title, id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            String,
            Option<WindowId>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.title, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::GetBounds {
    type Response = WindowBounds;
    const CHANNEL: &'static str = "tuffite.window.getBounds";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::GetBounds {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.getBounds",
        0xe5a8_1d21_f786_57f1u64,
        0x508f_19bc_5731_81eeu64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            Option<WindowId>,
        ) -> Result<WindowBounds, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.id))
    }
}
impl ::tuffite::ShellApiRequest for window::SetBounds {
    type Response = WindowInfo;
    const CHANNEL: &'static str = "tuffite.window.setBounds";
    fn encode(
        &self,
        encoder: &mut ::tuffite::__private::codec::Encoder,
    ) -> Result<(), ::tuffite::__private::codec::CodecError> {
        let _ = &encoder;
        ::tuffite::__private::codec::Encode::encode(&self.bounds, encoder)?;
        ::tuffite::__private::codec::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl window::SetBounds {
    pub const COMMAND: ::tuffite::CommandDescriptor = ::tuffite::CommandDescriptor::new(
        "tuffite.window.setBounds",
        0x5b73_dc63_3e97_3510u64,
        0xfd14_6d8b_bf8b_2e49u64,
    );
    #[doc(hidden)]
    pub fn __tuffite_decode(
        command: ::tuffite::Command<'_>,
    ) -> Result<Self, ::tuffite::CommandError> {
        if command.id() != Self::COMMAND.id() {
            return Err(::tuffite::CommandError::UnknownCommand);
        }
        if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
            return Err(
                ::tuffite::CommandError::SchemaMismatch(
                    "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                        .to_owned(),
                ),
            );
        }
        let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        let bounds: WindowBounds = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        let id: Option<WindowId> = ::tuffite::__private::codec::Decode::decode(
                &mut decoder,
            )
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "invalid ShellAPI request".to_owned(),
            ))?;
        decoder
            .finish()
            .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        Ok(Self { bounds, id })
    }
    #[doc(hidden)]
    pub fn __tuffite_dispatch<A>(
        command: ::tuffite::Command<'_>,
        application: &mut A,
        handler: fn(
            &mut A,
            WindowBounds,
            Option<WindowId>,
        ) -> Result<WindowInfo, ::tuffite::CommandError>,
    ) -> ::tuffite::CommandResult {
        let request = Self::__tuffite_decode(command)?;
        Self::respond(handler(application, request.bounds, request.id))
    }
}
