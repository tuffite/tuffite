//! Application icons, including the Windows executable resource.
use super::super::filesystem::{canonical_file, io_error, safe_relative_path};
use super::resources::copy_regular_file;
use serde::Deserialize;
use std::fs;
use std::path::Path;
#[derive(Debug, Default, Deserialize)]
pub(super) struct Icons {
    macos: Option<String>,
    windows: Option<String>,
}
impl Icons {
    pub(super) fn install(
        &self,
        root: &Path,
        project: &Path,
        platform: &str,
        _executable: &str,
    ) -> Result<(), String> {
        let (name, configured, default): (&str, Option<&str>, &[u8]) = match platform {
            "macos" => (
                "app.icns",
                self.macos.as_deref(),
                include_bytes!("../../../assets/tuffite.icns"),
            ),
            "windows" => (
                "app.ico",
                self.windows.as_deref(),
                include_bytes!("../../../assets/tuffite.ico"),
            ),
            _ => return Ok(()),
        };
        if let Some(value) = configured {
            let relative = safe_relative_path(value, "bundle.icons path")?;
            if relative.extension() != Path::new(name).extension() {
                return Err(format!(
                    "bundle.icons.{platform} requires a .{} file",
                    Path::new(name).extension().unwrap().to_string_lossy()
                ));
            }
            let source = canonical_file(&project.join(relative))?;
            copy_regular_file(&source, &root.join(name))?;
        } else {
            fs::write(root.join(name), default)
                .map_err(io_error("write default application icon"))?;
        }
        #[cfg(windows)]
        embed_windows(
            &root.join(_executable),
            &fs::read(root.join(name)).map_err(io_error("read icon"))?,
        )?;
        Ok(())
    }
}

#[cfg(any(windows, test))]
fn ico_resources(bytes: &[u8]) -> Result<(Vec<u8>, Vec<&[u8]>), String> {
    if bytes.len() < 6 || bytes[..4] != [0, 0, 1, 0] {
        return Err("invalid ICO header".into());
    }
    let count = u16::from_le_bytes([bytes[4], bytes[5]]);
    if count == 0 || count > 256 {
        return Err("invalid ICO image count".into());
    }
    let mut group = bytes[..6].to_vec();
    let mut images = Vec::new();
    for index in 0..usize::from(count) {
        let entry = bytes
            .get(6 + index * 16..22 + index * 16)
            .ok_or("truncated ICO directory")?;
        let length = u32::from_le_bytes(entry[8..12].try_into().unwrap()) as usize;
        let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
        let end = offset.checked_add(length).ok_or("ICO range overflow")?;
        let image = bytes
            .get(offset..end)
            .filter(|b| !b.is_empty())
            .ok_or("invalid ICO image range")?;
        group.extend_from_slice(&entry[..12]);
        group.extend_from_slice(
            &(1000 + u16::try_from(index).map_err(|_| "ICO resource identifier overflow")?)
                .to_le_bytes(),
        );
        images.push(image);
    }
    Ok((group, images))
}
#[cfg(windows)]
#[expect(unsafe_code, reason = "Win32 resource updates require native FFI")]
fn embed_windows(executable: &Path, icon: &[u8]) -> Result<(), String> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn BeginUpdateResourceW(filename: *const u16, delete: i32) -> *mut c_void;
        fn UpdateResourceW(
            handle: *mut c_void,
            kind: *const u16,
            name: *const u16,
            language: u16,
            data: *const c_void,
            length: u32,
        ) -> i32;
        fn EndUpdateResourceW(handle: *mut c_void, discard: i32) -> i32;
    }
    let (group, images) = ico_resources(icon)?;
    let filename: Vec<u16> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    // SAFETY: filename is NUL-terminated and remains live during the call.
    let handle = unsafe { BeginUpdateResourceW(filename.as_ptr(), 0) };
    if handle.is_null() {
        return Err(format!(
            "open executable icon resources: {}",
            std::io::Error::last_os_error()
        ));
    }
    let result = (|| {
        for (index, data) in images.iter().enumerate() {
            // SAFETY: the resource handle is valid; buffers remain live and the
            // integer resource identifiers use MAKEINTRESOURCEW's representation.
            if unsafe {
                UpdateResourceW(
                    handle,
                    3usize as *const u16,
                    (1000 + index) as *const u16,
                    0,
                    data.as_ptr().cast(),
                    u32::try_from(data.len()).map_err(|_| "icon resource too large")?,
                )
            } == 0
            {
                return Err(format!(
                    "write executable icon: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }
        // SAFETY: same handle/buffer/identifier contract as above.
        if unsafe {
            UpdateResourceW(
                handle,
                14usize as *const u16,
                1usize as *const u16,
                0,
                group.as_ptr().cast(),
                u32::try_from(group.len()).map_err(|_| "icon group too large")?,
            )
        } == 0
        {
            return Err(format!(
                "write executable icon group: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    })();
    // SAFETY: close the handle exactly once, discarding changes on failure.
    let ended = unsafe { EndUpdateResourceW(handle, i32::from(result.is_err())) };
    if ended == 0 && result.is_ok() {
        return Err(format!(
            "save executable icons: {}",
            std::io::Error::last_os_error()
        ));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_icons_have_complete_native_containers() {
        let icns = include_bytes!("../../../assets/tuffite.icns");
        assert_eq!(&icns[..4], b"icns");
        assert_eq!(
            u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize,
            icns.len()
        );
        let (group, images) = ico_resources(include_bytes!("../../../assets/tuffite.ico")).unwrap();
        assert_eq!(images.len(), 5);
        assert_eq!(group.len(), 6 + 14 * 5);
    }
    #[test]
    fn malformed_icon_ranges_are_rejected() {
        let mut bytes = include_bytes!("../../../assets/tuffite.ico").to_vec();
        bytes[18..22].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(ico_resources(&bytes).is_err());
        assert!(ico_resources(b"not an icon").is_err());
    }
}
