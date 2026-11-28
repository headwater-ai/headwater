// SPDX-License-Identifier: Apache-2.0
//! `relation.scope.unmatched`: a governed scope pattern that matches no entry
//! of the tree, as an error.
//!
//! `taxonomy validate` refuses a governed scope pattern that matches no entry
//! of the tree beside the taxonomy (#951), and `headwater check` read the same
//! scope through `Scope::reach`, which drops the resolver's reason. So a
//! taxonomy that `validate` refused passed `check --strict` at exit 0
//! ([#2099](https://github.com/headwater-ai/headwater/issues/2099)). An
//! adopter's pattern `mod-*/src/main/**` opens with a wildcard, the resolver
//! refuses it before any walk, and the scope admitted no file while the strict
//! check passed.
//!
//! This rule reports each pattern that `validate` refuses, one error each, on
//! the lock. Both verbs take the one decision,
//! `headwater_graph::scope::Scope::coverage`, so the two cannot drift apart
//! again. The message names the pattern and the resolver's reason in the words
//! `validate` uses.
//!
//! **Where it is silent.** Where no tree stands beside the taxonomy, as for a
//! taxonomy published as its own repository, the count is skipped (owner
//! ruling 2026-09-25, #951 and #1103), and the text report prints the notice
//! `validate` prints. Where no lock was read, which is the candidate taxonomy
//! of `taxonomy diff` and `gate`, the caller takes no reading.
//!
//! **Where it differs from `validate`.** The check counts with nothing
//! ignored, because [the check layer](../../../../docs/spec/check-layer.md) keeps every version control command off the check
//! loop, and `validate` leaves out what git ignores. So a pattern whose only
//! matches are files git ignores passes the check and fails `validate`, and a
//! top directory git ignores counts as a tree here and not there. A pattern
//! that opens on a wildcard is refused before any walk and before any ignore
//! rule, so the adopter's case is not affected.
//!
//! **The resolver set.** The caller binds through the session's resolvers,
//! so the walks the graph build already took are reused. That set adds the
//! shipped rules, the imports snapshots, the corpus exports and
//! `comment-scan` to the set `validate` binds through, so the two answers can
//! differ only for a scope on an anchor kind whose resolver is one of those.
//! No anchor kind of `headwater/standard` or of this repository's overlay
//! declares such a resolver: the one resolver either names is `source-tree`.
//!
//! The rule is about the taxonomy read against the tree, so it creates no
//! instance, and it enters the findings list for the reason
//! [`crate::lock_stale`] does. It is never cached: the reading is taken on
//! every run by the caller. It adds nothing to the read set, because a walk
//! of the tree is not a list of files a gate can hash.

use crate::finding::{Finding, Severity};
use crate::scope::Scope;
use headwater_graph::scope::Coverage;

pub const RULE: &str = "relation.scope.unmatched";

/// The grain of [`RULE`]: a fact about the taxonomy read against the tree, so
/// it creates no instance and accounts nothing against the census.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of [`findings`] reached a verdict.
///
/// Raise the rule version when a rule's behavior changes.
pub const VERSION: u32 = 1;

/// Empty: a front-matter schema has no instance to hold this against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// The remedy every finding of this rule names.
const REMEDY: &str = "correct the pattern so that it names a directory or a file of the tree, \
                      with a literal directory before the first `*` or `**`, or take it off the \
                      scope; `headwater taxonomy validate` names the same pattern";

/// One error on `source` for each pattern that matches no entry, and nothing
/// where the count is skipped or no reading was taken.
pub fn findings(coverage: Option<&Coverage>, source: &str) -> Vec<Finding> {
    let Some(coverage) = coverage else {
        return Vec::new();
    };
    coverage
        .unmatched()
        .iter()
        .map(|(pattern, why)| Finding {
            rule: RULE,
            severity: Severity::Error,
            obligation: None,
            path: source.to_string(),
            line: 0,
            column: 0,
            message: format!(
                "governed scope pattern `{pattern}` matches no entry of the tree: {why}"
            ),
            remediation: REMEDY.to_string(),
            patch: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK: &str = ".headwater/taxonomy.lock";

    #[test]
    fn no_reading_is_silent() {
        assert!(findings(None, LOCK).is_empty());
    }

    #[test]
    fn a_skipped_count_is_silent() {
        assert!(findings(Some(&Coverage::NoTree { patterns: 3 }), LOCK).is_empty());
    }

    #[test]
    fn every_pattern_matched_is_silent() {
        assert!(findings(Some(&Coverage::Unmatched(Vec::new())), LOCK).is_empty());
    }

    #[test]
    fn each_unmatched_pattern_is_one_error_on_the_lock_that_names_it_and_the_reason() {
        let coverage = Coverage::Unmatched(vec![
            (
                "mod-*/src/**".to_string(),
                "it opens with a wildcard".to_string(),
            ),
            (
                "nowhere/**".to_string(),
                "no directory `nowhere`".to_string(),
            ),
        ]);
        let found = findings(Some(&coverage), LOCK);
        assert_eq!(found.len(), 2);
        assert!(found
            .iter()
            .all(|finding| finding.severity == Severity::Error && finding.path == LOCK));
        assert_eq!(
            found[0].message,
            "governed scope pattern `mod-*/src/**` matches no entry of the tree: it opens with a \
             wildcard"
        );
        assert!(found[1].message.contains("`nowhere/**`"));
    }
}
