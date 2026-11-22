// SPDX-License-Identifier: Apache-2.0
//! #1971: `headwater taxonomy vendor <location>` against a server that holds
//! the connection and never answers, and against a vendor that dies while it
//! unpacks.
//!
//! The first case holds the bound on the whole fetch: before #1971 the client
//! set no timeout, and a vendor against a silent server waited for ever. The
//! second holds the sweep: a vendor killed while it unpacks leaves its
//! temporary directory behind, and the next vendor removes it.
//! Every server here listens on 127.0.0.1, and the suite reaches no other host.
#![cfg(feature = "fetch")]

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The bound on the whole fetch that `headwater-fetch` states as
/// `TOTAL_TIMEOUT`. A test crate cannot read a private constant, so the
/// number is written twice, and this case fails if the two disagree upward.
const TOTAL_TIMEOUT: Duration = Duration::from_secs(30);

const ANY_DIGEST: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// A new, empty directory under the system temporary directory, keyed on the
/// process, the case and a counter, because cargo runs the cases as threads
/// of one process.
fn scratch(label: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "hw-vendor-fetch-{label}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch directory is made");
    dir
}

/// A listener on 127.0.0.1 that accepts every connection and never writes a
/// byte. The streams stay open in the thread, so the client sees a server
/// that is there and silent rather than one that hangs up.
fn serve_silence() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port binds");
    let base = format!("http://{}", listener.local_addr().expect("the port reads"));
    std::thread::spawn(move || {
        let mut held = Vec::new();
        for stream in listener.incoming().flatten() {
            held.push(stream);
        }
    });
    base
}

/// A listener on 127.0.0.1 that answers `/x.zip` with `body` and every other
/// path with a 404.
#[cfg(unix)]
fn serve(body: Vec<u8>) -> String {
    use std::io::{Read, Write};
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port binds");
    let base = format!("http://{}", listener.local_addr().expect("the port reads"));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = [0_u8; 4096];
            let read = stream.read(&mut request).unwrap_or(0);
            let line = String::from_utf8_lossy(&request[..read]).to_string();
            let path = line.split_whitespace().nth(1).unwrap_or("").to_string();
            let (head, payload): (String, &[u8]) = if path == "/x.zip" {
                (
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    ),
                    &body,
                )
            } else {
                (
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        .to_string(),
                    &[],
                )
            };
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(payload);
        }
    });
    base
}

fn vendor(root: &Path, location: &str, tmpdir: Option<&Path>) -> Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_headwater"));
    command
        .current_dir(root)
        .args(["taxonomy", "vendor", location, "--expect", ANY_DIGEST])
        .env_remove("HTTP_PROXY")
        .env_remove("HTTPS_PROXY")
        .env_remove("ALL_PROXY")
        .env_remove("http_proxy")
        .env_remove("https_proxy")
        .env_remove("all_proxy")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(tmpdir) = tmpdir {
        command.env("TMPDIR", tmpdir);
    }
    command.spawn().expect("the binary starts")
}

/// Wait for `child` until `limit`, and kill it there, so that a case on a
/// client with no bound fails rather than hangs the suite.
fn wait_within(mut child: Child, limit: Duration) -> (Option<i32>, String, Duration) {
    use std::io::Read;
    let started = Instant::now();
    loop {
        if child.try_wait().expect("the child is polled").is_some() {
            break;
        }
        if started.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "the vendor was still running after {} s and was killed",
                limit.as_secs()
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let elapsed = started.elapsed();
    let status = child.wait().expect("the child is reaped");
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    (status.code(), stderr, elapsed)
}

#[test]
fn a_vendor_from_a_server_that_never_answers_exits_1_within_the_bound_and_names_the_host() {
    let root = scratch("silent");
    let location = format!("{}/x.zip", serve_silence());
    let child = vendor(&root, &location, None);
    let (code, stderr, elapsed) = wait_within(child, TOTAL_TIMEOUT + Duration::from_secs(10));
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        elapsed < TOTAL_TIMEOUT + Duration::from_secs(5),
        "the vendor took {elapsed:?}, past the bound of {TOTAL_TIMEOUT:?}"
    );
    assert!(
        stderr.contains("127.0.0.1") && stderr.contains("timed out"),
        "the refusal does not name the host and the timeout:\n{stderr}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The `headwater-fetch-*` entries of `dir`.
#[cfg(unix)]
fn fetch_dirs(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("headwater-fetch-"))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A zip with one member of `size` zero bytes, which deflates to a small body
/// and takes long enough to unpack that a kill can land inside it.
#[cfg(unix)]
fn zeros_zip(size: usize) -> Vec<u8> {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    writer
        .start_file(
            "zeros.bin",
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .large_file(false),
        )
        .expect("the member starts");
    let block = vec![0_u8; 1024 * 1024];
    let mut left = size;
    while left > 0 {
        let take = left.min(block.len());
        writer.write_all(&block[..take]).expect("the member writes");
        left -= take;
    }
    writer.finish().expect("the archive closes").into_inner()
}

/// Start a vendor of `location` under `tmpdir` and kill it the moment its
/// temporary directory appears. The directory it left, or `None` when the
/// vendor finished before the kill landed.
#[cfg(unix)]
fn kill_mid_unpack(root: &Path, location: &str, tmpdir: &Path) -> Option<PathBuf> {
    let mut child = vendor(root, location, Some(tmpdir));
    let started = Instant::now();
    loop {
        if let Some(left) = fetch_dirs(tmpdir).into_iter().next() {
            let _ = child.kill();
            // A signal leaves no exit code. A code means the vendor finished
            // before the kill landed, and its own drop removed the directory.
            let killed = child.wait().expect("the child is reaped").code().is_none();
            return (killed && left.exists()).then_some(left);
        }
        if child.try_wait().expect("the child is polled").is_some() {
            return None;
        }
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "the vendor made no temporary directory in 60 s"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(unix)]
#[test]
fn a_vendor_killed_mid_unpack_leaves_nothing_once_the_next_vendor_starts() {
    let root = scratch("killed-root");
    let tmpdir = scratch("killed-tmp");
    let base = serve(zeros_zip(60 * 1024 * 1024));
    let location = format!("{base}/x.zip");

    let mut left = None;
    for _ in 0..3 {
        left = kill_mid_unpack(&root, &location, &tmpdir);
        if left.is_some() {
            break;
        }
    }
    let left = left.expect("no kill landed inside the unpack in three tries");
    assert!(left.exists(), "the killed vendor left no directory to sweep");

    // A directory that a live vendor holds: its lock is taken by this process.
    let live = tmpdir.join("headwater-fetch-live-0-0");
    std::fs::create_dir_all(&live).expect("the live directory is made");
    let lock = std::fs::File::create(live.join(".lock")).expect("the lock file is made");
    lock.lock().expect("the lock is taken");

    let child = vendor(&root, &format!("{base}/missing.zip"), Some(&tmpdir));
    let (code, stderr, _elapsed) = wait_within(child, TOTAL_TIMEOUT + Duration::from_secs(10));
    assert_eq!(code, Some(1), "a 404 was accepted:\n{stderr}");
    assert!(
        !left.exists(),
        "the next vendor left the killed vendor's directory {} in place",
        left.display()
    );
    assert!(live.exists(), "the next vendor removed a directory a live vendor holds");

    drop(lock);
    let _ = std::fs::remove_dir_all(&tmpdir);
    let _ = std::fs::remove_dir_all(&root);
}
