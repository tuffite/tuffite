use tuffite::__private::mojom::Struct;
use tuffite::shell_api::*;

fn fixture(hex: &str) -> Vec<u8> {
    hex.trim()
        .trim_matches('"')
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn framework_payloads_match_official_cpp_and_js_fixtures() {
    let fs = FsReadFileRequest {
        base: BaseDirectory::Data,
        path: "配置.bin".into(),
    };
    let window = WindowCreateRequest {
        options: Some(WindowOptions {
            label: Some("child".into()),
            bounds: Some(WindowBounds {
                x: 10.,
                y: 20.,
                width: 640.,
                height: 480.,
            }),
            focused: Some(false),
            ..Default::default()
        }),
    };
    let remoting = RemotingStartRequest {
        options: RemotingStartOptions {
            auth_key: "secret".into(),
            ice_servers: vec![RemotingIceServer {
                urls: vec!["stun:example.test".into(), "turn:example.test".into()],
                username: Some("user".into()),
                credential: None,
            }],
            max_bitrate_kbps: None,
            target_framerate: Some(60.),
            screen_id: Some(0),
        },
    };
    macro_rules! check {
        ($value:expr,$ty:ty,$path:literal) => {{
            let bytes = fixture(include_str!($path));
            assert_eq!($value.encode_mojom().unwrap(), bytes);
            assert_eq!(<$ty>::decode_mojom(&bytes).unwrap(), $value);
            for length in 0..bytes.len() {
                assert!(<$ty>::decode_mojom(&bytes[..length]).is_err());
            }
        }};
    }
    check!(
        fs,
        FsReadFileRequest,
        "../../../components/dispatch/test/framework_fs.inc"
    );
    check!(
        window,
        WindowCreateRequest,
        "../../../components/dispatch/test/framework_window.inc"
    );
    check!(
        remoting,
        RemotingStartRequest,
        "../../../components/dispatch/test/framework_remoting.inc"
    );
}
