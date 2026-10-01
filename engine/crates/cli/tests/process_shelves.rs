// SPDX-License-Identifier: Apache-2.0
//! What this repository's overlay declares for a process obligation record.
//!
//! # The defect this target exists for
//!
//! [#1286](https://github.com/headwater-ai/headwater/issues/1286) split the
//! records about this repository's own tooling onto a process shelf,
//! `docs/process/obligations/`, and HW-PD-0024 rules that a record keeps its
//! identifier when it moves there. So the kind `process_obligation` binds the
//! scheme its product kind binds, `obligation_record_id`, and requires the
//! sections the product kind requires. Before this target, only the golden
//! resolved taxonomy under `engine/crates/resolve/fixtures/` held either
//! clause, and a golden file is re-recorded, not read. An overlay edit that
//! dropped the scheme, or gave the process shelf a count of its own, or
//! dropped the sections, would re-record green.
//!
//! # Why these cases drive the binary
//!
//! The claim is about what `headwater new` writes and what `check --strict`
//! reports, so each case assembles a scratch repository from this
//! repository's own taxonomy and overlay, the way `blank_facet.rs` does, and
//! runs the built binary over it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch repository with this repository's declarations, one product
/// obligation record at `HW-OBL-0300`, and one process obligation record
/// that `headwater new` scaffolded after it.
struct Root {
    at: PathBuf,
    process: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-process-shelves-{}-{label}",
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
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        // The overlay lists four paths outside the corpus root under
        // `regimes.language.ste_house.outside_root` (HW-DR-0084), and a path
        // that matches no file is an error of a strict run, so each gets a
        // stub that obeys the regime.
        for entry in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/SECURITY.md",
            ".github/ISSUE_TEMPLATE/issue.md",
        ] {
            let to = at.join(entry);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the directory is there");
            std::fs::write(to, "# A stub\n\nThis file stands in for the real one.\n")
                .expect("the stub writes");
        }

        // The product record the allocator must count past. It sits at a
        // number no scaffold would reach from an empty process shelf, so a
        // process shelf that counted on its own would mint `HW-OBL-0001`.
        let product = at.join("docs/obligations/0300-a-product-obligation.md");
        std::fs::create_dir_all(product.parent().expect("it has a parent"))
            .expect("the product shelf is there");
        std::fs::write(
            &product,
            "---\nid: HW-OBL-0300\ntitle: A product obligation\nstatus: current\n---\n\n# A product obligation\n",
        )
        .expect("the product record writes");

        let mut root = Root {
            at,
            process: PathBuf::new(),
        };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );

        let made = root.run(&[
            "new",
            "process_obligation",
            "--title",
            "A scratch process obligation",
            "--facet",
            "waiting_on=build",
        ]);
        assert_eq!(
            made.code,
            Some(0),
            "the process record scaffolds\n{}{}",
            made.out,
            made.err
        );
        root.process = std::fs::read_dir(root.at.join("docs/process/obligations"))
            .expect("the scaffolder wrote onto the process shelf")
            .map(|entry| entry.expect("the entry reads").path())
            .find(|path| path.extension().is_some_and(|kind| kind == "md"))
            .expect("the scaffolder wrote a document");
        root
    }

    /// The path the check layer names, relative to the corpus root.
    fn named(&self) -> String {
        let name = self.process.file_name().expect("the document has a name");
        format!("docs/process/obligations/{}", name.to_string_lossy())
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
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart. `headwater check` reports on one and states a
/// run statistic on the other, and a case that merged them reads the wrong one.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
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

/// HW-PD-0024: one scheme across both shelves. The record lands on the
/// process shelf and takes the next `HW-OBL-` past the product record, so a
/// scheme of its own or a count kept per shelf fails here.
#[test]
fn a_process_obligation_takes_the_next_identifier_of_the_product_scheme() {
    let root = Root::new("identifier");
    let text = std::fs::read_to_string(&root.process).expect("the record reads");
    let id = text
        .lines()
        .find_map(|line| line.strip_prefix("id:"))
        .map(str::trim)
        .expect("the scaffolder wrote an id");
    assert_eq!(
        id,
        "HW-OBL-0301",
        "the process record at {} took the wrong identifier",
        root.named()
    );
    assert!(
        root.named().starts_with("docs/process/obligations/0301-"),
        "the file name carries the sequence the shelf layout names: {}",
        root.named()
    );
}

/// The process kind requires the three sections the product kind requires.
/// Deleting `## Discharge` from a scaffolded record is a finding of the
/// sections rule on that path. The assertion reads the report, not the exit
/// status alone, because the seeded records can raise other findings.
#[test]
fn a_process_obligation_without_its_discharge_section_is_a_finding() {
    let root = Root::new("sections");
    let text = std::fs::read_to_string(&root.process).expect("the record reads");
    let at = text
        .find("## Discharge")
        .expect("the scaffolder wrote a Discharge section");
    std::fs::write(&root.process, &text[..at]).expect("the record writes");

    let checked = root.run(&["check", "--strict"]);
    let named = root.named();
    // A finding is a path line, then the rule, which wraps onto a second line
    // that names the section.
    let lines: Vec<&str> = checked.out.lines().collect();
    let found = lines.windows(3).any(|finding| {
        finding[0].trim_start().starts_with(&format!("{named} "))
            && finding[1].contains("section.required.missing")
            && finding[1].contains("`process_obligation`")
            && finding[2].contains("`Discharge`")
    });
    assert!(
        found,
        "no sections finding names Discharge on {named}\n{}{}",
        checked.out, checked.err
    );
}
