// SPDX-License-Identifier: Apache-2.0
//! What `check --strict` makes of a document whose folder and front matter
//! state two different values for one facet.
//!
//! # The defect this target exists for
//!
//! An adopter keeps one folder per bounded context and declared the shelf
//! `docs/contexts/{context}/contracts/**` (#2101). Before the binding existed,
//! `{context}` was a literal segment, so no document was on that shelf, and a
//! contract under `billing/` that declared `context: shipping` passed a strict
//! run. The owner's ruling on the issue makes the disagreement a finding, and
//! a document on the shelf with no value a finding too: the engine does not
//! fill the value in from the folder. The rule `shelf.bound_facet.disagrees`
//! is that finding.
//!
//! # Why these cases drive the binary
//!
//! The claim is about an exit status and a report, on the terms
//! `discriminator_unresolved.rs` states. Each case assembles a scratch
//! repository from this repository's taxonomy, adds a facet and two bound
//! shelves to the overlay, writes documents onto a shelf, and runs the built
//! binary over the result.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "shelf.bound_facet.disagrees";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The declarations each case adds to this repository's overlay: a `stable`
/// facet `context` that no kind requires, a shelf that binds it, and a shelf
/// that binds `title`, which the kind `review_record` requires, and a more
/// specific shelf under the bound one that binds nothing. A second facet
/// `team` and a shelf that binds it and `title` together hold the case where
/// one document breaks two bindings in two ways. Every shelf
/// are heterogeneous, because `review_record` requires `doc_type` and a
/// homogeneous shelf refuses it as a restatement; a heterogeneous shelf is
/// also the one where no shelf is "the" shelf of a kind.
const OVERLAY: &str = "
  facets.context:
    type: string
    required: false
    volatility: stable
  shelves.contracts:
    path: docs/contexts/{context}/contracts/**
    homogeneous: false
    discriminator: doc_type
    kinds: [review_record]
  shelves.special:
    path: docs/contexts/billing/contracts/special/**
    homogeneous: false
    discriminator: doc_type
    kinds: [review_record]
  shelves.titled:
    path: docs/titled/{title}/**
    homogeneous: false
    discriminator: doc_type
    kinds: [review_record]
  facets.team:
    type: string
    required: false
    volatility: stable
  shelves.two:
    path: docs/two/{team}/{title}/**
    homogeneous: false
    discriminator: doc_type
    kinds: [review_record]
";

/// A scratch repository holding this repository's taxonomy, with the three
/// shelves added, and nothing else.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-bound-facet-{}-{label}",
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
        let declarations = at.join(".headwater");
        std::fs::create_dir_all(&declarations).expect("the declaration directory is there");
        std::fs::copy(
            repository.join(".headwater/taxonomy.yml"),
            declarations.join("taxonomy.yml"),
        )
        .expect("the declaration copies");
        let overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            overlay.contains("\nadd:\n"),
            "the overlay has an `add` block"
        );
        std::fs::write(
            declarations.join("overlay.yml"),
            overlay.replacen("\nadd:\n", &format!("\nadd:\n{OVERLAY}"), 1),
        )
        .expect("the overlay writes");
        // The stubs `discriminator_unresolved.rs` writes, for the reason it
        // gives: the overlay names these paths outside the corpus root. The
        // rest stand in for one entry under each governed scope this
        // repository declares, so that `relation.scope.unmatched` is quiet and
        // a strict run over agreeing documents exits 0.
        for entry in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/SECURITY.md",
            ".github/ISSUE_TEMPLATE/issue.md",
            ".claude/agents/stub.md",
            ".claude/hooks/stub.md",
            ".claude/skills/stub.md",
            ".githooks/stub.md",
            "engine/crates/stub/src/lib.rs",
            "engine/crates/cli/harness/stub.md",
            "site/stub.md",
            "tools/stub.md",
        ] {
            let to = at.join(entry);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the directory is there");
            std::fs::write(to, "# A stub\n\nThis file stands in for the real one.\n")
                .expect("the stub writes");
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

    fn write(&self, path: &str, text: &str) {
        let to = self.at.join(path);
        std::fs::create_dir_all(to.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        std::fs::write(to, text).expect("the document writes");
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

    fn strict(&self) -> Ran {
        self.run(&["check", "--strict", "--no-cache"])
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart, on `discriminator_unresolved.rs`'s terms.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Ran {
    /// Every finding of one rule, as the text report prints it: the location
    /// line, then the rule, the message and the fix joined by one space.
    fn of(&self, rule: &str) -> Vec<String> {
        let lines: Vec<&str> = self.out.lines().map(str::trim).collect();
        // `<rule>: ` where no obligation is named, `<rule> (<obligation>): `
        // where one is.
        let opens = [format!("{rule}: "), format!("{rule} (")];
        (1..lines.len())
            .filter(|at| opens.iter().any(|open| lines[*at].starts_with(open)))
            .map(|at| {
                let body: Vec<&str> = lines[at..]
                    .iter()
                    .copied()
                    .take_while(|line| !line.is_empty())
                    .collect();
                format!("{}\n{}", lines[at - 1], body.join(" "))
            })
            .collect()
    }

    fn of_rule(&self) -> Vec<String> {
        self.of(RULE)
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

const A: &str = "docs/contexts/billing/contracts/a.md";
const B: &str = "docs/contexts/billing/contracts/b.md";
const C: &str = "docs/contexts/billing/contracts/c.md";
const TITLED: &str = "docs/titled/probe/a.md";

/// A complete review record whose front matter opens with the lines given,
/// so the first of them is on line 2. `slug` makes its identifier unique.
fn record(slug: &str, lines: &str) -> String {
    format!(
        "---\n{lines}doc_type: review_record\nid: HW-REV-{slug}\ntitle: A contract {slug}\nsummary: One contract of the billing context.\nstatus: draft\nstatus_since: 2026-11-26\nlast_verified: 2026-11-26\n---\n\n# A contract\n\nThis is a contract.\n"
    )
}

/// The decisive case: one document agrees with its folder and one does not.
/// The disagreeing one is an error at its facet's line, naming both values,
/// and the agreeing one raises nothing.
#[test]
fn a_value_that_disagrees_with_its_folder_fails_a_strict_run() {
    let root = Root::new("disagrees");
    root.write(A, &record("a", "context: billing\n"));
    root.write(B, &record("b", "context: shipping\n"));
    let ran = root.strict();
    assert_eq!(
        ran.code,
        Some(1),
        "a strict run passed a document its folder contradicts\n{}{}",
        ran.out,
        ran.err
    );
    let found = ran.of_rule();
    assert_eq!(found.len(), 1, "one finding of {RULE}\n{}", ran.out);
    assert!(
        found[0].starts_with(&format!("{B}:2:")),
        "the finding anchors at the facet's line of b.md\n{}",
        ran.out
    );
    let head = found[0].lines().next().expect("the finding has a location");
    assert!(head.ends_with("error"), "the finding is an error\n{head}");
    for named in ["`billing`", "`shipping`", "`context`", "`contracts`"] {
        assert!(found[0].contains(named), "it names {named}\n{}", found[0]);
    }
    assert!(
        !ran.out.contains(&format!("{A}:")),
        "nothing is reported on a.md\n{}",
        ran.out
    );
}

/// The control: an agreeing document alone raises nothing from this rule, and
/// the strict run passes, so the decisive case's exit is this rule's.
#[test]
fn a_value_that_agrees_with_its_folder_raises_nothing() {
    let root = Root::new("agrees");
    root.write(A, &record("a", "context: billing\n"));
    let ran = root.strict();
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
}

/// Owner ruling 2 on #2101: no value is a finding, and the engine does not
/// read the value off the folder in its place.
#[test]
fn an_absent_value_is_the_same_finding() {
    let root = Root::new("absent");
    root.write(A, &record("a", "context: billing\n"));
    root.write(C, &record("c", ""));
    let ran = root.strict();
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    let found = ran.of_rule();
    assert_eq!(found.len(), 1, "one finding of {RULE}\n{}", ran.out);
    assert!(found[0].starts_with(&format!("{C}:1:")), "{}", ran.out);
    assert!(found[0].contains("`billing`"), "{}", found[0]);
}

/// A facet the kind already requires has its absence reported by
/// `facet.required.missing`, and one fact gives one finding.
#[test]
fn an_absent_required_value_is_reported_once_by_the_required_rule() {
    let root = Root::new("required");
    root.write(
        TITLED,
        &record("t", "").replace("title: A contract t\n", ""),
    );
    let ran = root.strict();
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
    let required: Vec<String> = ran
        .of("facet.required.missing")
        .into_iter()
        .filter(|found| found.starts_with(TITLED))
        .collect();
    assert_eq!(required.len(), 1, "{}", ran.out);
}

/// The kind instantiates on every shelf, and a shelf that binds nothing asks
/// for no value: the same record with no `context` on `reviews` raises
/// nothing from this rule.
#[test]
fn a_record_on_a_shelf_that_binds_nothing_raises_nothing() {
    let root = Root::new("unbound");
    root.write("docs/reviews/r.md", &record("r", ""));
    let ran = root.strict();
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
}

/// A more specific shelf that binds nothing wins a path under a bound one, so
/// the folder of that document states no value and the rule asks for none.
#[test]
fn a_more_specific_shelf_that_binds_nothing_wins_and_raises_nothing() {
    let root = Root::new("specific");
    root.write(
        "docs/contexts/billing/contracts/special/s.md",
        &record("s", ""),
    );
    let ran = root.strict();
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
}

/// The same required facet with a value that disagrees is this rule's.
#[test]
fn a_required_value_that_disagrees_is_this_rule() {
    let root = Root::new("required-disagrees");
    root.write(
        TITLED,
        &record("t", "title: other\n").replace("title: A contract t\n", ""),
    );
    let ran = root.strict();
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    let found = ran.of_rule();
    assert_eq!(found.len(), 1, "{}", ran.out);
    assert!(found[0].starts_with(&format!("{TITLED}:2:")), "{}", ran.out);
}

/// One document on a shelf that binds two facets: `team` disagrees with its
/// folder and `title`, which the kind requires, is absent. The absence is
/// `facet.required.missing`'s, and the disagreement is still this rule's: the
/// required facet does not turn the whole document over to the other rule.
#[test]
fn a_disagreement_beside_an_absent_required_value_is_still_this_rule() {
    const D: &str = "docs/two/core/x/d.md";
    let root = Root::new("two");
    root.write(
        D,
        &record("d", "team: ops\n").replace("title: A contract d\n", ""),
    );
    let ran = root.strict();
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    let found = ran.of_rule();
    assert_eq!(found.len(), 1, "one finding of {RULE}\n{}", ran.out);
    assert!(found[0].starts_with(&format!("{D}:2:")), "{}", ran.out);
    for named in ["`team`", "`core`", "`ops`"] {
        assert!(found[0].contains(named), "it names {named}\n{}", found[0]);
    }
    let required: Vec<String> = ran
        .of("facet.required.missing")
        .into_iter()
        .filter(|found| found.starts_with(D))
        .collect();
    assert_eq!(required.len(), 1, "{}", ran.out);
}

/// A value that is no folder name, a null or a sequence, is the same finding,
/// and its remedy is the value the folder gives: there is no folder for a
/// null or a list to move the document under.
#[test]
fn a_value_that_names_no_folder_has_one_remedy() {
    let root = Root::new("no-folder");
    root.write(A, &record("a", "context: ~\n"));
    root.write(B, &record("b", "context: [billing, shipping]\n"));
    let ran = root.strict();
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    let found = ran.of_rule();
    assert_eq!(found.len(), 2, "{}", ran.out);
    for one in &found {
        assert!(one.contains("set `context: billing`"), "{one}");
        assert!(!one.contains("move the document"), "{one}");
        assert!(!one.contains("`~`"), "{one}");
        assert!(!one.contains("`a sequence`"), "{one}");
    }
    let null = found
        .iter()
        .find(|one| one.starts_with(A))
        .expect("a.md has a finding");
    assert!(null.contains("a null `context`"), "{null}");
    let list = found
        .iter()
        .find(|one| one.starts_with(B))
        .expect("b.md has a finding");
    assert!(list.contains("`context` as a sequence"), "{list}");
}

const D: &str = "docs/contexts/shipping/contracts/d.md";

/// Two contexts, each with one document that its folder contradicts: one
/// `shelf.bound_facet.disagrees` error under `billing/` and one under
/// `shipping/`, before a case changes either.
fn two_contexts(label: &str) -> Root {
    let root = Root::new(label);
    root.write(B, &record("b", "context: shipping\n"));
    root.write(D, &record("d", "context: billing\n"));
    root
}

/// The decisive case of clause 4 of #2101: `check --path` reports one context
/// alone. The finding under `billing/` is printed and fails the run, the one
/// under `shipping/` is held back, and the report names the pattern and the
/// count it held back.
#[test]
fn a_path_filter_reports_one_context_alone() {
    let root = two_contexts("path-one");
    let ran = root.run(&[
        "check",
        "--strict",
        "--no-cache",
        "--path",
        "docs/contexts/billing/**",
    ]);
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    let found = ran.of_rule();
    assert_eq!(found.len(), 1, "one finding of {RULE}\n{}", ran.out);
    assert!(found[0].starts_with(&format!("{B}:")), "{}", ran.out);
    assert!(
        !ran.out.contains(&format!("{D}:")),
        "nothing is reported on d.md\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("filter: --path docs/contexts/billing/**"),
        "the report names the pattern\n{}",
        ran.out
    );
    // The count held back is every finding of the unfiltered run that the
    // report leaves out: the error on d.md and the advisory findings this
    // repository's own doctrine pages carry.
    let whole = root.run(&["check", "--no-cache"]);
    assert!(
        whole.out.contains(&format!("{D}:")),
        "d.md has a finding to hold back\n{}",
        whole.out
    );
    assert_eq!(
        held_back(&ran),
        total(&whole) - total(&ran),
        "the report says what it held back\n{}",
        ran.out
    );
}

/// The count on the `held back:` line of a filtered text report.
fn held_back(ran: &Ran) -> usize {
    ran.out
        .lines()
        .find_map(|line| line.trim().strip_prefix("held back: "))
        .and_then(|rest| rest.split(' ').next())
        .and_then(|count| count.parse().ok())
        .unwrap_or_else(|| panic!("no held-back line\n{}", ran.out))
}

/// The run's total, the first line of the shape `<n> findings`.
fn total(ran: &Ran) -> usize {
    ran.out
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_suffix(" findings")
                .or_else(|| line.strip_suffix(" finding"))
                .and_then(|count| count.parse().ok())
        })
        .unwrap_or_else(|| panic!("no total\n{}", ran.out))
}

/// The half that proves the strict exit is narrowed and not only the print:
/// the one error left is outside the filter, so the strict run passes, and
/// the report counts it among what it held back. The control is the same
/// corpus with no filter, which fails.
#[test]
fn a_path_filter_narrows_the_strict_exit() {
    let root = two_contexts("path-exit");
    root.write(D, &record("d", "context: shipping\n"));
    let whole = root.strict();
    assert_eq!(whole.code, Some(1), "{}{}", whole.out, whole.err);
    let ran = root.run(&[
        "check",
        "--strict",
        "--no-cache",
        "--path",
        "docs/contexts/shipping/**",
    ]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
    assert_eq!(whole.of_rule().len(), 1, "{}", whole.out);
    assert_eq!(held_back(&ran), total(&whole) - total(&ran), "{}", ran.out);
    assert!(held_back(&ran) >= 1, "{}", ran.out);
}

/// The JSON report carries the filter, and its `findings` holds only what the
/// filter admits. A run with no `--path` writes no `filter` member.
#[test]
fn a_path_filter_is_in_the_json_report() {
    let root = two_contexts("path-json");
    let ran = root.run(&[
        "check",
        "--no-cache",
        "--format",
        "json",
        "--path",
        "docs/contexts/billing/**",
    ]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    let parsed = headwater_yaml::load(&ran.out).expect("the report parses");
    let filter = member(&parsed.value, "filter").expect("the report has a filter");
    assert_eq!(
        texts(member(filter, "patterns")),
        ["docs/contexts/billing/**"],
        "{}",
        ran.out
    );
    assert!(texts(member(filter, "unmatched")).is_empty(), "{}", ran.out);
    let paths: Vec<String> = member(&parsed.value, "findings")
        .and_then(headwater_yaml::Value::as_seq)
        .expect("findings is a list")
        .iter()
        .map(|finding| text(member(&finding.value, "path")))
        .collect();
    assert!(paths.iter().any(|path| path == B), "{}", ran.out);
    assert!(
        paths
            .iter()
            .all(|path| path.starts_with("docs/contexts/billing/")),
        "{paths:?}"
    );
    let whole = root.run(&["check", "--no-cache", "--format", "json"]);
    let whole = headwater_yaml::load(&whole.out).expect("the report parses");
    assert!(
        member(&whole.value, "filter").is_none(),
        "no filter member without --path"
    );
    let every = member(&whole.value, "findings")
        .and_then(headwater_yaml::Value::as_seq)
        .expect("findings is a list")
        .len();
    assert_eq!(
        text(member(filter, "held_back")),
        (every - paths.len()).to_string(),
        "{}",
        ran.out
    );
}

fn member<'a>(value: &'a headwater_yaml::Value, key: &str) -> Option<&'a headwater_yaml::Value> {
    value
        .as_map()
        .and_then(|map| map.get(key))
        .map(|spanned| &spanned.value)
}

fn text(value: Option<&headwater_yaml::Value>) -> String {
    value
        .and_then(headwater_yaml::Value::as_scalar)
        .map(headwater_yaml::core_schema::as_str)
        .expect("a scalar")
        .to_string()
}

fn texts(value: Option<&headwater_yaml::Value>) -> Vec<String> {
    value
        .and_then(headwater_yaml::Value::as_seq)
        .expect("a list")
        .iter()
        .map(|one| text(Some(&one.value)))
        .collect()
}

/// A pattern that matches no file and no finding fails a strict run and is
/// named, so a mistyped `--path` in a CI step cannot hold back every finding
/// and pass. The control is a filter that matches, over the same clean
/// corpus, which passes.
#[test]
fn a_path_filter_that_matches_nothing_fails_a_strict_run() {
    let root = Root::new("path-unmatched");
    root.write(A, &record("a", "context: billing\n"));
    let matched = root.run(&[
        "check",
        "--strict",
        "--no-cache",
        "--path",
        "docs/contexts/billing/**",
    ]);
    assert_eq!(matched.code, Some(0), "{}{}", matched.out, matched.err);
    let ran = root.run(&[
        "check",
        "--strict",
        "--no-cache",
        "--path",
        "docs/contexts/billing/**",
        "--path",
        "docs/contexts/nowhere/**",
    ]);
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    assert!(
        ran.out
            .contains("unmatched: docs/contexts/nowhere/** matches no file and no finding"),
        "{}",
        ran.out
    );
    assert!(
        !ran.out
            .contains("unmatched: docs/contexts/billing/**"),
        "a pattern that matches is not named unmatched\n{}",
        ran.out
    );
    let loose = root.run(&["check", "--no-cache", "--path", "docs/contexts/nowhere/**"]);
    assert_eq!(
        loose.code,
        Some(0),
        "an advisory run does not fail\n{}",
        loose.err
    );
}


/// `--fix --path` writes the patch of a finding the filter admits and leaves
/// the patch of a finding it holds back unwritten. The patch is the house
/// regime's spelling fix, in two doctrine pages the scratch repository copies:
/// the regime reaches a review record on no shelf here, and it reaches these.
#[test]
fn a_path_filter_narrows_what_fix_writes() {
    const INSIDE: &str = "docs/taxonomies/diataxis/doctrine.md";
    const OUTSIDE: &str = "docs/taxonomies/design-spec/doctrine.md";
    const BRITISH: &str = "The organisation is the reader.";
    const AMERICAN: &str = "The organization is the reader.";
    let root = Root::new("path-fix");
    let read = |path: &str| std::fs::read_to_string(root.at.join(path)).expect("it reads");
    for path in [INSIDE, OUTSIDE] {
        let text = read(path);
        assert!(!text.contains(BRITISH) && !text.contains(AMERICAN));
        root.write(path, &format!("{}\n\n{BRITISH}\n", text.trim_end()));
    }
    let fixed = root.run(&[
        "check",
        "--fix",
        "--no-cache",
        "--path",
        "docs/taxonomies/diataxis/**",
    ]);
    assert_eq!(fixed.code, Some(0), "{}{}", fixed.out, fixed.err);
    assert!(
        read(INSIDE).contains(AMERICAN),
        "the admitted finding's patch was not written\n{}",
        fixed.err
    );
    assert!(
        read(OUTSIDE).contains(BRITISH),
        "the held-back finding's patch was written\n{}",
        fixed.err
    );
    // The control: with no filter the same run writes the other, so the page
    // outside the filter had a patch to hold back.
    let all = root.run(&["check", "--fix", "--no-cache"]);
    assert_eq!(all.code, Some(0), "{}{}", all.out, all.err);
    assert!(read(OUTSIDE).contains(AMERICAN), "{}", all.err);
}
