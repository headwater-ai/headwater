// SPDX-License-Identifier: Apache-2.0
//! Two rules over the provenance block: a warrant outside the closed set, and
//! an acceptor the warrant does not pair with.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires)
//! states both in one paragraph. The warrant's "value set is closed", and
//! "each value requires a different part of the block, and the engine checks
//! the pairing". Before these rules no check read either half. A document at
//! `warrant: acepted` raised nothing, and a reader that tested for `asserted`
//! alone served the misspelling as a vouched document (#1438). No check read
//! `accepted_by` either (#1437).
//!
//! # The set is the engine's, so no taxonomy generates this rule
//!
//! The rule follows [`crate::facet_value`], and differs in where the set comes
//! from. A facet's values are a declaration. The provenance block is the
//! engine's, so the four values are [`headwater_doc::WARRANTS`], and one
//! template instantiates over every typed document. [`crate::placement`] is
//! universal on the same terms, so this rule makes no document count as
//! checked that was not already counted, and coverage loses nothing.
//!
//! # Absent is not a bad value, and this rule skips it
//!
//! Spec 3 also says "an absent value is a finding rather than a default". That
//! is a separate defect with a separate remedy, and 134 documents of this
//! repository declare no warrant on the day these rules landed.
//! [HW-OBL-0125](../../../../docs/obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md)
//! holds it. So an absent warrant, and a `warrant:` key with no value, skip
//! with a reason. A rule that reported them here would make one finding mean
//! two things.
//!
//! # A generated document is not read
//!
//! Spec 3 derives `regenerated` from the marker "and never from a
//! declaration". [`crate::scope::over_documents`] creates no document instance
//! over a generated file, so a stray `warrant:` that a hand edit put into one
//! is judged by neither rule, which is what [`headwater_doc::warrant_of`]
//! answers for it too.
//!
//! # Both are errors, and neither offers a fix
//!
//! A value outside the set makes a record look more trusted than it is, or
//! less, and every reader downstream takes it as the author wrote it. An
//! `accepted` warrant that names nobody claims a human act that the record
//! does not show. Neither has a false positive on this repository's corpus
//! except the nine documents at `proposed`, and those are adoption debt in
//! `.headwater/taxonomy.lock` until their owner rules. The remedy is a choice
//! between a typo and an intent, which only the author can make, so no
//! finding carries a patch.
//!
//! # What the pairing rule does not check
//!
//! It checks that `accepted_by` names somebody, and never that it names a
//! human. [HW-OBL-0108](../../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md)
//! records that an agent writes every acceptance stamp in this corpus, and a
//! rule that reads a name cannot tell who wrote it. The marker and snapshot
//! requirements of the `regenerated` and `transcribed` rows are not read here
//! either.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use headwater_doc::{ACCEPTED, ACCEPTED_BY, WARRANT, WARRANTS};
use headwater_yaml::core_schema::as_null;
use headwater_yaml::{Entry, Mapping, Value};

/// A warrant outside spec 3's closed set.
pub const VALUE: &str = "warrant.value.not_permitted";

/// An acceptor where the warrant forbids one, or none where it requires one.
pub const UNPAIRED: &str = "warrant.acceptance.unpaired";

/// The skip reason for a document that declares no warrant.
///
/// It names no identifier of this repository, because an adopter's report
/// carries it too.
const ABSENT: &str = "this document declares no warrant, and an absent warrant is not a value \
                      outside the closed set";

/// The declared warrant of one document, as this module reads it.
enum Declared<'a> {
    /// No provenance block, no `warrant` key, or a key with no value.
    Absent,
    /// One of the four values.
    Member(&'a str),
    /// Anything else, with the entry that holds it and the text to quote.
    Outside(&'a Entry, String),
}

fn declared(facets: &Mapping) -> Declared<'_> {
    let Some(entry) = headwater_doc::provenance(facets).and_then(|block| block.entry(WARRANT))
    else {
        return Declared::Absent;
    };
    match &entry.value.value {
        Value::Scalar(scalar) if as_null(scalar) => Declared::Absent,
        Value::Scalar(scalar) if WARRANTS.contains(&scalar.text.as_str()) => {
            Declared::Member(scalar.text.as_str())
        }
        Value::Scalar(scalar) => Declared::Outside(entry, scalar.text.clone()),
        other => Declared::Outside(entry, other.kind_name().to_string()),
    }
}

/// The four values as a message quotes them.
fn listed() -> String {
    WARRANTS
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The check for the value. It carries nothing: the set is the engine's.
pub struct Closed;

impl DocumentCheck for Closed {
    const RULE: &'static str = self::VALUE;
    /// The first edition. See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let (entry, written) = match declared(view.facets()) {
            Declared::Absent => return Outcome::Skipped(ABSENT.to_string()),
            Declared::Member(_) => return Outcome::Passed,
            Declared::Outside(entry, written) => (entry, written),
        };
        let quoted = match entry.value.value.as_scalar() {
            Some(_) => format!("`{written}`"),
            None => written,
        };
        let (line, column) = at(Some(entry.value.span));
        Outcome::failed_with(Finding {
            rule: self::VALUE,
            severity: Severity::Error,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "the warrant is {quoted}, and spec 3 closes the warrant at {}: a value outside \
                 the set says nothing about who read this document",
                listed()
            ),
            remediation: format!(
                "set `provenance.warrant` in {} to one of {}: `accepted` only where a person \
                 read it and is named in `accepted_by`, and `asserted` where nobody did",
                view.path(),
                listed()
            ),
            patch: None,
        })
    }
}

/// The check for the pairing. It carries nothing either.
pub struct Pairing;

impl DocumentCheck for Pairing {
    const RULE: &'static str = self::UNPAIRED;
    /// The first edition. See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let warrant = match declared(view.facets()) {
            Declared::Absent => return Outcome::Skipped(ABSENT.to_string()),
            // The value rule reports it, and an acceptor pairs with nothing
            // outside the set, so a second finding here would say the same
            // thing from a second place.
            Declared::Outside(..) => {
                return Outcome::Skipped(format!(
                    "the warrant is outside the closed set, which `{VALUE}` reports, so there is \
                     no row of spec 3's table to pair `{ACCEPTED_BY}` against"
                ))
            }
            Declared::Member(warrant) => warrant,
        };
        let Some(block) = headwater_doc::provenance(view.facets()) else {
            return Outcome::Skipped(ABSENT.to_string());
        };
        let acceptor = block.entry(ACCEPTED_BY);
        // Named means at least one name that is not empty. `accepted_by:` with
        // nothing after it names nobody, and a presence test would pass it. A
        // list names whoever it lists, as `drafted_by` does in this corpus, and
        // an empty list names nobody.
        let names = |value: &Value| {
            value
                .as_scalar()
                .is_some_and(|scalar| !as_null(scalar) && !scalar.text.trim().is_empty())
        };
        let named = acceptor.is_some_and(|entry| match &entry.value.value {
            Value::Seq(items) => items.iter().any(|item| names(&item.value)),
            other => names(other),
        });

        let finding = |anchor: &Entry, message: String, remediation: String| Finding {
            rule: self::UNPAIRED,
            severity: Severity::Error,
            obligation: None,
            path: view.path().to_string(),
            line: at(Some(anchor.key.span)).0,
            column: at(Some(anchor.key.span)).1,
            message,
            remediation,
            patch: None,
        };

        let warrant_entry = block.entry(WARRANT).expect("a member warrant has an entry");
        match (warrant == ACCEPTED, named, acceptor) {
            (true, true, _) | (false, _, None) => Outcome::Passed,
            (true, false, _) => Outcome::failed_with(finding(
                acceptor.unwrap_or(warrant_entry),
                format!(
                    "the warrant is `{ACCEPTED}`, and `{ACCEPTED_BY}` names nobody: spec 3 \
                     requires an accepted document to say who accepted it"
                ),
                format!(
                    "name the person who read {} in `provenance.{ACCEPTED_BY}`, or set the \
                     warrant to `asserted` if nobody did",
                    view.path()
                ),
            )),
            (false, _, Some(entry)) => Outcome::failed_with(finding(
                entry,
                format!(
                    "the warrant is `{warrant}`, and the document also names `{ACCEPTED_BY}`: \
                     spec 3 forbids an acceptor at `{warrant}`, because a document that names one \
                     is `{ACCEPTED}`"
                ),
                format!(
                    "remove `provenance.{ACCEPTED_BY}` from {}, or set the warrant to \
                     `{ACCEPTED}` if that person read it",
                    view.path()
                ),
            )),
        }
    }
}
