//! Native application bundles and host-specific installers.
use super::super::filesystem::io_error;
use super::archive::collect_files;
use super::resources::copy_regular_file;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub(super) enum Format {
    #[default]
    Zip,
    Directory,
    App,
    Dmg,
    Exe,
}
impl Format {
    pub(super) fn archive_extension(self) -> Option<&'static str> {
        match self {
            Self::Zip => Some("zip"),
            Self::Dmg => Some("dmg"),
            Self::Exe => Some("exe"),
            _ => None,
        }
    }
    pub(super) fn is_bundle(self) -> bool {
        matches!(self, Self::App | Self::Dmg)
    }
    pub(super) fn validate(self) -> Result<(), String> {
        if self.is_bundle() && !cfg!(target_os = "macos") {
            return Err("app/dmg packaging requires a macOS host".into());
        }
        if self == Self::Exe {
            if !cfg!(target_os = "windows") {
                return Err("exe installers require a Windows host".into());
            }
            run(super::nsis::Nsis::resolve()?.command().arg("/VERSION"))?;
        }
        Ok(())
    }
}
fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn numeric_version(version: &str) -> &str {
    version.split(['-', '+']).next().unwrap_or(version)
}
/// Chromium's flat output uses cwd-relative library names. Bundles must load
/// libraries beside their executable regardless of Finder's working directory.
pub(super) fn relocate_libraries(root: &Path, bundle: bool) -> Result<(), String> {
    let icd_path = root.join("vk_swiftshader_icd.json");
    if bundle && icd_path.is_file() {
        let mut icd: serde_json::Value =
            serde_json::from_slice(&fs::read(&icd_path).map_err(io_error("read SwiftShader ICD"))?)
                .map_err(|e| e.to_string())?;
        icd["ICD"]["library_path"] = "../MacOS/libvk_swiftshader.dylib".into();
        fs::write(
            &icd_path,
            serde_json::to_vec_pretty(&icd).map_err(|e| e.to_string())?,
        )
        .map_err(io_error("write bundle SwiftShader ICD"))?;
    }
    for entry in fs::read_dir(root).map_err(io_error("read bundle code"))? {
        let path = entry.map_err(io_error("read bundle image"))?.path();
        if !path.is_file() || !super::symbols::native_image(&path)? {
            continue;
        }
        let output = Command::new("otool")
            .arg("-L")
            .arg(&path)
            .output()
            .map_err(io_error("inspect bundle libraries"))?;
        if !output.status.success() {
            return Err(format!("otool failed for {}", path.display()));
        }
        let mut command = Command::new("install_name_tool");
        let mut changed = false;
        for line in String::from_utf8_lossy(&output.stdout).lines().skip(1) {
            let Some((dependency, _)) = line.trim().split_once(" (compatibility") else {
                continue;
            };
            if let Some(name) = dependency.strip_prefix("./")
                && root.join(name).is_file()
                && path.file_name().is_none_or(|own| own != name)
            {
                command.args(["-change", dependency, &format!("@loader_path/{name}")]);
                changed = true;
            }
        }
        if path.extension().is_some_and(|e| e == "dylib")
            && String::from_utf8_lossy(&output.stdout)
                .lines()
                .nth(1)
                .is_some_and(|line| line.trim_start().starts_with('/'))
        {
            command.arg("-id").arg(format!(
                "@loader_path/{}",
                path.file_name().unwrap().to_string_lossy()
            ));
            changed = true;
        }
        if changed {
            run(command.arg(&path))?;
        }
    }
    Ok(())
}
/// Metadata shared by native bundle layout and installer generation.
pub(super) struct ApplicationBundle<'a> {
    pub(super) identifier: &'a str,
    pub(super) product_name: &'a str,
    pub(super) executable: &'a str,
    pub(super) version: &'a str,
}
impl ApplicationBundle<'_> {
    pub(super) fn assemble(&self, root: &Path) -> Result<(), String> {
        let Self {
            identifier,
            product_name,
            executable,
            version,
        } = *self;
        let payload = root.with_extension("payload");
        if payload.exists() {
            return Err(format!(
                "staging bundle payload already exists: {}",
                payload.display()
            ));
        }
        fs::rename(root, &payload).map_err(io_error("stage bundle payload"))?;
        let result = (|| {
            fs::create_dir_all(root.join("Contents/MacOS")).map_err(io_error("create bundle"))?;
            fs::rename(&payload, root.join("Contents/Resources"))
                .map_err(io_error("move bundle runtime"))?;
            let resources = root.join("Contents/Resources");
            for entry in fs::read_dir(&resources).map_err(io_error("read bundle payload"))? {
                let path = entry.map_err(io_error("read bundle entry"))?.path();
                if path.is_file() && super::symbols::native_image(&path)? {
                    fs::rename(
                        &path,
                        root.join("Contents/MacOS").join(path.file_name().unwrap()),
                    )
                    .map_err(io_error("move bundle code"))?;
                }
            }
            let manifest_path = resources.join("manifest.json");
            let mut manifest: serde_json::Value = serde_json::from_slice(
                &fs::read(&manifest_path).map_err(io_error("read bundle manifest"))?,
            )
            .map_err(|e| e.to_string())?;
            for file in manifest["files"]
                .as_array_mut()
                .ok_or("missing bundle file list")?
            {
                let path = file["path"].as_str().ok_or("missing bundle file path")?;
                let directory = if root.join("Contents/MacOS").join(path).is_file() {
                    "MacOS"
                } else {
                    "Resources"
                };
                file["path"] = format!("{directory}/{path}").into();
            }
            fs::write(
                &manifest_path,
                serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
            )
            .map_err(io_error("write bundle manifest"))?;
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>{}</string>
<key>CFBundleExecutable</key><string>{}</string>
<key>CFBundleName</key><string>{}</string>
<key>CFBundleDisplayName</key><string>{}</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>{}</string>
<key>CFBundleShortVersionString</key><string>{}</string>
<key>CFBundleIconFile</key><string>app.icns</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
<key>NSBluetoothAlwaysUsageDescription</key><string>Connect to a nearby phone or security key to sign in with a passkey.</string>
</dict></plist>
"#,
                xml(identifier),
                xml(executable),
                xml(product_name),
                xml(product_name),
                xml(numeric_version(version)),
                xml(numeric_version(version))
            );
            fs::write(root.join("Contents/Info.plist"), plist)
                .map_err(io_error("write bundle Info.plist"))?;
            run(Command::new("codesign")
                .args(["--force", "--sign", "-"])
                .arg(root))
        })();
        if result.is_err() && payload.exists() {
            let _ = fs::remove_dir_all(root);
            let _ = fs::rename(payload, root);
        }
        result
    }
}

pub(super) fn run(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|e| format!("run {command:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command:?} failed: {status}"))
    }
}
fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(io_error("create installer payload"))?;
    for path in collect_files(source)? {
        copy_regular_file(&source.join(&path), &destination.join(path))?;
    }
    Ok(())
}
impl ApplicationBundle<'_> {
    pub(super) fn installer(
        &self,
        format: Format,
        root: &Path,
        output: &Path,
        name: &str,
    ) -> Result<(), String> {
        let Self {
            identifier,
            product_name,
            executable,
            version,
        } = *self;
        if format == Format::Exe {
            return nsis(root, output, identifier, product_name, executable, version);
        }
        let media = root.with_extension("media");
        if media.exists() {
            return Err("installer staging directory already exists".into());
        }
        let result = (|| {
            copy_tree(root, &media.join(name))?;
            match format {
                Format::Dmg => {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink("/Applications", media.join("Applications"))
                        .map_err(io_error("link Applications"))?;
                    run(Command::new("hdiutil")
                        .args([
                            "create",
                            "-fs",
                            "HFS+",
                            "-format",
                            "UDZO",
                            "-volname",
                            product_name,
                            "-srcfolder",
                        ])
                        .arg(&media)
                        .arg(output))
                }
                _ => Err("unsupported installer format".into()),
            }
        })();
        let _ = fs::remove_dir_all(media);
        result
    }
}

fn nsis_quote(value: &str) -> String {
    value
        .replace('$', "$$")
        .replace('"', "$\\\"")
        .replace('\r', "$\\r")
        .replace('\n', "$\\n")
}
fn nsis(
    root: &Path,
    output: &Path,
    identifier: &str,
    product_name: &str,
    executable: &str,
    version: &str,
) -> Result<(), String> {
    let script = output.with_extension("nsi");
    let product_name = nsis_quote(product_name);
    let quote = |p: &Path| nsis_quote(&p.to_string_lossy());
    let key = format!(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
        nsis_quote(identifier)
    );
    let mut contents = format!(
        r#"Unicode true
Name "{product_name}"
OutFile "{}"
InstallDir "$LOCALAPPDATA\Programs\{identifier}"
RequestExecutionLevel user
SetCompressor /SOLID lzma
Icon "{}"
UninstallIcon "{}"
Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles
Section "Install"
SetOutPath "$INSTDIR"
File /r "{}\*"
ExecWait '"$SYSDIR\icacls.exe" "$INSTDIR" /grant "*S-1-15-2-1:(OI)(CI)(RX)" "*S-1-15-2-2:(OI)(CI)(RX)" /T /C /Q' $0
${{IfThen}} $0 != 0 ${{|}} Abort "Unable to grant AppContainer access" ${{|}}
WriteUninstaller "$INSTDIR\uninstall.exe"
CreateShortcut "$SMPROGRAMS\{product_name}.lnk" "$INSTDIR\{executable}" "" "$INSTDIR\app.ico"
WriteRegStr HKCU "{key}" "DisplayName" "{product_name}"
WriteRegStr HKCU "{key}" "DisplayVersion" "{}"
WriteRegStr HKCU "{key}" "UninstallString" '$\"$INSTDIR\uninstall.exe$\"'
WriteRegStr HKCU "{key}" "InstallLocation" "$INSTDIR"
SectionEnd
Section "Uninstall"
Delete "$SMPROGRAMS\{product_name}.lnk"
DeleteRegKey HKCU "{key}"
"#,
        quote(output),
        quote(&root.join("app.ico")),
        quote(&root.join("app.ico")),
        quote(root),
        nsis_quote(version)
    );
    contents.insert_str(0, "!include LogicLib.nsh\n");
    let paths = collect_files(root)?;
    let mut directories = std::collections::BTreeSet::<PathBuf>::new();
    for path in paths {
        writeln!(
            contents,
            "Delete \"$INSTDIR\\{}\"",
            nsis_quote(&path.to_string_lossy().replace('/', "\\"))
        )
        .map_err(|e| e.to_string())?;
        let mut parent = path.parent();
        while let Some(p) = parent.filter(|p| !p.as_os_str().is_empty()) {
            directories.insert(p.to_path_buf());
            parent = p.parent();
        }
    }
    for directory in directories.into_iter().rev() {
        writeln!(
            contents,
            "RMDir \"$INSTDIR\\{}\"",
            nsis_quote(&directory.to_string_lossy().replace('/', "\\"))
        )
        .map_err(|e| e.to_string())?;
    }
    contents.push_str("Delete \"$INSTDIR\\uninstall.exe\"\nRMDir \"$INSTDIR\"\nSectionEnd\n");
    fs::write(&script, contents).map_err(io_error("write NSIS script"))?;
    let result = super::nsis::Nsis::resolve().and_then(|tool| run(tool.command().arg(&script)));
    let _ = fs::remove_file(script);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installer_values_cannot_expand_nsis_variables_or_statements() {
        assert_eq!(
            nsis_quote("C:\\$USER\\\"icon\"\n"),
            "C:\\$$USER\\$\\\"icon$\\\"$\\n"
        );
        assert_eq!(xml("A&B<logo>"), "A&amp;B&lt;logo&gt;");
    }
    #[test]
    fn apple_bundle_versions_omit_cargo_prerelease_suffixes() {
        assert_eq!(numeric_version("1.2.3-beta.1+build"), "1.2.3");
        assert!(Format::App.archive_extension().is_none());
        assert_eq!(Format::Dmg.archive_extension(), Some("dmg"));
        assert_eq!(Format::Exe.archive_extension(), Some("exe"));
    }
    #[test]
    fn wrong_host_installers_fail_before_building_an_application() {
        if cfg!(target_os = "macos") {
            assert!(Format::Exe.validate().is_err());
        } else {
            assert!(Format::App.validate().is_err());
        }
    }
}
