//! Wire compatibility checks against the official C++ Mojom fixture.
use crate::shell_api::MojomSample;
use tuffite::__private::mojom::Struct;

#[test]
fn matches_official_cpp_fixture_and_rejects_corrupt_data() {
    let hex = include_str!("../../components/dispatch/test/mojom_sample.inc")
        .trim()
        .trim_matches('"');
    let bytes = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let sample = MojomSample::decode_mojom(&bytes).unwrap();
    assert_eq!(sample.large, (1_u64 << 63) + 17);
    assert_eq!(sample.signed_value, -12345);
    assert_eq!(sample.nested.label, "你好");
    assert_eq!(sample.encode_mojom().unwrap(), bytes);
    for length in 0..bytes.len() {
        assert!(MojomSample::decode_mojom(&bytes[..length]).is_err());
    }
    let mut corrupt = bytes.clone();
    corrupt[4] = 1;
    assert!(MojomSample::decode_mojom(&corrupt).is_err());
    let mut corrupt = bytes.clone();
    corrupt[32..40].fill(255);
    assert!(MojomSample::decode_mojom(&corrupt).is_err());
    let mut corrupt = bytes.clone();
    corrupt.extend_from_slice(&[0; 8]);
    assert!(MojomSample::decode_mojom(&corrupt).is_err());
    let mut corrupt = bytes.clone();
    corrupt[48..56].copy_from_slice(&8_u64.to_le_bytes());
    assert!(MojomSample::decode_mojom(&corrupt).is_err());
}
