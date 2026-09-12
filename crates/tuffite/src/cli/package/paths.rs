//! Paths operations for application packaging.
use super::cargo::PackageTarget;

pub(super) fn validate_application_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err("application_id contains unsupported characters".to_owned());
    }
    Ok(())
}

pub(super) fn default_package_name(application_id: &str, target: &PackageTarget<'_>) -> String {
    let application = application_id
        .rsplit('.')
        .next()
        .unwrap_or(application_id)
        .to_ascii_lowercase();
    format!(
        "tuffite-{application}-{}-{}",
        target.platform, target.architecture
    )
}

pub(super) fn validate_package_name(value: &str) -> Result<(), String> {
    super::super::config::validate_output_name(value)
        .map_err(|error| error.replace("output name", "package name"))
}
