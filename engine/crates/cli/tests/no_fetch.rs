// SPDX-License-Identifier: Apache-2.0
//! The binary built without the `fetch` feature refuses a location (#1113).
//!
//! `--no-default-features` is a distributor's opt-out, and HW-DR-0075 states
//! what it builds: a binary that opens no socket, whose `taxonomy vendor`
//! answers a location with one line that names the path form. Every other
//! suite runs with the default features, so this file compiles only without
//! them, and CI runs it by name:
//!
//!     cargo test -p headwater-cli --no-default-features --locked --test no_fetch
//!
//! The location names `example.invalid`, which no resolver answers, so a
//! binary that did try to fetch would fail with a transport error and not
//! with the refusal these cases read.
#![cfg(not(feature = "fetch"))]

use std::path::PathBuf;
use std::process::Command;

/// A digest of the right shape. `vendor` refuses an unpinned location before
/// it reaches the fetch, so the pin is what lets the refusal under test speak.
const PIN: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// A new empty adopter root, keyed on the case name so that two cases running
/// as threads of one process never share one.
fn scratch(case: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "headwater-no-fetch-{}-{case}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the adopter root is made");
    dir
}

/// Run `taxonomy vendor <location> --expect <PIN>` and return the exit code,
/// the standard error and whether the package area was written.
fn vendor(case: &str, location: &str) -> (Option<i32>, String, bool) {
    let root = scratch(case);
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "vendor", location, "--expect", PIN, "--root"])
        .arg(&root)
        .output()
        .expect("the binary runs");
    let wrote = root.join(".headwater/packages").exists();
    let _ = std::fs::remove_dir_all(&root);
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        wrote,
    )
}

fn assert_refused(case: &str, location: &str) {
    let (code, stderr, wrote) = vendor(case, location);
    assert_eq!(code, Some(1), "a location was not refused:\n{stderr}");
    assert!(
        stderr.contains("built without the `fetch` feature"),
        "the refusal does not say why:\n{stderr}"
    );
    assert!(
        stderr.contains("taxonomy vendor <dir> --expect <digest>"),
        "the refusal does not name the path form:\n{stderr}"
    );
    assert!(!wrote, "a refused vendor wrote into the package area");
}

#[test]
fn a_binary_without_the_fetch_refuses_an_https_location() {
    assert_refused("https", "https://example.invalid/x.zip");
}

/// A scheme is case-insensitive, so an upper-case `HTTP://` is a location and
/// is refused as one, not read as a directory.
#[test]
fn a_binary_without_the_fetch_refuses_an_upper_case_http_location() {
    assert_refused("upper-http", "HTTP://127.0.0.1:9/x.zip");
}
