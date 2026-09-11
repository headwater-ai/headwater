// SPDX-License-Identifier: Apache-2.0
//! The `identity` block: what a generated document declares about itself.
//!
//! # Why a projection needs one at all
//!
//! A shelf index carries no front matter, and that is right for it. Nothing
//! cites `docs/decisions/README.md`, no edge names it, and a block on it would
//! state facts that no reader and no rule ever asks for.
//!
//! A projection that takes over the path of a document is a different artifact.
//! `docs/spec/09-open-questions.md` carries `HW-REG-open-questions`, and three
//! documents name that identifier. A file at that path with no front matter
//! holds no identifier, so it is no node, so every one of those three edges
//! resolves to nothing. The census admits a generated Markdown document that
//! carries the marker inside its front-matter block, and this block is what
//! makes an emitter able to write one.
//!
//! # The block is closed at three scalars
//!
//! `id`, `kind` and `name`. An identifier is minted once and is a declaration. A
//! kind is a declaration. A name is a declaration too: it is what the document
//! is called, and a taxonomy that requires the facet in the `name` role of a
//! generated document's kind is already asking for one. `decision_register`
//! requires `title`, and the tombstone at `docs/spec/09-open-questions.md` is a
//! `decision_register` that had no way to supply it, so
//! [`crate::label`] fell through to the identifier and a reader met
//! `HW-REG-open-questions` in the sidebar, the browser tab and the shelf's own
//! index page ([#627](https://github.com/headwater-ai/headwater/issues/627)).
//!
//! A summary, a status, a date and a body are prose, and no member here can
//! carry one. `name` is not that: it is a label, it cannot be a sentence, and
//! eight shelves of this repository already declare a free-text `title:` in a
//! taxonomy source that `crate::shelf_label` renders into a navigation key, a
//! heading and a served `<title>`. Admitting the same short string for a
//! document rather than for a shelf adds no class of prose that the taxonomy
//! source did not already hold.
//!
//! The member names the *role* and never the facet, for the reason
//! [`crate::label`] gives: `title` means something in one taxonomy and nothing
//! in the next.
//!
//! The refusal that this shape holds is the one the shelf-sections emitter
//! already made against a `template`: a taxonomy source sits outside the corpus
//! root, so prose written there moves to the one file that no census row covers
//! and no language regime binds. A member that took an open mapping of facets
//! would have re-admitted that prose under a different syntax, and the earlier
//! refusal would then have been a ruling about punctuation rather than about
//! substance. Two scalars cannot express a sentence, so it stays a ruling about
//! substance.
//!
//! # `kind` names a kind, and the shelf says what to write for it
//!
//! A declaration states the kind it wants and never the facet that carries it.
//! What the front matter needs in order to resolve that kind belongs to the
//! shelf that claims the output path:
//!
//! - a **heterogeneous** shelf declares a discriminator, so the block writes
//!   `{discriminator}: {kind}`, and the kind has to be one the shelf admits;
//! - a **homogeneous** shelf states the kind itself, and
//!   [spec 2](../../../../docs/spec/02-taxonomy-model.md) forbids restating it,
//!   so the block writes no facet at all and the declared kind has to agree with
//!   the shelf's.
//!
//! A path that no shelf claims resolves to no kind however the block is
//! written, and the declaration is refused rather than written as a file that
//! reaches no index.
//!
//! # The edges are derived, and none of them is declared here
//!
//! [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage) rules that
//! front matter is the one place an edge is authored. A taxonomy source is not
//! front matter, so an `identity` block that listed relations would be the
//! second authoring location that Q4 refused.
//!
//! A generated document still owes edges, because `reciprocal: required` obliges
//! the far end of every such pair to declare its half. Those halves are not new
//! facts. Each one is the inverse of an edge that another document already
//! wrote, and the corpus already holds every part of it: the relation, the
//! direction and the identifier at each end. So the emitter derives them, and it
//! derives nothing else. A generated document carries the reciprocal half of
//! every edge that another document declares into it, and no other edge.
//!
//! The derivation reads the *raw* target of each incoming edge rather than the
//! resolved one, and that is what makes one pass enough. A resolved target needs
//! this document to be in the identifier index already, which it is not on the
//! run that first writes it. Reading the raw target gives the same answer on the
//! first run and on every run after it.

use crate::{DeclaredIdentity, Kind};
use headwater_census::shelves::{DeclarationError, ShelfBody};
use headwater_graph::declarations::Direction;
use headwater_graph::Reciprocal;
use headwater_query::Surface;
use headwater_yaml::value::{Mapping, Value};
use headwater_yaml::Span;

/// Read the `identity` block of one `projections` entry.
///
/// A kind whose output is not a document of the corpus is refused here rather
/// than at the emitter, because the refusal is a property of the declaration and
/// a reader of the taxonomy is the person who can act on it.
pub(crate) fn read(
    body: &Mapping,
    kind: Kind,
    index: usize,
    span: Span,
    errors: &mut Vec<DeclarationError>,
) -> Option<DeclaredIdentity> {
    let node = body.get("identity")?;
    if !writes_a_document(kind) {
        errors.push(DeclarationError {
            message: format!(
                "`projections.{index}` states an `identity`, and `{}` does not write a document \
                 of this corpus. An identity says which document a generated file is, and only a \
                 projection that writes Markdown into the corpus produces one",
                kind.name()
            ),
            span,
        });
        return None;
    }
    let Value::Map(block) = &node.value else {
        errors.push(DeclarationError {
            message: format!(
                "`projections.{index}.identity` is {}, and an identity is a block",
                node.value.kind_name()
            ),
            span: node.span,
        });
        return None;
    };
    let id = scalar(block, "id", index, node.span, errors);
    let kind = scalar(block, "kind", index, node.span, errors);
    let name = optional_scalar(block, "name");
    Some(DeclaredIdentity {
        id: id?,
        kind: kind?,
        name,
    })
}

/// Whether a kind writes a Markdown document into the corpus.
///
/// The three that do are the two shelf emitters and the probe result.
/// `graph_export` writes JSON, which is not a document of this corpus whatever
/// its front matter would say. The unbuilt kinds are refused for the same reason
/// the generator gives for not emitting them: no document states the artifact's
/// form, so nothing states whether it is a document.
///
/// A probe result is the case that makes the block earn its keep a second time.
/// Spec 5 asks a result to be citable, and a rate that no document can name is a
/// number a reader reaches only by opening the file it happens to sit in.
fn writes_a_document(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::ShelfIndex | Kind::ShelfSections | Kind::ProbeResult
    )
}

/// A member the block may state and need not.
///
/// A missing `name` is the shape every declaration had before the member
/// existed, so it is not an error. An empty one is: a declaration that wrote
/// `name: ""` asked for a label and would get the identifier, which is the
/// defect rather than the fix. It is reported as absent so that the required
/// facet reports missing where a check reads it, rather than the block writing
/// a blank facet that resolves to nothing.
fn optional_scalar(block: &Mapping, member: &str) -> Option<String> {
    block
        .get(member)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn scalar(
    block: &Mapping,
    member: &str,
    index: usize,
    span: Span,
    errors: &mut Vec<DeclarationError>,
) -> Option<String> {
    match block.get(member).and_then(|node| node.value.as_scalar()) {
        Some(scalar) if !scalar.text.trim().is_empty() => Some(scalar.text.clone()),
        _ => {
            errors.push(DeclarationError {
                message: format!("`projections.{index}.identity` states no `{member}`"),
                span,
            });
            None
        }
    }
}

/// The front-matter block a generated document at `output` carries, or the
/// reason it cannot carry one.
///
/// The marker is a member of the block rather than a line above it. A file whose
/// first line is a comment puts the fences on line two, where no front-matter
/// parser looks for them, so a document that declares an identity and a file
/// that carries a first-line marker are two shapes and never one.
pub(crate) fn front_matter(
    surface: &Surface<'_>,
    identity: &DeclaredIdentity,
    output: &str,
    projection: Kind,
) -> Result<String, String> {
    let discriminator = placement(surface, identity, output)?;
    unwritable(surface, identity, discriminator.as_ref())?;
    let config = surface.config();
    let mut out = String::from("---\n");
    out.push_str(&headwater_mark::marker_member(projection.name()));
    out.push('\n');
    out.push_str(&format!("{}: {}\n", config.identifier_facet, identity.id));
    if let Some((facet, value)) = discriminator {
        out.push_str(&format!("{facet}: {value}\n"));
    }
    if let Some(name) = &identity.name {
        let Some(facet) = surface.name_facet() else {
            return Err(format!(
                "declares the name `{name}`, and this taxonomy puts no facet in the `name` role.                  A name is written under the facet that carries one, and a taxonomy that                  declares none names no document at all"
            ));
        };
        out.push_str(&format!("{facet}: {name}\n"));
    }
    let owed = reciprocals(surface, identity, output);
    if !owed.is_empty() {
        out.push_str(&format!("{}:\n", config.relations_facet));
        let mut written = String::new();
        for (relation, target) in &owed {
            if *relation != written {
                out.push_str(&format!("  {relation}:\n"));
                written = relation.clone();
            }
            out.push_str(&format!("    - {target}\n"));
        }
    }
    out.push_str("---\n\n");
    Ok(out)
}

/// The reason no block can be written when the kind requires a facet that this
/// block is the writer of and the declaration does not state it.
///
/// # What this reads, and what it deliberately does not
///
/// Three facets, and only three: the identifier facet, the discriminator of a
/// heterogeneous shelf, and the facet in the `name` role. Those are the ones a
/// member of this block writes, so a requirement on one of them is a
/// requirement the declaration can meet and did not
/// ([#780](https://github.com/headwater-ai/headwater/issues/780)).
///
/// It is not the whole set the kind requires, and the line is measured. Over
/// this repository's own lock, `governed_document` requires `status`,
/// `status_since`, `last_verified` and `summary`, and both kinds this corpus
/// generates inherit all four. A refusal over the whole required set would name
/// 5 facets on `docs/spec/09-open-questions.md` and 4 on the probe result,
/// refusing both of the two identity declarations this repository makes, and no
/// member of this block could answer any of the nine. Whether a generated
/// document should be excused from `status` and `summary` is a question about
/// the census exemption and about the position
/// [spec 6](../../../../docs/spec/06-engine-architecture.md) states, rather than
/// a question about this block, and #780 stays open holding it.
///
/// # Why the refusal is here and not at the read
///
/// The declaration is read by [`crate::Projections::read`], which takes the
/// resolved taxonomy as a `&Mapping` and holds no shape, so it can resolve no
/// kind against the facets that kind requires. This function has a
/// [`Surface`], and [`Surface::shape`] carries `required_facets`. It also has
/// the shelf that claims the output path, which is what decides whether the
/// discriminator is a facet the block writes at all.
fn unwritable(
    surface: &Surface<'_>,
    identity: &DeclaredIdentity,
    discriminator: Option<&(String, String)>,
) -> Result<(), String> {
    let config = surface.config();
    let missing: Vec<String> = surface
        .shape()
        .required_facets(&identity.kind)
        .into_iter()
        .filter(|facet| {
            // The identifier is a required member of the block, so this never
            // fires; it is written rather than assumed, because a block that
            // stopped writing the identifier would be the same defect.
            if facet == &config.identifier_facet {
                return false;
            }
            // A heterogeneous shelf gets its discriminator from `kind`. A
            // homogeneous one gets no facet at all, and spec 2 forbids
            // restating the kind there, so a requirement on it is outside what
            // this block answers for.
            if discriminator.is_some_and(|(written, _)| written == facet) {
                return false;
            }
            match surface.name_facet() {
                Some(name) if name == facet => identity.name.is_none(),
                _ => false,
            }
        })
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    let kind = &identity.kind;
    let facets = missing
        .iter()
        .map(|facet| format!("`{facet}`"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "declares the kind `{kind}`, and `{kind}` requires the facet {facets}. This block writes \
         that facet from its `name` member and the declaration states none. A generated document \
         is the one document whose only writer is a declaration, so a facet the declaration \
         leaves out is a facet no author can add and no check reads"
    ))
}

/// The discriminator the shelf that claims this path needs, and the reason no
/// block can be written when the placement and the declared kind disagree.
fn placement(
    surface: &Surface<'_>,
    identity: &DeclaredIdentity,
    output: &str,
) -> Result<Option<(String, String)>, String> {
    let Some(shelf) = crate::shelf_of(output, surface) else {
        return Err(format!(
            "declares the identity `{}`, and no shelf claims `{output}`. A document off every \
             shelf resolves to no kind, so the file would carry an identifier that no index \
             reads",
            identity.id
        ));
    };
    match &shelf.body {
        ShelfBody::Homogeneous { kind } if kind == &identity.kind => Ok(None),
        ShelfBody::Homogeneous { kind } => Err(format!(
            "declares the kind `{}`, and the shelf `{}` holds `{kind}` alone. A homogeneous \
             shelf states the kind, and no facet may restate it",
            identity.kind, shelf.name
        )),
        ShelfBody::Heterogeneous {
            discriminator,
            kinds,
        } => match kinds.contains(&identity.kind) {
            true => Ok(Some((discriminator.clone(), identity.kind.clone()))),
            false => Err(format!(
                "declares the kind `{}`, and the shelf `{}` admits {}",
                identity.kind,
                shelf.name,
                kinds
                    .iter()
                    .map(|kind| format!("`{kind}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        },
    }
}

/// Every reciprocal half this document owes, as `(relation, target)`, sorted so
/// that two runs over one corpus write one file.
///
/// The far end of the half that exists is the document that owes the other half,
/// which is the rule `relation.reciprocity.missing` reports against. This writes
/// the half rather than reporting it.
fn reciprocals(
    surface: &Surface<'_>,
    identity: &DeclaredIdentity,
    output: &str,
) -> Vec<(String, String)> {
    let declarations = surface.relations();
    let mut owed: Vec<(String, String)> = Vec::new();
    for edge in &surface.graph().edges {
        // A document does not owe itself a reciprocal half. The edge would be
        // one this emitter wrote on an earlier run, and reading it back would
        // make the output a function of its own last version.
        if edge.source.path == output || edge.raw_target.trim() != identity.id {
            continue;
        }
        let Some(relation) = declarations
            .relations
            .iter()
            .find(|known| known.name == edge.declared)
        else {
            continue;
        };
        if relation.reciprocal != Reciprocal::Required {
            continue;
        }
        // Whichever name the far end wrote, this end owes the other one.
        let name = match edge.direction {
            Direction::AsDeclared => relation
                .inverse
                .clone()
                .unwrap_or_else(|| relation.name.clone()),
            Direction::Inverse => relation.name.clone(),
        };
        // A source with no identifier is no node, so the edge it declares is
        // one this document cannot answer. The reciprocity rule reports that
        // pair; this emitter writes no half for it.
        if edge.source.id.trim().is_empty() {
            continue;
        }
        owed.push((name, edge.source.id.clone()));
    }
    owed.sort();
    owed.dedup();
    owed
}
