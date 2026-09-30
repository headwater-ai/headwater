// SPDX-License-Identifier: Apache-2.0
//! The defect of #1213: a link whose text names one document and whose path
//! reaches another.
//!
//! `[HW-DR-0089](0090-….md)` resolves, so `link.path.unresolved` is silent,
//! and it carries no fragment, so `link.fragment.unresolved` is silent too.
//! Before `link.identifier.mismatch` a strict run passed over it.
//!
//! # How the run is made green first
//!
//! `renamed_target.rs`'s instrument, unchanged: the tree under `fixtures/check/`
//! is a tree of deliberate defects, so the first run declares every error cell
//! it raises as adoption debt, and the copy then passes a strict run. The only
//! thing that moves after that is two lines appended to `00-both-halves.md`,
//! which carries no error cell of its own, so a new finding there cannot be
//! absorbed by the payload.
//!
//! # Why it also asserts what the neighbors say
//!
//! [HW-OBL-0140](../../../../docs/obligations/0140-a-check-can-meet-the-failing-fixture-bar-with-a-fixture-that-cannot-distinguish-the-rule-from-its-neighbour.md)
//! records that a failing fixture can turn red for a neighboring rule and
//! prove nothing about the new one. The mismatched line names a file that
//! exists and carries no fragment, and this test asserts that both neighbor
//! rules are silent on it.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Severity, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::link_identifier::RULE;

/// The citing document, which carries no error cell of its own.
const CITER: &str = "check/spec/00-both-halves.md";

/// Text that names `02-cited-only.md`, on a path that reaches
/// `20-fragment-target.md`.
const MISMATCHED: &str =
    "The text of [SPEC-FIX-cited-only](20-fragment-target.md) names one document and the path reaches another.";
/// The same text on the path of the document it names.
const MATCHED: &str =
    "The text of [SPEC-FIX-cited-only](02-cited-only.md) names the document its path reaches.";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// A copy of the fixture tree, keyed on the label and the process id, because
/// cargo runs the cases of one target as threads of one process.
fn scratch(label: &str) -> Scratch {
    let at = Scratch(std::env::temp_dir().join(format!(
        "headwater-check-linked-identifier-{}-{label}",
        std::process::id()
    )));
    let _ = std::fs::remove_dir_all(&*at);
    std::fs::create_dir_all(&*at).expect("a scratch directory");
    copy_into(&fixtures_dir().join("check"), &at.join("check"));
    std::fs::copy(
        fixtures_dir().join("check.taxonomy.yml"),
        at.join("check.taxonomy.yml"),
    )
    .expect("the fixture taxonomy copies");
    at
}

/// A directory that is removed when this value is dropped.
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

fn copy_into(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("a readable fixture tree") {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy_into(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a file copies");
            }
        }
    }
}

/// One run over the scratch tree, with whatever adoption payload it is handed.
fn run_over(base: &Path, adoption: Option<&Mapping>) -> Run {
    let corpus = Corpus::new(base.to_path_buf(), "check");
    let source = std::fs::read_to_string(base.join("check.taxonomy.yml"))
        .expect("the fixture taxonomy reads");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let lock = headwater_hash::hex(source.as_bytes());
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            pin: None,
            harvests: &[],
            adoption,
            source: "fixtures/check.taxonomy.yml",
        },
        &headwater_check::claim::Claims::at(&corpus.base),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// Every `(document, rule)` cell of this run that carries an error, as an
/// adoption payload that declares all of them as debt.
fn debt_of(run: &Run) -> Mapping {
    let mut cells: Vec<(String, String)> = run
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| (finding.path.clone(), finding.rule.to_string()))
        .collect();
    cells.sort();
    cells.dedup();
    let mut pairs = String::new();
    for (path, rule) in &cells {
        use std::fmt::Write;
        let _ = writeln!(pairs, "      - {{path: {path}, rule: {rule}}}");
    }
    headwater_yaml::load(&format!(
        "\
tasks:
  - id: AD-1
    statement: every error this tree raises before the edit, declared as debt
    owner: the fixture tree
    until: 2027-01-01
    pairs:
{pairs}"
    ))
    .expect("the payload loads")
    .value
    .as_map()
    .expect("a mapping")
    .clone()
}

/// Every finding of one rule on one line of the citing document.
fn at<'a>(run: &'a Run, rule: &str, line: usize) -> Vec<&'a headwater_check::Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == rule && finding.path == CITER && finding.line == line)
        .collect()
}

/// Append the given lines to the citing document, each as its own paragraph,
/// and return the line number each one landed on.
fn append(base: &Path, lines: &[&str]) -> Vec<usize> {
    let path = base.join(CITER);
    let mut text = std::fs::read_to_string(&path).expect("the citer reads");
    let mut landed = Vec::new();
    for line in lines {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push('\n');
        landed.push(text.lines().count() + 1);
        text.push_str(line);
        text.push('\n');
    }
    std::fs::write(&path, text).expect("the citer writes");
    landed
}

/// The whole issue: one link whose text and path disagree turns a green strict
/// run red, with one finding of this rule at that line and none from either
/// neighbor, and the matched link beside it says nothing.
///
/// On an engine with no `link.identifier.mismatch` the second assertion is
/// what fails: the mismatched link resolves, carries no fragment, and the run
/// stays green.
#[test]
fn a_link_whose_text_names_another_document_turns_a_green_strict_run_red() {
    let base = scratch("mismatch");

    let payload = debt_of(&run_over(&base, None));
    assert!(
        !run_over(&base, Some(&payload)).has_errors(),
        "every error of this tree is declared as debt, and debt never blocks"
    );

    let landed = append(&base, &[MISMATCHED, MATCHED]);
    let (mismatched, matched) = (landed[0], landed[1]);

    let after = run_over(&base, Some(&payload));
    assert!(
        after.has_errors(),
        "a link whose text names another document left a strict run passing"
    );

    let raised = at(&after, RULE, mismatched);
    assert_eq!(raised.len(), 1, "{:#?}", after.findings);
    assert_eq!(raised[0].severity, Severity::Error);
    assert!(
        raised[0].message.contains("SPEC-FIX-cited-only"),
        "the finding names the identifier the text says: {:#?}",
        raised[0]
    );
    assert!(
        raised[0].message.contains("SPEC-FIX-fragment-target"),
        "and the identifier the path reaches: {:#?}",
        raised[0]
    );

    assert!(
        at(&after, RULE, matched).is_empty(),
        "{:#?}",
        after.findings
    );

    for neighbor in [
        headwater_check::link_path::RULE,
        headwater_check::fragment::RULE,
    ] {
        for line in [mismatched, matched] {
            assert!(
                at(&after, neighbor, line).is_empty(),
                "{neighbor} fired on line {line}, so this fixture cannot tell the rules apart"
            );
        }
    }

    // The only error cell the edit added is this rule's, in the citer.
    let new_cells: Vec<&headwater_check::Finding> = after
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error && finding.path == CITER)
        .collect();
    assert_eq!(new_cells.len(), 1, "{new_cells:#?}");
}

/// Space inside the brackets is not part of the identifier, and a code span is
/// how most of this repository writes one. Both are read as the identifier they
/// hold, so neither hides a mismatch.
#[test]
fn padded_and_code_span_text_is_read_as_the_identifier_it_holds() {
    let base = scratch("padded");

    let payload = debt_of(&run_over(&base, None));
    let landed = append(
        &base,
        &[
            "Padded: [ SPEC-FIX-cited-only ](20-fragment-target.md) reaches the wrong file.",
            "Code: [`SPEC-FIX-cited-only`](20-fragment-target.md) reaches the wrong file.",
        ],
    );

    let after = run_over(&base, Some(&payload));
    for line in landed {
        assert_eq!(
            at(&after, RULE, line).len(),
            1,
            "line {line}: {:#?}",
            after.findings
        );
    }
}

/// Two forms the inline case does not reach. A reference-style link binds
/// through its definition, and a link into the same document reaches the
/// document that wrote it, whose identifier is `SPEC-FIX-both-halves`. Each
/// one, with text that names another document, is one finding at its line.
#[test]
fn a_reference_link_and_a_same_document_link_are_read_too() {
    let base = scratch("forms");

    let payload = debt_of(&run_over(&base, None));
    let landed = append(
        &base,
        &[
            "Reference: [SPEC-FIX-cited-only][elsewhere] reaches the wrong file.",
            "[elsewhere]: 20-fragment-target.md",
            "Same document: [SPEC-FIX-cited-only](#both-halves) lands on this file.",
            "Same document, agreeing: [SPEC-FIX-both-halves](#both-halves) lands on this file.",
        ],
    );
    let (reference, same, agreeing) = (landed[0], landed[2], landed[3]);

    let after = run_over(&base, Some(&payload));
    let at_reference = at(&after, RULE, reference);
    assert_eq!(at_reference.len(), 1, "{:#?}", after.findings);
    assert!(
        at_reference[0].message.contains("SPEC-FIX-fragment-target"),
        "{:#?}",
        at_reference[0]
    );
    let at_same = at(&after, RULE, same);
    assert_eq!(at_same.len(), 1, "{:#?}", after.findings);
    assert!(
        at_same[0].message.contains("SPEC-FIX-both-halves"),
        "{:#?}",
        at_same[0]
    );
    assert!(
        at(&after, RULE, agreeing).is_empty(),
        "{:#?}",
        after.findings
    );
    for line in [reference, same, agreeing] {
        for neighbor in [
            headwater_check::link_path::RULE,
            headwater_check::fragment::RULE,
        ] {
            assert!(
                at(&after, neighbor, line).is_empty(),
                "{neighbor} on {line}"
            );
        }
    }
}

/// The denominator of the rule's silence (#1347). A run over a corpus whose
/// identifier links all agree reports no finding, so the run carries how many
/// links the rule compared, and the report prints that count beside the
/// rule's own finding count. The population is the rule's own: an agreeing
/// inline link and an agreeing link into the same document each add one, and
/// neither adds a finding. A count that left the same-document form out would
/// go up by one here. Identifier text on a path that reaches nothing, and on
/// an address outside the repository, reaches no identifier, so the rule
/// compares neither and neither adds one.
#[test]
fn the_run_carries_how_many_links_the_rule_compared() {
    use headwater_check::paint::ColorMode;
    use headwater_check::Detail;

    let base = scratch("compared");

    let payload = debt_of(&run_over(&base, None));
    let before = run_over(&base, Some(&payload));
    append(
        &base,
        &[
            "Inline, agreeing: [SPEC-FIX-cited-only](02-cited-only.md) reaches its own document.",
            "Same document, agreeing: [SPEC-FIX-both-halves](#both-halves) lands on this file.",
            "Missing: [SPEC-FIX-cited-only](no-such-file.md) reaches nothing.",
            "External: [SPEC-FIX-cited-only](https://example.com/x) leaves the repository.",
        ],
    );
    let after = run_over(&base, Some(&payload));

    let compared = |run: &Run| {
        run.served
            .iter()
            .find(|served| served.rule == RULE)
            .and_then(|served| served.compared)
            .expect("the rule carries a compared count")
    };
    let reported = |run: &Run| run.findings.iter().filter(|f| f.rule == RULE).count();
    assert_eq!(compared(&after), compared(&before) + 2);
    assert_eq!(reported(&after), reported(&before));
    assert!(
        compared(&before) > 0,
        "the fixture tree has identifier links"
    );

    // No rule carries the member but the two link rules whose silence it
    // qualifies.
    for served in &after.served {
        if served.rule != RULE && served.rule != headwater_check::fragment::RULE {
            assert_eq!(served.compared, None, "{}", served.rule);
        }
    }

    // The report prints both numbers on the line under the rule's scope.
    let scope = after
        .served
        .iter()
        .find(|served| served.rule == RULE)
        .expect("the rule is served")
        .scope
        .render();
    let line = format!(
        "  {RULE}\n    {scope}\n    {} links compared, {} of them with a finding\n",
        compared(&after),
        reported(&after)
    );
    let text = after.render(Detail::Findings, ColorMode::Plain);
    assert!(text.contains(&line), "{line}\nnot in\n{text}");
    // Scripts read the run's total as the first line that ends in
    // `<n> findings` (`tools/repo/diataxis-facet-fixtures.sh`'s
    // `report_number`, `tools/taxonomy/drive_n8n.py`). The per-rule line comes
    // before the total, so it must not have that shape.
    let first_total = text
        .lines()
        .find_map(|line| {
            let head = line.strip_suffix(" findings")?;
            let number = head.rsplit([' ', ',']).next()?;
            number.parse::<usize>().ok()
        })
        .expect("the report has a findings total");
    assert_eq!(first_total, after.findings.len(), "{text}");
    // The totals carry no count of findings, by Detail::Totals's contract, so
    // they print the denominator alone.
    let totals = after.render(Detail::Totals, ColorMode::Plain);
    let alone = format!(
        "  {RULE}\n    {scope}\n    {} links compared\n",
        compared(&after)
    );
    assert!(totals.contains(&alone), "{alone}\nnot in\n{totals}");
}

/// The same denominator for `link.fragment.unresolved`, folded into #1347. The
/// rule compares a fragment against the heading list of the document it
/// names, so a fragment into this document and a fragment into another corpus
/// document each add one to the count, whether they resolve or not. A
/// fragment into a file outside the corpus root, into a file that carries no
/// parsed document, or onto an address outside the repository has no heading
/// list to compare against, and adds nothing.
#[test]
fn the_run_carries_how_many_fragment_links_the_fragment_rule_compared() {
    use headwater_check::paint::ColorMode;
    use headwater_check::Detail;
    const FRAGMENT: &str = headwater_check::fragment::RULE;

    let base = scratch("fragments-compared");

    let payload = debt_of(&run_over(&base, None));
    let before = run_over(&base, Some(&payload));
    // A file under the corpus root that no kind claims, so the census walks
    // it and parses no document out of it.
    std::fs::write(base.join("check/plain.txt"), "# A heading\n").expect("a plain file");
    let landed = append(
        &base,
        &[
            "Same document, resolving: [here](#both-halves).",
            "Across, dead: [there](02-cited-only.md#no-such-heading).",
            "Outside the corpus root: [taxonomy](../../check.taxonomy.yml#kinds).",
            "No parsed document: [plain](../plain.txt#a-heading).",
            "External: [site](https://example.com/page#part).",
        ],
    );
    let after = run_over(&base, Some(&payload));

    let compared = |run: &Run| {
        run.served
            .iter()
            .find(|served| served.rule == FRAGMENT)
            .and_then(|served| served.compared)
            .expect("the fragment rule carries a compared count")
    };
    let reported = |run: &Run| run.findings.iter().filter(|f| f.rule == FRAGMENT).count();
    assert_eq!(compared(&after), compared(&before) + 2);
    assert_eq!(reported(&after), reported(&before) + 1);
    assert_eq!(
        at(&after, FRAGMENT, landed[1]).len(),
        1,
        "{:#?}",
        after.findings
    );
    for line in [landed[0], landed[2], landed[3], landed[4]] {
        assert!(at(&after, FRAGMENT, line).is_empty(), "{line}");
    }

    let scope = after
        .served
        .iter()
        .find(|served| served.rule == FRAGMENT)
        .expect("the fragment rule is served")
        .scope
        .render();
    let line = format!(
        "  {FRAGMENT}\n    {scope}\n    {} links compared, {} of them with a finding\n",
        compared(&after),
        reported(&after)
    );
    let text = after.render(Detail::Findings, ColorMode::Plain);
    assert!(text.contains(&line), "{line}\nnot in\n{text}");
}

/// The matched link alone leaves the run green: the rule reads a link whose
/// text is an identifier, and passes it when the path agrees.
#[test]
fn a_link_whose_text_names_the_document_it_reaches_stays_green() {
    let base = scratch("matched");

    let payload = debt_of(&run_over(&base, None));
    let landed = append(&base, &[MATCHED]);

    let after = run_over(&base, Some(&payload));
    assert!(!after.has_errors(), "{:#?}", after.findings);
    assert!(at(&after, RULE, landed[0]).is_empty());
}
