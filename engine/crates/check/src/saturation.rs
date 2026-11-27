// SPDX-License-Identifier: Apache-2.0
//! How many entries of each lexical pattern set produced a finding (#1492,
//! HW-OBL-0168).
//!
//! `voice.forbidden_construction` reads one curated set of patterns per
//! category, and `language.retired_term.used` reads one list of terms per
//! regime. A zero from either rule is a clean corpus, or a set whose entries
//! match nothing an author writes, and the findings list cannot tell the two
//! apart. This projection prints, for each set, the entries that produced a
//! finding against the entries the set declares.
//!
//! It is a projection over findings, read after the rules and before the
//! adoption payload and the directives remove any finding. A suppressed
//! finding still shows that its pattern is live, so a count taken after the
//! filters would undercount. No finding and no cache key carries the count,
//! so it moves no rule version. The voice rule names only the first pattern of
//! a category that a sentence writes ([`crate::voice`]), so the count is of
//! patterns that produced a finding and not of patterns that occur.

use std::collections::BTreeSet;

use crate::finding::Finding;

/// One pattern set and its count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Set {
    /// The rule that reads the set.
    pub rule: &'static str,
    /// The voice category, or the name of the language regime.
    pub name: String,
    /// What one entry is called in the report: `patterns` or `terms`.
    pub unit: &'static str,
    /// The entries the set declares.
    pub declared: usize,
    /// The entries that a finding of this run names.
    pub matched: usize,
}

/// Every pattern set of the run, the voice categories first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Saturation {
    pub sets: Vec<Set>,
}

impl Saturation {
    /// The projection. `voice` and `retired` are the declared sets, from
    /// [`crate::voice::Voice::pattern_sets`] and
    /// [`crate::retired::Retired::pattern_sets`]. `findings` is the list
    /// before any filter.
    pub fn project(
        voice: Vec<(&'static str, usize)>,
        retired: Vec<(String, usize)>,
        findings: &[Finding],
    ) -> Self {
        let mut voiced: BTreeSet<(&str, &str)> = BTreeSet::new();
        let mut terms: BTreeSet<(&str, &str)> = BTreeSet::new();
        for finding in findings {
            if finding.rule == crate::voice::RULE {
                voiced.extend(crate::voice::named(&finding.message));
            } else if finding.rule == crate::retired::RULE {
                terms.extend(crate::retired::named(&finding.message));
            }
        }
        let count = |named: &BTreeSet<(&str, &str)>, set: &str| {
            named.iter().filter(|(name, _)| *name == set).count()
        };
        let mut sets: Vec<Set> = voice
            .into_iter()
            .map(|(category, declared)| Set {
                rule: crate::voice::RULE,
                name: category.to_string(),
                unit: "patterns",
                declared,
                matched: count(&voiced, category),
            })
            .collect();
        sets.extend(retired.into_iter().map(|(regime, declared)| Set {
            rule: crate::retired::RULE,
            matched: count(&terms, &regime),
            name: regime,
            unit: "terms",
            declared,
        }));
        Saturation { sets }
    }

    /// One line per set, and nothing for a run with no set.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for set in &self.sets {
            out.push_str(&format!(
                "pattern set {} {}: {} of {} {} produced a finding",
                set.rule, set.name, set.matched, set.declared, set.unit
            ));
            if set.matched == 0 {
                out.push_str(", so a zero here does not show a clean corpus");
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Severity;

    fn finding(rule: &'static str, message: &str) -> Finding {
        Finding {
            rule,
            severity: Severity::Warn,
            obligation: None,
            path: "docs/a.md".to_string(),
            line: 1,
            column: 1,
            message: message.to_string(),
            remediation: String::new(),
            patch: None,
        }
    }

    /// Two findings of one pattern count it once, a facet subject with its own
    /// backticks still reads back, and a set no finding names reads zero.
    #[test]
    fn a_pattern_counts_once_and_a_silent_set_reads_zero() {
        let findings = [
            finding(
                crate::voice::RULE,
                "`declarative` forbids future_intent, and this sentence writes `will be`",
            ),
            finding(
                crate::voice::RULE,
                "`declarative` forbids future_intent, and the `summary` facet writes `will be`",
            ),
            finding(
                crate::retired::RULE,
                "`house` retires `delve`, and this sentence writes it: a reason with `code`",
            ),
        ];
        let projected = Saturation::project(
            vec![("future_intent", 14), ("phased_rollout", 13)],
            vec![("house".to_string(), 2)],
            &findings,
        );
        assert_eq!(
            projected.render(),
            "pattern set voice.forbidden_construction future_intent: 1 of 14 patterns produced a finding\n\
             pattern set voice.forbidden_construction phased_rollout: 0 of 13 patterns produced a finding, so a zero here does not show a clean corpus\n\
             pattern set language.retired_term.used house: 1 of 2 terms produced a finding\n"
        );
    }
}
