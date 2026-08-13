// SPDX-License-Identifier: Apache-2.0
//! What `check --fix` writes, held against the engine's own checks.
//!
//! The claim [#71](https://github.com/headwater-ai/headwater/issues/71) is
//! about: a patch over prose anchors at an offset, the offsets come from
//! [`headwater_doc::sentences`], and
//! [spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots) holds
//! that splitter as a correctness root. A rewrite at a wrong offset **corrupts
//! a document** rather than reporting a wrong line, so a green run over the
//! result is not enough on its own and this file records four things.
//!
//! 1. Every finding of the first run that carried a patch is gone from the
//!    second.
//! 2. **No finding of the second run is new.** That is the test the issue asks
//!    for in as many words: if the fixer's own output does not pass the
//!    engine's own checks, that is the defect.
//! 3. The three spans a patch must not land at are each accounted for.
//!    `07-prose-defects.md` writes one contraction inside a code span, one
//!    inside a block quotation and one inside a link's text. The first two
//!    reach no rule at all, which is
//!    [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)'s
//!    construction rather than an exemption. The third is this author's prose,
//!    so it is reported, fixed, and the link survives the fix.
//! 4. The whole finding set of both runs, recorded. A rule that stopped
//!    generating instances over these documents would otherwise make this file
//!    pass by reading nothing.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-scaffold --test fixing
//!
//! # It reads the check crate's fixture corpus
//!
//! Because that corpus already carries one failing document per prose rule and
//! one missing reciprocal half, which is three of the four things a patch can
//! be. A second corpus here would be a second set of defects that could drift
//! from the first, and the rules being exercised are that corpus's own.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::shape::Shape;
use headwater_check::{Cache, Context, Date, Declared, Finding, Patch, Register, Run};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_scaffold::fix;
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-14";
const TAXONOMY: &str = "engine/crates/check/fixtures/check.taxonomy.yml";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../check/fixtures")
}

fn recorded_at() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/fixing.report")
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {:?}", path.display(), errors))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

/// A copy of the fixture corpus, under a directory this test owns.
///
/// A fix writes, and the fixture corpus is committed. A test that wrote into
/// the committed tree would hand the next run a different corpus.
struct Scratch(PathBuf);

/// One name per copy. These tests run in parallel and each one writes.
static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl Scratch {
    fn new() -> Self {
        let at = std::env::temp_dir().join(format!(
            "headwater-fixing-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&at);
        copy_tree(&fixtures_dir().join("check"), &at.join("check"));
        Scratch(at)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the scratch directory");
    for entry in std::fs::read_dir(from)
        .expect("the fixture corpus")
        .flatten()
    {
        let target = to.join(entry.file_name());
        match entry.path().is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a fixture file");
            }
        }
    }
}

/// One run of the whole check layer over a tree.
fn check_over(root: &Path) -> Run {
    let resolved = load_map(&fixtures_dir().join("check.taxonomy.yml"));
    let taxonomy = Taxonomy::read(&resolved).expect("the shelves read");
    let declarations = Declarations::read(&resolved).expect("the relations read");
    let register = Register::read(&resolved).expect("the register reads");
    let shape = Shape::read(&resolved).expect("the shape reads");
    let corpus = Corpus::new(root.to_path_buf(), "check");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let source = std::fs::read_to_string(fixtures_dir().join("check.taxonomy.yml"))
        .expect("the fixture taxonomy");
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &headwater_hash::hex(source.as_bytes()),
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: TAXONOMY,
        },
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// One finding as the line this test compares, without the offsets that a fix
/// moves. Two runs over one tree are being compared, and every line below the
/// first substitution shifts.
fn named(finding: &Finding) -> String {
    format!("{} {} {}", finding.path, finding.rule, finding.message)
}

#[test]
fn what_the_fixer_wrote_passes_the_engines_own_checks() {
    let scratch = Scratch::new();
    let root = &scratch.0;

    let before = check_over(root);
    let patches: Vec<Patch> = before
        .findings
        .iter()
        .filter_map(|finding| finding.patch.clone())
        .collect();
    assert!(
        !patches.is_empty(),
        "the fixture corpus offers no patch, so this test proves nothing"
    );

    let composed = fix::compose(root, &patches);
    assert!(
        composed.refused.is_empty(),
        "the fixture corpus refused a patch: {:?}",
        composed.refused
    );
    fix::apply(root, &composed.files).expect("the files write");

    let after = check_over(root);

    // 1. Every fixable finding is gone.
    let fixed: Vec<String> = before
        .findings
        .iter()
        .filter(|finding| finding.fixable())
        .map(named)
        .collect();
    let survives: Vec<&String> = fixed
        .iter()
        .filter(|line| {
            after
                .findings
                .iter()
                .any(|finding| named(finding) == **line)
        })
        .collect();
    assert!(
        survives.is_empty(),
        "a patch was written and the finding stands:\n{survives:#?}"
    );

    // 2. And nothing new arrived. This is the assertion the issue asks for.
    let was: Vec<String> = before.findings.iter().map(named).collect();
    let arrived: Vec<String> = after
        .findings
        .iter()
        .map(named)
        .filter(|line| !was.contains(line))
        .collect();
    assert!(
        arrived.is_empty(),
        "the fixer's own output does not pass the engine's own checks:\n{arrived:#?}"
    );

    // 3. The three spans, and what became of each.
    let defects = root.join("check/spec/07-prose-defects.md");
    let text = std::fs::read_to_string(&defects).expect("the prose fixture");
    assert!(
        text.contains("`it doesn't matter`"),
        "the contraction inside a code span was rewritten"
    );
    assert!(
        text.contains("> A quoted author writes what they write, and it doesn't answer"),
        "the contraction inside a block quotation was rewritten"
    );
    assert!(
        text.contains("[it does not escape](00-both-halves.md)"),
        "the contraction inside a link's text was not fixed, or the link did not survive it"
    );
    assert!(text.contains("It does not expand the contraction"));
    assert!(text.contains("The behavior of the resolver"));
    assert!(text.contains("The taxonomy resolves each name"));

    // The reciprocal half, written by the splice into the far document.
    let far = std::fs::read_to_string(root.join("check/spec/00-both-halves.md"))
        .expect("the far document");
    assert!(far.contains("cites_evidence"), "{far}");

    // 4. The recorded account, because a green run proves nothing on its own.
    let mut report = String::new();
    for file in &composed.files {
        report.push_str(&format!("fixed {} {}\n", file.path, file.applied));
    }
    report.push_str("\nfindings before\n");
    for line in &was {
        report.push_str(&format!("  {line}\n"));
    }
    report.push_str("\nfindings after\n");
    for finding in &after.findings {
        report.push_str(&format!("  {}\n", named(finding)));
    }
    compare(&recorded_at(), &report);
}

/// A patch computed over one tree and applied to another is refused, and the
/// run says so rather than writing at the offset it was handed.
#[test]
fn a_patch_that_names_bytes_the_file_does_not_hold_is_refused() {
    let scratch = Scratch::new();
    let root = &scratch.0;
    let refused = fix::compose(
        root,
        &[Patch::Text {
            path: "check/spec/07-prose-defects.md".to_string(),
            start: 200,
            end: 209,
            expect: "behaviour".to_string(),
            replacement: "behavior".to_string(),
        }],
    );
    assert!(refused.files.is_empty(), "{:?}", refused.files);
    assert!(
        matches!(refused.refused[0], fix::Refused::Moved { .. }),
        "{:?}",
        refused.refused
    );
}

fn compare(recorded: &Path, actual: &str) {
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::write(recorded, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(recorded).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            recorded.display()
        )
    });
    assert_eq!(
        expected,
        actual,
        "\nthe run over the fixed tree no longer matches {}",
        recorded.display()
    );
}
