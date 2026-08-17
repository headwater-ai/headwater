// SPDX-License-Identifier: Apache-2.0
//! Whether the verb index is derived, or a snapshot somebody typed.
//!
//! A tamper that fails the gate and a clean tree that passes it are both
//! necessary and neither one decides this. A file with the right bytes frozen
//! into the emitter passes both. What decides it is a **change to a source**:
//! the artifact has to go stale when a verb is added to the dispatch table, and
//! when a document that describes a verb leaves the corpus. Both directions are
//! below, and the third test is the control — an edit to a document on another
//! shelf must move nothing.
//!
//! [#257](https://github.com/headwater-ai/headwater/issues/257) asks for the
//! failing direction rather than the passing one, and every test here that
//! asserts a failure was watched to fail before the emitter existed.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{check, plan, write, Identity, Kind, Plan, Projections, Runs};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_verbs::Verb;
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The output every declaration in the fixture taxonomy writes.
const OUTPUT: &str = "verbs/interfaces/README.md";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// A synthetic dispatch table: two verbs the fixture describes nothing of, one
/// it describes, and one with second words.
///
/// Synthetic on purpose. A test that read the real table would move every time
/// a verb was added, which is a property of the *committed* index rather than
/// of the emitter, and the last test here is where that property belongs.
fn four() -> Vec<Verb> {
    vec![
        Verb {
            name: "check",
            words: &[],
        },
        Verb {
            name: "gate",
            words: &[],
        },
        Verb {
            name: "sweep",
            words: &["plan", "report"],
        },
        Verb {
            name: "route",
            words: &[],
        },
    ]
}

struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    config: Config,
}

impl Built {
    fn over(corpus: &Corpus, root: &Mapping) -> Self {
        let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
        let relations = Declarations::read(root).expect("the declarations read");
        let shape = Shape::read(root).expect("the shape reads");
        let census = census::take(corpus, &taxonomy);
        let graph = Graph::build(
            &census,
            &relations,
            &Resolvers::over(corpus),
            corpus,
            &Config::default(),
        );
        Built {
            census,
            graph,
            shape,
            taxonomy,
            relations,
            config: Config::default(),
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn identity() -> Identity {
    Identity {
        corpus_root: "verbs".to_string(),
        exclusions: Vec::new(),
        package: "headwater/fixture".to_string(),
        version: "1.0.0".to_string(),
        lock: "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

/// A copy of the fixture corpus, so that a test may edit a source.
///
/// Never the fixture tree itself. A test that wrote its output beside its input
/// would leave a generated file that the next run's census walks.
fn copy_of(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("verb-index-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    copy_tree(&fixtures_dir().join("verbs"), &root.join("verbs"));
    root
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a temporary tree");
    for entry in std::fs::read_dir(from).expect("the fixture tree") {
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

/// The plan over one tree, with one dispatch table.
fn plan_over(root: &Path, verbs: &[Verb]) -> (Plan, Built) {
    let corpus = Corpus::new(root.to_path_buf(), "verbs");
    let taxonomy_source = load_map(&fixtures_dir().join("verbs.taxonomy.yml"));
    let built = Built::over(&corpus, &taxonomy_source);
    let projections = Projections::read(&taxonomy_source).expect("the projections read");
    let plan = {
        let surface = built.surface();
        plan(
            &surface,
            &built.census,
            &projections,
            &identity(),
            &Runs::default(),
            verbs,
        )
    };
    (plan, built)
}

fn index_of(plan: &Plan) -> &str {
    &plan
        .outputs
        .iter()
        .find(|output| output.kind == Kind::VerbIndex)
        .unwrap_or_else(|| {
            panic!(
                "no verb index in the plan. Unwritten: {:?}",
                plan.unwritten
                    .iter()
                    .map(|entry| &entry.reason)
                    .collect::<Vec<_>>()
            )
        })
        .bytes
}

fn rows(bytes: &str) -> Vec<&str> {
    bytes
        .lines()
        .filter(|line| line.starts_with("| `"))
        .collect()
}

#[test]
fn every_verb_gets_a_row_and_the_undescribed_ones_are_marked() {
    let root = copy_of("rows");
    let (plan, _built) = plan_over(&root, &four());
    let bytes = index_of(&plan);
    let rows = rows(bytes);
    assert_eq!(rows.len(), 4, "one row per verb of the table\n{bytes}");
    assert_eq!(
        rows.iter().filter(|row| row.contains("**no contract**")).count(),
        3,
        "three of the four verbs are described by nothing\n{bytes}"
    );
    assert!(
        rows[0].contains("[headwater check](headwater-check.md)"),
        "the described verb links to the document that describes it\n{bytes}"
    );
    // The forms are the command lines a caller may type, and a verb with second
    // words names no bare one.
    assert!(rows[2].contains("`headwater sweep plan`, `headwater sweep report`"));
}

/// The test that decides whether this artifact is derived.
///
/// A verb joins the dispatch table and nothing else moves. The committed bytes
/// have to stop being what this corpus derives, and `generate --check` has to
/// say so.
#[test]
fn a_verb_added_to_the_dispatch_table_makes_the_committed_index_stale() {
    let root = copy_of("added-verb");
    let (before, _built) = plan_over(&root, &four());
    let written = write(&root, &before);
    assert!(!written.has_errors(), "the first write");
    assert!(
        !check(&root, &before).has_errors(),
        "the tree this plan just wrote is not stale"
    );

    let mut wider = four();
    wider.push(Verb {
        name: "explain",
        words: &[],
    });
    let (after, _built) = plan_over(&root, &wider);
    assert_eq!(rows(index_of(&after)).len(), 5);

    let stale = check(&root, &after);
    assert!(
        stale.has_errors(),
        "a verb the committed index does not carry has to fail the gate:\n{}",
        stale.render()
    );
    assert!(
        stale.render().contains(OUTPUT),
        "the gate has to name the file:\n{}",
        stale.render()
    );
}

/// The other direction of the same property. A document leaves the corpus and
/// the cell it filled goes back to the mark.
#[test]
fn a_contract_removed_from_the_corpus_makes_the_committed_index_stale() {
    let root = copy_of("removed-contract");
    let (before, _built) = plan_over(&root, &four());
    assert!(!write(&root, &before).has_errors(), "the first write");
    assert!(index_of(&before).contains("[headwater check](headwater-check.md)"));

    std::fs::remove_file(root.join("verbs/interfaces/headwater-check.md")).expect("the removal");
    let (after, _built) = plan_over(&root, &four());
    let bytes = index_of(&after);
    assert!(
        !bytes.contains("headwater-check.md"),
        "the link has to go with the document\n{bytes}"
    );
    assert_eq!(
        rows(bytes).iter().filter(|row| row.contains("**no contract**")).count(),
        4,
        "every verb is now undescribed\n{bytes}"
    );
    let stale = check(&root, &after);
    assert!(
        stale.has_errors(),
        "the committed index describes a document that is gone:\n{}",
        stale.render()
    );
}

/// The control. Without it the two tests above prove only that *something*
/// moves the file, and a projection that regenerated on every edit would pass
/// them both.
#[test]
fn an_edit_to_a_document_on_another_shelf_moves_nothing() {
    let root = copy_of("unrelated-edit");
    let (before, _built) = plan_over(&root, &four());
    assert!(!write(&root, &before).has_errors(), "the first write");
    let bytes = index_of(&before).to_string();

    let note = root.join("verbs/notes/why.md");
    let text = std::fs::read_to_string(&note).expect("the note");
    std::fs::write(&note, format!("{text}\nA sentence somebody added.\n")).expect("the edit");

    let (after, _built) = plan_over(&root, &four());
    assert_eq!(bytes, index_of(&after), "an unrelated edit moved the index");
    assert!(
        !check(&root, &after).has_errors(),
        "an unrelated edit must not fail the gate"
    );
}

/// A description of a verb the binary does not dispatch. The index would drop
/// it, and a dropped description is worse than a missing one.
#[test]
fn a_document_that_describes_no_verb_declines_the_whole_file() {
    let root = copy_of("unknown-verb");
    std::fs::write(
        root.join("verbs/interfaces/headwater-wibble.md"),
        "---\nid: IF-FIX-headwater-wibble\ntitle: headwater wibble\nstatus: current\n\
         status_since: 2026-08-17\nsummary: a contract for a verb nothing dispatches\n---\n\n\
         # headwater wibble\n\nNothing dispatches this.\n",
    )
    .expect("the document");
    let (plan, _built) = plan_over(&root, &four());
    assert!(
        !plan.outputs.iter().any(|output| output.kind == Kind::VerbIndex),
        "a decline is whole, so no file is written"
    );
    let reason = plan
        .unwritten
        .iter()
        .find(|entry| entry.kind == Kind::VerbIndex)
        .map(|entry| entry.reason.clone())
        .expect("the decline is reported");
    assert!(reason.contains("headwater-wibble.md"), "{reason}");
    assert!(reason.contains("dispatches no such verb"), "{reason}");
}

/// A document on the shelf with nothing in the facet that carries the join.
#[test]
fn a_document_with_no_name_declines_the_whole_file() {
    let root = copy_of("no-name");
    std::fs::write(
        root.join("verbs/interfaces/nameless.md"),
        "---\nid: IF-FIX-nameless\nstatus: current\nstatus_since: 2026-08-17\n\
         summary: a contract that says which verb it describes nowhere\n---\n\n\
         # Nameless\n\nNothing here names a verb.\n",
    )
    .expect("the document");
    let (plan, _built) = plan_over(&root, &four());
    assert!(!plan.outputs.iter().any(|output| output.kind == Kind::VerbIndex));
    let reason = plan
        .unwritten
        .iter()
        .find(|entry| entry.kind == Kind::VerbIndex)
        .map(|entry| entry.reason.clone())
        .expect("the decline is reported");
    assert!(reason.contains("nameless.md"), "{reason}");
    assert!(reason.contains("title"), "{reason}");
}

/// The committed artifact of this repository, against the table of the binary
/// that a reader of it is holding.
///
/// `generate --check` already holds the bytes, and this holds the count that
/// #257 asks for by name: an index with two rows is a list of work already
/// done, whatever else is right about it.
#[test]
fn the_committed_index_carries_one_row_for_every_verb_this_binary_dispatches() {
    let path = repository_root().join("docs/interfaces/README.md");
    let bytes = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let rows = rows(&bytes);
    assert_eq!(
        rows.len(),
        headwater_verbs::VERBS.len(),
        "the committed index and the dispatch table carry different verbs"
    );
    for verb in headwater_verbs::VERBS {
        assert!(
            rows.iter().any(|row| row.starts_with(&format!("| `{}` |", verb.name))),
            "`{}` is dispatched and the committed index has no row for it",
            verb.name
        );
    }
    let undescribed = rows.iter().filter(|row| row.contains("**no contract**")).count();
    assert!(
        undescribed > 0,
        "an index whose every cell is filled is an index nobody needs to read"
    );
}
