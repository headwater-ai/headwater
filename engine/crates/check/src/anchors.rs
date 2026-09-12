// SPDX-License-Identifier: Apache-2.0
//! The `check-rule` anchor resolver: a rule identifier this engine ships.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#behavior-at-the-limits)
//! rules that exactly one resolver owns each anchor kind, and that "a resolver
//! reads repository content or a committed snapshot, and never a live service".
//! This one reads neither. It reads [`crate::RULES`], the list of rule
//! templates the binary was compiled with, so its answer is a property of the
//! engine rather than of the corpus. That is the whole of it, and the rest of
//! this comment is what follows from it.
//!
//! # What it is for
//!
//! A requirement names what verifies it. 29148 names four verifying artifacts,
//! and before this resolver existed a check rule was the one of the four that
//! no edge could reach: a rule identifier is not a document, and no anchor kind
//! claimed it, so `verified_by` had nowhere to point
//! ([#411](https://github.com/headwater-ai/headwater/issues/411)). The prose of
//! a `Verification` section carried the coverage instead, and prose is not a
//! graph: nothing could report a requirement whose named rule this engine
//! stopped shipping.
//!
//! # Why the resolver lives in this crate
//!
//! `headwater-check` owns [`crate::RULES`] and already depends on
//! `headwater-graph`, so a type here can implement
//! [`headwater_graph::anchors::Resolver`] with no new crate and no new
//! dependency edge. `headwater_import::anchors::Items` took exactly this shape
//! for the same reason, and `engine/crates/cli/src/main.rs` is where a run puts
//! the two together, because the graph crate cannot name either of them.
//!
//! # A template is addressable and an instance is not
//!
//! [`crate::RULES`] is a list of *templates*. Twenty-five of them are generated
//! from the taxonomy, and a generated instance carries no identifier of its
//! own: `mechanism: check:<rule-id>` names a template
//! ([`crate::register::Register::mechanism`]), a suppression directive writes
//! `allow=language.retired_term.used`, and the adoption block keys on the same
//! strings. Nothing anywhere spells an instance, so nothing can point at one,
//! and this resolver binds templates alone.
//!
//! One consequence is worth stating rather than discovering. A taxonomy edit
//! that stops generating instances of a bound template does not break the
//! binding: the engine still ships the template, so the anchor still resolves,
//! and what moved is the instance count that `headwater check` reports. That is
//! the honest reading of the edge, which says that a rule exists and not that a
//! corpus runs it. The rule that reports a requirement nothing verifies is the
//! participation expectation, and it reads the edge.
//!
//! # It trims, and it normalizes nothing else
//!
//! `SourceTree` normalizes a path, because two spellings of one path name one
//! file and the rules of that equality are the rules of a path. A rule
//! identifier has no such rules: the identity is the string the engine ships,
//! and `Section.Required.Missing` is not that string. So the whole of the
//! normalization is [`str::trim`], for the reason
//! `headwater_import::anchors` gives: leading space in a YAML scalar belongs to
//! the format and not to the identity. A resolver that guessed would bind a
//! typo to a real rule, and [#158] rules that a resolver that binds anything is
//! worse than none.
//!
//! [HW-OBL-0061](../../../../docs/obligations/0061-an-anchor-resolver-normalizes-and-nothing-states-how.md)
//! records that the corpus states no normalization rule for a resolver in
//! general. This module states its own, which is evidence for that record and
//! not a discharge of it.
//!
//! [#158]: https://github.com/headwater-ai/headwater/issues/158

use headwater_graph::anchors::{Binding, Resolver};

/// The resolver namespace `anchors.check_rule` claims.
pub const RESOLVER: &str = "check-rule";

/// A resolver over the rule templates one engine ships.
///
/// The list is a parameter rather than a constant read inside [`Resolver`], so
/// that a case table can build one over a list it states in full. Every
/// production caller takes [`Rules::shipped`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    held: &'static [&'static str],
}

impl Rules {
    /// The rules this binary was compiled with.
    pub fn shipped() -> Self {
        Rules { held: &crate::RULES }
    }

    /// A resolver over a stated list, for a case table.
    pub fn over(held: &'static [&'static str]) -> Self {
        Rules { held }
    }

    /// How many rule identities this resolver can bind.
    pub fn held(&self) -> usize {
        self.held.len()
    }
}

impl Resolver for Rules {
    fn name(&self) -> &str {
        RESOLVER
    }

    fn resolve(&self, raw: &str) -> Binding {
        let id = raw.trim();
        if id.is_empty() {
            return Binding::Unresolved("an empty anchor names nothing".to_string());
        }
        match self.held.contains(&id) {
            // The identity is the string the engine ships, so the normalized
            // form is that string and never a form this resolver invented.
            true => Binding::Resolved {
                normalized: id.to_string(),
                excluded_by: None,
                // A rule template is at no revision. It moves with the binary,
                // and the lock digest that every read set already carries is
                // what a cached verdict keys on.
                revision: None,
            },
            false => Binding::Unresolved(format!(
                "this engine implements no rule `{id}`, and it ships {}",
                match self.held.len() {
                    1 => "1 rule".to_string(),
                    other => format!("{other} rules"),
                }
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list a case table binds against, stated here in full so that a
    /// failure names what the resolver was given. It is not `crate::RULES`:
    /// the parity test below is what holds the two together.
    const STATED: [&str; 2] = ["section.required.missing", "language.retired_term.used"];

    fn over() -> Rules {
        Rules::over(&STATED)
    }

    /// A rule the list holds binds to itself, and the binding carries no
    /// revision and no exclusion. Both empty fields are asserted rather than
    /// assumed: a corpus exclusion is about a path in this repository and a
    /// rule identifier is under no path, and a rule template is at no revision
    /// a run can name.
    #[test]
    fn a_rule_this_engine_ships_binds_to_the_string_it_ships() {
        assert_eq!(
            over().resolve("section.required.missing"),
            Binding::Resolved {
                normalized: "section.required.missing".to_string(),
                excluded_by: None,
                revision: None,
            }
        );
    }

    /// Surrounding space is the format's and not the identity's, so it is
    /// trimmed, and nothing else about the string is touched.
    #[test]
    fn it_trims_and_normalizes_nothing_else() {
        assert_eq!(
            over().resolve("  language.retired_term.used\n"),
            Binding::Resolved {
                normalized: "language.retired_term.used".to_string(),
                excluded_by: None,
                revision: None,
            }
        );
        // Case, separator and prefix are three ways to spell a rule this engine
        // does not ship. A resolver that admitted any of them would bind a typo
        // to a real rule, which is the failure the module comment names.
        for spelling in [
            "Section.Required.Missing",
            "section_required_missing",
            "check:section.required.missing",
            "section.required",
        ] {
            assert!(
                matches!(over().resolve(spelling), Binding::Unresolved(_)),
                "`{spelling}` is not the string this engine ships"
            );
        }
    }

    /// A refusal names the offending identifier, because the author is looking
    /// at the line that spells it.
    #[test]
    fn a_refusal_names_the_identifier_and_says_how_many_rules_there_are() {
        let Binding::Unresolved(why) = over().resolve("no.such.rule") else {
            panic!("a rule this engine does not ship binds to nothing");
        };
        assert_eq!(why, "this engine implements no rule `no.such.rule`, and it ships 2 rules");
    }

    /// An empty string is refused with its own sentence rather than with the
    /// membership one, which would report that this engine ships no rule `` .
    #[test]
    fn an_empty_anchor_is_refused_on_its_own_terms() {
        assert_eq!(
            over().resolve("   "),
            Binding::Unresolved("an empty anchor names nothing".to_string())
        );
    }

    /// The name is the resolver namespace `anchors.check_rule` claims, and the
    /// taxonomy and this string are the two halves of one declaration.
    #[test]
    fn it_answers_to_the_namespace_the_taxonomy_names() {
        assert_eq!(over().name(), "check-rule");
        assert_eq!(Rules::shipped().held(), crate::RULES.len());
    }

    /// **The parity case.** A control and an edge that name one rule name one
    /// thing, and the two readers of [`crate::RULES`] are
    /// [`crate::register::Register::mechanism`] and this resolver. The case
    /// runs both over the same strings and asserts they agree, so a later
    /// change that widens one of them and not the other fails here.
    ///
    /// Without it the two could drift in silence: a rule the register calls
    /// implemented and the graph calls unresolvable is a corpus in which
    /// `mechanism: check:x` discharges an obligation while `verified_by: x`
    /// reports a defect on the same line of the same engine.
    #[test]
    fn the_resolver_binds_exactly_what_the_register_calls_implemented() {
        let resolver = Rules::shipped();
        let mut checked = 0;
        for rule in crate::RULES {
            assert!(
                matches!(resolver.resolve(rule), Binding::Resolved { .. }),
                "the register calls `{rule}` implemented, so the resolver binds it"
            );
            checked += 1;
        }
        assert_eq!(checked, crate::RULES.len(), "every shipped rule was asked");
        // The other arm, which is what stops the first from passing vacuously.
        // `no.such.rule` is the string the register's own case table pins as
        // unimplemented.
        assert!(matches!(
            resolver.resolve("no.such.rule"),
            Binding::Unresolved(_)
        ));
    }
}
