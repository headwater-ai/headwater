// SPDX-License-Identifier: Apache-2.0
//! The templates a package ships, held to the taxonomy it ships them with.
//!
//! Every case here provokes the refusal it is about, over a package on disk,
//! through the verb an adopter's publisher runs. The one live defect this reader
//! was filed for — `spec_layer: functional` against a facet declaring
//! `functional_spec` — was fixed before
//! [PR #376](https://github.com/headwater-ai/headwater/pull/376) merged, so the
//! decisive case is built here rather than found in the corpus. The last case in
//! the file is the other half: this repository's own library, held to the same
//! reader, so that a template nobody expected to trip stays silent.

use headwater_resolve::package;
use headwater_resolve::template;
use std::path::{Path, PathBuf};

/// A tree that removes itself, named for the case that made it.
///
/// `cargo` runs the cases of one target as threads of one process, so the
/// process id alone is not a key. The case name is what separates them.
struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let at =
            std::env::temp_dir().join(format!("headwater-templates-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Scratch(at)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, text: &str) {
        self.write_bytes(relative, text.as_bytes());
    }

    fn write_bytes(&self, relative: &str, bytes: &[u8]) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the parent is made");
        std::fs::write(path, bytes).expect("the file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const MANIFEST: &str = "\
package: acme/fixture
version: 1.0.0
contents:
  taxonomy: taxonomy.yml
  bundles: bundles
";

const TAXONOMY: &str = "\
taxonomy: acme/fixture
version: 1.0.0
purposes:
  rationale: {intent: explain why a choice was made and what it forecloses}
";

/// One bundle, in the shape the `standards-spec` entry has: an enumerated facet,
/// a heterogeneous shelf that discriminates on it, one kind that requires it and
/// one kind that forbids it.
const BUNDLE: &str = "\
bundle: specs
extends: acme/fixture
add:
  facets:
    status:
      role: state
      values: [draft, current]
      volatility: mutable
    spec_layer:
      type: string
      values: [functional_spec, technical_spec]
      volatility: stable
  kinds:
    governed_document: {abstract: true}
    functional_spec: {is_a: governed_document, purpose: rationale}
    technical_spec: {is_a: governed_document, purpose: rationale}
    standard: {is_a: governed_document, purpose: rationale, facets: {forbid: [spec_layer]}}
  relations:
    supersedes:
      family: succession
      from: [governed_document]
      to:   [governed_document]
      inverse: superseded_by
      reciprocal: required
      created_by: scaffold
  shelves:
    component_specs:
      path: docs/component-specs/**
      homogeneous: false
      discriminator: spec_layer
      kinds: [functional_spec, technical_spec]
    standards:
      path: docs/standards/**
      homogeneous: true
      kind: standard
";

/// The template the issue measured, verbatim: the layer name where the facet
/// declares the kind name.
const BAD_LAYER: &str = "\
---
status: draft
spec_layer: functional
---

# {{the component}}
";

/// The same template as the library ships it, with the value the facet declares.
const GOOD_LAYER: &str = "\
---
status: draft
status_since: \"{{today}}\"
spec_layer: functional_spec
---

# {{the component}}
";

/// A package that ships one bundle, with the templates a case plants in it.
fn publisher(scratch: &Scratch, templates: &[(&str, &str)]) -> PathBuf {
    scratch.write("publisher/.headwater/packages/acme-fixture/package.yml", MANIFEST);
    scratch.write("publisher/.headwater/packages/acme-fixture/taxonomy.yml", TAXONOMY);
    scratch.write(
        "publisher/.headwater/packages/acme-fixture/bundles/specs/bundle.yml",
        BUNDLE,
    );
    for (name, body) in templates {
        scratch.write(
            &format!("publisher/.headwater/packages/acme-fixture/bundles/specs/templates/{name}"),
            body,
        );
    }
    scratch.path().join("publisher")
}

/// Publish the fixture and hand back what the refusal said.
fn refused(root: &Path, out: &Path) -> String {
    let errors = package::publish_from(root, &root.join(".headwater/packages/acme-fixture"), out)
        .expect_err("the publish is refused");
    headwater_resolve::render_errors(&errors)
}

/// Every branch this reader reported over one package.
fn variants(root: &Path) -> Vec<&'static str> {
    let directory = root.join(".headwater/packages/acme-fixture");
    let manifest = package::manifest_at(&directory).expect("the manifest reads");
    let taxonomy = package::maximal(root, &directory).expect("the shipped set resolves");
    template::refusals(root, &directory, &manifest, &taxonomy.taxonomy)
        .iter()
        .map(|one| one.reason.variant())
        .collect()
}

/// **The decisive case.** A facet value outside the declared set is refused, and
/// the refusal names the file, the value, the permitted set and what a document
/// carrying it loses.
///
/// This is the defect [#378](https://github.com/headwater-ai/headwater/issues/378)
/// was filed for, in the shape it shipped in: `spec_layer: functional` against
/// `values: [functional_spec, technical_spec]`. Kind resolution matches a
/// discriminator against the shelf's kinds by identity, so a document carrying
/// it is untyped, no generated rule reads it, and `headwater check --strict`
/// exits 0 over it. Before this reader existed the publish below exited 0 and
/// wrote the artifact with the bad template inside it.
#[test]
fn a_facet_value_outside_the_declared_set_is_refused_at_publish() {
    let scratch = Scratch::new("bad-layer");
    let root = publisher(&scratch, &[("functional_spec.md", BAD_LAYER)]);
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("bundles/specs/templates/functional_spec.md"),
        "the refusal does not name the file a person edits:\n{message}"
    );
    assert!(
        message.contains("`spec_layer: functional` is not a value `facets.spec_layer` declares"),
        "the refusal does not name the value and the facet:\n{message}"
    );
    assert!(
        message.contains("(`functional_spec`, `technical_spec`)"),
        "the refusal does not name the permitted set:\n{message}"
    );
    assert!(
        message.contains(
            "a document carrying this value resolves to no kind, so no rule reads it and no \
             check reports it"
        ),
        "the refusal does not name the consequence, which is what makes it a refusal rather \
         than a style note:\n{message}"
    );
    assert!(
        !out.exists(),
        "the refused publish left an output directory behind"
    );
}

/// Front matter no loader accepts is its own refusal, and the message names the
/// repair.
///
/// Skipping it would leave the reader with a documented bypass: a publisher who
/// wants none of the checks above ships a template that does not parse and the
/// verb reports a pass. Six of this repository's own thirteen templates were in
/// this state when the reader landed, which spec 13 recorded as *No template
/// parses*.
#[test]
fn front_matter_that_does_not_parse_is_refused_at_publish() {
    let scratch = Scratch::new("bare-placeholder");
    let root = publisher(
        &scratch,
        &[(
            "functional_spec.md",
            "---\nstatus: draft\nstatus_since: {{today}}\nspec_layer: functional_spec\n---\n\n# x\n",
        )],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("the front matter of this template does not parse"),
        "the refusal does not say what it could not read:\n{message}"
    );
    assert!(
        message.contains("a quoted placeholder parses"),
        "the refusal does not name the repair:\n{message}"
    );
    assert!(!out.exists(), "the refused publish wrote an artifact");
}

/// A file name that names no kind is refused, because the file name is what
/// states the kind a template teaches.
#[test]
fn a_template_whose_name_is_not_a_kind_is_refused_at_publish() {
    let scratch = Scratch::new("unknown-kind");
    let root = publisher(&scratch, &[("widget.md", GOOD_LAYER)]);
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("`widget` is not a kind this package declares"),
        "the refusal does not name the kind it could not find:\n{message}"
    );
    assert!(!out.exists(), "the refused publish wrote an artifact");
}

/// A file name that names an abstract kind is refused. Spec 2 says no document
/// is one, so a template teaching one teaches a document that resolves to
/// nothing.
#[test]
fn a_template_that_teaches_an_abstract_kind_is_refused_at_publish() {
    let scratch = Scratch::new("abstract-kind");
    let root = publisher(
        &scratch,
        &[("governed_document.md", "---\nstatus: draft\n---\n\n# x\n")],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("`governed_document` is abstract"),
        "the refusal does not name the abstract kind:\n{message}"
    );
    assert!(!out.exists(), "the refused publish wrote an artifact");
}

/// A discriminator whose value is in the set and is not the kind the file name
/// states is refused. This is what makes the value-set check not enough on its
/// own.
#[test]
fn a_discriminator_that_disagrees_with_the_file_name_is_refused_at_publish() {
    let scratch = Scratch::new("disagreeing-discriminator");
    let root = publisher(
        &scratch,
        &[(
            "functional_spec.md",
            "---\nstatus: draft\nspec_layer: technical_spec\n---\n\n# x\n",
        )],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message
            .contains("`spec_layer: technical_spec` disagrees with the kind this template teaches"),
        "the refusal does not name the disagreement:\n{message}"
    );
    assert!(
        message.contains("`functional_spec`"),
        "the refusal does not name the kind the file name states:\n{message}"
    );
    assert!(!out.exists(), "the refused publish wrote an artifact");
}

/// A facet the kind forbids is refused on presence, and a placeholder does not
/// excuse it.
///
/// `standards-spec/templates/standard.md` is the one live template that
/// exercises this check positively — it carries no `spec_layer` and its kind
/// forbids one — so the negative half is built here.
#[test]
fn a_facet_the_kind_forbids_is_refused_at_publish() {
    let scratch = Scratch::new("forbidden-facet");
    let root = publisher(
        &scratch,
        &[(
            "standard.md",
            "---\nstatus: draft\nspec_layer: \"{{functional_spec | technical_spec}}\"\n---\n\n# x\n",
        )],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("`spec_layer` is a facet `kinds.standard` forbids"),
        "the refusal does not name the facet and the kind:\n{message}"
    );
    assert!(!out.exists(), "the refused publish wrote an artifact");
}

/// A relation written as a top-level key is refused, and the inverse half of a
/// relation is refused on the same ground as the declared half.
///
/// This is the defect [#507](https://github.com/headwater-ai/headwater/issues/507)
/// was filed for, in the shape it shipped in: three templates of this
/// repository's library wrote four relation names at the top level, and the
/// artifact an adopter vendors carried all four.
/// [HW-DR-0004](../../../../docs/decisions/0004-relation-storage.md) puts a
/// relation under a `relations:` block and nowhere else, so an edge written
/// beside the facets is an edge the graph never sees.
///
/// **Both halves are planted, because two of the four live keys were inverse
/// halves.** `applies` and `cited_by` are not keys of any `relations:` map — a
/// bundle names them as the `inverse` of `applied_in` and `cites_evidence` — so
/// a reader that walks the keys alone refuses two of the four and looks like a
/// working check.
#[test]
fn a_relation_written_at_the_top_level_is_refused_at_publish() {
    let scratch = Scratch::new("relation-at-top-level");
    let root = publisher(
        &scratch,
        &[(
            "standard.md",
            "---\nstatus: draft\nsupersedes:\n  - \"{{what this replaces}}\"\nsuperseded_by: \
             \"{{what replaces this}}\"\n---\n\n# x\n",
        )],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("`supersedes` is a relation this package declares"),
        "the refusal does not name the relation the taxonomy declares:\n{message}"
    );
    assert!(
        message.contains("`superseded_by` is a relation this package declares"),
        "the refusal does not name the inverse half:\n{message}"
    );
    assert!(
        message.contains("under a `relations:` block"),
        "the refusal does not say where the key belongs:\n{message}"
    );
    assert!(
        message.contains("bundles/specs/templates/standard.md"),
        "the refusal does not name the file a person edits:\n{message}"
    );
    assert!(!out.exists(), "the refused publish wrote an artifact");
}

/// A template that cannot be read as text at all is refused rather than skipped.
#[test]
fn a_template_that_is_not_text_is_refused_at_publish() {
    let scratch = Scratch::new("not-text");
    let root = publisher(&scratch, &[]);
    scratch.write_bytes(
        "publisher/.headwater/packages/acme-fixture/bundles/specs/templates/functional_spec.md",
        &[0xff, 0xfe, 0x00, 0x9f],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("this template cannot be read"),
        "the refusal does not say the file could not be opened as text:\n{message}"
    );
    assert!(!out.exists(), "the refused publish wrote an artifact");
}

/// Every bad template is reported, not the first one.
///
/// `reachable` and `agrees` collect on the same path for the reason `reachable`
/// states: a second run should not have to discover the second defect. A
/// publisher fixing a library entry reads one report.
#[test]
fn every_bad_template_is_reported_rather_than_the_first() {
    let scratch = Scratch::new("two-bad");
    let root = publisher(
        &scratch,
        &[("functional_spec.md", BAD_LAYER), ("widget.md", GOOD_LAYER)],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("templates/functional_spec.md"),
        "the first bad template is not reported:\n{message}"
    );
    assert!(
        message.contains("templates/widget.md"),
        "the second bad template is not reported, so a second run would have to find it:\n\
         {message}"
    );
}

/// A template that agrees with the bundle beside it publishes, and reaches the
/// artifact.
///
/// The reader is not trivially *refuse everything*: a placeholder is skipped
/// rather than typed, a facet with no enumerated set is not held to one, and a
/// kind that forbids a facet the template does not carry is silent.
#[test]
fn a_template_that_agrees_with_its_bundle_publishes() {
    let scratch = Scratch::new("good");
    let root = publisher(
        &scratch,
        &[
            ("functional_spec.md", GOOD_LAYER),
            (
                "standard.md",
                "---\nstatus: current\nstatus_since: \"{{today}}\"\n---\n\n# x\n",
            ),
        ],
    );
    let out = scratch.path().join("artifact");

    let record = package::publish_from(&root, &root.join(".headwater/packages/acme-fixture"), &out)
        .expect("a package whose templates agree with its bundles publishes");
    let paths: Vec<&str> = record.members.iter().map(|m| m.path.as_str()).collect();
    assert!(
        paths.contains(&"bundles/specs/templates/functional_spec.md"),
        "the template did not reach the artifact: {paths:?}"
    );
    assert!(
        variants(&root).is_empty(),
        "the reader refused a template the publish accepted"
    );
}

/// A Markdown file that declares no front matter, and whose own stem names no
/// kind, is prose beside the templates, and this reader says nothing about it.
///
/// Spec 7's own example manifest ships `templates/note.md` and its comment calls
/// it prose for a publisher's own authors. Every check here reads front matter,
/// so a file that declares none takes no check away — which is what separates it
/// from a block that opens and does not parse, where the declarations exist and
/// nothing can read them. **This silence is scoped to the stem, not just to the
/// block**: see `a_markdown_file_with_no_front_matter_and_a_known_kind_stem_is_refused_at_publish`
/// below for the case where the stem does name a kind, which refuses instead.
#[test]
fn a_markdown_file_with_no_front_matter_is_not_a_template() {
    let scratch = Scratch::new("prose-beside");
    let root = publisher(
        &scratch,
        &[
            (
                "note.md",
                "# Templates\n\nProse for a publisher's authors.\n",
            ),
            ("functional_spec.md", GOOD_LAYER),
        ],
    );
    let out = scratch.path().join("artifact");

    package::publish_from(&root, &root.join(".headwater/packages/acme-fixture"), &out)
        .expect("prose beside the templates is not a template");
}

/// A file whose stem names a real kind is a template's own claim to teach it,
/// and that claim is checkable even with no front-matter block at all.
///
/// This is the case verification found this reader silent over: a leading blank
/// line before an otherwise-valid `---` block makes `headwater-doc` report
/// `NoFrontMatter`, identically to a file that never had a block, and the naive
/// rule — skip whenever the parser reports no front matter — cannot tell them
/// apart. Skipping on stem-blindness there would leave every one of the four
/// checks silently off for a file a publisher (or an accident) can trigger with
/// one newline. This case is the bare version, with no block at all; the
/// leading-blank-line version is the next case below.
#[test]
fn a_markdown_file_with_no_front_matter_and_a_known_kind_stem_is_refused_at_publish() {
    let scratch = Scratch::new("no-front-matter-known-kind");
    let root = publisher(
        &scratch,
        &[(
            "functional_spec.md",
            "# functional_spec\n\nNo front matter here at all.\n",
        )],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("functional_spec") && message.contains("no front-matter block"),
        "the refusal does not name the kind and the missing block:\n{message}"
    );
}

/// The exact bypass verification measured: a leading blank line hides a
/// perfectly normal front-matter block from the parser, and the file underneath
/// carries the same defect this reader exists to catch.
///
/// Before this fix, `headwater-doc` reported `NoFrontMatter` for this file
/// (identical to a file with no block at all), the reader skipped it on stem
/// alone, and `taxonomy publish` exited 0 with the bad template inside the
/// artifact. This case is the regression guard: it must refuse, and it must
/// refuse for the missing-block reason rather than silently disappearing again.
#[test]
fn a_leading_blank_line_does_not_hide_a_template_from_the_reader() {
    let scratch = Scratch::new("leading-blank-line");
    let root = publisher(
        &scratch,
        &[(
            "technical_spec.md",
            "\n---\nstatus: draft\nspec_layer: functional_spec\n---\n\n# x\n",
        )],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("technical_spec") && message.contains("no front-matter block"),
        "the leading blank line hid the template again:\n{message}"
    );
}

/// A front-matter block that opens and never closes is refused, and it is the
/// line between the case above and the parse refusal.
#[test]
fn a_front_matter_block_that_never_closes_is_refused_at_publish() {
    let scratch = Scratch::new("unterminated");
    let root = publisher(
        &scratch,
        &[(
            "functional_spec.md",
            "---\nstatus: draft\nspec_layer: functional_spec\n\n# x\n",
        )],
    );
    let out = scratch.path().join("artifact");

    let message = refused(&root, &out);
    assert!(
        message.contains("never closed"),
        "the refusal does not say the block was left open:\n{message}"
    );
}

/// A bundle directory with no `bundle.yml` ships templates nothing resolves, and
/// the reader skips it on the rule `package::shipped` already applies.
///
/// The taxonomy this reader holds a template against is the base with every
/// bundle a consumer can select. A directory that declares no bundle is not one
/// of them, so its templates are outside the set the reader can say anything
/// about, and refusing them would be a rule about a directory nothing reads.
#[test]
fn a_directory_that_declares_no_bundle_is_not_walked() {
    let scratch = Scratch::new("no-bundle-yml");
    let root = publisher(&scratch, &[("functional_spec.md", GOOD_LAYER)]);
    scratch.write(
        "publisher/.headwater/packages/acme-fixture/bundles/README.md",
        "# The bundles this package ships\n",
    );
    scratch.write(
        "publisher/.headwater/packages/acme-fixture/bundles/draft/templates/functional_spec.md",
        BAD_LAYER,
    );
    let out = scratch.path().join("artifact");

    package::publish_from(&root, &root.join(".headwater/packages/acme-fixture"), &out)
        .expect("a directory that declares no bundle is not a bundle");
}

/// Every branch of `template::Reason` is reached by a case of this file.
///
/// `BRANCHES` is written by the `branches!` macro out of the same list the
/// `variant` match reads, so no branch can be left out of it. PR #461 found what
/// two hand-kept lists cost: three variants were in the match and absent from
/// the constant, and the coverage test claimed totality it did not have.
#[test]
fn every_refusal_branch_has_a_case() {
    let scratch = Scratch::new("coverage");
    let mut reached: Vec<&'static str> = Vec::new();

    for (name, body) in [
        ("functional_spec.md", BAD_LAYER),
        ("widget.md", GOOD_LAYER),
        ("governed_document.md", "---\nstatus: draft\n---\n\n# x\n"),
        (
            "technical_spec.md",
            "---\nstatus: draft\nspec_layer: functional_spec\n---\n\n# x\n",
        ),
        (
            "standard.md",
            "---\nstatus: draft\nspec_layer: technical_spec\n---\n\n# x\n",
        ),
        (
            "functional_spec.md",
            "# functional_spec\n\nNo front matter here at all.\n",
        ),
        (
            "standard.md",
            "---\nstatus: draft\nsuperseded_by: \"{{what replaces this}}\"\n---\n\n# x\n",
        ),
    ] {
        let root = publisher(&scratch, &[(name, body)]);
        reached.extend(variants(&root));
        let _ = std::fs::remove_dir_all(
            scratch
                .path()
                .join("publisher/.headwater/packages/acme-fixture/bundles/specs/templates"),
        );
    }

    let root = publisher(&scratch, &[]);
    scratch.write(
        "publisher/.headwater/packages/acme-fixture/bundles/specs/templates/functional_spec.md",
        "---\nstatus_since: {{today}}\n---\n\n# x\n",
    );
    reached.extend(variants(&root));
    scratch.write_bytes(
        "publisher/.headwater/packages/acme-fixture/bundles/specs/templates/functional_spec.md",
        &[0xff, 0xfe],
    );
    reached.extend(variants(&root));

    let missing: Vec<&&str> = template::BRANCHES
        .iter()
        .filter(|branch| !reached.contains(branch))
        .collect();
    assert!(
        missing.is_empty(),
        "these refusal branches have no case: {missing:?}"
    );
}

/// The library this repository ships passes its own reader.
///
/// This is the half of the change that a synthetic fixture cannot state. A rule
/// that stops a thing from existing cannot be scoped to that thing: the reader
/// is general over every template in every bundle a package ships, so a template
/// nobody expected to trip has to stay silent.
///
/// **A count of them is asserted rather than written down.** Seventeen templates
/// across six bundles reached this case on the commit that added the sentence,
/// and nothing read either number, so a bundle that shipped no template at all
/// would have left the case green and the sentence true-looking. The `walked`
/// assertion below is what holds both, and it fails on the number rather than on
/// the silence.
#[test]
fn the_library_this_repository_ships_passes_its_own_reader() {
    // Canonical, so that the paths a refusal names are repository-relative:
    // `package::display` strips the root it is handed, and a root spelled with
    // `../../..` is not a prefix of the path it reaches.
    let root = std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("the repository root resolves");
    let directory = root.join("taxonomy-source/headwater-standard");
    let manifest = package::manifest_at(&directory).expect("the base package manifest reads");
    let taxonomy = package::maximal(&root, &directory).expect("the shipped bundle set resolves");

    // What the reader walks, counted the way `template::files` walks it: every
    // directory under the declared bundle root that carries a `bundle.yml`, and
    // every `*.md` directly under its `templates/`.
    let bundles = root.join("docs/taxonomies");
    let mut carrying = 0usize;
    let mut walked = 0usize;
    for entry in std::fs::read_dir(&bundles).expect("the bundle root reads") {
        let at = entry.expect("the entry reads").path();
        if !at.join("bundle.yml").is_file() {
            continue;
        }
        let Ok(templates) = std::fs::read_dir(at.join("templates")) else {
            continue;
        };
        let here = templates
            .filter_map(|one| one.ok())
            .filter(|one| one.path().extension().is_some_and(|kind| kind == "md"))
            .count();
        if here > 0 {
            carrying += 1;
        }
        walked += here;
    }
    assert_eq!(
        (walked, carrying),
        (17, 6),
        "the library this case reads is not the one its doc comment describes"
    );

    let refused = template::refusals(&root, &directory, &manifest, &taxonomy.taxonomy);
    let named: Vec<String> = refused
        .iter()
        .map(|one| format!("{}: {}", one.path, one.reason))
        .collect();
    assert!(
        refused.is_empty(),
        "this repository ships templates its own publish refuses:\n{}",
        named.join("\n")
    );
}
