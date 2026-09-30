// SPDX-License-Identifier: Apache-2.0
//! What `check --strict` makes of a warrant outside spec 3's closed set.
//!
//! [#1438](https://github.com/headwater-ai/headwater/issues/1438) measured a
//! document at `warrant: acepted` passing a strict run at exit 0, and a pointer
//! to it served as vouched. The claim is about an exit status and a report,
//! which are the caller's, so the case drives the built binary over a scratch
//! repository with one scaffolded document, as `blank_facet.rs` does.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch repository holding one scaffolded decision record.
struct Root {
    at: PathBuf,
    document: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-warrant-value-{}-{label}",
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

        let mut root = Root {
            at,
            document: PathBuf::new(),
        };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );

        // Scaffolded rather than written out here, so a taxonomy that adds a
        // required facet fails this fixture rather than typing the document as
        // nothing and raising the finding for another reason.
        let made = root.run(&["new", "decision", "--title", "A scratch ruling"]);
        assert_eq!(
            made.code,
            Some(0),
            "the corpus document scaffolds\n{}{}",
            made.out,
            made.err
        );
        root.document = std::fs::read_dir(root.at.join("docs/decisions"))
            .expect("the decisions shelf is there")
            .map(|entry| entry.expect("the entry reads").path())
            .find(|path| path.extension().is_some_and(|kind| kind == "md"))
            .expect("the scaffolder wrote a document");
        root
    }

    /// The path the check layer names, relative to the corpus root.
    fn named(&self) -> String {
        let name = self.document.file_name().expect("the document has a name");
        format!("docs/decisions/{}", name.to_string_lossy())
    }

    /// Write a provenance block carrying this warrant into the front matter.
    /// The scaffolder writes none, because no taxonomy declares one.
    fn warrant(&self, value: &str) {
        let text = std::fs::read_to_string(&self.document).expect("the document reads");
        let body = text.strip_prefix("---\n").expect("the document opens front matter");
        let close = body.find("\n---\n").expect("the front matter closes");
        let written = format!(
            "---\n{}\nprovenance:\n  warrant: {value}\n  agency: human\n  accepted_by: a.person{}",
            &body[..close],
            &body[close..]
        );
        std::fs::write(&self.document, written).expect("the document writes");
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

/// The end-to-end witness: one misspelling stops a strict run, and the report
/// names the rule and the value at the document.
#[test]
fn a_misspelled_warrant_stops_a_strict_run() {
    let root = Root::new("misspelled");
    root.warrant("acepted");
    let ran = root.run(&["check", "--strict", "--no-cache"]);
    assert_eq!(
        ran.code,
        Some(1),
        "`warrant: acepted` passed a strict run\n{}{}",
        ran.out,
        ran.err
    );
    let named = root.named();
    let reported = ran
        .out
        .split(&named)
        .skip(1)
        .any(|after| after.contains("warrant.value.not_permitted") && after.contains("`acepted`"));
    assert!(
        reported,
        "no `warrant.value.not_permitted` finding on {named} naming the value\n{}",
        ran.out
    );
}

/// The negative arm: the same block at `accepted` raises neither rule, so the
/// finding above is the value and nothing else about the block.
#[test]
fn an_accepted_warrant_with_its_acceptor_is_not_reported() {
    let root = Root::new("control");
    root.warrant("accepted");
    let ran = root.run(&["check", "--strict", "--no-cache"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the accepted corpus does not pass a strict run\n{}{}",
        ran.out,
        ran.err
    );
    for rule in ["warrant.value.not_permitted", "warrant.acceptance.unpaired"] {
        assert!(
            !ran.out.contains(&format!("{rule}:")),
            "{rule} reported an accepted document\n{}",
            ran.out
        );
    }
}
