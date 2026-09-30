// SPDX-License-Identifier: Apache-2.0
//! A committed imports snapshot that binds nothing is a finding that names the
//! pin, whether or not an anchor names its resolver.
//!
//! [Q19](../../../../docs/spec/09-decisions.md#q19--inbound-integration-an-external-system-of-record) says a
//! committed snapshot "is a pin". `harvest.pin.unread` made an unread corpus
//! export visible on those terms (#1311), and its imports twin was missing: a
//! declared import whose snapshot was absent, unpinned or not the pinned
//! artifact reached a reader only through an edge into it, so one that no
//! anchor named produced nothing
//! ([#1345](https://github.com/headwater-ai/headwater/issues/1345)).
//!
//! Each case drives the built binary, because the defect is that nothing
//! carried the reading from the resolver set to the checks, and a `Declared`
//! a test assembles by hand measures the test.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

const RULE: &str = "import.pin.unread";
const AT: &str = ".headwater/imports/upstream";

/// A snapshot payload with one item, in the shape a fetch script leaves.
const PAYLOAD: &str = "\
snapshot:
  format: 1
  source: acme/work-items
  fetched: 2026-09-30
  items:
    - id: \"12345\"
      revision: \"7\"
      title: The service refuses an unauthenticated request
";

/// A scratch corpus with this repository's taxonomy and one import declared
/// under `imports.upstream`, whose resolver no anchor kind names.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str, digest: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-import-pin-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        std::fs::copy(
            repository.join(".headwater/overlay.yml"),
            at.join(".headwater/overlay.yml"),
        )
        .expect("the overlay copies");

        let root = Root { at };
        // The files this repository's language regime names outside the corpus
        // root. A scratch corpus that lacks them is refused for that, which is
        // a finding about the scratch and not about the pin.
        for stub in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/ISSUE_TEMPLATE/issue.md",
            ".github/SECURITY.md",
        ] {
            root.write(stub, "# Scratch\n\nThis file is a stub.\n");
        }
        root.declare(digest);
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        root
    }

    /// Write `.headwater/taxonomy.yml` as this repository's, with the one
    /// import appended and pinned at `digest`.
    fn declare(&self, digest: &str) {
        let consumer = std::fs::read_to_string(repository().join(".headwater/taxonomy.yml"))
            .expect("the declaration reads");
        let imports = format!(
            "\nimports:\n  upstream:\n    at: {AT}\n    digest: {digest}\n    channel: the \
             publisher's release page\n    resolver: upstream-snapshot\n"
        );
        self.write(".headwater/taxonomy.yml", &format!("{consumer}{imports}"));
    }

    /// Commit a snapshot at [`AT`] with its release record, and return the
    /// digest a pin names it by.
    fn snapshot(&self) -> String {
        self.write(&format!("{AT}/snapshot.yml"), PAYLOAD);
        let dir = self.at.join(AT);
        let manifest = headwater_yaml::load("package: acme/work-items\nversion: \"2026-09-30\"\n")
            .expect("the manifest reads")
            .value
            .as_map()
            .expect("it is a mapping")
            .clone();
        let release = headwater_resolve::release::compute(&dir, &manifest).expect("it computes");
        std::fs::write(
            dir.join(headwater_resolve::release::RECORD),
            headwater_resolve::release::render(&release),
        )
        .expect("the record is written");
        release.digest
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.at.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the directory is made");
        std::fs::write(path, text).expect("the file writes");
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Every error a check reports on one file, one block each: the header
    /// line that names the file and the lines indented under it.
    fn errors_on(ran: &Ran, path: &str) -> Vec<String> {
        let header = format!("  {path}");
        let mut out: Vec<String> = Vec::new();
        let mut open = false;
        for line in ran.out.lines() {
            if line.starts_with("  ") && !line.starts_with("   ") {
                open = line.starts_with(&header) && line.contains("error");
                if open {
                    out.push(line.to_string());
                }
            } else if open && line.starts_with("    ") {
                let last = out.last_mut().expect("a block is open");
                last.push(' ');
                last.push_str(line.trim());
            } else {
                open = false;
            }
        }
        out
    }

    /// The errors on the declaration that name this rule.
    fn pins(ran: &Ran) -> Vec<String> {
        Root::errors_on(ran, ".headwater/taxonomy.yml")
            .into_iter()
            .filter(|block| block.contains(RULE))
            .collect()
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// The two streams held apart.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

/// The decisive case of #1345. The import is pinned, no snapshot is on disk,
/// and no anchor kind names its resolver, so no edge reaches it. The pin itself
/// is reported, once, on the declaration. Then a valid snapshot is committed
/// and pinned, and the declaration is silent.
#[test]
fn an_unread_snapshot_that_no_anchor_names_is_a_finding_that_names_the_pin() {
    let root = Root::new("unread", "sha256:0000");
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "the strict gate fails: {ran:?}");
    let pins = Root::pins(&ran);
    assert_eq!(pins.len(), 1, "one finding, for the one pin: {ran:?}");
    assert!(
        pins[0].contains(&format!(
            "`imports.upstream` pins a snapshot at `{AT}` that binds nothing"
        )),
        "the finding names the pin and where it is: {}",
        pins[0]
    );
    assert!(
        pins[0].contains("the snapshot of `upstream` is not the pinned artifact"),
        "the finding carries the resolver's reason: {}",
        pins[0]
    );

    let digest = root.snapshot();
    root.declare(&digest);
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(
        Root::errors_on(&ran, ".headwater/taxonomy.yml"),
        Vec::<String>::new(),
        "{ran:?}"
    );
    assert_eq!(ran.code, Some(0), "{ran:?}");
}

/// An import with no digest binds nothing either, and the finding says what to
/// write.
#[test]
fn a_snapshot_with_no_digest_is_a_finding_that_says_what_to_write() {
    let root = Root::new("undigested", "sha256:0000");
    root.snapshot();
    let declared = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    root.write(
        ".headwater/taxonomy.yml",
        &declared.replacen("    digest: sha256:0000\n", "", 1),
    );
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "{ran:?}");
    let pins = Root::pins(&ran);
    assert_eq!(pins.len(), 1, "{ran:?}");
    assert!(
        pins[0].contains("`imports.upstream`") && pins[0].contains("imports.upstream.digest"),
        "{}",
        pins[0]
    );
}

/// Each file of the snapshot joins the read set with the digest of its bytes,
/// so a gate over a later tree names a snapshot that moved, and names none over
/// the tree the read set was taken from.
#[test]
fn each_snapshot_file_joins_the_read_set_and_a_gate_sees_it_move() {
    let root = Root::new("read-set", "sha256:0000");
    let digest = root.snapshot();
    root.declare(&digest);
    let read_set = root.at.join("clean.readset");
    let read_set = read_set.to_str().expect("the read set path is UTF-8");
    let ran = root.run(&["check", "--read-set", read_set]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let recorded = std::fs::read_to_string(read_set).expect("the read set reads");
    let payload = format!("{AT}/snapshot.yml");
    let record = format!("{AT}/release.yml");
    for file in [&payload, &record] {
        let bytes = std::fs::read(root.at.join(file)).expect("the file reads");
        let digest = headwater_hash::digest(&bytes);
        assert!(
            recorded
                .lines()
                .any(|line| line.contains(file.as_str()) && line.contains(&digest)),
            "the read set lists `{file}` at {digest}: {recorded}"
        );
    }
    let still = root.run(&["gate", "--read-set", read_set]);
    assert!(!still.out.contains(&payload), "{still:?}");

    root.write(&payload, &PAYLOAD.replace("\"7\"", "\"8\""));
    let moved = root.run(&["gate", "--read-set", read_set]);
    assert!(
        moved.out.contains(&payload),
        "the gate names the snapshot file that moved: {moved:?}"
    );
}

/// A snapshot that is absent joins the read set too, with no digest, so a read
/// set never drops the file whose absence the rule reports.
#[test]
fn an_absent_snapshot_joins_the_read_set_with_no_digest() {
    let root = Root::new("read-set-absent", "sha256:0000");
    let read_set = root.at.join("absent.readset");
    let read_set = read_set.to_str().expect("the read set path is UTF-8");
    let ran = root.run(&["check", "--read-set", read_set]);
    let recorded = std::fs::read_to_string(read_set)
        .unwrap_or_else(|error| panic!("the read set is written: {error}: {ran:?}"));
    let payload = format!("{AT}/snapshot.yml");
    assert!(
        recorded.lines().any(|line| line.contains(&payload)),
        "the read set lists the absent payload: {recorded}"
    );
    let still = root.run(&["gate", "--read-set", read_set]);
    assert!(
        still
            .out
            .contains(&format!("{payload} carried no hash when it was read")),
        "{still:?}"
    );
}
