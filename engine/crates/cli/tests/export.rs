// SPDX-License-Identifier: Apache-2.0
//! The operator-facing export fixture for an answer that one profile withholds.
//!
//! The fixture is a repository root rather than a library value. This target
//! invokes the built binary twice and gives both JSON artifacts to Python's
//! parser. The parser proves the answer exists only in the control artifact and
//! that the filtered artifact accounts for one withheld document.

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

    let parsed = Command::new("python3")
        .arg(root.join("verify.py"))
        .arg(&control_path)
        .arg(&filtered_path)
        .output()
        .expect("Python runs");
    assert_eq!(
        parsed.status.code(),
        Some(0),
        "the external parser rejected the differential\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&parsed.stdout),
        String::from_utf8_lossy(&parsed.stderr)
    );

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
