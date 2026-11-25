// SPDX-License-Identifier: Apache-2.0
//! A document that names another document's identifier in its prose and
//! declares no relation to it (#1977).
//!
//! The phase B forensics of #1881 found an agent that wrote "This follows from
//! HW-DR-0089" in the body of a new obligation and declared no `traces_to`. No
//! rule read the sentence, so the graph lost the edge the author meant. This
//! rule reads it and offers the relations the taxonomy declares for the pair.
//! The cases below are the ways a rule about it is easy to get wrong.
//!
//! **A rule that ignored the inverse half** reports `obligations/inverse-only.md`,
//! whose edge the decision wrote.
//!
//! **A rule that offered every relation** reports `decisions/names-obligation.md`,
//! for which the taxonomy declares no relation at all (HW-OBL-0055). A rule
//! that read only the target kind reports it too, because `refines` reaches
//! an obligation from another obligation. A rule that read only the source
//! kind reports `obligations/names-note.md`, because `traces_to` leaves an
//! obligation and no relation reaches a note.
//!
//! **A rule that read a document's own identifier** reports
//! `obligations/self-and-ghost.md`, which `refines` would join to itself.
//!
//! **A rule that read every run of text** reports the code span, the fenced and
//! indented blocks and the block quote of `obligations/not-prose.md`.
//!
//! **A rule that reported each mention** reports `obligations/thrice.md` three
//! times for one missing edge.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Finding, Register, Run, Severity, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

/// Named as a literal rather than through the crate, so this table compiles
/// and fails on the rule's absence before the rule exists.
const RULE: &str = "relation.mention.undeclared";

const TREE: &str = "mentioned-identifier";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// One run over the fixture tree and the taxonomy of the same name.
fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), TREE);
    let source = std::fs::read_to_string(fixtures_dir().join(format!("{TREE}.taxonomy.yml")))
        .expect("the fixture taxonomy");
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
    let lock = format!("sha256:{TREE}-fixture");
    let source = format!("engine/crates/check/fixtures/{TREE}.taxonomy.yml");
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
            lock_stale: None,
            agent_budgets: &[],
            harvests: &[],
            imports: &[],
            adoption: None,
            source: &source,
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

fn path(file: &str) -> String {
    format!("{TREE}/{file}")
}

/// Every finding of the rule on one file, in report order.
fn on<'a>(run: &'a Run, file: &str) -> Vec<&'a Finding> {
    let wanted = path(file);
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE && finding.path == wanted)
        .collect()
}

/// The decisive case: an obligation that names a decision in prose and
/// declares no relation gets one note, at the mention, that names the
/// identifier and the relation it could declare. No patch, because whether the
/// mention is a dependency is the author's call.
#[test]
fn an_obligation_naming_a_decision_in_prose_is_offered_the_relation() {
    let run = run();
    let found = on(&run, "obligations/names-in-prose.md");
    assert_eq!(found.len(), 1, "{:#?}", run.findings);
    let finding = found[0];
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!((finding.line, finding.column), (9, 19), "{finding:#?}");
    assert!(finding.message.contains("`DEC-MEN-0001`"), "{finding:#?}");
    assert!(finding.message.contains("`traces_to`"), "{finding:#?}");
    assert!(
        finding
            .remediation
            .contains("headwater explain DEC-MEN-0001"),
        "{finding:#?}"
    );
    assert!(finding.patch.is_none(), "{:?}", finding.patch);
}

/// The relation, declared, silences the rule.
#[test]
fn a_declared_relation_is_silent() {
    assert!(on(&run(), "obligations/declared.md").is_empty());
}

/// An edge in either direction joins the pair, whichever document wrote it.
#[test]
fn an_inverse_half_written_by_the_named_document_is_silent() {
    let run = run();
    assert!(on(&run, "obligations/inverse-only.md").is_empty());
    assert!(on(&run, "decisions/inverse-writer.md").is_empty());
}

/// No relation goes from a decision to an obligation, so there is nothing to
/// offer and no finding.
#[test]
fn a_pair_no_relation_admits_is_silent() {
    assert!(on(&run(), "decisions/names-obligation.md").is_empty());
}

/// A note is the end of no relation, so an obligation that names one has
/// nothing to be offered, though `traces_to` leaves obligations.
#[test]
fn a_named_kind_no_relation_reaches_is_silent() {
    assert!(on(&run(), "obligations/names-note.md").is_empty());
}

/// A code span, a fenced block, an indented block and a block quote are not
/// the author's prose.
#[test]
fn an_identifier_outside_authored_prose_is_silent() {
    assert!(on(&run(), "obligations/not-prose.md").is_empty());
}

/// A document's own identifier, a token no document holds, and a token that
/// only contains an identifier are not mentions of another document.
#[test]
fn its_own_identifier_and_a_ghost_token_are_silent() {
    assert!(on(&run(), "obligations/self-and-ghost.md").is_empty());
}

/// A generated document's prose and front matter are the generator's, so its
/// author has nowhere to declare the relation.
#[test]
fn a_generated_source_is_silent() {
    let run = run();
    assert!(
        run.findings.iter().all(|finding| finding.path != path("obligations/generated.md")
            || finding.rule != RULE),
        "{:#?}",
        run.findings
    );
}

/// One missing edge is one finding, at the first mention, however many times
/// the prose names the document. A trailing hyphen is trimmed from the token.
#[test]
fn three_mentions_of_one_document_are_one_finding_at_the_first() {
    let run = run();
    let found = on(&run, "obligations/thrice.md");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!((found[0].line, found[0].column), (9, 8), "{:#?}", found[0]);
}

/// A relation that names an abstract ancestor at both ends admits every pair
/// of its descendants. `cites` names `record`, and the guide and the rationale
/// are each `is_a: record`, so the guide is offered `cites` by inheritance and
/// by nothing else.
#[test]
fn a_relation_between_ancestors_is_offered_to_their_descendants() {
    let run = run();
    let found = on(&run, "guides/names-rationale.md");
    assert_eq!(found.len(), 1, "{:#?}", run.findings);
    let finding = found[0];
    assert_eq!((finding.line, finding.column), (8, 15), "{finding:#?}");
    assert!(finding.message.contains("`RAT-MEN-0001`"), "{finding:#?}");
    assert!(finding.message.contains("`cites`"), "{finding:#?}");
}

/// The whole population of the tree: the three cases above and nothing else.
#[test]
fn the_tree_reports_exactly_three_findings() {
    let run = run();
    let all: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        all,
        [
            path("guides/names-rationale.md"),
            path("obligations/names-in-prose.md"),
            path("obligations/thrice.md")
        ],
        "{:#?}",
        run.findings
    );
}
