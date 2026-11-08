//! C ABI declarations generated from Framework's public headers.
//!
//! Applications should use the safe `tuffite` crate instead.
#[allow(
    clippy::all,
    missing_docs,
    non_snake_case,
    unpredictable_function_pointer_comparisons
)]
mod generated {
    #[cfg(feature = "bindgen")]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    #[cfg(not(feature = "bindgen"))]
    include!("bindings.rs");
}
pub use generated::*;

/// ABI crate release. Major changes break the binary contract; minor changes
/// append compatible capabilities; patch releases preserve its layout.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Binary compatibility major, derived from this crate's version.
pub const ABI_MAJOR: u16 = version_component(env!("CARGO_PKG_VERSION_MAJOR"));
/// Required binary capability minor, derived from this crate's version.
pub const ABI_MINOR: u16 = version_component(env!("CARGO_PKG_VERSION_MINOR"));
/// Compatible fixes, derived from this crate's version.
pub const ABI_PATCH: u16 = version_component(env!("CARGO_PKG_VERSION_PATCH"));

#[allow(clippy::cast_possible_truncation)] // The bound is checked before conversion.
const fn version_component(value: &str) -> u16 {
    let bytes = value.as_bytes();
    let mut result = 0_u32;
    let mut index = 0;
    while index < bytes.len() {
        result = result * 10 + (bytes[index] - b'0') as u32;
        assert!(
            result <= u16::MAX as u32,
            "ABI version component exceeds u16"
        );
        index += 1;
    }
    result as u16
}

/// A provider must match the required major and satisfy the full minimum
/// version. This binary minimum rule also applies before 1.0; Cargo requirements
/// follow Cargo's separate pre-1.0 compatibility rules.
#[must_use]
pub const fn is_compatible(provided: (u16, u16, u16), required: (u16, u16, u16)) -> bool {
    provided.0 == required.0
        && (provided.1 > required.1 || (provided.1 == required.1 && provided.2 >= required.2))
}

impl Version {
    /// Version of this ABI crate.
    pub const CURRENT: Self = Self {
        major: ABI_MAJOR,
        minor: ABI_MINOR,
        patch: ABI_PATCH,
    };

    /// Minimum original contract: 0.1.0 before 1.0, then major.0.0.
    /// Optional capabilities declare their own minimum when they evolve.
    pub const BASELINE: Self = Self {
        major: ABI_MAJOR,
        minor: if ABI_MAJOR == 0 { 1 } else { 0 },
        patch: 0,
    };

    /// Whether this provider satisfies a stable semver minimum requirement.
    #[must_use]
    pub const fn is_compatible_with(self, required: Self) -> bool {
        is_compatible(
            (self.major, self.minor, self.patch),
            (required.major, required.minor, required.patch),
        )
    }
}

impl Header {
    /// Describes an outer ABI table using this crate's version.
    #[must_use]
    pub const fn for_type<T>() -> Self {
        Self {
            struct_size: std::mem::size_of::<T>(),
            version: Version::CURRENT,
            reserved: 0,
        }
    }
}

impl ByteSlice {
    /// Creates an ABI view over a Rust slice.
    #[must_use]
    pub const fn from_slice(bytes: &[u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}

impl Default for UtilitySessionClient {
    fn default() -> Self {
        Self {
            header: Header::for_type::<Self>(),
            user_data: std::ptr::null_mut(),
            create: None,
            send: None,
            acknowledge: None,
            cancel: None,
            close: None,
        }
    }
}

#[allow(non_upper_case_globals, clippy::cast_possible_wrap)]
impl Status {
    pub const Ok: Self = Self(TUFFITE_STATUS_OK as i32);
    pub const InvalidArgument: Self = Self(TUFFITE_STATUS_INVALID_ARGUMENT as i32);
    pub const IncompatibleAbi: Self = Self(TUFFITE_STATUS_INCOMPATIBLE_ABI as i32);
    pub const InvalidHandle: Self = Self(TUFFITE_STATUS_INVALID_HANDLE as i32);
    pub const InvalidState: Self = Self(TUFFITE_STATUS_INVALID_STATE as i32);
    pub const ResourceExhausted: Self = Self(TUFFITE_STATUS_RESOURCE_EXHAUSTED as i32);
    pub const Internal: Self = Self(TUFFITE_STATUS_INTERNAL as i32);
    pub const NotFound: Self = Self(TUFFITE_STATUS_NOT_FOUND as i32);
}

#[allow(non_upper_case_globals, clippy::cast_possible_wrap)]
impl RuntimeEventKind {
    pub const Started: Self = Self(TUFFITE_RUNTIME_EVENT_STARTED);
    pub const DispatchAccepted: Self = Self(TUFFITE_RUNTIME_EVENT_DISPATCH_ACCEPTED);
    pub const Stopping: Self = Self(TUFFITE_RUNTIME_EVENT_STOPPING);
    pub const CommandResponse: Self = Self(TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE);
    pub const ApplicationEvent: Self = Self(TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT);
}

#[allow(non_upper_case_globals, clippy::cast_possible_wrap)]
impl TaskPriority {
    pub const BestEffort: Self = Self(TUFFITE_TASK_PRIORITY_BEST_EFFORT);
    pub const UserVisible: Self = Self(TUFFITE_TASK_PRIORITY_USER_VISIBLE);
    pub const UserBlocking: Self = Self(TUFFITE_TASK_PRIORITY_USER_BLOCKING);
}

/// Capability selectors are converted to raw integers before crossing FFI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Capability {
    /// Browser-side native client for supervised utility sessions.
    UtilitySession = TUFFITE_CAPABILITY_UTILITY_SESSION,
    /// Framework supports interface-to-utility routing and blocks raw access to typed owners.
    UtilityInterfaces = TUFFITE_CAPABILITY_UTILITY_INTERFACES,
    /// Optional DOM isolated-world registry capability.
    Isolated = TUFFITE_CAPABILITY_ISOLATED,
    /// Trusted profile lifetime and window/content composition operations.
    Contexts = TUFFITE_CAPABILITY_CONTEXTS,
    /// Optional application-owned native desktop remoting operations.
    Remoting = TUFFITE_CAPABILITY_REMOTING,
}
pub mod abi {
    use super::{ABI_MAJOR, ABI_MINOR, ABI_PATCH, RuntimeEventKind, TaskPriority};

    /// Canonical event-kind enumerants exposed to C headers.
    pub const RUNTIME_EVENT_KINDS: &[(&str, u32)] = &[
        ("STARTED", RuntimeEventKind::Started.0),
        ("DISPATCH_ACCEPTED", RuntimeEventKind::DispatchAccepted.0),
        ("STOPPING", RuntimeEventKind::Stopping.0),
        ("COMMAND_RESPONSE", RuntimeEventKind::CommandResponse.0),
        ("APPLICATION_EVENT", RuntimeEventKind::ApplicationEvent.0),
    ];

    /// Canonical task priorities exposed to C headers.
    pub const TASK_PRIORITIES: &[(&str, u32)] = &[
        ("BEST_EFFORT", TaskPriority::BestEffort.0),
        ("USER_VISIBLE", TaskPriority::UserVisible.0),
        ("USER_BLOCKING", TaskPriority::UserBlocking.0),
    ];

    /// Canonical ABI version reported by the C header.
    pub const VERSION: (u16, u16, u16) = (ABI_MAJOR, ABI_MINOR, ABI_PATCH);
}

#[cfg(all(test, feature = "bindgen"))]
#[test]
fn checked_in_bindings_match_headers() {
    assert!(
        include_str!("bindings.rs") == include_str!(concat!(env!("OUT_DIR"), "/bindings.rs")),
        "ABI bindings are stale; copy {} to crates/tuffite-abi/src/bindings.rs",
        concat!(env!("OUT_DIR"), "/bindings.rs"),
    );
}

#[cfg(test)]
mod header_parity {
    //! Cross-checks the Rust-side ABI constants against the canonical C header so
    //! divergence is caught at `cargo test` time, not at ninja/link time.

    use super::abi;

    const HEADER: &str = concat!(
        include_str!("../../../framework/public/c_api/host.h"),
        include_str!("../../../framework/public/c_api/types.h"),
    );

    fn find_define(source: &str, name: &str) -> Option<String> {
        // Join `\`-continued preprocessor lines before matching so multi-line
        // enumerants defined for column-limited headers still parse.
        let joined = source.replace("\\\r\n", " ").replace("\\\n", " ");
        for line in joined.lines() {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix("#define ") else {
                continue;
            };
            let Some((token, rest)) = rest.split_once(char::is_whitespace) else {
                continue;
            };
            if token == name {
                return Some(rest.trim().to_owned());
            }
        }
        None
    }

    fn expect_number(source: &str, header: &str, name: &str) -> u64 {
        let raw =
            find_define(source, name).unwrap_or_else(|| panic!("{header}: missing #define {name}"));
        // Common shapes: `1`, `((tuffite_status_t)0)`, `((tuffite_runtime_event_kind_t)5)`.
        // Select the value, not digits embedded in a cast such as uint32_t.
        raw.split(|character: char| !character.is_ascii_digit())
            .rfind(|part| !part.is_empty())
            .unwrap_or_else(|| panic!("{header}: missing number in #define {name}: {raw}"))
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("{header}: cannot parse #define {name}: {raw}"))
    }

    #[test]
    fn unknown_status_values_round_trip_through_a_callback() {
        unsafe extern "C" fn echo(
            _: *mut std::ffi::c_void,
            status: super::Status,
            _: super::ByteSlice,
        ) -> super::Status {
            status
        }
        let callback: super::UtilityReplyFn = echo;
        for value in [i32::MIN, -1, 8, i32::MAX] {
            // SAFETY: The local callback does not dereference either pointer.
            let result = unsafe {
                callback(
                    std::ptr::null_mut(),
                    super::Status(value),
                    super::ByteSlice::from_slice(&[]),
                )
            };
            assert_eq!(result.0, value);
        }
    }

    #[test]
    fn abi_version_matches_header() {
        let header = include_str!("../../../framework/public/c_api/version.h");
        assert!(HEADER.contains("tuffite/framework/public/c_api/version.h"));
        let (major, minor, patch) = abi::VERSION;
        assert_eq!(
            expect_number(header, "generated", "TUFFITE_FFI_ABI_MAJOR"),
            u64::from(major),
            "canonical header ABI major disagrees with Rust",
        );
        assert_eq!(
            expect_number(header, "generated", "TUFFITE_FFI_ABI_MINOR"),
            u64::from(minor),
            "canonical header ABI minor disagrees with Rust",
        );
        assert_eq!(
            expect_number(header, "generated", "TUFFITE_FFI_ABI_PATCH"),
            u64::from(patch),
            "canonical header ABI patch disagrees with Rust",
        );
    }

    #[test]
    fn compatibility_requires_matching_major_and_sufficient_minor() {
        assert!(super::is_compatible((2, 4, 0), (2, 4, 0)));
        assert!(super::is_compatible((2, 5, 0), (2, 4, 0)));
        assert!(super::is_compatible((2, 4, 10), (2, 4, 9)));
        assert!(!super::is_compatible((2, 4, 1), (2, 4, 9)));
        assert!(super::is_compatible((2, 5, 0), (2, 4, 9)));
        assert!(!super::is_compatible((2, 3, 9), (2, 4, 0)));
        assert!(!super::is_compatible((1, 9, 9), (2, 0, 0)));
        assert!(!super::is_compatible((3, 0, 0), (2, 0, 0)));
    }

    #[test]
    fn current_release_satisfies_reset_baseline() {
        assert!(super::Version::CURRENT.is_compatible_with(super::Version::BASELINE));
        assert!(
            !super::Version {
                major: 2,
                minor: 5,
                patch: 0
            }
            .is_compatible_with(super::Version {
                major: 0,
                minor: 1,
                patch: 0
            })
        );
        assert!(!super::is_compatible((0, 0, 9), (0, 1, 0)));
        assert!(super::is_compatible((0, 1, 1), (0, 1, 0)));
    }

    #[test]
    fn tables_share_a_fixed_size_semver_prefix() {
        use super::{Application, Header, Host, Version};
        use std::mem::{offset_of, size_of};

        let size = size_of::<usize>();
        assert_eq!(offset_of!(Host, header), 0);
        assert_eq!(offset_of!(Application, header), 0);
        assert_eq!(offset_of!(Header, struct_size), 0);
        assert_eq!(offset_of!(Header, version), size);
        assert_eq!(offset_of!(Header, reserved), size + 6);
        assert_eq!(offset_of!(Version, major), 0);
        assert_eq!(offset_of!(Version, minor), 2);
        assert_eq!(offset_of!(Version, patch), 4);
        assert_eq!(size_of::<Version>(), 6);
        assert_eq!(size_of::<Header>(), size + 8);
        assert_eq!(offset_of!(Host, user_data), size + 8);
        assert_eq!(offset_of!(Application, start), size + 8);
    }

    #[test]
    fn callback_tables_have_fixed_prefixes() {
        use super::{
            Application, Header, Host, UtilityService, UtilitySessionClient, UtilitySessionService,
        };
        use std::mem::{offset_of, size_of};
        let pointer = size_of::<usize>();
        let header = size_of::<Header>();
        assert_eq!(offset_of!(Host, query_extension), header + 3 * pointer);
        assert_eq!(size_of::<Host>(), header + 4 * pointer);
        assert_eq!(offset_of!(UtilitySessionClient, user_data), header);
        assert_eq!(offset_of!(UtilitySessionClient, create), header + pointer);
        assert_eq!(
            offset_of!(UtilitySessionClient, close),
            header + 5 * pointer
        );
        assert_eq!(size_of::<UtilitySessionClient>(), header + 6 * pointer);
        assert_eq!(offset_of!(super::Remoting, header), 0);
        assert_eq!(offset_of!(super::Remoting, capabilities), header + pointer);
        assert_eq!(offset_of!(super::Remoting, capture), header + 9 * pointer);
        assert_eq!(size_of::<super::Remoting>(), header + 10 * pointer);
        assert_eq!(offset_of!(Application, utility), header + 3 * pointer);
        assert_eq!(offset_of!(UtilityService, session), 5 * pointer);
        assert_eq!(
            offset_of!(Application, prepare_browser),
            header + 9 * pointer
        );
        assert_eq!(
            offset_of!(Application, prepare_process),
            header + 10 * pointer
        );
        assert_eq!(
            offset_of!(Application, utility_runtime),
            header + 11 * pointer
        );
        assert_eq!(
            offset_of!(Application, utility_start_with_context),
            header + 13 * pointer
        );
        assert_eq!(
            offset_of!(Application, resolve_utility_route),
            header + 14 * pointer
        );
        assert_eq!(
            offset_of!(Application, setup_browser),
            header + 15 * pointer
        );
        assert_eq!(
            offset_of!(Application, initialize_browser),
            header + 16 * pointer
        );
        assert_eq!(
            offset_of!(Application, start_profile),
            header + 18 * pointer
        );
        assert_eq!(
            offset_of!(Application, command_scope),
            header + 19 * pointer
        );
        assert_eq!(
            offset_of!(Application, configuration),
            header + 20 * pointer
        );
        assert_eq!(
            offset_of!(Application, resolve_mojom_method),
            header + 21 * pointer
        );
        assert_eq!(
            offset_of!(Application, dispatch_mojom),
            header + 22 * pointer
        );
        assert_eq!(
            offset_of!(Application, invoke_utility_mojom),
            header + 23 * pointer
        );
        assert_eq!(size_of::<Application>(), header + 24 * pointer);
        assert_eq!(offset_of!(super::MojomMethod, command_id), 2 * pointer);
        assert_eq!(
            offset_of!(super::MojomMethod, schema_fingerprint),
            2 * pointer + 8
        );
        assert_eq!(offset_of!(UtilitySessionService, header), 0);
        assert_eq!(offset_of!(UtilitySessionService, create), header);
        assert_eq!(
            offset_of!(UtilitySessionService, close),
            header + 4 * pointer
        );
        assert_eq!(size_of::<UtilitySessionService>(), header + 5 * pointer);
    }

    #[test]
    fn capability_ids_match_header() {
        for (name, value) in [
            ("UTILITY_SESSION", super::Capability::UtilitySession),
            ("UTILITY_INTERFACES", super::Capability::UtilityInterfaces),
            ("CONTEXTS", super::Capability::Contexts),
            ("ISOLATED", super::Capability::Isolated),
            ("REMOTING", super::Capability::Remoting),
        ] {
            assert_eq!(
                expect_number(HEADER, "canonical", &format!("TUFFITE_CAPABILITY_{name}")),
                u64::from(value as u32)
            );
        }
    }

    #[test]
    fn runtime_event_kinds_match_header() {
        for (name, value) in abi::RUNTIME_EVENT_KINDS {
            let symbol = format!("TUFFITE_RUNTIME_EVENT_{name}");
            assert_eq!(
                expect_number(HEADER, "canonical", &symbol),
                u64::from(*value),
                "canonical header {symbol} disagrees with Rust value {value}",
            );
        }
    }

    #[test]
    fn task_priorities_match_header() {
        for (name, value) in abi::TASK_PRIORITIES {
            let symbol = format!("TUFFITE_TASK_PRIORITY_{name}");
            assert_eq!(
                expect_number(HEADER, "canonical", &symbol),
                u64::from(*value),
                "canonical header {symbol} disagrees with Rust value {value}",
            );
        }
    }
}
