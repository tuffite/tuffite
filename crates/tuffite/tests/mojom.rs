//! This test deliberately has no Framework resolver or native build dependency.
#[allow(dead_code, clippy::all, clippy::pedantic)]
mod generated {
    include!("fixtures/mojom_generated.rs");
}

use tuffite::__private::mojom::Struct;

#[test]
fn cargo_only_generated_types_match_cpp_wire_fixture() {
    let hex = include_str!("../../../components/dispatch/test/mojom_sample.inc")
        .trim()
        .trim_matches('"');
    let bytes = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let value = generated::MojomSample::decode_mojom(&bytes).unwrap();
    assert_eq!(value.large, (1_u64 << 63) + 17);
    assert_eq!(value.signed_value, -12345);
    assert_eq!(value.nested.label, "你好");
    assert_eq!(value.encode_mojom().unwrap(), bytes);
    for length in 0..bytes.len() {
        assert!(generated::MojomSample::decode_mojom(&bytes[..length]).is_err());
    }
    for offset in [4, 32, 48, 64, 68, 72] {
        let mut corrupt = bytes.clone();
        corrupt[offset..offset + 4].fill(255);
        assert!(generated::MojomSample::decode_mojom(&corrupt).is_err());
    }
}

#[allow(dead_code, clippy::all, clippy::pedantic)]
mod collision {
    include!("fixtures/mojom_collision.rs");
}

#[test]
fn method_alias_and_dto_with_the_same_name_compile_and_round_trip() {
    let value = collision::Status { ready: true };
    let bytes = collision::commands::Status::respond(Ok(value.clone())).unwrap();
    let response = collision::CommandsStatusResponse::decode_mojom(&bytes).unwrap();
    assert_eq!(response.result, value);
}
