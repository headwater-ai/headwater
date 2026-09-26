// SPDX-License-Identifier: Apache-2.0
//! `language.outside_root.refused`: a path a language regime lists outside the
//! corpus root that reads nothing, as an error.
//!
//! [HW-DR-0084](../../../../docs/decisions/0084-a-language-rule-reaches-front-door-prose-outside-the-corpus-root-and-no-other-rule-does.md)
//! clause 2 refuses three kinds of entry and names a fourth. Two of the
//! refusals need no tree, a pattern that climbs out with `..` and an absolute
//! one, and `taxonomy resolve` refuses both under the projection-targets rule,
//! so a lock that carries either is never written. The rest need the tree: a
//! pattern that matches a path under the corpus root, a path a second regime
//! lists, a symlink, and a pattern that matches no file. The census reads the
//! tree, so this rule reports each of them, and an error is what makes
//! `check --strict` fail on it. A mistyped or deleted path is then a red gate
//! rather than a report line nobody reads.
//!
//! The rule is about the declaration and not about the prose of a file, so it
//! is not a fourth rule over an outside path in the sense of clause 5: it
//! creates no instance and reads no file. It enters the findings list for the
//! reason [`crate::adoption::expired`] does.

use crate::finding::{Finding, Severity};
use crate::scope::Scope;
use headwater_census::census::Census;

pub const RULE: &str = "language.outside_root.refused";

/// The grain of [`RULE`]: a fact about the taxonomy read against the tree, so
/// it creates no instance and accounts nothing against the census.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of [`findings`] reached a verdict.
pub const VERSION: u32 = 1;

/// Empty: a front-matter schema has no instance to hold this against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// One error for each listed pattern that reads nothing.
pub fn findings(census: &Census, source: &str) -> Vec<Finding> {
    let at = |message: String, remediation: &str| Finding {
        rule: RULE,
        severity: Severity::Error,
        obligation: None,
        path: source.to_string(),
        line: 0,
        column: 0,
        message,
        remediation: remediation.to_string(),
        patch: None,
    };
    let unmatched = census.outside.unmatched.iter().map(|entry| {
        at(
            format!(
                "`regimes.language.{}.outside_root` lists `{}`, and it matches no file: {}",
                entry.regime, entry.pattern, entry.reason
            ),
            "correct the pattern, or take it off the list if the file is gone",
        )
    });
    let refused = census.outside.refused.iter().map(|entry| {
        at(
            format!(
                "`regimes.language.{}.outside_root` lists `{}`, and HW-DR-0084 refuses it: {}",
                entry.regime, entry.pattern, entry.reason
            ),
            "take the pattern off the list, or list the real file outside the corpus root",
        )
    });
    unmatched.chain(refused).collect()
}
