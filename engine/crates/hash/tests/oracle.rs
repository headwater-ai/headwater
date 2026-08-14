// SPDX-License-Identifier: Apache-2.0
//! This implementation of SHA-256 against one that nobody here wrote.
//!
//! # Why this file exists
//!
//! The module doc of this crate argues that SHA-256 is a fixed function of
//! about eighty lines, and that writing it beat six transitive crates. It also
//! named the one digest that would deserve a vetted implementation: the package
//! digest of a fetched artifact, which is a supply-chain check rather than an
//! integrity mark inside a repository that already holds the bytes.
//!
//! That digest exists now, in `headwater_resolve::release`, and it reuses this
//! function. [Q22](../../../../docs/spec/09-decisions.md#q22--the-integrity-posture-of-a-published-package)
//! is where the reuse was decided, and the argument turns on what a dependency
//! would have bought. It would buy a correct implementation, and it would buy no
//! authentication at all, because the check gets its strength from a pin that a
//! person committed rather than from who wrote the compression function.
//!
//! So the risk a dependency answers is an implementation defect, and this file
//! is the evidence that answers it instead. The published FIPS 180-4 vectors in
//! `src/lib.rs` hold the function against the standard. This holds it against a
//! second implementation, over bytes that no vector anticipated.
//!
//! # Skipping, and what turns a skip into a failure
//!
//! `sha256sum` is coreutils, and it is absent from some minimal images. Absent,
//! this skips with a printed note, on the terms the two stock-validator
//! differentials already take. Set `HEADWATER_SHA256_ORACLE` and the skip
//! becomes a failure. Continuous integration sets it, so the cross-check cannot
//! go quiet by losing a dependency.

use std::process::Command;

/// What `sha256sum` makes of some bytes, or why it could not be asked.
fn oracle(bytes: &[u8]) -> Result<String, String> {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("oracle");
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!("subject-{}", std::process::id()));
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;

    let ran = Command::new("sha256sum")
        .arg(&path)
        .output()
        .map_err(|error| format!("sha256sum did not run: {error}"))?;
    let _ = std::fs::remove_file(&path);
    if !ran.status.success() {
        return Err(format!(
            "sha256sum exited {}: {}",
            ran.status,
            String::from_utf8_lossy(&ran.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&ran.stdout).to_string();
    text.split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| format!("sha256sum wrote nothing this reads: {text}"))
}

/// The subjects. The first four are the shapes a package artifact carries: a
/// manifest, a member list of the form the release digest is taken over, bytes
/// that are not text, and the empty file. The rest walk the block boundary,
/// which is where a padding defect that no fixed vector meets would hide.
fn subjects() -> Vec<(String, Vec<u8>)> {
    let mut out = vec![
        (
            "a manifest".to_string(),
            b"package: acme/fixture\nversion: 1.0.0\nrequires_engine: \">=0.1 <2\"\n".to_vec(),
        ),
        (
            "a member list".to_string(),
            b"bundles/extra/bundle.yml\tsha256:00\npackage.yml\tsha256:01\n".to_vec(),
        ),
        ("bytes that are not text".to_string(), (0u8..=255).collect()),
        ("the empty file".to_string(), Vec::new()),
    ];
    for length in [1usize, 54, 55, 56, 57, 63, 64, 65, 119, 120, 128, 1000] {
        out.push((
            format!("{length} bytes"),
            (0..length).map(|index| (index % 251) as u8).collect(),
        ));
    }
    out
}

/// Every subject hashes to the same sixty-four digits under both
/// implementations.
#[test]
fn a_second_implementation_agrees_on_every_subject() {
    let required = std::env::var_os("HEADWATER_SHA256_ORACLE").is_some();

    let subjects = subjects();
    assert!(
        subjects.len() >= 16,
        "a cross-check over a handful of subjects is a cross-check that proves a handful"
    );

    for (name, bytes) in &subjects {
        let theirs = match oracle(bytes) {
            Ok(digest) => digest,
            Err(reason) => {
                assert!(
                    !required,
                    "HEADWATER_SHA256_ORACLE is set and sha256sum did not run, so nothing \
                     holds this implementation against a second one: {reason}"
                );
                eprintln!(
                    "note: sha256sum did not run, so this implementation is held only against \
                     the published vectors. Install coreutils, or set HEADWATER_SHA256_ORACLE \
                     to make its absence a failure.\n{reason}"
                );
                return;
            }
        };
        assert_eq!(
            headwater_hash::hex(bytes),
            theirs,
            "the two implementations disagree on {name}"
        );
        assert_eq!(
            headwater_hash::digest(bytes),
            format!("sha256:{theirs}"),
            "the named form is not the hexadecimal one for {name}"
        );
    }
}
