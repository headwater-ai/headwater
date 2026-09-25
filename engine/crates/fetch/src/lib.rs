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

/// The most bytes one fetch writes to disk when it unpacks, counted on the
/// bytes written and not on the sizes the archive declares, which can lie. It
/// is the same number as `LIMIT` for the same reason: the most the fetch
/// writes should be the most it is willing to read. The published
/// `headwater-standard` artifact unpacks to 392,598 bytes, so an honest
/// artifact never meets it.
const UNPACKED_LIMIT: u64 = 64 * 1024 * 1024;

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

/// What one request answered: a redirect to the target its `Location` header
/// names, as written, or the response to read.
enum Hop<T> {
    Redirect(String),
    Done(T),
}

/// Request `location` through `call`, following redirects. Every hop, the
/// first included, goes through [`allowed`] with the scheme the fetch began
/// on, so a redirect can neither turn an https fetch into plain http nor send
/// plain http off this machine. More than [`MAX_REDIRECTS`] redirects is a
/// refusal, and the target past the cap is never requested.
fn follow<T>(
    location: &str,
    mut call: impl FnMut(&str) -> Result<Hop<T>, Error>,
) -> Result<T, Error> {
    let started_https = scheme(location) == Some("https");
    let mut at = location.to_string();
    let mut hops = 0;
    loop {
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
        match call(&at)? {
            Hop::Done(answer) => return Ok(answer),
            Hop::Redirect(next) => {
                hops += 1;
                if hops > MAX_REDIRECTS {
                    return Err(Error::Transport(format!(
                        "{location} redirected more than {MAX_REDIRECTS} times"
                    )));
                }
                at = resolve(&at, &next);
            }
        }
    }
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
    // The client follows no redirect itself. `follow` reads each hop and puts
    // it through the same gate as the first request.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .build()
        .into();
    let mut response = follow(location, |at| {
        let response = agent
            .get(at)
            .call()
            .map_err(|error| Error::Transport(format!("fetching {at} failed: {error}")))?;
        if !response.status().is_redirection() {
            return Ok(Hop::Done(response));
        }
        let next = response
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                Error::Transport(format!("{at} answered with a redirect and no location"))
            })?;
        Ok(Hop::Redirect(next.to_string()))
    })?;
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
    unpack(&mut archive, &fetched.dir, UNPACKED_LIMIT, location)?;
    Ok(fetched)
}

/// Unpack `archive` into `dir`, refusing once more than `bound` bytes would
/// be written. The sizes an archive declares are read first, as a cheap
/// refusal, and then the bound is held again on the bytes written, because a
/// deflate member decompresses past its declared size and a header can lie.
/// A member whose name leaves `dir`, and a symbolic link, are refused: a
/// published artifact has neither. Whatever was written before a refusal is
/// removed by the caller's [`Fetched`] when it drops.
fn unpack<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    dir: &Path,
    bound: u64,
    location: &str,
) -> Result<(), Error> {
    let too_large = || {
        Error::Archive(format!(
            "the archive at {location} unpacks to more than {bound} bytes, the most this verb \
             writes"
        ))
    };
    let broken = |error: &dyn fmt::Display| {
        Error::Archive(format!(
            "the archive at {location} does not unpack: {error}"
        ))
    };
    let io = |path: &Path, error: std::io::Error| {
        Error::Io(format!("{} could not be written: {error}", path.display()))
    };
    if archive
        .decompressed_size()
        .is_some_and(|declared| declared > u128::from(bound))
    {
        return Err(too_large());
    }
    let mut written: u64 = 0;
    for index in 0..archive.len() {
        let mut member = archive.by_index(index).map_err(|error| broken(&error))?;
        let Some(name) = member.enclosed_name() else {
            return Err(broken(&format!(
                "the member {} names a path outside the directory",
                member.name()
            )));
        };
        let path = dir.join(name);
        if member.is_symlink() {
            return Err(broken(&format!(
                "the member {} is a symbolic link",
                member.name()
            )));
        }
        if member.is_dir() {
            std::fs::create_dir_all(&path).map_err(|error| io(&path, error))?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        let mut file = std::fs::File::create(&path).map_err(|error| io(&path, error))?;
        let remaining = bound - written;
        let copied = std::io::copy(
            &mut std::io::Read::take(&mut member, remaining.saturating_add(1)),
            &mut file,
        )
        .map_err(|error| broken(&error))?;
        if copied > remaining {
            return Err(too_large());
        }
        written += copied;
    }
    Ok(())
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

    /// Drive [`follow`] from `location` over a transport that answers each
    /// URL with `answer`, and return the result with every URL requested.
    fn followed(
        location: &str,
        mut answer: impl FnMut(usize, &str) -> Hop<()>,
    ) -> (Result<(), Error>, Vec<String>) {
        let mut asked = Vec::new();
        let result = follow(location, |at| {
            asked.push(at.to_string());
            Ok(answer(asked.len(), at))
        });
        (result, asked)
    }

    /// A transport that redirects `count` times, `/0` to `/1` and on, and
    /// then answers.
    fn chain(count: usize) -> impl FnMut(usize, &str) -> Hop<()> {
        move |request, _| {
            if request <= count {
                Hop::Redirect(format!("/{request}"))
            } else {
                Hop::Done(())
            }
        }
    }

    #[test]
    fn exactly_max_redirects_are_followed() {
        let (result, asked) = followed("http://127.0.0.1:9/0", chain(10));
        result.unwrap();
        assert_eq!(asked.len(), 11, "{asked:?}");
        assert_eq!(asked.last().unwrap(), "http://127.0.0.1:9/10");
    }

    #[test]
    fn one_redirect_past_max_redirects_is_refused_and_not_requested() {
        let (result, asked) = followed("http://127.0.0.1:9/0", chain(11));
        let refused = result.unwrap_err();
        assert!(matches!(refused, Error::Transport(_)), "{refused}");
        assert!(
            refused
                .to_string()
                .contains("redirected more than 10 times"),
            "{refused}"
        );
        assert_eq!(
            asked.len(),
            11,
            "the first request and ten redirects: {asked:?}"
        );
    }

    #[test]
    fn a_fetch_that_began_on_https_is_not_redirected_to_loopback_http() {
        let (result, asked) = followed("https://objects.example.org/x.zip", |_, _| {
            Hop::Redirect("http://127.0.0.1/x.zip".to_string())
        });
        let refused = result.unwrap_err();
        assert!(matches!(refused, Error::Scheme(_)), "{refused}");
        assert!(
            refused.to_string().contains("http://127.0.0.1/x.zip"),
            "{refused}"
        );
        assert_eq!(asked, ["https://objects.example.org/x.zip"]);
    }

    /// A deflate zip with one member, `zeros.bin`, of `len` zero bytes.
    fn zeros(len: usize) -> Vec<u8> {
        members(&[("zeros.bin", len)])
    }

    /// A deflate zip with one member of zero bytes for each name and length.
    fn members(files: &[(&str, usize)]) -> Vec<u8> {
        use std::io::Write;
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, len) in files {
            writer.start_file(*name, options).unwrap();
            writer.write_all(&vec![0_u8; *len]).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// Rewrite the `uncompressed_size` of every member to `size`, in the local
    /// header (22 bytes after `PK\x03\x04`) and in the central directory (24
    /// bytes after `PK\x01\x02`), so the archive declares less than it holds.
    fn lie_about_the_size(mut bytes: Vec<u8>, size: u32) -> Vec<u8> {
        let (mut local, mut central) = (0, 0);
        for at in 0..bytes.len().saturating_sub(4) {
            let offset = match &bytes[at..at + 4] {
                b"PK\x03\x04" => {
                    local += 1;
                    22
                }
                b"PK\x01\x02" => {
                    central += 1;
                    24
                }
                _ => continue,
            };
            bytes[at + offset..at + offset + 4].copy_from_slice(&size.to_le_bytes());
        }
        assert!(local > 0, "no local header was found");
        assert_eq!(local, central, "a central record for each local header");
        bytes
    }

    fn unpacked(bytes: Vec<u8>, bound: u64) -> (Result<(), Error>, Fetched) {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let fetched = Fetched {
            dir: scratch().unwrap(),
        };
        let result = unpack(&mut archive, &fetched.dir, bound, "test.zip");
        (result, fetched)
    }

    #[test]
    fn an_archive_that_unpacks_past_the_bound_is_refused() {
        let (result, _fetched) = unpacked(zeros(1025), 1024);
        let refused = result.unwrap_err();
        assert!(matches!(refused, Error::Archive(_)), "{refused}");
        assert!(refused.to_string().contains("1024"), "{refused}");
        assert!(refused.to_string().contains("test.zip"), "{refused}");
    }

    #[test]
    fn an_archive_that_unpacks_to_the_bound_is_accepted() {
        let (result, fetched) = unpacked(zeros(1024), 1024);
        result.unwrap();
        let written = std::fs::metadata(fetched.path().join("zeros.bin")).unwrap();
        assert_eq!(written.len(), 1024);
    }

    #[test]
    fn the_bound_holds_on_the_bytes_written_when_the_header_lies() {
        let (result, _fetched) = unpacked(lie_about_the_size(zeros(1025), 10), 1024);
        let refused = result.unwrap_err();
        assert!(matches!(refused, Error::Archive(_)), "{refused}");
        // Refused by the count of bytes written, not by a checksum or a size
        // mismatch the reader might raise first.
        assert!(
            refused.to_string().contains("more than 1024 bytes"),
            "{refused}"
        );
    }

    #[test]
    fn the_bound_holds_across_members_and_not_for_each_one() {
        // Each member is under the bound and declares 10 bytes, so neither
        // the declared total nor any one member meets it. The 1,200 bytes the
        // two write together do.
        let lying = lie_about_the_size(members(&[("a.bin", 600), ("b.bin", 600)]), 10);
        let (result, _fetched) = unpacked(lying, 1024);
        let refused = result.unwrap_err();
        assert!(matches!(refused, Error::Archive(_)), "{refused}");
        assert!(
            refused.to_string().contains("more than 1024 bytes"),
            "{refused}"
        );
    }

    #[test]
    fn a_declared_size_past_the_bound_is_refused_before_a_byte_is_written() {
        let (result, fetched) = unpacked(zeros(1025), 1024);
        let refused = result.unwrap_err();
        assert!(matches!(refused, Error::Archive(_)), "{refused}");
        let written: Vec<_> = std::fs::read_dir(fetched.path()).unwrap().collect();
        assert!(
            written.is_empty(),
            "written before the refusal: {written:?}"
        );
    }
}
