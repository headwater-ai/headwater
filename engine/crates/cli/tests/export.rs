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

/// A directory under the temporary directory that is removed when this value
/// is dropped, so a case that fails an assertion leaves nothing behind (#1158).
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Scratch {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for Scratch {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

fn scratch() -> Scratch {
    let suffix = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "headwater-cli-answered-export-{}-{suffix}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("the scratch directory is made");
    Scratch(path)
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

/// Copy a directory tree.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination");
    for entry in std::fs::read_dir(from).expect("the source tree") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the copy");
            }
        }
    }
}

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs")
}

fn status(output: &std::process::Output) -> (Option<i32>, String) {
    (
        output.status.code(),
        format!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

/// A `graph_export` declared `committed: false` is written by `headwater
/// export` and by nothing else, and neither gate requires or compares it
/// ([#1261](https://github.com/headwater-ai/headwater/issues/1261)).
///
/// The generate crate holds the same table over its library functions. This
/// case holds the binary to it, because the verb picks which of those
/// functions runs: an `export` that called `write` in place of `publish`
/// would pass every library case and never produce the artifact.
#[test]
fn an_uncommitted_graph_export_is_written_by_export_and_not_by_generate() {
    let root = scratch();
    copy_tree(&fixtures(), &root);
    // The fixture keeps its package where engines before 0.2.0 read it, and
    // the member has to reach the lock, so the package moves and resolves.
    std::fs::rename(root.join("packages"), root.join(".headwater/packages"))
        .expect("the package moves");
    let package = root.join(".headwater/packages/acme-answered-export/taxonomy.yml");
    let source = std::fs::read_to_string(&package).expect("the package reads");
    let declared = "    output: exports/control.json\n";
    assert!(source.contains(declared), "the fixture moved: {source}");
    std::fs::write(
        &package,
        source.replace(declared, &format!("{declared}    committed: false\n")),
    )
    .expect("the package writes");
    let (code, said) = status(&run(&root, &["taxonomy", "resolve"]));
    assert_eq!(code, Some(0), "the taxonomy does not resolve\n{said}");
    let at = root.join("exports/control.json");

    let (code, said) = status(&run(&root, &["generate"]));
    assert_eq!(code, Some(0), "`generate` failed\n{said}");
    assert!(
        !at.exists(),
        "`generate` wrote an export declared as built at publish time\n{said}"
    );
    assert!(
        root.join("exports/filtered.json").exists(),
        "`generate` did not write the committed export beside it\n{said}"
    );
    for gate in [&["generate", "--check"][..], &["export", "--check"][..]] {
        let (code, said) = status(&run(&root, gate));
        assert_eq!(code, Some(0), "{gate:?} requires the absent file\n{said}");
    }

    let (code, said) = status(&run(&root, &["export"]));
    assert_eq!(code, Some(0), "`export` failed\n{said}");
    let bytes = std::fs::read_to_string(&at).unwrap_or_else(|error| {
        panic!("`export` did not write the publish-time file: {error}\n{said}")
    });

    std::fs::write(&at, format!("{bytes}\n")).expect("the stale copy");
    for gate in [&["generate", "--check"][..], &["export", "--check"][..]] {
        let (code, said) = status(&run(&root, gate));
        assert_eq!(
            code,
            Some(0),
            "{gate:?} compares a stale local copy\n{said}"
        );
    }
}

/// The answered-export fixture with its `control` export declared
/// `committed: false`, resolved into the lock.
fn uncommitted_control() -> Scratch {
    let root = scratch();
    copy_tree(&fixtures(), &root);
    std::fs::rename(root.join("packages"), root.join(".headwater/packages"))
        .expect("the package moves");
    let package = root.join(".headwater/packages/acme-answered-export/taxonomy.yml");
    let source = std::fs::read_to_string(&package).expect("the package reads");
    let declared = "    output: exports/control.json\n";
    assert!(source.contains(declared), "the fixture moved: {source}");
    std::fs::write(
        &package,
        source.replace(declared, &format!("{declared}    committed: false\n")),
    )
    .expect("the package writes");
    let (code, said) = status(&run(&root, &["taxonomy", "resolve"]));
    assert_eq!(code, Some(0), "the taxonomy does not resolve\n{said}");
    root
}

/// `headwater export --at` dates the export that `committed: false` declares,
/// and its marker says which verb builds it
/// ([#1343](https://github.com/headwater-ai/headwater/issues/1343)).
///
/// Spec 6 asks a filtered export that leaves the repository to state when it
/// was generated. An uncommitted export is that artifact, and no gate compares
/// it by byte, so the date the byte gate forbids elsewhere is allowed here. A
/// run whose plan holds a committed export is still refused whole, because a
/// date inside that one would fail `generate --check` on the next morning.
#[test]
fn export_at_dates_an_uncommitted_graph_export_and_refuses_a_committed_one() {
    let root = uncommitted_control();
    let (code, said) = status(&run(&root, &["generate"]));
    assert_eq!(code, Some(0), "`generate` failed\n{said}");
    let control = root.join("exports/control.json");
    let filtered = root.join("exports/filtered.json");
    let committed = std::fs::read_to_string(&filtered).expect("the committed export");

    // The whole plan holds the committed `filtered` export, so the date is
    // refused for the run, and nothing is written.
    let refused = run(&root, &["export", "--at", "2026-09-30"]);
    let (code, said) = status(&refused);
    assert_eq!(code, Some(1), "`export --at` dated a committed export\n{said}");
    assert!(refused.stdout.is_empty(), "the refusal wrote a report\n{said}");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("exports/filtered.json"),
        "the refusal does not name the committed output\n{said}"
    );
    assert!(
        !stderr.contains("exports/control.json"),
        "the refusal names the uncommitted output as committed\n{said}"
    );
    assert!(
        !control.exists(),
        "the refused run wrote the uncommitted export\n{said}"
    );
    assert_eq!(
        std::fs::read_to_string(&filtered).expect("the committed export"),
        committed,
        "the refused run rewrote the committed export"
    );

    let (code, said) = status(&run(
        &root,
        &["export", "--profile", "control", "--at", "2026-09-30"],
    ));
    assert_eq!(
        code,
        Some(0),
        "`export --at` refused an uncommitted export\n{said}"
    );
    let written = std::fs::read_to_string(&control).expect("the dated export");
    assert!(
        written.contains("\"generated_at\": \"2026-09-30\""),
        "the uncommitted export does not state the injected date\n{written}"
    );
    let marker = written
        .lines()
        .find(|line| line.contains("\"headwater:generated\""))
        .expect("the marker member");
    assert!(
        marker.contains("\"graph_export. ") && marker.contains("`headwater export`"),
        "the marker does not name the verb that builds the file: {marker}"
    );
    assert!(
        !marker.contains("`headwater generate --check` holds it"),
        "the marker claims a gate holds an export that no gate compares: {marker}"
    );
    assert!(
        committed.contains("`headwater generate --check` holds it"),
        "the committed export lost the marker its gate relies on\n{committed}"
    );

    for gate in [&["generate", "--check"][..], &["export", "--check"][..]] {
        let (code, said) = status(&run(&root, gate));
        assert_eq!(
            code,
            Some(0),
            "{gate:?} failed after the dated export\n{said}"
        );
    }
    // A second publish recognizes its own dated file rather than refusing to
    // overwrite it.
    let (code, said) = status(&run(
        &root,
        &["export", "--profile", "control", "--at", "2026-10-01"],
    ));
    assert_eq!(
        code,
        Some(0),
        "`export` refused its own earlier output\n{said}"
    );
    let rewritten = std::fs::read_to_string(&control).expect("the dated export");
    assert!(
        rewritten.contains("\"generated_at\": \"2026-10-01\""),
        "the second publish did not rewrite the date\n{rewritten}"
    );
}

/// `--at` with `--check` is refused, because nothing `--check` compares
/// carries a date.
#[test]
fn export_at_is_refused_with_check() {
    let root = uncommitted_control();
    let (code, said) = status(&run(&root, &["export", "--check", "--at", "2026-09-30"]));
    assert_eq!(code, Some(2), "`export --check --at` ran\n{said}");
}
