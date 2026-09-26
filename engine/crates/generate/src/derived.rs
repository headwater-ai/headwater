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
//! # The date a state was entered, when no edge set the state
//!
//! An edge that sets the state also dates it: the state was entered when the
//! document that declares the edge says it was, so [`members`] folds the
//! setters' dates, and takes the stalest of them. With no such edge the state
//! is the `live` fallback of [`standing`], and the date is the **newest**
//! `state_entered` over the documents the projection read, which HW-DR-0063
//! rules as amended on 2026-09-25.
//!
//! The owner ruled on
//! [#820](https://github.com/headwater-ai/headwater/issues/820) on 2026-09-25
//! that a page is no fresher than its newest input. The page is a new page
//! each time an input changes, so the date it entered its state is the date of
//! the newest input it carries. The minimum stays right for freshness, where a
//! file is only as fresh as its oldest confirmation. So the two folds over the
//! read set go in opposite directions: [`newest_of`] for the date a state was
//! entered, and [`stalest_of`] for freshness.
//!
//! `a_state_no_edge_sets_is_dated_by_the_newest_document_the_projection_read`
//! in `tests/lifecycle_state_admission.rs` holds the fallback fold, and
//! `a_state_a_relation_sets_that_the_kinds_own_regime_admits_is_written_with_the_setters_date`
//! holds the setter's date and freshness over the same two read documents.
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
//!
//! # One of them is checked, against the one thing that computed it
//!
//! `lifecycle.state.not_admitted` is the check-layer rule that holds a state
//! against the lifecycle regime the document's own kind binds
//! ([`headwater_check::lifecycle_state::StateAdmitted`]), and the census
//! exemption above keeps it from ever instantiating over a generated document.
//! [`standing`] is therefore the only reader left that could catch a state its
//! own kind's regime does not name, and until
//! [#945](https://github.com/headwater-ai/headwater/issues/945) it did not
//! look: a kind that binds a narrower regime than the vocabulary's could
//! receive the vocabulary's `live`-role value regardless. [`members`] now
//! reads [`headwater_check::shape::Shape::lifecycle_of`] for the document's own
//! kind and holds the computed state against
//! [`headwater_check::shape::LifecycleRegime::states`] — the same set
//! `StateAdmitted::evaluate` reads, so this module does not keep a second
//! opinion about which states a regime names. A kind that binds no regime is
//! unaffected: that is [HW-OBL-0196](../../../../docs/obligations/0196-a-relation-writes-a-state-onto-a-kind-that-binds-no-lifecycle-regime-and-nothing-reads-that-pair.md)'s
//! question, at a different layer.

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

/// What [`members`] computed, and why it could not compute the rest.
pub(crate) struct Members {
    /// Every required facet this module answered, as `(facet, value)` in the
    /// order the kind requires them.
    pub values: Vec<(String, String)>,
    /// Every required facet this module reads and could not answer here, as
    /// `(facet, cause)`. The cause is a clause that names where the repair is.
    ///
    /// A required facet that is in neither list is one this module does not
    /// read at all: no role it derives and no layout names it. That cause is
    /// the taxonomy's, and [`crate::identity`] keeps that wording for it.
    /// A facet here is different. Its role or its layout is one this module
    /// reads, so the repair is where the value should have come from (#820).
    pub unsupplied: Vec<(String, String)>,
}

/// Every required facet of this kind that the `identity` block does not write,
/// and the cause of each one this module reads and could not answer.
///
/// `written` names the facets the block already wrote, so that a facet is
/// stated once. A required facet that neither the block nor this module can
/// answer is left out of [`Members::values`], and [`crate::identity`] refuses
/// the declaration over what remains, with the cause from
/// [`Members::unsupplied`] where this module has one.
///
/// The one facet this can refuse outright, rather than merely leave out, is
/// the state: a value [`standing`] computes and the kind's own lifecycle
/// regime does not name is not a fact this document can be written to state,
/// on the terms the module note above gives.
pub(crate) fn members(
    surface: &Surface<'_>,
    identity: &crate::DeclaredIdentity,
    output: &str,
    composed: &Composed<'_>,
    written: &[String],
) -> Result<Members, String> {
    let kind = identity.kind.as_str();
    let id = identity.id.as_str();
    let shape = surface.shape();
    let role_of = |role: &str| shape.facet_in_role(role).map(|facet| facet.name.clone());
    let scent = role_of(SCENT);
    let state = role_of(STATE);
    let entered = role_of(STATE_ENTERED);
    let freshness = role_of(FRESHNESS);
    let sources = documents(surface, composed, output);

    let read: Vec<String> = sources
        .iter()
        .map(|document| document.path.to_string())
        .collect();

    let mut out = Members {
        values: Vec::new(),
        unsupplied: Vec::new(),
    };
    for facet in shape.required_facets(kind) {
        if written.contains(&facet) {
            continue;
        }
        // `Ok(None)` is a facet this module does not read. `Err` is one it
        // reads and could not answer, with the cause.
        let value: Result<Option<String>, String> = if scent.as_deref() == Some(facet.as_str()) {
            Ok(Some(headwater_resolve::render::quoted(&composed.summary)))
        } else if state.as_deref() == Some(facet.as_str()) {
            // No standing means no edge sets a state and the vocabulary holds
            // no `live` value. That is a taxonomy cause, so it keeps the
            // taxonomy wording.
            match standing(surface, id, output) {
                Some(standing) => Ok(Some(admitted(surface, kind, standing)?)),
                None => Ok(None),
            }
        } else if entered.as_deref() == Some(facet.as_str()) {
            match set_by(surface, id, output) {
                // The state came from an incoming edge, so the date it was
                // entered is a date of the documents that declare that edge
                // rather than of the documents this projection read.
                Some(setters) => stalest(surface, &setters, entered.as_deref()).ok_or_else(|| {
                    unsourced(
                        &facet,
                        STATE_ENTERED,
                        "stalest",
                        "the documents whose edges set this document's state",
                        &setters,
                    )
                }),
                // No edge sets the state, so the file stands at the `live`
                // fallback and entered it with its newest input (#820).
                None => newest_of(&sources, entered.as_deref()).ok_or_else(|| {
                    unsourced(
                        &facet,
                        STATE_ENTERED,
                        "newest",
                        "the documents this projection read",
                        &read,
                    )
                }),
            }
            .map(Some)
        } else if freshness.as_deref() == Some(facet.as_str()) {
            stalest_of(&sources, freshness.as_deref())
                .ok_or_else(|| {
                    unsourced(
                        &facet,
                        FRESHNESS,
                        "stalest",
                        "the documents this projection read",
                        &read,
                    )
                })
                .map(Some)
        } else {
            from_layout(surface, output, &facet)
        };
        match value {
            Ok(Some(value)) => out.values.push((facet, value)),
            Ok(None) => {}
            Err(cause) => out.unsupplied.push((facet, cause)),
        }
    }
    Ok(out)
}

/// The cause of a date this module reads from documents and found in none.
///
/// It names the role, the facet, the direction of the fold (`fold`, which is
/// `stalest` or `newest`) and every document the fold read, because the repair
/// is a value on one of those documents. It does not name the
/// taxonomy, where nothing is wrong.
fn unsourced(facet: &str, in_role: &str, fold: &str, what: &str, paths: &[String]) -> String {
    if paths.is_empty() {
        return format!(
            "the engine derives `{facet}`, the facet in the `{in_role}` role, from {what}, and there \
             are none, so the file carries no value for it. Add a document that carries \
             `{facet}` to what this declaration reads"
        );
    }
    format!(
        "the engine derives `{facet}`, the facet in the `{in_role}` role, as the {fold} value over \
         {what} ({}), and each of them carries no value for it. Add `{facet}` to one of those \
         documents",
        paths.join(", ")
    )
}

/// Where a computed state came from, which a refusal over it must name.
///
/// A generator cannot repair a relation declaration by rewriting a vocabulary,
/// and it cannot repair a vocabulary by rewriting a relation declaration. Two
/// different repairs need two different reasons, so the site that produced
/// the value travels with it rather than being re-derived from the value
/// alone.
enum Locus {
    /// An incoming edge whose relation declares `on_target.set_state`, with
    /// the paths of every document whose edge agreed, in the order
    /// [`set_by`] finds them.
    Edge(Vec<String>),
    /// No such edge exists, so the value is the facet's own `live`-role
    /// default — the vocabulary's answer rather than any relation's.
    Fallback,
}

/// Whether the kind this document declares admits the state [`standing`]
/// computed, against the lifecycle regime that kind binds.
///
/// Two refusals, and they read two different declarations. A value the state
/// facet does not admit **at all** is [`headwater_check::lifecycle_state::Stood::NotAState`]'s
/// case: `StateAdmitted::evaluate` skips it because `facet.value.not_permitted`
/// owns that defect, and the census exemption means nothing else ever will for
/// a generated document, so this function is the only reader left. It can only
/// arise from [`Locus::Edge`]: the fallback draws its value from the facet's
/// own vocabulary, so it can never fail this half. A value the vocabulary
/// holds but the kind's own regime does not name is the second refusal, on the
/// same terms `StateAdmitted::evaluate` reads: `regime.states()` is the set,
/// unchanged from there.
///
/// A kind that binds no regime answers every state, on the same reading
/// [`headwater_check::lifecycle_state::StateAdmitted::instantiates`] takes:
/// an absent regime is a kind this rule says nothing about, and
/// [HW-OBL-0196](../../../../docs/obligations/0196-a-relation-writes-a-state-onto-a-kind-that-binds-no-lifecycle-regime-and-nothing-reads-that-pair.md)
/// is the open question about that shape, at the resolve-time layer rather
/// than here. A regime that names no state at all is a declaration `lifecycle
/// soundness` refuses in the resolver, and this function skips it for the
/// same reason `StateAdmitted::evaluate` does: reporting it here would put one
/// taxonomy defect on every generated document of every kind that binds the
/// regime.
fn admitted(
    surface: &Surface<'_>,
    kind: &str,
    standing: (String, Locus),
) -> Result<String, String> {
    let (state, locus) = standing;
    let shape = surface.shape();
    if let Locus::Edge(ref setters) = locus {
        if let Some(facet) = shape.facet_in_role(STATE) {
            if !facet.values.iter().any(|value| value.value == state) {
                return Err(format!(
                    "derives the state `{state}` for `{kind}`, where {} declares the incoming \
                     edge that sets it, and this taxonomy's state facet admits no such value at \
                     all. Change the relation's `on_target.set_state`, or add `{state}` to the \
                     state facet's own vocabulary",
                    setters.join(", ")
                ));
            }
        }
    }
    let Some(regime) = shape.lifecycle_of(kind) else {
        return Ok(state);
    };
    let states = regime.states();
    if states.is_empty() || states.contains(&state.as_str()) {
        return Ok(state);
    }
    let offers = states
        .iter()
        .map(|state| format!("`{state}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = match locus {
        Locus::Edge(setters) => format!(
            "{} declares the incoming edge that sets it",
            setters.join(", ")
        ),
        Locus::Fallback => {
            "no incoming edge sets a state, so `derived::standing` fell back to the facet value \
             whose role is `live`, this taxonomy's vocabulary-wide default"
                .to_string()
        }
    };
    Err(format!(
        "derives the state `{state}` for `{kind}`, and the lifecycle regime `{}` that `{kind}` \
         binds names no such state: a `{kind}` stands at one of {offers}. {source}",
        regime.name
    ))
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
/// that set is a function of its own previous answer. Both folds over the set
/// keep such a date: [`stalest_of`] takes a minimum, so a committed date older
/// than every source stays the minimum on every later run, and [`newest_of`]
/// takes a maximum, so a committed date newer than every source stays the
/// maximum. `generate --check` holds either one, because the emitter agrees
/// with itself. Neither declaration in this repository reaches that shape, and this
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

/// The state this document stands in, and [`Locus`] names where that value
/// came from.
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
fn standing(surface: &Surface<'_>, id: &str, output: &str) -> Option<(String, Locus)> {
    if let Some(state) = set_state(surface, id, output) {
        let setters = set_by(surface, id, output).unwrap_or_default();
        return Some((state, Locus::Edge(setters)));
    }
    let facet = surface.shape().facet_in_role(STATE)?;
    let value = facet
        .values
        .iter()
        .find(|held| held.role.as_deref() == Some(LIVE))
        .map(|held| held.value.clone())?;
    Some((value, Locus::Fallback))
}

/// The state that an incoming edge puts on this document, where one does.
///
/// Two relations declaring one state agree. Two declaring different states
/// get the first in sorted order, so the answer is the same on every run. The
/// check rule `lifecycle.state.set_twice` (`headwater_check::state_set_twice`,
/// #1086) reports the disagreement at the target and names the state this
/// picks.
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
/// freshest date would state a confidence that no source supports. That is the
/// fold for freshness, and for the date a state was entered when an edge set
/// the state.
///
/// `None` where no source carries the facet, as [`values_of`] states.
fn stalest_of(documents: &[Document<'_>], facet: Option<&str>) -> Option<String> {
    values_of(documents, facet)?.min()
}

/// The newest value of one facet over a set of documents.
///
/// The maximum. It is the fold for the date a state was entered when no edge
/// set the state, because the owner ruled on
/// [#820](https://github.com/headwater-ai/headwater/issues/820) that a page is
/// no fresher than its newest input.
///
/// `None` where no source carries the facet, as [`values_of`] states.
fn newest_of(documents: &[Document<'_>], facet: Option<&str>) -> Option<String> {
    values_of(documents, facet)?.max()
}

/// The non-empty values of one facet over a set of documents, for a fold.
///
/// The values are ISO dates, which sort as text in the order they sort as
/// dates, so a fold takes the minimum or the maximum of the text.
///
/// `None` where the declaration names no facet in the role. A fold over the
/// result is `None` where no source carries the facet. An invented date is
/// worse than an absent one, and the declaration is then refused by the caller
/// rather than written with a value nothing supports.
fn values_of<'d>(
    documents: &'d [Document<'_>],
    facet: Option<&'d str>,
) -> Option<impl Iterator<Item = String> + 'd> {
    let facet = facet?;
    Some(
        documents
            .iter()
            .filter_map(move |document| {
                document
                    .facets
                    .get(facet)
                    .and_then(|node| node.value.as_scalar())
                    .map(|scalar| scalar.text.trim().to_string())
            })
            .filter(|text| !text.is_empty()),
    )
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
/// A placeholder is matched to a facet by its key alone. `{seq}` in an
/// identifier pattern names the sequence of an identifier scheme and never a
/// facet, so a layout that wrote `{seq}` names no facet here, and only a key
/// that is the facet's own name, such as `{sequence:02d}`, does.
///
/// `Ok(None)` where the layout names no such facet, which is a cause this
/// module does not own. `Err` where it names the facet and the file name does
/// not read back as exactly one value: see [`read_layout`].
fn from_layout(surface: &Surface<'_>, output: &str, facet: &str) -> Result<Option<String>, String> {
    let Some(shelf) = crate::shelf_of(output, surface) else {
        return Ok(None);
    };
    let Some(layout) = shelf.layout.as_deref() else {
        return Ok(None);
    };
    let name = output.rsplit('/').next().unwrap_or(output);
    let segments = headwater_scaffold::segments(layout);
    let named = segments.iter().any(|segment| {
        matches!(segment, headwater_scaffold::Segment::Placeholder { key, .. } if *key == facet)
    });
    if !named {
        return Ok(None);
    }
    match read_layout(&segments, name, facet) {
        Some(value) => Ok(Some(value)),
        None => Err(format!(
            "the shelf layout `{layout}` names `{facet}`, and the file name `{name}` does not read \
             back against that layout as exactly one value. Change the output path so that it \
             reads back as one value"
        )),
    }
}

/// The value of `facet` in `name`, where `name` reads back against `segments`
/// in exactly one way.
///
/// A placeholder whose specifier is a width, such as `02d`, matches digits
/// alone, which is what [`headwater_scaffold::render_layout`] writes for it.
/// Any other placeholder matches one character or more. Every split of the
/// name that fits the whole layout is found, and **more than one is no
/// answer**. The earlier reader took the first occurrence of the literal after
/// each placeholder, so under `{slug}-{sequence:02d}.md` the slug `a-b` read
/// `sequence` as `b-07` and wrote that text as the value (#820). Digits alone
/// read that name once, as `7`. A layout of two free placeholders around a
/// separator that a value also contains reads more than once, and is refused
/// rather than guessed.
///
/// The padding specifier is deliberately dropped. `{sequence:02d}` writes `09`
/// and the facet is an integer, so the value is `9`, which is what every
/// authored document of that shelf carries.
fn read_layout(
    segments: &[headwater_scaffold::Segment<'_>],
    name: &str,
    facet: &str,
) -> Option<String> {
    let mut found = Vec::new();
    splits(segments, name, facet, None, &mut found);
    if found.len() != 1 {
        return None;
    }
    let value = found.pop()??;
    // An integer facet carries the number and never the padding the layout
    // asked a file name for.
    match value.parse::<i64>() {
        Ok(number) => Some(number.to_string()),
        Err(_) => Some(value),
    }
}

/// Every way `rest` fits `segments`, as the value of `facet` in each. Stops
/// at two, which is already no answer.
fn splits(
    segments: &[headwater_scaffold::Segment<'_>],
    rest: &str,
    facet: &str,
    wanted: Option<&str>,
    found: &mut Vec<Option<String>>,
) {
    if found.len() > 1 {
        return;
    }
    let Some((segment, after)) = segments.split_first() else {
        if rest.is_empty() {
            found.push(wanted.map(str::to_string));
        }
        return;
    };
    match segment {
        headwater_scaffold::Segment::Literal(text) => {
            if let Some(tail) = rest.strip_prefix(text) {
                splits(after, tail, facet, wanted, found);
            }
        }
        headwater_scaffold::Segment::Placeholder { key, specifier } => {
            let digits = specifier.is_some_and(|specifier| specifier.ends_with('d'));
            for end in 1..=rest.len() {
                if !rest.is_char_boundary(end) {
                    continue;
                }
                let (value, tail) = rest.split_at(end);
                if digits && !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    break;
                }
                let wanted = if *key == facet { Some(value) } else { wanted };
                splits(after, tail, facet, wanted, found);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_layout;

    fn read(layout: &str, name: &str, facet: &str) -> Option<String> {
        read_layout(&headwater_scaffold::segments(layout), name, facet)
    }

    /// The layouts this repository's lock declares, which must read as before.
    #[test]
    fn a_leading_number_reads_as_before() {
        assert_eq!(
            read(
                "{sequence:02d}-{slug}.md",
                "09-open-questions.md",
                "sequence"
            )
            .as_deref(),
            Some("9")
        );
        assert_eq!(
            read("{seq:04d}-{slug}.md", "0042-a-b.md", "slug").as_deref(),
            Some("a-b")
        );
    }

    /// The case #820 named: the value before the number carries the separator.
    /// The first-occurrence reader wrote `b-07`.
    #[test]
    fn a_separator_inside_an_earlier_value_does_not_move_the_number() {
        assert_eq!(
            read("{slug}-{sequence:02d}.md", "a-b-07.md", "sequence").as_deref(),
            Some("7")
        );
        assert_eq!(
            read("{slug}-{sequence:02d}.md", "a-b-07.md", "slug").as_deref(),
            Some("a-b")
        );
    }

    /// Two free placeholders around a separator a value contains read twice,
    /// and a name that fits no split reads never. Both are no answer.
    #[test]
    fn a_name_that_reads_back_more_than_once_or_never_is_no_answer() {
        assert_eq!(read("{slug}-{tier}.md", "a-b-c.md", "tier"), None);
        assert_eq!(
            read("{slug}-{sequence:02d}.md", "a-b-xy.md", "sequence"),
            None
        );
    }
}
