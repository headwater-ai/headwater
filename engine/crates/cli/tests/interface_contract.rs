// SPDX-License-Identifier: Apache-2.0
//! The `interface_contract` kind, held against the rule its declaration runs.
//!
//! # What this target is evidence of
//!
//! `.headwater/overlay.yml` declares one kind, one shelf and one identifier
//! scheme, and it declares no rule. The rule that reads the declaration is
//! `section.required.missing`, which ships already. So the thing under test is
//! the declaration rather than the engine, and a case that asserted the rule
//! fires somewhere would assert what
//! `engine/crates/check/fixtures/check/spec/09-contract-missing.md` already
//! asserts about a different taxonomy.
//!
//! [#211](https://github.com/headwater-ai/headwater/issues/211) names the
//! defect that makes such a case worthless: a fixture whose asserted outcome is
//! also the ambient outcome passes whether the rule ran or not. So every case
//! below runs against **this repository's own resolved taxonomy**, and the two
//! decisive cases differ by exactly one heading and reach opposite verdicts.
//!
//! # The four cases, and what each one is for
//!
//! - **The constructor.** `headwater new interface_contract` writes a document
//!   that a strict run then passes. A kind that no document instantiates is
//!   invisible to every check, so the constructor is the only reading that
//!   reaches a defect in a kind with no corpus.
//! - **The rule against its negation.** Two contracts, eight headings and
//!   seven, in one corpus. One finding, naming the heading and the file, and
//!   the complete document in no finding at all. Then the incomplete document
//!   is removed and the same run reports nothing, so the verdict follows the
//!   heading rather than the corpus.
//! - **What the contract does not read.** The eight headings in reverse order
//!   pass, and the eight headings with `TODO` under every one of them pass. The
//!   overlay says so in prose and this is where it is measured. A kind whose
//!   whole purpose is a description must not imply that a check read the
//!   description.
//! - **The `governs` edge.** A contract reaches its crate by the edge the base
//!   package already declares, and
//!   [Q29](../../../../docs/decisions/0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md)
//!   rules that the edge binds on existence alone. One target that is on the
//!   tree and one that is not, in one document, and only the second is
//!   reported.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The eight headings the overlay declares, in the order it declares them.
///
/// The list is here rather than read back out of the lock, because a case that
/// derived it from the declaration would follow any change to the declaration
/// and assert nothing about it. Seven are the man-pages(7) headings the
/// evaluation named, and `Preconditions` is the one the man tradition has no
/// slot for.
const SECTIONS: [&str; 8] = [
    "Synopsis",
    "Description",
    "Preconditions",
    "Options",
    "Exit status",
    "Environment",
    "Files",
    "See also",
];

/// The one heading an incomplete contract leaves out.
///
/// It is the heading the evaluation says every manual page in sections 1 and 8
/// is expected to carry, and the one this engine's verbs are least described
/// by.
const OMITTED: &str = "Exit status";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch corpus that holds this repository's taxonomy and none of its prose.
///
/// The corpus root is `docs`, and `docs/taxonomies/**` is excluded, so a scratch
/// that copies the taxonomy sources alone has a corpus of exactly the documents
/// a case writes. That is what makes a finding count assertable: 523 findings
/// stand over the real corpus, and a case that ran there could only ever assert
/// a difference.
struct Root {
    at: PathBuf,
}

impl Root {
    /// `label` names the case rather than the target, because cargo runs the
    /// cases of one target as threads of one process and a directory keyed on
    /// the process identifier alone is one that a second case removes while the
    /// first reads it.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-interface-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(&repository.join("packages"), &at.join("packages"));
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

        let root = Root { at };
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

    /// One contract on the shelf, with the headings a case names.
    ///
    /// Everything except the heading list is held constant, so two documents
    /// that differ in the list differ in nothing else a rule reads. The return
    /// value is the path as the report writes it.
    fn contract(&self, slug: &str, sections: &[&str], under: &str) -> String {
        let relative = format!("docs/interfaces/{slug}.md");
        let path = self.at.join(&relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf directory is there");

        let mut text = format!(
            "---\n\
             id: HW-IFACE-{slug}\n\
             status: draft\n\
             status_since: 2026-08-16\n\
             summary: \"What this verb takes, what it prints, and what its exit status means.\"\n\
             last_verified: 2026-08-16\n\
             title: \"{slug}\"\n\
             ---\n\n\
             # {slug}\n"
        );
        for section in sections {
            text.push_str(&format!("\n## {section}\n\n{under}\n"));
        }
        std::fs::write(&path, text).expect("the contract writes");
        relative
    }

    fn remove(&self, relative: &str) {
        std::fs::remove_file(self.at.join(relative)).expect("the contract is removed");
    }

    /// A file on the source tree, for the end of a `governs` edge that binds.
    fn source_file(&self, relative: &str) {
        let path = self.at.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the source directory is there");
        std::fs::write(&path, "// a file the anchor resolver finds\n").expect("the source writes");
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

/// The two streams held apart. `headwater check` writes its report to standard
/// output and a run statistic to standard error, so a case that merged them
/// reads a correct report as a broken one.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Ran {
    /// Every finding line of this rule, which is the form that carries the
    /// obligation. The coverage account above the findings names the rule too,
    /// and a case that counted the bare name would count that line as a
    /// finding.
    fn section_findings(&self) -> Vec<&str> {
        self.out
            .lines()
            .filter(|line| line.contains("section.required.missing (OB-SECT-1)"))
            .collect()
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

/// A kind with no document of it is invisible to every check, so the
/// constructor is the audit.
///
/// The recorded case is `headwater new specification`, which surfaced in one
/// command that the base package's `specification` kind is an endpoint of four
/// relations and mints under no identifier scheme
/// ([HW-OBL-0107](../../../../docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md)).
/// `identifier.unusable` will never report that, because it generates zero
/// instances over a kind with zero documents.
#[test]
fn the_constructor_writes_a_contract_that_a_strict_run_passes() {
    let root = Root::new("constructor");

    let wrote = root.run(&["new", "interface_contract", "--title", "headwater check"]);
    assert_eq!(
        wrote.code,
        Some(0),
        "the kind can be constructed\n{}{}",
        wrote.out,
        wrote.err
    );
    assert!(
        wrote.out.contains("wrote docs/interfaces/headwater-check.md"),
        "the shelf and the layout name the file:\n{}",
        wrote.out
    );

    // Every heading, in the declared order. The scaffolder writes them from the
    // contract, so this is also the reading that says the contract reached it.
    let text = std::fs::read_to_string(root.at.join("docs/interfaces/headwater-check.md"))
        .expect("the document reads");
    let written: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .collect();
    assert_eq!(written, SECTIONS, "the document carries the contract");

    // The identifier the scheme mints, which is the half of a declaration that
    // `headwater new` refuses a kind for when it is missing.
    assert!(
        text.contains("id: HW-IFACE-headwater-check"),
        "the scheme mints an identifier:\n{text}"
    );

    let checked = root.run(&["check", "--strict"]);
    assert_eq!(
        checked.code,
        Some(0),
        "a scaffolded contract passes a strict run\n{}{}",
        checked.out,
        checked.err
    );
    assert!(
        checked.out.contains("  0 findings"),
        "and it passes with nothing reported:\n{}",
        checked.out
    );

    // The reader-facing surface of the declaration, which is what an author
    // meets before writing anything.
    let explained = root.run(&["explain", "docs/interfaces/headwater-check.md"]);
    assert!(
        explained.out.contains("kind interface_contract")
            && explained
                .out
                .contains("requires the facets status, status_since, summary, last_verified, title")
            && explained.out.contains(
                "requires the sections Synopsis, Description, Preconditions, Options, \
                 Exit status, Environment, Files, See also"
            ),
        "`explain` prints the kind, its facets and its sections:\n{}",
        explained.out
    );
}

/// The rule against its negation, in one corpus, one heading apart.
///
/// Both documents are written by one function from one heading list, so the
/// only input that differs between them is the one the rule reads. The ambient
/// outcome of this corpus is no finding, and the asserted outcome is one
/// finding, so a rule that did not run could not produce it.
#[test]
fn a_contract_missing_one_heading_is_reported_by_its_name_and_its_file() {
    let root = Root::new("missing-one");

    let complete = root.contract("headwater-route", &SECTIONS, "One sentence of description.");
    let short: Vec<&str> = SECTIONS
        .iter()
        .copied()
        .filter(|section| *section != OMITTED)
        .collect();
    assert_eq!(short.len(), SECTIONS.len() - 1, "one heading is left out");
    let incomplete = root.contract("headwater-explain", &short, "One sentence of description.");

    let checked = root.run(&["check"]);
    assert_eq!(
        checked.section_findings().len(),
        1,
        "one finding, and it is about the document that is short one heading:\n{}",
        checked.out
    );
    assert!(
        checked.out.contains(&format!(
            "  {incomplete} error\n    \
             section.required.missing (OB-SECT-1): `interface_contract` requires the section \
             `{OMITTED}`, and no heading of this document says so\n    \
             fix: add a `{OMITTED}` heading to {incomplete}, with the content the kind is for"
        )),
        "the finding names the rule, the kind, the heading and the file:\n{}",
        checked.out
    );
    assert!(
        !checked
            .section_findings()
            .iter()
            .any(|line| line.contains(&complete)),
        "the complete contract is in no finding:\n{}",
        checked.out
    );

    // An error, so it stops a commit rather than being reported past.
    let strict = root.run(&["check", "--strict"]);
    assert_eq!(
        strict.code,
        Some(1),
        "a missing section fails a strict run\n{}{}",
        strict.out,
        strict.err
    );

    // The negation. The same corpus without the short document reports nothing,
    // so the verdict above followed the heading and not the run.
    root.remove(&incomplete);
    let again = root.run(&["check", "--strict"]);
    assert_eq!(
        again.code,
        Some(0),
        "the complete contract alone passes\n{}{}",
        again.out,
        again.err
    );
    assert!(
        again.section_findings().is_empty() && again.out.contains("  0 findings"),
        "and it passes with nothing reported:\n{}",
        again.out
    );
}

/// What the contract does not read, measured rather than asserted in prose.
///
/// `section.required.missing` matches the text of a heading at any level,
/// case-insensitively, and reads no word under one. Both readings below are
/// therefore silent, and the overlay says so where an author meets the
/// declaration. A later change that made either one a finding is a change to
/// what this kind promises, and it fails here first.
///
/// **The third document is why this case is evidence of anything.** No finding
/// is also what a corpus reports when the rule never ran, so a case that held
/// only the two silent documents would pass over a taxonomy that declared no
/// section contract at all. That is the defect
/// [#211](https://github.com/headwater-ai/headwater/issues/211) records. The
/// control is short one heading, so the ambient outcome here is one finding,
/// and the assertion is that it is the only one.
#[test]
fn the_contract_reads_a_heading_and_never_its_order_or_its_content() {
    let root = Root::new("what-it-does-not-read");

    let reversed: Vec<&str> = SECTIONS.iter().copied().rev().collect();
    let out_of_order = root.contract("headwater-generate", &reversed, "One sentence.");
    let hollow = root.contract("headwater-import", &SECTIONS, "TODO write this section.");

    let short: Vec<&str> = SECTIONS
        .iter()
        .copied()
        .filter(|section| *section != OMITTED)
        .collect();
    let control = root.contract("headwater-export", &short, "One sentence.");

    let checked = root.run(&["check"]);
    let findings = checked.section_findings();
    assert_eq!(
        findings.len(),
        1,
        "the rule ran, and it reported the control and nothing else:\n{}",
        checked.out
    );
    assert!(
        checked.out.contains(&format!("  {control} error")),
        "the one finding is about the document that is short a heading:\n{}",
        checked.out
    );
    assert!(
        !checked.out.contains(&format!("  {out_of_order} error"))
            && !checked.out.contains(&format!("  {hollow} error")),
        "the order of the headings and the words under them reach no rule:\n{}",
        checked.out
    );
}

/// The `governs` edge, and the concession Q29 made about it.
///
/// The base package declares `governs` from `governed_document` to `code_path`,
/// and this kind is a `governed_document`, so nothing about the relation is
/// declared for it and nothing needs to be. The edge binds when the path exists
/// and reads no byte at it. It declares no inverse and no reciprocal, and it
/// cannot: the far end is an anchor rather than a node, so there is no document
/// there to write the other half onto.
#[test]
fn a_governs_edge_binds_on_existence_and_reports_a_path_that_is_not_there() {
    let root = Root::new("governs");
    root.source_file("engine/crates/route/src/lib.rs");

    let relative = "docs/interfaces/headwater-route.md";
    let path = root.at.join(relative);
    std::fs::create_dir_all(path.parent().expect("it has a parent"))
        .expect("the shelf directory is there");
    let mut text = String::from(
        "---\n\
         id: HW-IFACE-headwater-route\n\
         status: draft\n\
         status_since: 2026-08-16\n\
         summary: \"What this verb takes, what it prints, and what its exit status means.\"\n\
         last_verified: 2026-08-16\n\
         title: \"headwater route\"\n\
         relations:\n  \
         governs:\n    \
         - engine/crates/route/src/lib.rs\n    \
         - engine/crates/route/src/nowhere.rs\n\
         ---\n\n\
         # headwater route\n",
    );
    for section in SECTIONS {
        text.push_str(&format!("\n## {section}\n\nOne sentence of description.\n"));
    }
    std::fs::write(&path, text).expect("the contract writes");

    let checked = root.run(&["check"]);

    // The endpoint rule is silent, which is the reading that says the abstract
    // source kind reached this one. A rule that resolved `from: [...]` by name
    // alone would report both halves here.
    assert!(
        !checked.out.contains("relation.endpoint.not_permitted (OB-REL-2)"),
        "`governs` admits an `interface_contract` as its source:\n{}",
        checked.out
    );

    // No reciprocal half is owed, and none is reported missing.
    assert!(
        !checked.out.contains("relation.reciprocity.missing (OB-REL-1)"),
        "`governs` declares no reciprocal, so neither end owes one:\n{}",
        checked.out
    );

    // One target is on the tree and one is not, and only the second is
    // reported. That is the whole of what the edge asserts: existence.
    let unresolved: Vec<&str> = checked
        .out
        .lines()
        .filter(|line| line.contains("relation.target.unresolved (OB-REL-4)"))
        .collect();
    assert_eq!(
        unresolved.len(),
        1,
        "the path that is on the tree binds and the other does not:\n{}",
        checked.out
    );
    assert!(
        unresolved[0].contains("`HW-IFACE-headwater-route` declares `governs: \
                                engine/crates/route/src/nowhere.rs`"),
        "the finding names the edge that resolved to nothing:\n{}",
        unresolved[0]
    );

    // And the edge that bound is in the graph, which is where #256 and #257
    // read it from.
    let explained = root.run(&["explain", relative]);
    assert!(
        explained
            .out
            .contains("to engine/crates/route/src/lib.rs governs"),
        "the bound edge is on the document:\n{}",
        explained.out
    );
}
