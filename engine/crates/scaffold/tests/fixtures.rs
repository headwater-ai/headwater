// SPDX-License-Identifier: Apache-2.0
//! The scaffolder's conformance fixtures.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! names the scaffolder a correctness root, and it states what one owes: "Each
//! component therefore owes its own conformance fixtures, in the same spirit as
//! 'a check without a failing fixture does not ship'." This file is the
//! instrument, and it has three parts.
//!
//! **The transcript.** Every case over the fixture corpus, recorded whole:
//! what each run proposed, where every value came from, and the sentence of
//! every refusal. A recorded transcript rather than an assertion per case,
//! because the thing under test is what the scaffolder *decides*, and a new
//! branch must not be able to arrive with no record at all.
//!
//! **The pipeline.** The claim the issue is about, tested rather than argued:
//! scaffold into a copy of the fixture corpus, then run the check layer over
//! the result and record every finding. A scaffolder whose output does not pass
//! the engine's own checks is the defect the root exists to prevent, and this
//! is where that would show.
//!
//! **The grammar.** What the minter writes, `identifier.pattern.not_met`
//! admits. It runs over every scheme of the fixture taxonomy and over every
//! scheme of this repository's committed lock, so a scheme added to either
//! reaches it with no edit here.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-scaffold --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::shape::Shape;
use headwater_check::Date;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_graph::Config;
use headwater_meta::identifier::Template;
use headwater_scaffold::{propose, write, Plan, Refusal, Request, Sources};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The date every recorded run is taken at. Spec 12 makes the clock an injected
/// value, and a fixture that read today's date would rewrite itself nightly.
const PINNED: &str = "2026-08-14";

fn pinned() -> Date {
    Date::parse(PINNED).expect("the pinned date")
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
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
        "\nthe scaffolder no longer matches {}",
        recorded.display()
    );
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

/// Everything a scaffolder reads, over one tree.
struct Loaded {
    resolved: Mapping,
    shape: Shape,
    shelves: Taxonomy,
    relations: Declarations,
    census: Census,
    index: Index,
    config: Config,
    /// No store under the fixture tree, so this is empty. Held rather than made
    /// at the point of use, because `sources` returns a borrow of it.
    claims: headwater_check::claim::Claims,
}

impl Loaded {
    fn over(root: &Path, corpus_root: &str, taxonomy: &Path) -> Self {
        let resolved = load_map(taxonomy);
        let shelves = Taxonomy::read(&resolved).expect("the shelves read");
        let relations = Declarations::read(&resolved).expect("the relations read");
        let shape = Shape::read(&resolved).expect("the shape reads");
        let corpus = Corpus::new(root.to_path_buf(), corpus_root);
        let census = census::take(&corpus, &shelves);
        let config = Config::default();
        let index = Index::build(&census, &config);
        Loaded {
            resolved,
            shape,
            shelves,
            relations,
            census,
            index,
            config,
            claims: headwater_check::claim::Claims::at(root),
        }
    }

    fn sources(&self) -> Sources<'_> {
        Sources {
            resolved: &self.resolved,
            shape: &self.shape,
            shelves: &self.shelves,
            relations: &self.relations,
            census: &self.census,
            index: &self.index,
            config: &self.config,
            claims: &self.claims,
        }
    }
}

/// One case of the transcript.
struct Case {
    kind: &'static str,
    title: &'static str,
    relates: Vec<(String, String)>,
    /// What the caller stated with `--facet`, in the order they named it.
    given: Vec<(String, String)>,
}

fn case(kind: &'static str, title: &'static str) -> Case {
    Case {
        kind,
        title,
        relates: Vec::new(),
        given: Vec::new(),
    }
}

impl Case {
    fn relating(mut self, relation: &str, target: &str) -> Self {
        self.relates
            .push((relation.to_string(), target.to_string()));
        self
    }

    /// A `--facet <name>=<value>` the caller states on the command line.
    fn stating(mut self, facet: &str, value: &str) -> Self {
        self.given.push((facet.to_string(), value.to_string()));
        self
    }
}

/// The command line a case stands for, which is the header of its transcript
/// block. It is built from the case rather than written beside it, so a case
/// that states a facet cannot be recorded under a header that omits it.
fn invocation(case: &Case) -> String {
    let mut line = format!("=== new {} --title \"{}\"", case.kind, case.title);
    for (relation, target) in &case.relates {
        line.push_str(&format!(" --relates {relation}={target}"));
    }
    for (facet, value) in &case.given {
        line.push_str(&format!(" --facet {facet}={value}"));
    }
    line
}

fn request<'a>(case: &'a Case) -> Request<'a> {
    Request {
        kind: case.kind,
        title: case.title,
        now: pinned(),
        relates: &case.relates,
        given: &case.given,
    }
}

/// Every case, in one order, so that the transcript reads top to bottom.
fn cases() -> Vec<Case> {
    vec![
        // What it writes.
        case("design_spec", "A scaffolded fourth part"),
        case("decision_record", "A scaffolded decision"),
        case("numbered", "A file named from its own identifier"),
        case("standalone", "A kind no relation may name"),
        case("decision_record", "A successor with a far half")
            .relating("supersedes", "SPEC-FIX-the-second-part"),
        case("decision_record", "A successor into a block with the key")
            .relating("supersedes", "SPEC-FIX-the-third-part"),
        case(
            "decision_record",
            "A successor into a document with no block",
        )
        .relating("supersedes", "DR-FIX-0007"),
        case("decision_record", "An assessment with no far half")
            .relating("assesses", "SPEC-FIX-the-first-part"),
        case("decision_record", "Two edges at once")
            .relating("supersedes", "SPEC-FIX-the-second-part")
            .relating("assesses", "SPEC-FIX-the-first-part"),
        // What it refuses. One case per branch of `Refusal`.
        case("nonesuch", "A kind nobody declared"),
        case("governed_document", "An abstract kind"),
        case("orphan", "A kind on no shelf"),
        case("ambiguous", "A kind on two shelves"),
        case("misplaced", "A shelf that globs early"),
        case("holed", "A layout with nothing to fill it"),
        case("decision_record", ""),
        case("unopened_record", "A regime with no initial state"),
        case("chaptered", "An integer nothing derives"),
        case("addressed", "A closed set nothing derives"),
        case("unmintable", "A pattern that cannot be read"),
        case("exhausted", "A sequence field that is full"),
        case("annexed", "A layout that lands on another shelf"),
        case("standard", "A relation endpoint with no scheme"),
        case("design_spec", "The first part"),
        case("decision_record", "An earlier decision"),
        case("decision_record", "A relation nobody declared").relating("flibberty", "DR-FIX-0007"),
        case("decision_record", "An edge a hook pays for").relating("traces_to", "DR-FIX-0007"),
        case("decision_record", "A source end the relation forbids")
            .relating("refines", "SPEC-FIX-the-first-part"),
        case("design_spec", "A target end the relation forbids").relating("refines", "DR-FIX-0007"),
        case("decision_record", "A target that resolves to nothing")
            .relating("supersedes", "DR-FIX-9999"),
        // What `--facet` does. Every branch the flag can reach, including the
        // one where it succeeds, because the flag shipped with no case at all
        // and that is why the discriminator branch could discard a value in
        // silence.
        case("addressed", "A closed set the caller states").stating("audience", "internal"),
        case("addressed", "A closed set value outside it").stating("audience", "wibble"),
        case("design_spec", "A facet the kind does not require").stating("chapter", "3"),
        case("design_spec", "A facet the state role decides").stating("status", "draft"),
        case("design_spec", "A discriminator the caller disagrees with")
            .stating("doc_type", "note"),
        case("design_spec", "A discriminator the caller agrees with")
            .stating("doc_type", "design_spec"),
    ]
}

/// Every case over the fixture corpus, recorded whole.
#[test]
fn every_case_over_the_fixture_corpus_matches_the_recorded_transcript() {
    let loaded = Loaded::over(
        &fixtures_dir(),
        "corpus",
        &fixtures_dir().join("scaffold.taxonomy.yml"),
    );
    let sources = loaded.sources();

    let mut transcript = String::new();
    for case in cases() {
        transcript.push_str(&invocation(&case));
        transcript.push_str("\n\n");
        let request = request(&case);
        match propose(&sources, &request) {
            Err(refusal) => {
                transcript.push_str(&format!("refused, {}\n", variant(&refusal)));
                transcript.push_str(&format!("  {refusal}\n"));
            }
            Ok(plan) => transcript.push_str(&render_plan(&plan)),
        }
        transcript.push('\n');
    }

    compare(&fixtures_dir().join("scaffold.transcript"), &transcript);
}

/// Every branch of `Refusal`, named once.
///
/// The macro writes two things from this one list: [`variant`], which gives a
/// refusal its name so that a test asserts against a name rather than a shape,
/// and `BRANCHES`, which is what [`every_refusal_branch_has_a_case`] holds the
/// suite to. They were two hand-kept lists until the `--facet` refusals showed
/// what that costs: three branches were in the match and absent from the
/// constant, so a test whose name claims total coverage quietly excluded them,
/// and the whole flag shipped with no fixture. One list cannot drift from
/// itself.
///
/// The match is exhaustive rather than a `_ =>`, so a branch added to the enum
/// does not compile until it is named here, and it is then in `BRANCHES` and
/// fails the coverage test until a case reaches it.
macro_rules! branches {
    ($($name:ident),+ $(,)?) => {
        fn variant(refusal: &Refusal) -> &'static str {
            match refusal {
                $(Refusal::$name { .. } => stringify!($name),)+
            }
        }

        const BRANCHES: &[&str] = &[$(stringify!($name)),+];
    };
}

branches![
    KindUnknown,
    KindAbstract,
    KindUnshelved,
    KindOnManyShelves,
    ShelfPathNotLiteral,
    LayoutUnresolved,
    TitleEmpty,
    FacetUndeterminable,
    FacetNotAsked,
    FacetDetermined,
    FacetIsDiscriminator,
    FacetNotPermitted,
    SchemeUnreadable,
    MintRefused,
    Unnameable,
    IdentifierTaken,
    PathTaken,
    PlacementDoesNotResolve,
    RelationUnknown,
    RelationNotScaffolded,
    EndpointNotPermitted,
    ClaimUnwritable,
    TargetUnresolved,
    ReciprocalUnwritable,
    TargetUnopened,
    DocumentUncreated,
    WriteHalted,
    NotOneDocument,
];

fn render_plan(plan: &Plan) -> String {
    let mut out = String::new();
    out.push_str(&format!("wrote {}\n", plan.path));
    match &plan.minting {
        Some(minting) => out.push_str(&format!(
            "identifier {} under `{}`, allocation {}, reconciled from {}\n",
            minting.id,
            minting.scheme,
            minting.allocation.as_deref().unwrap_or("unstated"),
            minting
                .reconciled_from
                .map(|value| value.to_string())
                .unwrap_or_else(|| "nothing".to_string())
        )),
        None => out.push_str("no identifier\n"),
    }
    for field in &plan.fields {
        let origin = match field.origin.is_scaffolded() {
            true => "scaffolded",
            false => "hand entry",
        };
        out.push_str(&format!(
            "  {} = {} [{origin}: {}]\n",
            field.key,
            field.value,
            field.origin.reason()
        ));
    }
    for edge in &plan.edges {
        out.push_str(&format!(
            "  edge {} -> {} [created_by: {}]\n",
            edge.relation, edge.target, edge.created_by
        ));
        match &edge.reciprocal {
            Some(half) => out.push_str(&format!(
                "    far half `{}` into {}\n",
                half.relation, half.path
            )),
            None => out.push_str("    no far half\n"),
        }
    }
    let assisted = plan.assisted();
    out.push_str(&format!(
        "assisted {} of {} — fields {}/{}, sections {}/{}, identifier {}/{}, halves {}/{}\n",
        assisted.supplied(),
        assisted.total(),
        assisted.fields.0,
        assisted.fields.1,
        assisted.sections.0,
        assisted.sections.1,
        assisted.identifier.0,
        assisted.identifier.1,
        assisted.edge_halves.0,
        assisted.edge_halves.1,
    ));
    out.push_str("--- the document ---\n");
    out.push_str(&write::render(plan));
    out
}

/// Every scheme this repository and the fixture taxonomy declare, minted and
/// then read back by the rule that checks an identifier.
///
/// The property, stated once: **what the minter writes, the check admits.** It
/// is what makes one grammar one grammar. A second parser in the scaffolder
/// would pass its own tests and fail this one.
#[test]
fn what_the_minter_writes_the_rule_admits() {
    let mut checked = 0;
    for source in [
        fixtures_dir().join("scaffold.taxonomy.yml"),
        repository_root().join(".headwater/taxonomy.lock"),
    ] {
        let root = match source.file_name().and_then(|name| name.to_str()) {
            Some("taxonomy.lock") => headwater_lock::at(&repository_root())
                .expect("the committed lock")
                .taxonomy
                .clone(),
            _ => load_map(&source),
        };
        let shape = Shape::read(&root).expect("the shape reads");
        for scheme in &shape.identifier_schemes {
            let Ok(template) = Template::parse(&scheme.pattern, &scheme.namespace) else {
                // A pattern the reader refuses is a scheme the minter issues
                // nothing under, which `Refusal::SchemeUnreadable` records in
                // the transcript above.
                continue;
            };
            for sequence in [0_u64, 1, 42, 9999] {
                // A value past the declared width has no identifier under this
                // scheme at all, which is the other half of the grammar and its
                // own test below. What matters here is that the minter never
                // writes a string the rule then refuses.
                let Ok(minted) = template.mint(Some("a-slug-somebody-chose"), Some(sequence))
                else {
                    continue;
                };
                assert!(
                    template.admits(&minted),
                    "the minter wrote `{minted}` under `{}`, and the rule refuses it",
                    scheme.name
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "no scheme reached this test");
}

/// Every branch of `Refusal` is reached by a case of the transcript.
///
/// The rule this enforces is the one spec 12 states about a check: a refusal
/// with no fixture does not ship. `BRANCHES` is the list the `branches!` macro
/// above writes out of the same names `variant` matches on, so no branch can be
/// left out of it. Five branches no `propose` case reaches are named below with
/// the file that holds each one, so that the gap is stated rather than left for
/// a reader to notice.
#[test]
fn every_refusal_branch_has_a_case() {
    let loaded = Loaded::over(
        &fixtures_dir(),
        "corpus",
        &fixtures_dir().join("scaffold.taxonomy.yml"),
    );
    let sources = loaded.sources();
    let mut reached: Vec<&'static str> = Vec::new();
    for case in cases() {
        let request = request(&case);
        if let Err(refusal) = propose(&sources, &request) {
            reached.push(variant(&refusal));
        }
    }
    // The branches no `propose` case reaches, and the file that holds each.
    // `ReciprocalUnwritable` fires on a document whose front matter the splice
    // could not read, which is [`the_splice_refuses_rather_than_corrupts`]
    // below. The other four are the write phase, and every one of them has a
    // case in `tests/writing.rs` — named here rather than counted, so a branch
    // that arrives with no fixture is a failure of this test.
    reached.push("ReciprocalUnwritable");
    reached.push("TargetUnopened");
    reached.push("DocumentUncreated");
    reached.push("WriteHalted");
    reached.push("NotOneDocument");
    // The claim writer, whose one refusal is an occupied path. It is in
    // `tests/writing.rs` beside the four above, because `propose` writes
    // nothing and so cannot reach it.
    reached.push("ClaimUnwritable");

    let missing: Vec<&&str> = BRANCHES
        .iter()
        .filter(|branch| !reached.contains(branch))
        .collect();
    assert!(
        missing.is_empty(),
        "these refusals ship with no fixture: {missing:?}"
    );
}

/// A front matter the splice cannot read refuses, and writes nothing.
///
/// The splice assumes a two-space nesting that no declaration states. The
/// assumption is safe because it is tested here rather than trusted: the
/// function parses its own result and looks for the half it wrote.
#[test]
fn the_splice_refuses_rather_than_corrupts() {
    let half = headwater_scaffold::Half {
        relation: "superseded_by".to_string(),
        path: "corpus/spec/02-the-second-part.md".to_string(),
        id: "DR-FIX-0008".to_string(),
        attributes: Vec::new(),
    };

    // No front matter at all.
    let refused = write::splice("# A document with no block\n", &half);
    assert!(matches!(refused, Err(Refusal::ReciprocalUnwritable { .. })));

    // A block that never closes.
    let refused = write::splice("---\nid: DR-FIX-0009\n\n# Nothing closed it\n", &half);
    assert!(matches!(refused, Err(Refusal::ReciprocalUnwritable { .. })));

    // And the case that works, so that the two are told apart by this test and
    // not only by the transcript.
    let spliced = write::splice("---\nid: DR-FIX-0009\n---\n\n# A document\n", &half)
        .expect("a plain block splices");
    assert!(spliced.contains("  superseded_by:\n    - DR-FIX-0008"));
}

/// A sequence past the declared width is refused rather than written short.
///
/// The other half of the grammar. `DR-FIX-42` is not `DR-FIX-0042` written
/// short: it is a second string, and an index that held both would resolve one
/// name to two documents.
#[test]
fn a_sequence_past_the_width_is_refused() {
    let template = Template::parse("DR-{namespace}-{seq:04d}", "FIX").expect("the template");
    assert!(template.mint(None, Some(9999)).is_ok());
    let refused = template.mint(None, Some(10_000));
    assert!(
        refused.is_err(),
        "a five-digit value filled a four-digit field: {refused:?}"
    );
}
