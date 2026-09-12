use super::{
    AppManifest, PackageApp, PackageManifest, application_build_command, configure_assets,
    copy_resources, default_package_name, executable_name, package_target, publish_development,
    publish_release, remove_dir_all_with_retry, safe_relative_path, validate_executable_name,
    validate_internal_asset_root, validate_package_name,
};
use std::fs;
use std::path::Path;

#[test]
fn accepts_nested_relative_resource_paths() {
    assert_eq!(
        safe_relative_path("generated/shell_api.js", "resource").unwrap(),
        Path::new("generated/shell_api.js")
    );
}

#[test]
fn rejects_resource_path_traversal_and_absolute_paths() {
    assert!(safe_relative_path("../secret", "resource").is_err());
    assert!(safe_relative_path("assets/../../secret", "resource").is_err());
    assert!(safe_relative_path("/absolute", "resource").is_err());
    assert!(safe_relative_path("", "resource").is_err());
}

#[test]
fn validates_package_names() {
    assert!(validate_package_name("demo-tuffite-windows-x86_64").is_ok());
    assert!(validate_package_name("../escape").is_err());
    assert!(validate_package_name("nested/package").is_err());
    assert!(validate_package_name("").is_err());
}

#[test]
fn force_removal_replaces_an_existing_directory() {
    let root = std::env::temp_dir().join(format!(
        "tuffite-package-force-removal-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let package = root.join("application");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("stale.txt"), b"stale").unwrap();

    remove_dir_all_with_retry(&package).unwrap();

    assert!(!package.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn development_publish_replaces_stale_package_contents() {
    let root = std::env::temp_dir().join(format!(
        "tuffite-development-publish-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let package = root.join("application");
    let staging = root.join(".application.staging");
    fs::create_dir_all(package.join("assets/extensions/removed")).unwrap();
    fs::write(
        package.join("assets/extensions/removed/extension.json"),
        b"stale",
    )
    .unwrap();
    fs::create_dir_all(staging.join("assets/extensions/current")).unwrap();
    fs::write(
        staging.join("assets/extensions/current/extension.json"),
        b"current",
    )
    .unwrap();

    publish_development(&staging, &package).unwrap();

    assert!(
        package
            .join("assets/extensions/current/extension.json")
            .is_file()
    );
    assert!(!package.join("assets/extensions/removed").exists());
    assert!(!staging.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn release_publish_replaces_directory_and_archive_together() {
    let root = std::env::temp_dir().join(format!(
        "tuffite-package-atomic-publish-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let package = root.join("application");
    let archive = root.join("application.zip");
    let staging_package = root.join(".application.staging");
    let staging_archive = root.join(".application.zip.staging");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("version"), b"old").unwrap();
    fs::write(&archive, b"old archive").unwrap();
    fs::create_dir_all(&staging_package).unwrap();
    fs::write(staging_package.join("version"), b"new").unwrap();
    fs::write(&staging_archive, b"new archive").unwrap();

    publish_release(&staging_package, &staging_archive, &package, &archive).unwrap();

    assert_eq!(fs::read(package.join("version")).unwrap(), b"new");
    assert_eq!(fs::read(&archive).unwrap(), b"new archive");
    assert!(!staging_package.exists());
    assert!(!staging_archive.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn release_publish_restores_previous_outputs_on_archive_failure() {
    let root = std::env::temp_dir().join(format!(
        "tuffite-package-atomic-rollback-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let package = root.join("application");
    let archive = root.join("application.zip");
    let staging_package = root.join(".application.staging");
    let missing_staging_archive = root.join(".missing.zip.staging");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("version"), b"old").unwrap();
    fs::write(&archive, b"old archive").unwrap();
    fs::create_dir_all(&staging_package).unwrap();
    fs::write(staging_package.join("version"), b"new").unwrap();

    assert!(
        publish_release(
            &staging_package,
            &missing_staging_archive,
            &package,
            &archive,
        )
        .is_err()
    );

    assert_eq!(fs::read(package.join("version")).unwrap(), b"old");
    assert_eq!(fs::read(&archive).unwrap(), b"old archive");
    assert_eq!(fs::read(staging_package.join("version")).unwrap(), b"new");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn derives_package_name_from_application_id() {
    let target = package_target(if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else if cfg!(target_os = "windows") {
        "x86_64-pc-windows-msvc"
    } else {
        "x86_64-unknown-linux-gnu"
    })
    .unwrap();
    let name = default_package_name("com.tuffite.demo", &target);
    assert_eq!(
        name,
        format!("tuffite-demo-{}-{}", target.platform, target.architecture)
    );
}

#[test]
fn framework_target_drives_package_platform_and_architecture() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let x64 = package_target("x86_64-apple-darwin").unwrap();
    assert_eq!(x64.platform, "macos");
    assert_eq!(x64.architecture, "x64");

    let arm64 = package_target("aarch64-apple-darwin").unwrap();
    assert_eq!(arm64.platform, "macos");
    assert_eq!(arm64.architecture, "arm64");
}

#[test]
fn framework_target_rejects_cross_platform_packaging() {
    let other_target = if cfg!(target_os = "windows") {
        "x86_64-unknown-linux-gnu"
    } else {
        "x86_64-pc-windows-msvc"
    };
    assert!(package_target(other_target).is_err());
}

#[test]
fn application_build_pins_framework_target_and_distribution() {
    let command = application_build_command(
        Path::new("/tmp/example/Cargo.toml"),
        "x86_64-apple-darwin",
        Path::new("/tmp/verified-framework"),
        true,
    );
    assert!(command.get_envs().any(|(name, value)| {
        name == "TUFFITE_FRAMEWORK_DIST"
            && value == Some(std::ffi::OsStr::new("/tmp/verified-framework"))
    }));
    let arguments = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        arguments
            .windows(2)
            .any(|pair| { pair == ["--target", "x86_64-apple-darwin"] })
    );
}

#[test]
fn derives_and_validates_business_executable_name() {
    let name = executable_name(None, "playground");
    if cfg!(target_os = "windows") {
        assert_eq!(name, "playground.exe");
        assert!(validate_executable_name("playground.exe").is_ok());
        assert!(validate_executable_name("playground").is_err());
    } else {
        assert_eq!(name, "playground");
        assert!(validate_executable_name("playground").is_ok());
    }
    assert_eq!(
        executable_name(Some("custom"), "playground"),
        if cfg!(target_os = "windows") {
            "custom.exe"
        } else {
            "custom"
        }
    );
    assert!(validate_executable_name("nested/playground.exe").is_err());
}

#[test]
fn internal_asset_root_cannot_collide_with_package_files() {
    assert!(
        validate_internal_asset_root(Path::new("assets"), "playground", "libplayground.dylib")
            .is_ok()
    );
    assert!(
        validate_internal_asset_root(
            Path::new("playground/ui"),
            "playground",
            "libplayground.dylib"
        )
        .is_err()
    );
    assert!(
        validate_internal_asset_root(
            Path::new("tuffite.json/assets"),
            "playground",
            "libplayground.dylib"
        )
        .is_err()
    );
}

#[test]
fn reads_product_name_without_source_schema_version() {
    let manifest: AppManifest = serde_json::from_str(
        r#"{
              "productName": "Demo App",
              "app": {
                "identifier": "com.example.demo"
              },
              "bundle": {
                "resources": ["index.html"]
              }
            }"#,
    )
    .unwrap();
    assert_eq!(manifest.product_name.as_deref(), Some("Demo App"));
    assert_eq!(manifest.app.identifier, "com.example.demo");
    assert_eq!(manifest.bundle.resources, ["index.html"]);
}

#[test]
fn assets_include_only_explicit_sources_without_implicit_routes() {
    let project = std::env::temp_dir().join(format!("tuffite-assets-{}", std::process::id()));
    let _ = fs::remove_dir_all(&project);
    fs::create_dir_all(project.join("ui/assets")).unwrap();
    fs::write(project.join("ui/index.html"), "demo").unwrap();
    let mut manifest = serde_json::json!({"app": {
        "identifier": "com.example.demo",
        "assets": [
            {"origins": ["demo://main"], "paths": ["/main"], "file": "ui/index.html"},
            {"origins": ["demo://*.artifact"], "paths": ["/assets/*"], "directory": "ui/assets"}
        ]
    }});
    let app: PackageApp = serde_json::from_value(manifest["app"].clone()).unwrap();
    let before = manifest.clone();
    let resources = configure_assets(&project, &app, &[], &mut manifest).unwrap();
    assert_eq!(resources, ["ui/index.html", "ui/assets"]);
    assert_eq!(manifest, before);
    assert!(manifest["app"].get("assetRoutes").is_none());
    fs::remove_dir_all(project).unwrap();
}

#[test]
fn package_manifest_records_development_capability() {
    let manifest = PackageManifest {
        schema_version: 2,
        framework: "tuffite",
        framework_version: "0.1.0".to_owned(),
        framework_target: "x86_64-apple-darwin".to_owned(),
        framework_profile: "release".to_owned(),
        framework_variant: "desktop".to_owned(),
        ffi_abi_major: 1,
        ffi_abi_minor: 3,
        ffi_abi_patch: 7,
        application: "com.example.demo".to_owned(),
        platform: "windows",
        architecture: "x64",
        executable: "demo.exe".to_owned(),
        executable_integrity: "sha256",
        runtime_library: "demo.dll".to_owned(),
        development_enabled: true,
        files: Vec::new(),
    };
    let encoded = serde_json::to_value(manifest).unwrap();
    assert_eq!(encoded["development_enabled"], true);
    assert_eq!(encoded["framework_target"], "x86_64-apple-darwin");
    assert_eq!(encoded["ffi_abi_minor"], 3);
    assert_eq!(encoded["ffi_abi_patch"], 7);
}

#[test]
fn copies_a_resource_directory_without_symlinks() {
    let root = std::env::temp_dir().join(format!(
        "tuffite-package-resource-directory-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let project = root.join("project");
    let output = root.join("output");
    fs::create_dir_all(project.join("ui/dist/assets")).unwrap();
    fs::create_dir_all(&output).unwrap();
    fs::write(project.join("ui/dist/index.html"), b"index").unwrap();
    fs::write(project.join("ui/dist/assets/app.js"), b"app").unwrap();

    copy_resources(
        &project,
        Path::new("application"),
        &["ui/dist".to_owned()],
        &output,
    )
    .unwrap();

    assert_eq!(
        fs::read(output.join("application/ui/dist/index.html")).unwrap(),
        b"index"
    );
    assert_eq!(
        fs::read(output.join("application/ui/dist/assets/app.js")).unwrap(),
        b"app"
    );
    fs::remove_dir_all(root).unwrap();
}
