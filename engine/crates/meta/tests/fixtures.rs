// SPDX-License-Identifier: Apache-2.0
//! The meta-schema's fixture corpus.
//!
//! Every case is a pair of files: a source under `fixtures/taxonomy/` or
//! `fixtures/overlay/`, and the `.record` beside it that holds what the
//! meta-schema makes of it. A source that validates records `valid`, because a
//! record that was empty when a source passed and empty when a fixture went
//! missing would say the same thing twice.
//!
//! `fixtures/schema.record` is the meta-schema as this crate read it: every
//! declaration, every closed value set, and every position the file declines to
//! constrain. It is the file to read in a diff when `meta-schema.yml` changes,
//! because a shape edit is otherwise visible only in the cases it moves.
//!
//! To re-record after a deliberate change:
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-meta --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_meta::schema::Position;
use headwater_meta::shape::{Form, Shape};
use headwater_meta::{render, MetaSchema};
use headwater_ref::Address;
use headwater_yaml::Spanned;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn blessing() -> bool {
    std::env::var_os("HEADWATER_BLESS").is_some()
}

fn cases(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "yml"))
        .collect();
    found.sort();
    assert!(!found.is_empty(), "no fixtures in {}", dir.display());
    found
}

fn compare(path: &Path, actual: &str) {
    if blessing() {
        std::fs::write(path, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            path.display()
        )
    });
    assert_eq!(expected, actual, "\n{} is out of date", path.display());
}

/// `valid`, or every rejection in the order it was found.
fn verdict(errors: &[headwater_meta::MetaError]) -> String {
    if errors.is_empty() {
        "valid\n".to_string()
    } else {
        render(errors)
    }
}

fn load(path: &Path) -> Spanned<headwater_yaml::Value> {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    headwater_yaml::load(&source).unwrap_or_else(|errors| {
        panic!(
            "{} does not load:\n{}",
            path.display(),
            headwater_yaml::error::render(&errors)
        )
    })
}

#[test]
fn taxonomy_sources_validate_to_the_recorded_verdict() {
    let schema = MetaSchema::shipped().expect("the shipped meta-schema");
    for path in cases(&fixtures_dir().join("taxonomy")) {
        let root = load(&path);
        let found = headwater_meta::validate::taxonomy(&schema, &root);
        compare(&path.with_extension("record"), &verdict(&found));
    }
}

#[test]
fn overlay_sources_validate_to_the_recorded_verdict() {
    let schema = MetaSchema::shipped().expect("the shipped meta-schema");
    for path in cases(&fixtures_dir().join("overlay")) {
        let root = load(&path);
        let found = headwater_meta::validate::overlay(&schema, &root);
        compare(&path.with_extension("record"), &verdict(&found));
    }
}

/// This repository's own three taxonomy sources.
///
/// The base package, the design-spec bundle and the adopter overlay are the
/// whole of what a meta-schema has to read today, and none of the three was
/// written against one. A constructed fixture cannot fail the way a file that
/// somebody wrote for another purpose can.
///
/// The base package is a file now. When this fixture was first recorded it was
/// a fenced YAML block inside an evaluation, read by parsing Markdown, and it
/// was refused on its first line: it opened `package: headwater/standard`, and
/// `package` is a reserved reference root that no taxonomy may declare.
/// [#50](https://github.com/headwater-ai/headwater/issues/50) split the
/// manifest from the taxonomy source, which is what the third `valid` below is.
#[test]
fn this_repositorys_own_sources_validate_to_the_recorded_verdict() {
    let schema = MetaSchema::shipped().expect("the shipped meta-schema");
    let root = repository_root();
    let mut out = String::new();

    const BASE: &str = ".headwater/packages/headwater-standard/taxonomy.yml";
    const BUNDLE: &str = "docs/taxonomies/design-spec/bundle.yml";
    const OVERLAY: &str = ".headwater/overlay.yml";

    let loaded = load(&root.join(BASE));
    let found = headwater_meta::validate::taxonomy(&schema, &loaded);
    out.push_str(&format!("{BASE}\n"));
    out.push_str(&indent(&verdict(&found)));

    for source in [BUNDLE, OVERLAY] {
        let loaded = load(&root.join(source));
        let found = headwater_meta::validate::overlay(&schema, &loaded);
        out.push_str(&format!("\n{source}\n"));
        out.push_str(&indent(&verdict(&found)));
    }

    compare(&fixtures_dir().join("corpus.meta"), &out);
}

/// The meta-schema as the crate read it, plus the rules it does not run.
#[test]
fn the_meta_schema_reads_to_the_recorded_shapes() {
    let schema = MetaSchema::shipped().expect("the shipped meta-schema");
    let mut out = String::new();
    out.push_str(&format!("{} {}\n\n", schema.name(), schema.version()));

    out.push_str("declarations\n");
    for member in schema.declarations() {
        out.push_str(&format!(
            "  {}{} {}\n",
            member.name,
            if member.required { " (required)" } else { "" },
            render_shape(&schema, &member.shape, 0)
        ));
    }
    out.push_str("\noperations\n");
    for member in schema.operations() {
        out.push_str(&format!(
            "  {} {}\n",
            member.name,
            render_shape(&schema, &member.shape, 0)
        ));
    }

    out.push_str("\nunconstrained\n");
    for (at, reason) in schema.unconstrained() {
        out.push_str(&format!("  {at}: {reason}\n"));
    }

    out.push_str("\nnot run without a resolved taxonomy\n");
    for (rule, why) in headwater_meta::validate::skipped() {
        out.push_str(&format!("  {rule}: {why}\n"));
    }

    compare(&fixtures_dir().join("schema.record"), &out);
}

/// One line per shape, with a block followed rather than named, so that a
/// change inside a definition shows up at every place that uses it. The
/// modifiers ride on the form word rather than after the body, because a
/// `members` body ends several lines below the shape that carries them.
fn render_shape(schema: &MetaSchema, shape: &Shape, depth: usize) -> String {
    let resolved = schema.resolve(shape);
    let modifiers = modifiers(resolved);
    match &resolved.form {
        Form::Scalar(kind) => format!("scalar:{}{modifiers}", kind.name()),
        Form::Enum(values) => format!("enum[{}]{modifiers}", values.join(" ")),
        Form::Free(reason) => format!("free ({reason}){modifiers}"),
        Form::Block(name) => format!("block:{name}{modifiers}"),
        Form::Seq(inner) => format!(
            "seq{modifiers} of {}",
            render_shape(schema, inner, depth + 1)
        ),
        Form::Map(inner) => format!(
            "map{modifiers} of {}",
            render_shape(schema, inner, depth + 1)
        ),
        Form::OneOf(alternatives) => format!(
            "one of{modifiers} [{}]",
            alternatives
                .iter()
                .map(|alternative| render_shape(schema, alternative, depth + 1))
                .collect::<Vec<_>>()
                .join(" | ")
        ),
        Form::Members(members) => {
            let pad = "  ".repeat(depth + 2);
            let mut out = format!("members{modifiers}\n");
            for member in members {
                out.push_str(&format!(
                    "{pad}{}{} {}\n",
                    member.name,
                    if member.required { " (required)" } else { "" },
                    render_shape(schema, &member.shape, depth + 1)
                ));
            }
            out.trim_end().to_string()
        }
    }
}

fn modifiers(shape: &Shape) -> String {
    let mut out = String::new();
    if shape.reference {
        out.push_str(" reference:allowed");
    }
    if !shape.addressable {
        out.push_str(" addressable:false");
    }
    out
}

/// One line, whatever the shape. The address record asks where an address
/// lands, and a whole nested shape printed there buries the answer.
fn summary(schema: &MetaSchema, shape: &Shape) -> String {
    let resolved = schema.resolve(shape);
    let modifiers = modifiers(resolved);
    match &resolved.form {
        Form::Members(members) => format!(
            "members({}){modifiers}",
            members
                .iter()
                .map(|member| member.name.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        ),
        Form::Seq(inner) => format!("seq{modifiers} of {}", summary(schema, inner)),
        Form::Map(inner) => format!("map{modifiers} of {}", summary(schema, inner)),
        Form::OneOf(alternatives) => {
            format!("one of{modifiers} {} alternatives", alternatives.len())
        }
        _ => render_shape(schema, resolved, 0),
    }
}

/// The three rules that [#48](https://github.com/headwater-ai/headwater/issues/48)
/// could not put in the grammar, asked of the meta-schema directly.
///
/// Each one is a question about an address rather than about a source, so the
/// fixture asks it that way. The list-index case is the one that matters: the
/// grammar parses `…transitions.0` because `0` is a legal key name, and the
/// answer has to come from something that knows which positions hold lists.
#[test]
fn addresses_land_where_the_record_says() {
    let schema = MetaSchema::shipped().expect("the shipped meta-schema");
    let cases = [
        "kinds.design_spec.identifier",
        "kinds.design_spec.language",
        "shelves.decisions.path",
        "regimes.language.ste_house",
        "regimes.lifecycle.standard.transitions",
        "regimes.lifecycle.standard.transitions.draft",
        "regimes.lifecycle.standard.transitions.draft.0",
        "core.requires",
        "core.requires.0",
        "projections.0",
        "kinds.decision.sections.require.0",
        "identifier_schemes.decision_id.pattern.prefix",
        "facets.status.values",
        "package.optional.forbids",
        "vocabularies.lifecycle_state",
    ];
    let mut out = String::new();
    for case in cases {
        let address = Address::parse(case).expect("the fixture holds addresses");
        let verdict = match schema.at(&address) {
            Position::At(shape) => format!("lands on {}", summary(&schema, shape)),
            Position::Undeclared { stopped_at } => format!("undeclared at `{stopped_at}`"),
            Position::InsideList { list } => format!("inside the list at `{list}`"),
            Position::BelowScalar { scalar } => format!("below the scalar at `{scalar}`"),
        };
        out.push_str(&format!("{case} => {verdict}\n"));
    }
    compare(&fixtures_dir().join("addresses.record"), &out);
}

fn indent(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    out
}
