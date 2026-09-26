// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: an edge names its target at the revision the edge was
//! verified against.
//!
//! # The gap this closes
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#upstream-awareness)
//! states the rule: "A snapshot pin reports drift on each affected edge, and
//! not only on the pin. A snapshot carries the upstream identity and revision
//! of every item in it, so an advance says which items changed. Every edge into
//! a changed item is then a finding until a person re-verifies it. Requirements
//! practice reached the same mechanism and calls such an edge *suspect*."
//!
//! Everything that sentence needs was already written down and nothing read it
//! together. The importer records `verified_revision` on every edge it writes,
//! and the snapshot records the revision each item is at now. This rule is the
//! comparison, and it is the whole computation.
//!
//! # A `governs` edge ages on the same comparison
//!
//! #946 fixed a hook, and six documents that govern it stayed wrong with no
//! report, because this rule read only a relation an importer writes and the
//! `source-tree` resolver answered no revision. The governs evaluation's
//! "Aging" section is the design: an edge records a digest of what it reached,
//! and a later run compares it against the bytes that are there now.
//! [`headwater_graph::anchors::tree_revision`] is that digest, so the rule
//! below needed no second comparison, only a wider denominator and the
//! message a working tree is owed rather than a snapshot's.
//!
//! # The report is at the origin
//!
//! A proposal against the whole snapshot names a file, and this names the
//! document whose author can act, at the entry of the `relations:` block that
//! carries the edge. That is [spec 4](../../../../docs/spec/04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s
//! report-at-the-origin rule, and it is why the drift report is a check-layer
//! output rather than a verb of its own.
//!
//! # What it cannot read, and what that forced
//!
//! `headwater-import` depends on this crate, so this crate can never read a
//! snapshot. The revision therefore has to reach the rule off the graph, which
//! is why `headwater_graph::anchors::Binding::Resolved` and
//! `headwater_graph::Target::Anchor` carry one. That is the same change
//! [#160](https://github.com/headwater-ai/headwater/issues/160) demands of the
//! cache key: `Target::resolution()` is the derived `Debug`, so a revision the
//! type carries is a revision the key names, and a cached verdict about an edge
//! cannot survive the advance that falsifies it.
//!
//! # The denominator
//!
//! Every relation the taxonomy declares with `created_by: import`, and every
//! relation whose `to` names an anchor kind. The first set is the edges an
//! importer writes a `verified_revision` onto. The second is the edges whose
//! target a resolver can state a revision for, which is `governs` in the
//! standard package and every relation a corpus declares onto a `code_path`.
//! A relation outside both reaches only documents, and a document target holds
//! no revision, so an instance over one would report a denominator of edges
//! that could never be suspect. A relation inside the second set can still
//! admit a document as well, and the instances over those edges are skipped
//! (see below). Declaration-driven, so a new relation produces
//! its instances with no code here, which is the property [`crate::target`]
//! states as its own.
//!
//! The instance exists for an edge that is not suspect as well as for one that
//! is. A rule whose instances are only its findings reports a count that reads
//! as its own denominator.
//!
//! # The three silences, and the one report an unrecorded edge gets
//!
//! An edge with no recorded revision passes, with one exception below. A
//! person who types an entry by hand records nothing to compare, and every
//! `governs` entry this corpus held when the rule widened was such an entry.
//!
//! An edge whose resolver offered no revision passes. A snapshot and the
//! source tree offer one. Every other resolver has no such notion. A rule must
//! not invent a comparison a resolver did not offer.
//!
//! **A literal that names a directory is reported, not passed.** The source
//! tree takes no digest of one (see
//! [`headwater_graph::anchors::tree_revision`]), so the edge could never go
//! suspect, and an author who wrote it would not learn that from a silence.
//! The rule reports it at `Info` on every run, and names `<literal>/**` as
//! the remedy. It offers no patch, because that remedy widens what the edge
//! reaches ([HW-OBL-0104](../../../../docs/obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md)).
//! A list member that names a directory is reported the same way, in one
//! finding per entry that names each such member, because one directory
//! member leaves the whole list without a digest (#1104).
//!
//! A target that is not an anchor passes, and a document target is skipped
//! with its reason. A document holds no revision, and an unbound target is
//! [`crate::target`]'s. The skip is there because instantiation is per
//! relation, so a relation that admits both a document and an anchor has an
//! instance over each of its document edges, and a pass there would count an
//! edge that could never be suspect.
//!
//! **The exception is an unrecorded edge on a document verified today.** The
//! document's freshness facet (`last_verified` in the standard package) is at
//! or after the run's clock, so its author has just re-read it, and the rule
//! reports at `Info` with a patch that records the digest the edge reaches
//! now. Only where the relation declares `verified_revision`, because spec 2
//! makes an attribute the relation does not declare a finding. That patch is the one way a digest is recorded without typing it.
//!
//! # When a fix is offered, and why the clock decides it
//!
//! A patch that writes `verified_revision` records a verification. So it is
//! offered only where the document says one happened today: its freshness
//! facet is at or after the injected clock, compared as dates because the
//! facet is a date. On any other document the finding carries its remedy as
//! prose and no patch, because `headwater check --fix` would then record a
//! verification nobody performed. The patch is a [`crate::Patch::Half`] with
//! the edge's attributes, which [`headwater_scaffold::write::splice`] writes
//! over the entry that names the target and reads back. It turns a bare entry
//! into the mapping form, and keeps every other attribute the entry had. An
//! entry with a list target, or an attribute that is not a scalar, gets no
//! patch, because the splice matches an entry by one scalar `to`.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use crate::shape::Shape;
use crate::{Date, Patch};
use headwater_graph::declarations::Relation;
use headwater_graph::edges::VERIFIED_REVISION;
use headwater_graph::{Declarations, Direction, Edge, Target};

pub const RULE: &str = "relation.target.suspect";

/// A group with no half at all, which the instantiation never produces. As
/// [`crate::target`]: recorded rather than panicked on.
const NO_HALF: &str = "the entry carries no declared half";

/// Why an instance over an edge onto a document decides nothing.
const DOCUMENT_TARGET: &str =
    "the target is a document, which holds no revision for an edge to be verified against";

/// The resolver whose revision is a digest of working-tree bytes.
const SOURCE_TREE: &str = "source-tree";

/// The check, generated from the relation declarations that can carry a
/// revision.
pub struct Suspect<'a> {
    /// Every relation whose `created_by` is `import`, and every relation whose
    /// `to` names an anchor kind. See the module comment.
    declared: Vec<&'a Relation>,
    /// The facet that carries the freshness role, which says when a person
    /// last re-read the document. Nothing where the taxonomy declares none,
    /// and then no fix is ever offered.
    freshness: Option<&'a str>,
}

impl<'a> Suspect<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        Suspect {
            declared: declarations
                .relations
                .iter()
                .filter(|relation| {
                    relation.created_by.as_deref() == Some("import")
                        || relation
                            .to
                            .iter()
                            .any(|kind| declarations.anchor(kind).is_some())
                })
                .collect(),
            freshness: shape
                .facet_in_role("freshness")
                .map(|facet| facet.name.as_str()),
        }
    }

    /// Whether the document that declared this edge says it was verified on
    /// or after the run's clock.
    fn verified_today(&self, view: &EdgeView<'_>) -> bool {
        let (Some(facet), Some(now), Some(facets)) =
            (self.freshness, view.now(), view.declarer_facets())
        else {
            return false;
        };
        facets
            .get(facet)
            .and_then(|node| node.value.as_scalar())
            .and_then(|scalar| Date::parse(&scalar.text))
            .is_some_and(|verified| verified >= now)
    }
}

impl EdgeCheck for Suspect<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`]. 2: the denominator
    /// widened to every relation onto an anchor kind, and the rule reads the
    /// clock (#952). 3: a literal that names a directory is reported rather
    /// than passed, so a verdict cached at 2 over such an edge is stale. 4: a
    /// list with a directory member is reported rather than passed, so a
    /// verdict cached at 3 over such an edge is stale (#1104).
    const VERSION: u32 = 4;
    /// The fix is offered only on a document verified on or after the clock,
    /// so the clock is an input and has to be in the key.
    const NEEDS_CLOCK: bool = true;
    /// As [`crate::target`]: an anchor target has no far document, so there is
    /// no pair to group two halves into.
    const UNIT: EdgeUnit = EdgeUnit::Entry;

    fn instantiates(&self, relation: &str) -> bool {
        self.declared.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        let Some(edge) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };

        let (revision, resolver, patterns) = match &edge.target {
            Target::Anchor {
                revision,
                resolver,
                patterns,
                ..
            } => (revision, resolver, patterns),
            // A relation that admits a document and an anchor, `traces_to` in
            // the standard package, has an instance over each of its document
            // edges too. It is skipped with the reason rather than passed, so
            // that the count of instances this rule passed is a count of edges
            // that could have gone suspect.
            Target::Document { .. } => return Outcome::Skipped(DOCUMENT_TARGET.to_string()),
            _ => return Outcome::Passed,
        };

        let verified = edge.verified_revision();

        // A fix writes `verified_revision`, so it is offered only where the
        // relation declares that attribute: spec 2 makes an undeclared one a
        // finding, and a fix must not write what a later rule refuses.
        let today = self.verified_today(view)
            && self
                .declared
                .iter()
                .find(|known| known.name == view.relation())
                .is_some_and(|known| known.attributes.iter().any(|a| a == VERIFIED_REVISION));
        let (line, column) = at(Some(edge.span));
        let finding = |severity, message, remediation, patch| Finding {
            rule: self::RULE,
            severity,
            obligation: None,
            path: edge.source.path.clone(),
            line,
            column,
            message,
            remediation,
            patch,
        };

        let Some(current) = revision.as_deref() else {
            // The source tree binds a literal that names a directory and gives
            // it no digest (see `tree_revision`), so this edge could never go
            // suspect. That is reported rather than passed, with the wildcard
            // that does carry a digest as the remedy. No patch: the remedy
            // widens what the edge reaches, which is the author's to decide.
            // Every other edge with no revision keeps its silence. A list with
            // one directory member has no digest either (#1104), and gets one
            // finding that names every such member.
            let members = directory_members(resolver, patterns);
            return match (patterns.as_slice(), members.as_slice()) {
                (_, []) => Outcome::Passed,
                ([_], [literal]) => Outcome::failed_with(finding(
                    Severity::Info,
                    directory(&edge.source.id, &edge.name, literal),
                    format!(
                        "write `{literal}/**` to govern the entries under it, which carries a \
                         digest this rule compares, or name the one file the document governs"
                    ),
                    None,
                )),
                (_, members) => Outcome::failed_with(finding(
                    Severity::Info,
                    directories(&edge.source.id, &edge.name, &edge.raw_target, members),
                    format!(
                        "write {} in the list to govern the entries under each directory, which \
                         carries a digest this rule compares, or name the files the document \
                         governs",
                        spoken(members.iter().map(|member| format!("`{member}/**`")))
                    ),
                    None,
                )),
            };
        };

        let Some(verified) = verified else {
            // The one report an unrecorded edge gets. See the module comment.
            if !today || resolver != SOURCE_TREE {
                return Outcome::Passed;
            }
            let Some(patch) = recording(edge, current) else {
                return Outcome::Passed;
            };
            return Outcome::failed_with(finding(
                Severity::Info,
                unrecorded(&edge.source.id, &edge.name, &edge.raw_target, current),
                format!(
                    "run `headwater check --fix` to record `{VERIFIED_REVISION}: {current}` on \
                     this entry, so that a later change to what it reaches is reported"
                ),
                Some(patch),
            ));
        };

        // The comparison is `Edge::suspect_revisions`, which `headwater route`
        // reads too (#953), so the two surfaces name the same edges.
        let Some((verified, current)) = edge.suspect_revisions() else {
            return Outcome::Passed;
        };

        let reached: usize = {
            let mut union: Vec<&String> = patterns.iter().flat_map(|p| p.matched.iter()).collect();
            union.sort();
            union.dedup();
            union.len()
        };
        let literal = patterns.len() == 1
            && patterns[0].matched.len() == 1
            && patterns[0].matched[0] == patterns[0].pattern;

        let (message, remediation) = match resolver.as_str() {
            SOURCE_TREE => (
                moved(
                    &edge.source.id,
                    &edge.name,
                    &edge.raw_target,
                    verified,
                    current,
                    match literal {
                        true => Reach::One,
                        false => Reach::Set(reached),
                    },
                ),
                reread(current),
            ),
            _ => (
                message(
                    &edge.source.id,
                    &edge.name,
                    &edge.raw_target,
                    verified,
                    current,
                ),
                remediation(current),
            ),
        };
        // A snapshot's revision is never fixed here: see `remediation`. A
        // source-tree digest is, on a document verified today, because then
        // the verification the patch records is one the author stated.
        let patch = match resolver == SOURCE_TREE && today {
            true => recording(edge, current),
            false => None,
        };
        Outcome::failed_with(finding(Severity::Warn, message, remediation, patch))
    }
}

/// Each member of a `source-tree` anchor that is one literal pattern, matched
/// only itself, and got no digest from the resolver: a directory, which
/// [`headwater_graph::anchors::tree_revision`] refuses to digest. One such
/// member leaves a list with no digest over its union too. Empty for every
/// other resolver, and for an anchor with no such member.
fn directory_members<'e>(
    resolver: &str,
    patterns: &'e [headwater_graph::edges::PatternMember],
) -> Vec<&'e str> {
    if resolver != SOURCE_TREE {
        return Vec::new();
    }
    patterns
        .iter()
        .filter(|member| {
            member.revision.is_none()
                && member.matched.len() == 1
                && member.matched[0] == member.pattern
        })
        .map(|member| member.pattern.as_str())
        .collect()
}

fn directory(id: &str, name: &str, literal: &str) -> String {
    format!(
        "`{id}` declares `{name}: {literal}`, and `{literal}` names a directory, which the source \
         tree takes no digest of, so this edge never goes suspect when what it governs changes"
    )
}

/// [`directory`] for a list: the declaration as written, and each member that
/// names a directory.
fn directories(id: &str, name: &str, raw: &str, members: &[&str]) -> String {
    let verb = match members {
        [_] => "names",
        _ => "each name",
    };
    format!(
        "`{id}` declares `{name}: {raw}`, and {} {verb} a directory, which the source tree takes \
         no digest of, so this edge never goes suspect when what it governs changes",
        spoken(members.iter().map(|member| format!("`{member}`")))
    )
}

/// `a`, `a and b`, or `a, b and c`.
fn spoken(items: impl Iterator<Item = String>) -> String {
    let items: Vec<String> = items.collect();
    match items.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

/// The patch that records `current` on the entry that declared `edge`, and
/// nothing where the splice could not find that entry by one scalar `to` or
/// could not carry an attribute the entry already has.
fn recording(edge: &Edge, current: &str) -> Option<Patch> {
    if edge.direction != Direction::AsDeclared {
        return None;
    }
    let Target::Anchor { patterns, .. } = &edge.target else {
        return None;
    };
    if patterns.len() != 1 {
        return None;
    }
    let mut attributes = Vec::with_capacity(edge.attributes.len() + 1);
    for entry in &edge.attributes {
        if entry.key.value == VERIFIED_REVISION {
            continue;
        }
        let scalar = entry.value.value.as_scalar()?;
        attributes.push((entry.key.value.clone(), scalar.text.clone()));
    }
    attributes.push((VERIFIED_REVISION.to_string(), current.to_string()));
    Some(Patch::Half {
        path: edge.source.path.clone(),
        relation: edge.name.clone(),
        id: edge.raw_target.clone(),
        attributes,
    })
}

/// How many entries a moved source-tree anchor reaches, and whether that is a
/// count of changed entries.
#[derive(Clone, Copy)]
enum Reach {
    /// A literal path: one entry, so the one that changed.
    One,
    /// A pattern or a list: this many entries now, and the digest does not say
    /// how many of them changed.
    Set(usize),
}

/// A moved digest, in the terms of the document that declares the edge.
fn moved(id: &str, name: &str, raw: &str, verified: &str, current: &str, reach: Reach) -> String {
    match reach {
        Reach::One => format!(
            "`{id}` declares `{name}: {raw}`, which was verified against content `{verified}`, \
             and the 1 entry it reaches has changed since: it now reads `{current}`"
        ),
        Reach::Set(count) => format!(
            "`{id}` declares `{name}: {raw}`, which was verified against content `{verified}`, \
             and the {count} {entries} it matches now read `{current}`; the digest covers the \
             set, so how many of them changed is not recorded",
            entries = match count {
                1 => "entry",
                _ => "entries",
            }
        ),
    }
}

/// What to do about a moved digest.
fn reread(current: &str) -> String {
    format!(
        "re-read what the entry reaches and correct this document where it no longer holds; then \
         set its `last_verified` to today and run `headwater check --fix`, which records \
         `{VERIFIED_REVISION}: {current}` on this entry"
    )
}

/// An unrecorded edge on a document verified today.
fn unrecorded(id: &str, name: &str, raw: &str, current: &str) -> String {
    format!(
        "`{id}` declares `{name}: {raw}` with no `{VERIFIED_REVISION}`, and the document was \
         verified today, so the content it reaches now, `{current}`, can be recorded"
    )
}

/// What moved, in the terms of the document that declares the edge.
fn message(id: &str, name: &str, raw: &str, verified: &str, current: &str) -> String {
    format!(
        "`{id}` declares `{name}: {raw}`, which was verified at revision `{verified}` and the \
         snapshot now pins revision `{current}`"
    )
}

/// What to do about it, which is one sentence because there is one remedy.
///
/// It is a hand edit and not a re-import, and that is a choice about who looks
/// at the changed item rather than about what a splice writes.
/// `headwater_import::write::declares` reads "same target, different revision"
/// as absent on purpose, so that a moved revision keeps proposing the edge
/// rather than going quiet — silencing it there would defeat the drift report
/// this whole rule exists for. `headwater_scaffold::write::splice` then
/// replaces the one entry a target already has rather than appending beside it
/// ([#384](https://github.com/headwater-ai/headwater/issues/384)), so a
/// re-import over a moved revision now writes the correct single entry rather
/// than a second one. What it still would not do is have a person decide
/// whether the edge holds against the item as it now reads — see the `patch:
/// None` comment above this function's call site — and that decision is the
/// whole reason the remedy stays a sentence naming the field a person sets by
/// hand, not an instruction to run the import again.
fn remediation(current: &str) -> String {
    format!(
        "re-read the upstream item, and where the edge still holds set \
         `{VERIFIED_REVISION}: {current}` on this entry"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two sentences a reader gets, on the precedent of
    /// [`crate::target`]'s table: the fixture corpus of this crate declares no
    /// importable relation, so no case there can reach either string.
    #[test]
    fn the_finding_names_both_revisions_and_the_remedy_names_the_attribute() {
        let text = message("SPEC-FIX-one", "audited_by", "12345", "7", "8");
        assert!(text.contains("SPEC-FIX-one"), "{text}");
        assert!(text.contains("audited_by: 12345"), "{text}");
        assert!(text.contains("verified at revision `7`"), "{text}");
        assert!(text.contains("pins revision `8`"), "{text}");

        let remedy = remediation("8");
        assert!(remedy.contains("verified_revision: 8"), "{remedy}");
        // And it never names a verb that would write a second entry for one
        // target. See the comment on `remediation`.
        assert!(!remedy.contains("headwater import"), "{remedy}");
    }

    /// The message says which revision is which. A sentence that named them in
    /// the other order would send an author to re-verify against the value the
    /// edge already carries.
    #[test]
    fn the_recorded_revision_and_the_pinned_one_are_never_the_same_side() {
        let text = message("SPEC-FIX-one", "audited_by", "12345", "7", "8");
        let verified = text.find("verified at revision").expect("it is there");
        let pinned = text.find("now pins revision").expect("it is there");
        assert!(verified < pinned, "{text}");
    }
}
