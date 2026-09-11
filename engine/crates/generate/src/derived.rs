// SPDX-License-Identifier: Apache-2.0
//! The required facets of a generated document that no declaration states.
//!
//! # What this module is for
//!
//! A generated document's kind requires what every other document of that kind
//! requires. Over this repository's own lock that is seven facets on
//! `docs/spec/09-open-questions.md` and five on a probe result, and the
//! `identity` block answers three of them: the identifier, the discriminator of
//! a heterogeneous shelf, and the facet in the `name` role. The rest went
//! unwritten and nothing reported it
//! ([#780](https://github.com/headwater-ai/headwater/issues/780)).
//!
//! Two shapes could have closed that. The block could have grown a member for
//! each facet, and [`crate::identity`] states why it did not: a taxonomy source
//! sits outside the corpus root, so prose written there moves to the one file
//! that no census row covers and no language regime binds. This module is the
//! other shape. **A value the engine computes costs the declaration nothing**,
//! and it is the channel this emitter already used for two facts: the kind of a
//! heterogeneous shelf's document comes from the shelf, and the reciprocal half
//! of every incoming edge comes from the graph.
//!
//! So the block stays closed at three scalars and the engine writes the rest.
//!
//! # Every value here is a function of committed bytes
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#projections) rules
//! that no generated file states when it was generated, because `generate
//! --check` compares bytes and a clock would make every run differ from the
//! last over a corpus nobody touched. Two of the facets below are dates, and
//! neither one is a clock: each folds the dates that the documents this
//! projection read already carry. A date derived from an input is not a
//! statement about when the run happened, so the rule stands untouched and the
//! gate holds these bytes like every other byte of the file.
//!
//! That is also what makes one pass enough. Nothing here reads the file being
//! written, and nothing reads git, so the first run and every run after it
//! write one answer.
//!
//! **That claim is held by two filters rather than by the shape of the corpus.**
//! A generated file that declares an identity is a node of the census, so a
//! projection whose output sits on the shelf it reads finds its own last version
//! among the documents it folds. [`incoming`] refuses the output path and so
//! does [`documents`], and without the second one a committed date would be its
//! own answer on every later run. Neither declaration in this repository reaches
//! that shape, which is why a fixture taxonomy has to build it.
//!
//! # The roles, and never the facet names
//!
//! `title` means something in one taxonomy and nothing in the next, which is
//! the reason [`crate::label`] reads the `name` role rather than a facet called
//! `title`. The same holds for every facet here, so each one is found through
//! [`headwater_check::shape::Shape::facet_in_role`] and none is named in this
//! file. A taxonomy that puts no facet in a role writes no value for it, and a
//! taxonomy that renames every facet is read unchanged.
//!
//! The one exception is not an exception to that rule. A facet that the shelf's
//! `layout` names is found by that declaration rather than by a role, because a
//! layout is where a taxonomy says which facet a file name carries. `sequence`
//! has no role in the registry spec 2 closes at five members, and
//! `{sequence:02d}-{slug}.md` is how the numbered specification series says
//! that the number is in the path.
//!
//! # What is deliberately not derived
//!
//! The facet in the `scent` role. Nothing computes a summary, so the emitter
//! composes one and hands it here in [`Composed`]. The owner ruled on
//! 2026-09-11 that this is right: a generated document's body is already prose
//! that its emitter wrote and that no rule reads, so a summary beside it is no
//! more ungoverned than the body, and refusing the summary while shipping the
//! body is an asymmetry that no stated principle supports.
//!
//! # Writing these does not end the census exemption
//!
//! No check reads a generated document, and this module changes nothing about
//! that. The facets below are written and unchecked. They are written because a
//! reader meets them, because an index and an export carry them, and because a
//! relation that puts a state on its target had nowhere to put one
//! ([#227](https://github.com/headwater-ai/headwater/issues/227)).

use headwater_graph::declarations::Direction;
use headwater_query::{Document, Surface};

/// The state a reader may rely on, as the state vocabulary's roles name it.
///
/// The one reader of this role outside [`headwater_check::lifecycle_state`],
/// and it reads it for the same reason: no component here holds a list of which
/// state means what.
const LIVE: &str = "live";

/// The facet role that carries the state a document stands in.
const STATE: &str = "state";

/// The facet role that carries the date a document entered its state.
const STATE_ENTERED: &str = "state_entered";

/// The facet role that carries the date a document was last confirmed to hold.
const FRESHNESS: &str = "freshness";

/// The facet role that carries the cue a reader scans a list for.
const SCENT: &str = "scent";

/// What the emitter knows and this module cannot compute.
pub(crate) struct Composed<'a> {
    /// The summary this emitter composed. See the module note on the `scent`
    /// role: this is the one required facet of a generated document that
    /// nothing derives.
    pub summary: String,
    /// The paths of the documents this projection read, which are where the
    /// derived dates come from. An emitter that read no document of the corpus
    /// supplies none, and the dates are then unwritten rather than invented.
    pub sources: Vec<&'a str>,
}

/// Every required facet of this kind that the `identity` block does not write,
/// as `(facet, value)` in the order the kind requires them.
///
/// `written` names the facets the block already wrote, so that a facet is
/// stated once. A required facet that neither the block nor this module can
/// answer is left out, and [`crate::identity`] refuses the declaration over
/// what remains.
pub(crate) fn members(
    surface: &Surface<'_>,
    identity: &crate::DeclaredIdentity,
    output: &str,
    composed: &Composed<'_>,
    written: &[String],
) -> Vec<(String, String)> {
    let kind = identity.kind.as_str();
    let id = identity.id.as_str();
    let shape = surface.shape();
    let role_of = |role: &str| shape.facet_in_role(role).map(|facet| facet.name.clone());
    let scent = role_of(SCENT);
    let state = role_of(STATE);
    let entered = role_of(STATE_ENTERED);
    let freshness = role_of(FRESHNESS);
    let sources = documents(surface, composed, output);

    let mut out = Vec::new();
    for facet in shape.required_facets(kind) {
        if written.contains(&facet) {
            continue;
        }
        let value = if scent.as_deref() == Some(facet.as_str()) {
            Some(headwater_resolve::render::quoted(&composed.summary))
        } else if state.as_deref() == Some(facet.as_str()) {
            standing(surface, id, output)
        } else if entered.as_deref() == Some(facet.as_str()) {
            match set_by(surface, id, output) {
                // The state came from an incoming edge, so the date it was
                // entered is a date of the documents that declare that edge
                // rather than of the documents this projection read.
                Some(setters) => stalest(surface, &setters, entered.as_deref()),
                None => stalest_of(&sources, entered.as_deref()),
            }
        } else if freshness.as_deref() == Some(facet.as_str()) {
            stalest_of(&sources, freshness.as_deref())
        } else {
            from_layout(surface, output, &facet)
        };
        if let Some(value) = value {
            out.push((facet, value));
        }
    }
    out
}

/// The documents this projection read, as the census classified them.
///
/// A path the census holds no row for contributes nothing. That is a projection
/// reading a file outside the corpus root, and such a file carries no facet of
/// this taxonomy to fold.
///
/// **The output is never one of them**, which is the same refusal [`incoming`]
/// makes and for the same reason. A generated file that declares an identity is
/// a node of the census
/// ([`headwater_census::census::Outcome::node`]), so a projection whose output
/// sits on the shelf it reads finds its own last version in the set. A fold over
/// that set is a function of its own previous answer: [`stalest_of`] takes a
/// minimum, so a committed date no source supports stays the minimum on every
/// later run, and `generate --check` holds it because the emitter agrees with
/// itself. Neither declaration in this repository reaches that shape, and this
/// filter is what keeps the claim in the module comment true of the module
/// rather than true only of its present callers.
fn documents<'a>(
    surface: &Surface<'a>,
    composed: &Composed<'_>,
    output: &str,
) -> Vec<Document<'a>> {
    surface
        .documents()
        .into_iter()
        .filter(|document| document.path != output)
        .filter(|document| composed.sources.contains(&document.path))
        .collect()
}

/// The state this document stands in.
///
/// An incoming edge of a relation that declares `on_target.set_state` **puts**
/// its target in that state, which is the reading
/// [`headwater_check::dependency`] takes of the same declaration. A generated
/// document had no way to carry the state such an edge wrote, so the fact was
/// stated by a relation and reachable by no rule
/// ([#227](https://github.com/headwater-ai/headwater/issues/227)).
///
/// With no such edge the value is the state whose role is `live`. A generated
/// document is never a draft: it is always exactly what its inputs say, so the
/// initial state of the regime would be false about every one of them. That is
/// also what `headwater new` writes for an authored document, which reaches
/// `current` in the commit that creates it rather than by a movement.
fn standing(surface: &Surface<'_>, id: &str, output: &str) -> Option<String> {
    if let Some(state) = set_state(surface, id, output) {
        return Some(state);
    }
    let facet = surface.shape().facet_in_role(STATE)?;
    facet
        .values
        .iter()
        .find(|held| held.role.as_deref() == Some(LIVE))
        .map(|held| held.value.clone())
}

/// The state that an incoming edge puts on this document, where one does.
///
/// Two relations declaring one state agree, and two declaring different states
/// are a defect of the taxonomy rather than of this document, so the first in
/// sorted order answers and the disagreement is left to the rule that owns it.
fn set_state(surface: &Surface<'_>, id: &str, output: &str) -> Option<String> {
    let mut found: Vec<String> = incoming(surface, id, output)
        .into_iter()
        .filter_map(|(relation, _)| relation)
        .collect();
    found.sort();
    found.dedup();
    found.into_iter().next()
}

/// The paths of the documents whose edges put a state on this document.
fn set_by(surface: &Surface<'_>, id: &str, output: &str) -> Option<Vec<String>> {
    let paths: Vec<String> = incoming(surface, id, output)
        .into_iter()
        .filter_map(|(relation, path)| relation.map(|_| path))
        .collect();
    match paths.is_empty() {
        true => None,
        false => Some(paths),
    }
}

/// Every edge into this document, as `(the state it sets, the source's path)`.
///
/// The *raw* target is what names this document, for the reason
/// [`crate::identity`] gives about the reciprocal halves: a resolved target
/// needs this document in the identifier index already, which it is not on the
/// run that first writes the file. Reading the raw target gives one answer on
/// the first run and on every run after it.
fn incoming(surface: &Surface<'_>, id: &str, output: &str) -> Vec<(Option<String>, String)> {
    let declarations = surface.relations();
    let mut found = Vec::new();
    for edge in &surface.graph().edges {
        // A document does not set its own state. The edge would be one an
        // earlier run of this emitter wrote, and reading it back would make the
        // output a function of its own last version.
        if edge.source.path == output || edge.raw_target.trim() != id {
            continue;
        }
        let Some(relation) = declarations
            .relations
            .iter()
            .find(|known| known.name == edge.declared)
        else {
            continue;
        };
        // `on_target` is a statement about the target of the relation as
        // declared. An edge written from the inverse name points the other way,
        // so the state it sets is not this document's.
        let sets = match edge.direction {
            Direction::AsDeclared => relation.sets_target_state.clone(),
            Direction::Inverse => None,
        };
        found.push((sets, edge.source.path.clone()));
    }
    found
}

/// The stalest value of one facet over a set of paths.
fn stalest(surface: &Surface<'_>, paths: &[String], facet: Option<&str>) -> Option<String> {
    let held: Vec<Document<'_>> = surface
        .documents()
        .into_iter()
        .filter(|document| paths.iter().any(|path| path == document.path))
        .collect();
    stalest_of(&held, facet)
}

/// The stalest value of one facet over a set of documents.
///
/// The minimum, and the reason is what a reader does with it. A file assembled
/// from other documents is only as fresh as the oldest thing it carries, so the
/// freshest date would state a confidence that no source supports. The values
/// are ISO dates, which sort as text in the order they sort as dates.
///
/// `None` where no source carries the facet. An invented date is worse than an
/// absent one, and the declaration is then refused by the caller rather than
/// written with a value nothing supports.
fn stalest_of(documents: &[Document<'_>], facet: Option<&str>) -> Option<String> {
    let facet = facet?;
    documents
        .iter()
        .filter_map(|document| {
            document
                .facets
                .get(facet)
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.trim().to_string())
        })
        .filter(|text| !text.is_empty())
        .min()
}

/// The value of a facet that the shelf's `layout` names, read out of the path.
///
/// A layout is where a taxonomy states that a file name carries a facet, so a
/// path written from one can be read back for that facet. The tokenizer is
/// [`headwater_scaffold::segments`], which is the same one that writes a name
/// from a layout: a second copy of that grammar here would be the drift that
/// [principle 2](../../../../docs/spec/00-vision-and-scope.md#design-principles)
/// rules against, and the two would disagree about a path this engine wrote
/// itself.
///
/// The padding specifier is deliberately dropped. `{sequence:02d}` writes `09`
/// and the facet is an integer, so the value is `9`, which is what every
/// authored document of that shelf carries.
fn from_layout(surface: &Surface<'_>, output: &str, facet: &str) -> Option<String> {
    let shelf = crate::shelf_of(output, surface)?;
    let layout = shelf.layout.as_deref()?;
    let name = output.rsplit('/').next()?;
    let segments = headwater_scaffold::segments(layout);
    let mut rest = name;
    let mut wanted = None;
    for (index, segment) in segments.iter().enumerate() {
        match segment {
            headwater_scaffold::Segment::Literal(text) => {
                rest = rest.strip_prefix(text)?;
            }
            headwater_scaffold::Segment::Placeholder { key, .. } => {
                // The literal that follows delimits this value. A placeholder
                // at the end of the layout runs to the end of the name.
                let end = match segments.get(index + 1) {
                    Some(headwater_scaffold::Segment::Literal(next)) => rest.find(next)?,
                    _ => rest.len(),
                };
                let (value, tail) = rest.split_at(end);
                if *key == facet {
                    wanted = Some(value.to_string());
                }
                rest = tail;
            }
        }
    }
    let value = wanted?;
    // An integer facet carries the number and never the padding the layout
    // asked a file name for.
    match value.parse::<i64>() {
        Ok(number) => Some(number.to_string()),
        Err(_) => Some(value),
    }
}
