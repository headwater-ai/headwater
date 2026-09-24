// SPDX-License-Identifier: Apache-2.0
//! The one crate of this engine that opens a socket.
//!
//! [HW-DR-0075](../../../../docs/decisions/0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md)
//! lets `headwater taxonomy vendor` take a location as well as a directory, on
//! the condition that the client lives in a crate no part of the checking loop
//! links. This is that crate, and `headwater-cli` is its only dependent.
//! `crates/cli/tests/network_boundary.rs` reads `engine/Cargo.lock` and fails
//! when any other crate reaches this one or the packages under it.
//!
//! A location is the URL of a published artifact zip, the shape
//! `release-taxonomy.yml` writes: `headwater-standard-<version>.zip`, with the
//! artifact at the root of the archive. This crate downloads it, follows
//! redirects (a release asset answers with a 302 to another host), unpacks it
//! into a new temporary directory and hands back that directory. It checks
//! nothing about the bytes. The digest check stays in
//! `headwater_resolve::package::vendor`, which reads the directory exactly as
//! it reads one a person fetched by hand.

use std::fmt;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The largest body this reads. A published taxonomy artifact is well under a
/// megabyte; the cap is there so that a wrong URL cannot fill the disk.
const LIMIT: u64 = 64 * 1024 * 1024;

/// The most redirects one fetch follows. A release asset takes one.
const MAX_REDIRECTS: u32 = 10;

/// Whether a command-line argument names a location rather than a directory.
/// A directory whose name begins `http://` or `https://` is not a case this
/// verb serves, and `./https:/...` still reaches it as a path.
#[must_use]
pub fn is_location(argument: &str) -> bool {
    scheme(argument).is_some()
}

/// The scheme of `location` when it is `http` or `https`, in lower case. A
/// scheme is case-insensitive, so `HTTPS://` is a location and not a path.
fn scheme(location: &str) -> Option<&'static str> {
    let (scheme, _) = location.split_once("://")?;
    if scheme.eq_ignore_ascii_case("https") {
        Some("https")
    } else if scheme.eq_ignore_ascii_case("http") {
        Some("http")
    } else {
        None
    }
}

/// Whether a request may go to `location`. A fetch that began over https
/// stays on https for every hop, and plain http reaches only this machine,
/// whichever hop names it. This is the one gate, read at the first request
/// and again at every redirect.
fn allowed(started_https: bool, location: &str) -> bool {
    match scheme(location) {
        Some("https") => true,
        Some("http") => !started_https && loopback(host(location)),
        _ => false,
    }
}

/// The absolute URL a `Location` header names, read against the URL that
/// answered with it.
fn resolve(base: &str, target: &str) -> String {
    if scheme(target).is_some() {
        return target.to_string();
    }
    let (base_scheme, rest) = base.split_once("://").unwrap_or(("https", base));
    if let Some(network) = target.strip_prefix("//") {
        return format!("{base_scheme}://{network}");
    }
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let origin = &base[..base_scheme.len() + 3 + authority_end];
    if target.starts_with('/') {
        return format!("{origin}{target}");
    }
    let path = &rest[authority_end..];
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let directory = path.rfind('/').map_or("/", |at| &path[..=at]);
    format!("{origin}{directory}{target}")
}

/// Why a fetch did not produce a directory. Each variant carries the one
/// sentence a person reads.
#[derive(Debug)]
pub enum Error {
    /// The location is not `https://`, and not `http://` to a loopback host.
    Scheme(String),
    /// The request failed, or the server answered with an error status.
    Transport(String),
    /// The body is not a zip this crate can unpack.
    Archive(String),
    /// The temporary directory could not be made or written.
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scheme(text) | Self::Transport(text) | Self::Archive(text) | Self::Io(text) => {
                f.write_str(text)
            }
        }
    }
}

impl std::error::Error for Error {}

/// An unpacked artifact in a temporary directory, removed when this drops.
#[derive(Debug)]
pub struct Fetched {
    dir: PathBuf,
}

impl Fetched {
    /// The directory that holds the artifact, as `vendor` reads it.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for Fetched {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The host of an `http://` or `https://` URL, without userinfo or port.
fn host(location: &str) -> &str {
    let rest = location
        .split_once("://")
        .map_or(location, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let authority = authority.rsplit_once('@').map_or(authority, |(_, a)| a);
    if let Some(bracketed) = authority.strip_prefix('[') {
        return bracketed.split(']').next().unwrap_or(bracketed);
    }
    authority.split(':').next().unwrap_or(authority)
}

fn loopback(host: &str) -> bool {
    host == "localhost"
        || host == "::1"
        || host
            .parse::<std::net::Ipv4Addr>()
            .is_ok_and(|ip| ip.is_loopback())
}

/// Download the artifact zip at `location` and unpack it into a new temporary
/// directory.
///
/// # Errors
///
/// [`Error::Scheme`] for a location that is neither `https://` nor `http://`
/// to a loopback host, [`Error::Transport`] for a failed request or an error
/// status, [`Error::Archive`] for a body that is not a readable zip, and
/// [`Error::Io`] when the directory cannot be written.
pub fn fetch(location: &str) -> Result<Fetched, Error> {
    let started_https = scheme(location) == Some("https");
    // The client follows no redirect itself. Each hop is read here and put
    // through the same gate as the first request, so a redirect can neither
    // turn an https fetch into plain http nor send plain http off this machine.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .build()
        .into();
    let mut at = location.to_string();
    let mut hops = 0;
    let mut response = loop {
        if !allowed(started_https, &at) {
            return Err(Error::Scheme(if at == location {
                format!(
                    "{location} is not an https:// location. This verb fetches over https only, \
                     and plain http:// only from this machine"
                )
            } else {
                format!(
                    "{location} redirected to {at}, which this verb does not fetch. A fetch that \
                     began over https stays on https, and plain http:// reaches only this machine"
                )
            }));
        }
        let response = agent
            .get(&at)
            .call()
            .map_err(|error| Error::Transport(format!("fetching {at} failed: {error}")))?;
        if !response.status().is_redirection() {
            break response;
        }
        hops += 1;
        let next = response
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                Error::Transport(format!("{at} answered with a redirect and no location"))
            })?;
        if hops > MAX_REDIRECTS {
            return Err(Error::Transport(format!(
                "{location} redirected more than {MAX_REDIRECTS} times"
            )));
        }
        at = resolve(&at, next);
    };
    let body = response
        .body_mut()
        .with_config()
        .limit(LIMIT)
        .read_to_vec()
        .map_err(|error| Error::Transport(format!("reading {location} failed: {error}")))?;

    let mut archive = zip::ZipArchive::new(Cursor::new(body)).map_err(|error| {
        Error::Archive(format!(
            "{location} did not answer with a zip archive: {error}"
        ))
    })?;
    let fetched = Fetched { dir: scratch()? };
    // `extract` refuses a member whose name leaves the directory.
    archive.extract(&fetched.dir).map_err(|error| {
        Error::Archive(format!(
            "the archive at {location} does not unpack: {error}"
        ))
    })?;
    Ok(fetched)
}

/// A new, empty directory under the system temporary directory. The name
/// carries the process id, the time and a counter, because `cargo test` runs
/// cases as threads of one process and a name keyed on the process alone races.
fn scratch() -> Result<PathBuf, Error> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let dir = std::env::temp_dir().join(format!(
        "headwater-fetch-{}-{nanos}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir)
        .map_err(|error| Error::Io(format!("{} could not be made: {error}", dir.display())))?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_host_is_read_past_userinfo_port_and_path() {
        assert_eq!(host("http://127.0.0.1:8080/x.zip"), "127.0.0.1");
        assert_eq!(host("https://u:p@example.org/a"), "example.org");
        assert_eq!(host("http://[::1]:9/a"), "::1");
        assert_eq!(host("http://localhost"), "localhost");
    }

    #[test]
    fn an_https_fetch_stays_on_https_at_every_hop() {
        assert!(allowed(true, "https://objects.example.org/x.zip"));
        assert!(!allowed(true, "http://127.0.0.1/x.zip"));
        assert!(!allowed(true, "http://example.org/x.zip"));
    }

    #[test]
    fn plain_http_reaches_only_this_machine_at_every_hop() {
        assert!(allowed(false, "http://127.0.0.1:9/x.zip"));
        assert!(allowed(false, "https://example.org/x.zip"));
        assert!(!allowed(false, "http://example.org/x.zip"));
        assert!(!allowed(false, "ftp://127.0.0.1/x.zip"));
    }

    #[test]
    fn a_scheme_is_read_in_any_case() {
        assert!(is_location("HTTPS://example.org/x.zip"));
        assert!(is_location("Http://127.0.0.1/x.zip"));
        assert!(!is_location("./https:/x"));
        assert!(allowed(true, "HTTPS://example.org/x.zip"));
    }

    #[test]
    fn a_redirect_target_is_read_against_the_url_that_sent_it() {
        let base = "http://127.0.0.1:8/a/b/moved?x=1";
        assert_eq!(resolve(base, "/x.zip"), "http://127.0.0.1:8/x.zip");
        assert_eq!(resolve(base, "x.zip"), "http://127.0.0.1:8/a/b/x.zip");
        assert_eq!(resolve(base, "//h.org/y"), "http://h.org/y");
        assert_eq!(resolve(base, "https://h.org/y"), "https://h.org/y");
        assert_eq!(resolve("http://h:1", "/x"), "http://h:1/x");
    }

    #[test]
    fn plain_http_is_refused_unless_the_host_is_this_machine() {
        let refused = fetch("http://example.org/x.zip").unwrap_err();
        assert!(matches!(refused, Error::Scheme(_)), "{refused}");
        let refused = fetch("ftp://127.0.0.1/x.zip").unwrap_err();
        assert!(matches!(refused, Error::Scheme(_)), "{refused}");
        assert!(loopback("127.0.0.1") && loopback("::1") && !loopback("10.0.0.1"));
    }
}
