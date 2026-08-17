// SPDX-License-Identifier: Apache-2.0
//! The fixture set that lets an imported edge carry full weight.
//!
//! [Q19](../../../../docs/spec/09-decisions.md#q19--inbound-integration-an-external-system-of-record)
//! rules that an imported edge carries full weight from the first release, and
//! its argument for that is not a posture. It is that "an importer is not a
//! rule. It is a producer of graph facts, and a wrong imported edge produces a
//! correct check result over a wrong graph. No advisory posture ever finds that.
//! The instrument is a fixture set over the importer." This file is that
//! instrument, and the weight rests on it rather than on the ruling.
//!
//! Each case provokes one refusal, and the cases are the whole set of ways a
//! link can be wrong. An importer that has refused nothing is an importer
//! nobody has seen work.
//!
//! The tree is built in a scratch directory rather than against this
//! repository's corpus, on the shape `engine/crates/resolve/tests/publish.rs`
//! set for the publishing verbs. Testing an importer against the corpus it will
//! type would measure this repository's taxonomy, which declares no relation an
//! importer may write, so every case would pass for one reason.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::shape::Shape;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_graph::Config;
use headwater_import::write::Unwritten;
use headwater_import::{Declaration, Proposed, Refusal};
use std::path::{Path, PathBuf};

/// A tree that removes itself, named for the case that made it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let at =
            std::env::temp_dir().join(format!("headwater-import-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Scratch(at)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the parent is made");
        std::fs::write(path, text).expect("the file is written");
    }

    fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.0.join(relative)).expect("the file is read")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The taxonomy the fixture corpus is typed and wired by.
///
/// `audited_by` is the one relation an importer may write. `cites_evidence` is
/// the authored one beside it, and it exists so that the refusal about a
/// creator has something true to be refused against. `governs` ends on a
/// document kind and never on an anchor, which is the third case.
const TAXONOMY: &str = "\
kinds:
  governed_document: {abstract: true}
  design_spec: {is_a: governed_document}
  evaluation: {is_a: governed_document}

shelves:
  spec_series:
    path: docs/spec/**
    homogeneous: true
    kind: design_spec
  evaluations:
    path: docs/evaluations/**
    homogeneous: true
    kind: evaluation

relations:
  audited_by:
    family: evidence
    from: [design_spec]
    to:   [ado_work_item]
    created_by: import
    attributes:
      verified_revision: {type: string, owner: edge}

  cites_evidence:
    family: evidence
    from: [design_spec]
    to:   [evaluation]
    created_by: author

  tracked_by:
    family: evidence
    from: [evaluation]
    to:   [ado_work_item]
    created_by: import

  governs:
    family: governance
    from: [design_spec]
    to:   [evaluation]
    created_by: import

anchors:
  ado_work_item: {resolver: ado-snapshot}
";

const PAYLOAD: &str = "\
snapshot:
  format: 1
  source: acme/work-items
  fetched: 2026-08-14
  items:
    - id: \"12345\"
      revision: \"7\"
      title: The service refuses an unauthenticated request
    - id: \"12346\"
      revision: \"2\"
  links:
    - from: SPEC-FIX-one
      relation: audited_by
      to: \"12345\"
    - from: SPEC-FIX-two
      relation: audited_by
      to: \"12346\"
";

/// A repository with two typed documents in it and one snapshot beside them.
///
/// The snapshot's own record is written the way a fetch script leaves one: the
/// digest is computed over the directory, and the caller is then handed it. That
/// is the position [HW-OBL-0115](../../../../docs/obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md)
/// describes, and the pin in `.headwater/taxonomy.yml` is what refuses a whole
/// re-publication of it.
fn tree(scratch: &Scratch, payload: &str) -> String {
    scratch.write(
        "docs/spec/00-first.md",
        "---\nid: SPEC-FIX-one\ndoc_type: design_spec\n---\n\n# The first part\n",
    );
    scratch.write(
        "docs/spec/01-second.md",
        "---\nid: SPEC-FIX-two\ndoc_type: design_spec\nrelations:\n  cites_evidence:\n    - EVAL-FIX-alpha\n---\n\n# The second part\n",
    );
    scratch.write(
        "docs/evaluations/alpha.md",
        "---\nid: EVAL-FIX-alpha\n---\n\n# An evaluation\n",
    );
    scratch.write("taxonomy.yml", TAXONOMY);
    scratch.write("imports/ado/snapshot.yml", payload);
    record(scratch)
}

/// Write the pin record over the snapshot directory, and return its digest.
fn record(scratch: &Scratch) -> String {
    let dir = scratch.path().join("imports/ado");
    let manifest = headwater_yaml::load("package: acme/work-items\nversion: \"2026-08-14\"\n")
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

/// The declaration a person authors, with the three fields an import rests on.
fn declaration(digest: Option<&str>, channel: Option<&str>) -> Declaration {
    Declaration {
        name: "ado".to_string(),
        at: "imports/ado".to_string(),
        digest: digest.map(str::to_string),
        channel: channel.map(str::to_string),
        resolver: Some(RESOLVER.to_string()),
    }
}

/// The name the fixture taxonomy's `anchors` block gives the resolver, and so
/// the name the import declaration has to supply for anything it writes to bind.
const RESOLVER: &str = "ado-snapshot";

fn channel() -> Option<&'static str> {
    Some("the platform team's fetch job, digest read from the ADO project release feed")
}

/// The corpus, as the importer reads it. Held together rather than returned
/// piecewise, because the three parts have to come from one walk.
struct Read {
    index: Index,
    relations: Declarations,
    shape: Shape,
}

fn corpus(scratch: &Scratch) -> Read {
    let source = scratch.read("taxonomy.yml");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy reads")
        .value
        .as_map()
        .expect("it is a mapping")
        .clone();
    let corpus = Corpus::new(scratch.path(), "docs");
    let taxonomy = Taxonomy::read(&root).expect("the shelves read");
    let taken = census::take(&corpus, &taxonomy);
    Read {
        index: Index::build(&taken, &Config::default()),
        relations: Declarations::read(&root).expect("the relations read"),
        shape: Shape::read(&root).expect("the kinds read"),
    }
}

impl Read {
    fn view(&self) -> headwater_import::Corpus<'_> {
        headwater_import::Corpus {
            index: &self.index,
            relations: &self.relations,
            shape: &self.shape,
        }
    }
}

/// Plan one import over a scratch tree, with whatever declaration the case has.
fn plan(
    scratch: &Scratch,
    declaration: &Declaration,
) -> Result<headwater_import::Plan, Vec<Refusal>> {
    let read = corpus(scratch);
    headwater_import::plan(scratch.path(), declaration, None, &read.view())
}

/// The whole verb, over a snapshot that is right.
///
/// It is the passing half of "a check without a failing fixture does not ship",
/// and it is also where the write is asserted: two documents take an edge each,
/// with the revision the snapshot pinned, and the second run over the same
/// snapshot writes nothing.
#[test]
fn a_pinned_snapshot_writes_one_edge_per_link_and_writes_it_once() {
    let scratch = Scratch::new("whole");
    let digest = tree(&scratch, PAYLOAD);
    let declaration = declaration(Some(&digest), channel());

    let first = plan(&scratch, &declaration).expect("it plans");
    assert_eq!(first.snapshot.source, "acme/work-items");
    assert_eq!(first.release.digest, digest);
    assert_eq!(
        first.edges,
        vec![
            Proposed {
                from: "SPEC-FIX-one".to_string(),
                path: "docs/spec/00-first.md".to_string(),
                relation: "audited_by".to_string(),
                to: "12345".to_string(),
                verified_revision: "7".to_string(),
                present: false,
            },
            Proposed {
                from: "SPEC-FIX-two".to_string(),
                path: "docs/spec/01-second.md".to_string(),
                relation: "audited_by".to_string(),
                to: "12346".to_string(),
                verified_revision: "2".to_string(),
                present: false,
            },
        ]
    );

    let pending = first.to_write();
    let composed = headwater_import::write::compose(scratch.path(), &pending).expect("it composes");
    headwater_import::write::apply(scratch.path(), &composed).expect("it writes");

    // The edge carries the revision, in the mapping form an instance attribute
    // needs, and the relation the document already declared survived it.
    let second = scratch.read("docs/spec/01-second.md");
    assert!(second.contains("audited_by:"), "{second}");
    assert!(second.contains("- to: 12346"), "{second}");
    assert!(second.contains("verified_revision: 2"), "{second}");
    assert!(second.contains("cites_evidence:"), "{second}");

    // The graph reads it back as one edge of a relation an importer pays for.
    let read = corpus(&scratch);
    let node = read.index.node("SPEC-FIX-two").expect("it is a node");
    assert_eq!(node.path, "docs/spec/01-second.md");

    // Second run, same snapshot: every edge is present and nothing is written.
    let again = plan(&scratch, &declaration).expect("it plans again");
    assert!(
        again.edges.iter().all(|edge| edge.present),
        "a second run over one snapshot proposes to write again: {:?}",
        again.edges
    );
    assert!(again.to_write().is_empty());
}

/// Lock a file against writing.
///
/// `compose` never sees this. Its only read of a document is a
/// `read_to_string`, which succeeds on a read-only file, so the failure this
/// provokes lands in the write and not one phase before it.
fn lock(at: &Path) {
    let mut permissions = std::fs::metadata(at)
        .expect("the file is there")
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(at, permissions).expect("the file locks");
}

/// One import over the fixture tree, with the second of its two documents
/// unwritable by the time the write runs.
///
/// The lock goes on **after** the compose, because a document `compose` cannot
/// read is `compose`'s own refusal and would stop the run one phase earlier
/// than the writer these cases are about. Returns the tree, what the *other*
/// document held before the write, and the refusal.
fn stopped(case: &str) -> (Scratch, String, Unwritten) {
    let scratch = Scratch::new(case);
    let digest = tree(&scratch, PAYLOAD);
    let planned = plan(&scratch, &declaration(Some(&digest), channel())).expect("it plans");
    let pending = planned.to_write();
    let composed = headwater_import::write::compose(scratch.path(), &pending).expect("it composes");
    assert_eq!(
        composed
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["docs/spec/00-first.md", "docs/spec/01-second.md"],
        "one run writes two documents, and the one locked below is the second of them"
    );

    let first = scratch.read("docs/spec/00-first.md");
    lock(&scratch.path().join("docs/spec/01-second.md"));
    let refused = headwater_import::write::apply(scratch.path(), &composed)
        .expect_err("the second document cannot be opened for writing");
    (scratch, first, refused)
}

/// An edge half nobody can open leaves every other document of the run as it
/// was.
///
/// The exposure here is width. One `headwater import --write` puts an edge half
/// into every document the snapshot names, and the loop this replaces wrote
/// them one at a time and stopped on its first error — leaving a corpus part
/// way through one import, with nothing on the tree to say which run stopped or
/// where.
///
/// **The *before* assertion is deliberately loose.** That `apply` returns an
/// error is true of the loop this replaces too, so making that the tight
/// assertion would attribute the case's failure to the wrong thing. The
/// decisive assertion is the first one below: the ambient outcome of this exact
/// scenario, measured against the writer this change replaces, is
/// `00-first.md` on the tree carrying its edge.
#[test]
fn an_edge_half_that_cannot_be_opened_leaves_every_other_document_as_it_was() {
    let (scratch, first, _) = stopped("write-half-written");

    assert_eq!(
        scratch.read("docs/spec/00-first.md"),
        first,
        "the other document of the run is as it was, and the writer this replaces had already put \
         its edge on the tree"
    );
    assert!(
        !scratch
            .read("docs/spec/01-second.md")
            .contains("audited_by"),
        "and so is the one that could not be opened"
    );
}

/// A run refused before it wrote says nothing was written, rather than saying
/// it stopped part way.
///
/// One sentence over two phases is true of neither. `headwater import --write`
/// printed *the write stopped part way* over every failure of the write,
/// including this one, where the run stopped before it started — the same fault
/// the scaffolder's own refusal carried when it printed *Nothing was written*
/// to an author whose tree had just gained a document.
#[test]
fn a_write_refused_before_it_started_says_nothing_was_written() {
    let (_scratch, _first, refused) = stopped("write-nothing-written");

    assert_eq!(refused.headline(), "nothing was written");
    assert_eq!(refused.path(), "docs/spec/01-second.md");
    assert!(matches!(refused, Unwritten::Unopened { .. }), "{refused}");
}

/// Nothing pins the snapshot, so the import refuses rather than records what it
/// received. `taxonomy vendor` takes the same position for the same reason.
#[test]
fn an_unpinned_snapshot_is_refused_rather_than_self_certified() {
    let scratch = Scratch::new("unpinned");
    tree(&scratch, PAYLOAD);
    assert_eq!(
        plan(&scratch, &declaration(None, channel())).expect_err("nothing pins it"),
        vec![Refusal::Unpinned {
            name: "ado".to_string()
        }]
    );
}

/// A digest with no channel beside it is refused.
///
/// This is the sharpest thing the issue asked for. A digest authenticates the
/// pin and never the publisher, so an edge imported against a digest alone
/// carries the authority of nothing anybody wrote down. The refusal is what
/// stops that from being the quiet default.
#[test]
fn a_digest_with_no_channel_beside_it_is_refused() {
    let scratch = Scratch::new("no-channel");
    let digest = tree(&scratch, PAYLOAD);
    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), None)).expect_err("nothing names a channel"),
        vec![Refusal::NoChannel {
            name: "ado".to_string()
        }]
    );
    // A channel that is present and empty is a channel nobody stated.
    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), Some("   "))).expect_err("it is blank"),
        vec![Refusal::NoChannel {
            name: "ado".to_string()
        }]
    );
}

/// A declaration that names no resolver is refused.
///
/// An import writes an edge onto an anchor kind, and only a resolver binds one.
/// A declaration with no resolver therefore writes edges that the next
/// `headwater check` reports as resolving to nothing, which is the whole of
/// [HW-OBL-0116](../../../../docs/obligations/0116-no-anchor-resolver-reads-a-committed-snapshot-so-every-imported-edge-lands-unresolved.md).
/// This refusal is what stops that state from being reachable by omission.
#[test]
fn a_declaration_that_names_no_resolver_is_refused() {
    let scratch = Scratch::new("no-resolver");
    let digest = tree(&scratch, PAYLOAD);
    let without = |resolver: Option<&str>| Declaration {
        resolver: resolver.map(str::to_string),
        ..declaration(Some(&digest), channel())
    };
    assert_eq!(
        plan(&scratch, &without(None)).expect_err("nothing names a resolver"),
        vec![Refusal::NoResolver {
            name: "ado".to_string()
        }]
    );
    // A resolver that is present and empty names nothing, on the terms the
    // channel refusal beside it takes.
    assert_eq!(
        plan(&scratch, &without(Some("  "))).expect_err("it is blank"),
        vec![Refusal::NoResolver {
            name: "ado".to_string()
        }]
    );
    let rendered = headwater_import::render(&[Refusal::NoResolver {
        name: "ado".to_string(),
    }]);
    assert!(rendered.contains("imports.ado.resolver"), "{rendered}");
}

/// A changed byte inside the snapshot is refused, and the file is named.
#[test]
fn a_changed_byte_is_refused_and_the_file_is_named() {
    let scratch = Scratch::new("tampered");
    let digest = tree(&scratch, PAYLOAD);

    // The revision of one item moves after the pin was written. Every edge into
    // it would record a revision the upstream never had.
    scratch.write(
        "imports/ado/snapshot.yml",
        &PAYLOAD.replace("\"7\"", "\"9\""),
    );

    let refusals =
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("the artifact moved");
    let rendered = headwater_import::render(&refusals);
    assert!(rendered.contains("snapshot.yml"), "{rendered}");
    assert!(rendered.contains("was published as"), "{rendered}");
}

/// A whole consistent re-publication is refused by the pin.
///
/// The record travels inside the artifact it describes, so a snapshot that was
/// rewritten and re-recorded passes every internal check. The pin is the only
/// thing that refuses it, and the pin is the number a person wrote down.
#[test]
fn a_consistent_forgery_is_refused_by_the_pin() {
    let scratch = Scratch::new("forgery");
    let digest = tree(&scratch, PAYLOAD);

    scratch.write(
        "imports/ado/snapshot.yml",
        &PAYLOAD.replace("\"7\"", "\"9\""),
    );
    let forged = record(&scratch);
    assert_ne!(forged, digest, "the forgery has to be a different artifact");

    // It verifies against itself, which is the whole point of the case.
    headwater_resolve::release::verify(&scratch.path().join("imports/ado"), &forged)
        .expect("the forgery is internally consistent");

    let refusals = plan(&scratch, &declaration(Some(&digest), channel()))
        .expect_err("it is not the pinned artifact");
    let rendered = headwater_import::render(&refusals);
    assert!(rendered.contains("this repository pins"), "{rendered}");
    assert!(rendered.contains(&digest), "{rendered}");
    assert!(rendered.contains(&forged), "{rendered}");
}

/// A file smuggled into the snapshot directory is refused.
#[test]
fn a_file_the_record_does_not_name_is_refused() {
    let scratch = Scratch::new("smuggled");
    let digest = tree(&scratch, PAYLOAD);
    scratch.write("imports/ado/extra.yml", "snapshot: {}\n");

    let refusals = plan(&scratch, &declaration(Some(&digest), channel()))
        .expect_err("the artifact grew a file");
    let rendered = headwater_import::render(&refusals);
    assert!(rendered.contains("extra.yml"), "{rendered}");
    assert!(
        rendered.contains("the record names no such member"),
        "{rendered}"
    );
}

/// A relation the taxonomy does not declare never reaches the graph.
///
/// This is the correctness-root case in its plainest form. Nothing downstream
/// could find it: a front-matter key that names no relation is reported by the
/// graph build, but a wrong relation *name* that happens to be declared would
/// not be, and neither would land as the edge the snapshot meant.
#[test]
fn a_link_whose_relation_is_not_declared_is_refused() {
    let scratch = Scratch::new("no-relation");
    let digest = tree(
        &scratch,
        &PAYLOAD.replace(
            "relation: audited_by\n      to: \"12345\"",
            "relation: audited\n      to: \"12345\"",
        ),
    );

    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("no such relation"),
        vec![Refusal::UnknownRelation {
            relation: "audited".to_string(),
            from: "SPEC-FIX-one".to_string(),
            to: "12345".to_string(),
        }]
    );
}

/// A relation an importer may not pay for is refused.
///
/// Q4 puts `created_by` on the relation type, so the relation is the only grain
/// at which the graph separates an imported edge from an authored one. Writing
/// an imported edge onto an authored relation would spend that distinction, and
/// no later reader could recover it.
#[test]
fn a_relation_an_importer_may_not_pay_for_is_refused() {
    let scratch = Scratch::new("wrong-creator");
    let digest = tree(
        &scratch,
        &PAYLOAD.replace(
            "relation: audited_by\n      to: \"12345\"",
            "relation: cites_evidence\n      to: \"12345\"",
        ),
    );

    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("an author pays for it"),
        vec![Refusal::NotAnImportRelation {
            relation: "cites_evidence".to_string(),
            created_by: Some("author".to_string()),
        }]
    );
}

/// A relation whose far end admits no anchor is refused.
///
/// `governs` declares `created_by: import` here, which is a taxonomy a
/// validator accepts. It ends on a document kind, so no external item can sit at
/// its far end, and an import of it would produce an edge whose target resolves
/// to nothing at all.
#[test]
fn a_relation_that_ends_on_no_anchor_is_refused() {
    let scratch = Scratch::new("no-anchor");
    let digest = tree(
        &scratch,
        &PAYLOAD.replace(
            "relation: audited_by\n      to: \"12345\"",
            "relation: governs\n      to: \"12345\"",
        ),
    );

    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("it ends on a document"),
        vec![Refusal::NoAnchorEnd {
            relation: "governs".to_string(),
            permitted: vec!["evaluation".to_string()],
        }]
    );
}

/// A near end that names no document of this corpus is refused.
#[test]
fn a_link_out_of_a_document_that_does_not_exist_is_refused() {
    let scratch = Scratch::new("no-source");
    let digest = tree(&scratch, &PAYLOAD.replace("SPEC-FIX-one", "SPEC-FIX-nine"));

    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("no such document"),
        vec![Refusal::NoSuchSource {
            from: "SPEC-FIX-nine".to_string(),
            relation: "audited_by".to_string(),
            to: "12345".to_string(),
        }]
    );
}

/// A near end whose kind the relation does not admit is refused.
///
/// `tracked_by` runs from an evaluation and `audited_by` runs from a design
/// spec, and both are relations an importer may write. So this case separates
/// the creator rule from the endpoint rule rather than passing on either one.
#[test]
fn a_link_out_of_a_kind_the_relation_refuses_is_refused() {
    let scratch = Scratch::new("wrong-kind");
    let digest = tree(
        &scratch,
        &PAYLOAD.replace(
            "from: SPEC-FIX-one\n      relation: audited_by",
            "from: EVAL-FIX-alpha\n      relation: audited_by",
        ),
    );

    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("the kind is refused"),
        vec![Refusal::SourceKindRefused {
            from: "EVAL-FIX-alpha".to_string(),
            kind: "evaluation".to_string(),
            relation: "audited_by".to_string(),
            permitted: vec!["design_spec".to_string()],
        }]
    );
}

/// A relation that runs from an abstract kind admits every kind under it.
///
/// The refusal above has to be the endpoint rule and not a string comparison,
/// so this is the same shape with the `is_a` chain in the way. Without
/// `descends_from` this case fails and the one above passes, which is how an
/// endpoint rule that is really an equality test looks from outside.
#[test]
fn a_relation_from_an_abstract_kind_admits_the_kinds_under_it() {
    let scratch = Scratch::new("abstract-kind");
    let digest = tree(&scratch, PAYLOAD);
    scratch.write(
        "taxonomy.yml",
        &TAXONOMY.replace(
            "    from: [design_spec]\n    to:   [ado_work_item]\n    created_by: import\n    attributes:",
            "    from: [governed_document]\n    to:   [ado_work_item]\n    created_by: import\n    attributes:",
        ),
    );

    let composed = plan(&scratch, &declaration(Some(&digest), channel()))
        .expect("a design_spec descends from governed_document");
    assert_eq!(composed.edges.len(), 2);
}

/// A far end the snapshot does not pin is refused.
///
/// An edge into an item the snapshot never named would record no revision, and
/// Q19's drift report is per edge. The anchor resolver would bind the string
/// happily, which is exactly why the importer has to be the one that refuses.
#[test]
fn a_link_into_an_item_the_snapshot_does_not_hold_is_refused() {
    let scratch = Scratch::new("no-item");
    let digest = tree(
        &scratch,
        &PAYLOAD.replace("      to: \"12345\"", "      to: \"99999\""),
    );

    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("no such item"),
        vec![Refusal::NoSuchItem {
            to: "99999".to_string(),
            from: "SPEC-FIX-one".to_string(),
            relation: "audited_by".to_string(),
        }]
    );
}

/// One snapshot naming one triple twice is refused.
#[test]
fn a_repeated_link_is_refused() {
    let scratch = Scratch::new("repeat");
    let digest = tree(
        &scratch,
        &format!(
            "{PAYLOAD}    - from: SPEC-FIX-one\n      relation: audited_by\n      to: \"12345\"\n"
        ),
    );

    assert_eq!(
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("it is named twice"),
        vec![Refusal::RepeatedLink {
            from: "SPEC-FIX-one".to_string(),
            relation: "audited_by".to_string(),
            to: "12345".to_string(),
        }]
    );
}

/// Every refusal is collected, so one fetch is one report.
///
/// A snapshot with three wrong links in it reports three times. Returning the
/// first one would turn one fetch to redo into three.
#[test]
fn every_wrong_link_is_reported_and_not_only_the_first() {
    let scratch = Scratch::new("all-of-them");
    let payload = PAYLOAD.replace("SPEC-FIX-one", "SPEC-FIX-nine").replace(
        "    - from: SPEC-FIX-two\n      relation: audited_by\n      to: \"12346\"\n",
        "    - from: SPEC-FIX-two\n      relation: audited_by\n      to: \"99999\"\n\
         \x20   - from: SPEC-FIX-two\n      relation: cites_evidence\n      to: \"12346\"\n",
    );
    let digest = tree(&scratch, &payload);

    let refusals = plan(&scratch, &declaration(Some(&digest), channel())).expect_err("three wrong");
    assert_eq!(refusals.len(), 3, "{refusals:?}");
    assert!(refusals
        .iter()
        .any(|refusal| matches!(refusal, Refusal::NoSuchSource { .. })));
    assert!(refusals
        .iter()
        .any(|refusal| matches!(refusal, Refusal::NoSuchItem { .. })));
    assert!(refusals
        .iter()
        .any(|refusal| matches!(refusal, Refusal::NotAnImportRelation { .. })));
}

/// An import refused at the plan writes nothing at all.
///
/// The refusals above are worth what this test says they are worth. An importer
/// that reported a wrong link and wrote the rest would leave a corpus that is
/// part imported, and the next run over the corrected snapshot would have no
/// way to tell which half it was looking at.
///
/// **The name says *at the plan* because that is the whole of what this holds.**
/// `plan` refuses before `compose` and `apply` are called at all, so the tree
/// this asserts about is a tree no writer has touched under any implementation
/// of the writer — including the loop that stopped part way. The case that
/// holds the writer is
/// [`an_edge_half_that_cannot_be_opened_leaves_every_other_document_as_it_was`],
/// which reaches `apply` and asserts the opposite of that loop's outcome.
#[test]
fn a_link_refused_at_the_plan_leaves_every_document_as_it_found_it() {
    let scratch = Scratch::new("plan-refused");
    let digest = tree(&scratch, &PAYLOAD.replace("SPEC-FIX-one", "SPEC-FIX-nine"));
    let before = scratch.read("docs/spec/01-second.md");

    plan(&scratch, &declaration(Some(&digest), channel())).expect_err("one link is wrong");
    assert_eq!(before, scratch.read("docs/spec/01-second.md"));
    assert!(!scratch
        .read("docs/spec/01-second.md")
        .contains("audited_by"));
}

/// A snapshot payload with no fetch time is refused, and the directory is not
/// a snapshot at all without one.
#[test]
fn a_directory_with_no_payload_is_not_a_snapshot() {
    let scratch = Scratch::new("no-payload");
    tree(&scratch, PAYLOAD);
    std::fs::remove_file(scratch.path().join("imports/ado/snapshot.yml")).expect("it is removed");
    let digest = record(&scratch);

    let refusals =
        plan(&scratch, &declaration(Some(&digest), channel())).expect_err("there is no payload");
    let rendered = headwater_import::render(&refusals);
    assert!(rendered.contains("carries no snapshot.yml"), "{rendered}");
}

/// The declaration is read out of `.headwater/taxonomy.yml` and never written.
#[test]
fn an_import_is_declared_beside_the_taxonomy_pin() {
    let scratch = Scratch::new("declared");
    scratch.write(
        ".headwater/taxonomy.yml",
        "\
taxonomy:
  package: acme/fixture
  version: 1.0.0
corpus:
  root: docs
imports:
  ado:
    at: imports/ado
    digest: sha256:0
    channel: the platform team's fetch job
    resolver: ado-snapshot
",
    );
    assert_eq!(
        headwater_import::declared(scratch.path()).expect("it reads"),
        vec![Declaration {
            name: "ado".to_string(),
            at: "imports/ado".to_string(),
            digest: Some("sha256:0".to_string()),
            channel: Some("the platform team's fetch job".to_string()),
            resolver: Some("ado-snapshot".to_string()),
        }]
    );

    // A repository that declares no import declares none, rather than failing.
    let empty = Scratch::new("declared-none");
    empty.write(
        ".headwater/taxonomy.yml",
        "taxonomy:\n  package: acme/fixture\n  version: 1.0.0\ncorpus:\n  root: docs\n",
    );
    assert_eq!(
        headwater_import::declared(empty.path()).expect("it reads"),
        Vec::new()
    );
}
