//! Web child process, authenticated readiness protocol, and loopback endpoint policy.
use processkit::{Command as ProcessCommand, RunningProcess, StdioMode};
use sha2::{Digest as _, Sha256};
use std::fmt::Write as _;
use std::io::{Read as _, Write as _};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use url::{Host, Position, Url};

pub(super) const DEV_TOKEN_ENV: &str = "TUFFITE_DEV_TOKEN";
const DEV_PROBE_PATH: &str = "/.well-known/tuffite-dev";

pub(super) struct DevelopmentToken {
    value: String,
    digest: String,
}

impl DevelopmentToken {
    pub(super) fn new() -> Result<Self, String> {
        let value = development_token()?;
        let digest = token_digest(&value);
        Ok(Self { value, digest })
    }
    pub(super) fn value(&self) -> &str {
        &self.value
    }
    pub(super) fn digest(&self) -> &str {
        &self.digest
    }
}

pub(super) struct ServerConfiguration {
    pub(super) url: Url,
    pub(super) command: PathBuf,
    pub(super) args: Vec<std::ffi::OsString>,
    pub(super) cwd: PathBuf,
}

pub(super) struct WebServer {
    pub(super) process: RunningProcess,
}

impl WebServer {
    pub(super) fn start(
        runtime: &Runtime,
        configuration: &ServerConfiguration,
        token: &DevelopmentToken,
    ) -> Result<Self, String> {
        let cwd = configuration.cwd.as_path();
        let resolved_command = resolve_executable(&configuration.command, &configuration.cwd)?;
        let command = ProcessCommand::new(&resolved_command)
            .args(&configuration.args)
            .current_dir(cwd)
            .env(DEV_TOKEN_ENV, token.value())
            .stdout(StdioMode::Inherit)
            .stderr(StdioMode::Inherit)
            .kill_on_parent_death();
        let mut process = runtime.block_on(command.start()).map_err(|error| {
            format!(
                "start Web development server {}: {error}",
                configuration.command.display()
            )
        })?;
        let deadline = Instant::now() + Duration::from_secs(30);
        let digest = token.digest();
        let ready = runtime.block_on(async {
            while Instant::now() < deadline {
                let mut processes = [&mut process];
                tokio::select! {
                    outcome = processkit::wait_any(&mut processes) => {
                        return Err(format!(
                            "Web development server exited before becoming ready: {outcome:?}"
                        ));
                    }
                    signal = tokio::signal::ctrl_c() => {
                        return match signal {
                            Ok(()) => Err("development session interrupted".to_owned()),
                            Err(error) => Err(format!("listen for interruption: {error}")),
                        };
                    }
                    () = tokio::time::sleep(Duration::from_millis(100)) => {}
                }
                if probe_server(&configuration.url, digest).unwrap_or(false) {
                    return Ok(());
                }
            }
            Err(
                "Web development server did not answer the authenticated Tuffite probe within 30 seconds"
                    .to_owned(),
            )
        });
        if let Err(error) = ready {
            drop(process);
            return Err(error);
        }
        Ok(Self { process })
    }
}
fn probe_server(url: &Url, token: &str) -> Result<bool, String> {
    let addresses = url
        .socket_addrs(|| None)
        .map_err(|error| format!("resolve development endpoint: {error}"))?;
    let authority = &url[Position::BeforeHost..Position::AfterPort];
    for address in addresses {
        let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(250))
        else {
            continue;
        };
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .map_err(|error| format!("set development probe timeout: {error}"))?;
        write!(
            stream,
            "GET {DEV_PROBE_PATH} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n"
        )
        .map_err(|error| format!("write development probe: {error}"))?;
        let mut response = String::new();
        stream
            .take(16 * 1024)
            .read_to_string(&mut response)
            .map_err(|error| format!("read development probe: {error}"))?;
        if authenticated_probe_response(&response, token) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn authenticated_probe_response(response: &str, digest: &str) -> bool {
    response.starts_with("HTTP/1.1 204")
        && response.lines().any(|line| {
            line.split_once(':').is_some_and(|(name, value)| {
                name.eq_ignore_ascii_case("x-tuffite-dev-token") && value.trim() == digest
            })
        })
}

fn development_token() -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| format!("generate development token: {error}"))?;
    Ok(bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut value, byte| {
            write!(value, "{byte:02x}").expect("writing to a String cannot fail");
            value
        }))
}

fn resolve_executable(command: &Path, cwd: &Path) -> Result<PathBuf, String> {
    if command.components().count() > 1 || command.is_absolute() {
        let path = if command.is_absolute() {
            command.to_path_buf()
        } else {
            cwd.join(command)
        };
        return std::fs::canonicalize(path).map_err(|error| {
            format!(
                "resolve Web development server {}: {error}",
                command.display()
            )
        });
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    for directory in std::env::split_paths(&path) {
        let base = if directory.is_absolute() {
            directory
        } else {
            cwd.join(directory)
        }
        .join(command);
        let candidates = if cfg!(windows) {
            vec![
                base.clone(),
                base.with_extension("exe"),
                base.with_extension("cmd"),
                base.with_extension("bat"),
            ]
        } else {
            vec![base]
        };
        if let Some(found) = candidates.into_iter().find(|candidate| candidate.is_file()) {
            return std::fs::canonicalize(&found).map_err(|error| {
                format!(
                    "resolve Web development server {}: {error}",
                    found.display()
                )
            });
        }
    }
    Err(format!(
        "Web development server executable was not found: {}",
        command.display()
    ))
}

fn token_digest(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

pub(super) fn validate_development_url(value: &str) -> Result<Url, String> {
    let url = Url::parse(value).map_err(|error| format!("invalid development URL: {error}"))?;
    if url.scheme() != "http"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !is_loopback_host(url.host().as_ref())
    {
        return Err("development URL must be a credential-free loopback HTTP URL".to_owned());
    }
    Ok(url)
}

fn is_loopback_host(host: Option<&Host<&str>>) -> bool {
    match host {
        Some(Host::Domain(host)) => {
            host.eq_ignore_ascii_case("localhost")
                || host.to_ascii_lowercase().ends_with(".localhost")
        }
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_commands_resolve_against_server_working_directory() {
        let root =
            std::env::temp_dir().join(format!("tuffite-server-command-{}", std::process::id()));
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::write(root.join("bin/server"), "fixture").unwrap();
        assert_eq!(
            resolve_executable(Path::new("bin/server"), &root).unwrap(),
            std::fs::canonicalize(root.join("bin/server")).unwrap()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_only_loopback_http_urls() {
        assert!(validate_development_url("http://127.0.0.1:5173/").is_ok());
        assert!(validate_development_url("http://localhost:5173/").is_ok());
        assert!(validate_development_url("http://[::1]:5173/").is_ok());
        assert!(validate_development_url("https://localhost:5173/").is_err());
        assert!(validate_development_url("https://example.com/").is_err());
        assert!(validate_development_url("file:///tmp/index.html").is_err());
        assert!(validate_development_url("http://user@localhost:5173/").is_err());
    }

    #[test]
    fn development_tokens_are_high_entropy_hex() {
        let first = development_token().unwrap();
        let second = development_token().unwrap();
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, second);
        assert_eq!(token_digest(&first).len(), 64);
    }

    #[test]
    fn readiness_probe_requires_matching_digest() {
        let response = "HTTP/1.1 204 No Content\r\nX-Tuffite-Dev-Token: abc123\r\n\r\n";
        assert!(authenticated_probe_response(response, "abc123"));
        assert!(!authenticated_probe_response(response, "different"));
        assert!(!authenticated_probe_response(
            "HTTP/1.1 204 No Content\r\n\r\n",
            "abc123"
        ));
        assert!(!authenticated_probe_response(
            "HTTP/1.1 200 OK\r\nX-Tuffite-Dev-Token: abc123\r\n\r\n",
            "abc123"
        ));
    }
}
