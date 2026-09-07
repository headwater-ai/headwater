// SPDX-License-Identifier: Apache-2.0
//! The operator-facing export fixture for an answer that one profile withholds.
//!
//! The fixture is a repository root rather than a library value. This target
//! invokes the built binary twice and gives both JSON artifacts to Python's
//! parser. The parser proves the answer exists only in the control artifact and
//! that the filtered artifact accounts for one withheld document.
//!
//! The parser is `python3`, which the pinned build container does not carry, so
//! its absence is a skip and `HEADWATER_STOCK_VALIDATOR` turns that skip into a
//! failure. That is the posture `crates/generate/tests/differential.rs` and
//! `crates/adapter/tests/fixtures.rs` already took, and this target was the one
//! of the three that did not: it called `.expect("Python runs")`, so the
//! container command in `engine/README.md` exited 101 on a machine with no
//! Python while that page said the differential suites skip there. Continuous
//! integration sets the variable, so nothing is softened where Python exists.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/answered-export")
}

fn scratch() -> PathBuf {
    let suffix = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "headwater-cli-answered-export-{}-{suffix}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("the scratch directory is made");
    path
}

fn export(root: &Path, profile: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["export", "--profile", profile, "--format", "json", "--root"])
        .arg(root)
        .output()
        .expect("the binary runs")
}

#[test]
fn stock_export_exposes_the_control_answer_and_the_filtered_tombstone() {
    let root = fixtures();
    let control = export(&root, "control");
    assert_eq!(
        control.status.code(),
        Some(0),
        "control export failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&control.stdout),
        String::from_utf8_lossy(&control.stderr)
    );
    let filtered = export(&root, "filtered");
    assert_eq!(
        filtered.status.code(),
        Some(0),
        "filtered export failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&filtered.stdout),
        String::from_utf8_lossy(&filtered.stderr)
    );

    let scratch = scratch();
    let control_path = scratch.join("control.json");
    let filtered_path = scratch.join("filtered.json");
    std::fs::write(&control_path, control.stdout).expect("the control artifact writes");
    std::fs::write(&filtered_path, filtered.stdout).expect("the filtered artifact writes");

    let required = std::env::var_os("HEADWATER_STOCK_VALIDATOR").is_some();
    let ran = Command::new("python3")
        .arg(root.join("verify.py"))
        .arg(&control_path)
        .arg(&filtered_path)
        .output();
    let parsed = match ran {
        Ok(output) if output.status.success() => output,
        other => {
            let reason = match other {
                Ok(output) => format!(
                    "the external parser rejected the differential\nstdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
                Err(error) => error.to_string(),
            };
            assert!(
                !required,
                "HEADWATER_STOCK_VALIDATOR is set and the external parser did not \
                 run, so nothing pins the withheld answer: {reason}"
            );
            eprintln!(
                "note: the external parser did not run, so the withheld answer in \
                 this file is unpinned. Install `python3`, or set \
                 HEADWATER_STOCK_VALIDATOR to make its absence a failure.\n{reason}"
            );
            return;
        }
    };

    let actual = String::from_utf8(parsed.stdout).expect("the parser writes UTF-8");
    let record = root.join("export.record");
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::write(&record, &actual).expect("the record writes");
    } else {
        let expected = std::fs::read_to_string(&record)
            .unwrap_or_else(|error| panic!("{}: {error}", record.display()));
        assert_eq!(expected, actual, "the export differential moved");
    }
}
