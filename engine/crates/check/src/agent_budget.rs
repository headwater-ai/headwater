// SPDX-License-Identifier: Apache-2.0
//! `projection.budget.exceeded`: an agent rule file past its projection's byte
//! budget, as a warning.
//!
//! [HW-DR-0115](../../../../docs/decisions/0115-an-agent-rules-file-is-a-budgeted-pointer-at-the-copilot-instructions-path.md)
//! gives an `agent_rules` projection two figures: `per_file`, which bounds one
//! rule file, and `aggregate`, which bounds every rule file of the projection
//! together. `headwater generate` writes no rule file past `per_file`, and it
//! prunes rules from the satellite end of the derived reading precedence until
//! the rest fit `aggregate` (#2090). A prune that nothing reported would drop a
//! rule in silence, so this rule reports each one:
//!
//! - **Per file.** One finding for each rule file past `per_file`, against the
//!   path the file would have. It names the size, the figure, and the layer to
//!   trim: the `summary` of the source document, or the figure.
//! - **Aggregate.** One finding for each projection whose rules, after the
//!   per-file withholds, total more than `aggregate`. It names the projection,
//!   the total, the figure, and how many rules the prune withholds.
//!
//! **Why the caller takes the reading.** The emitter measures the rules, and
//! `headwater-generate` depends on this crate, so this crate cannot call it.
//! The caller calls `headwater_generate::agent_rules::readings` once per run
//! and hands the result here, so the finding and the prune read one
//! measurement and cannot disagree.
//!
//! **Why a warning.** The remedy is a rewrite of a summary or a choice of
//! budget, and neither is mechanical. A prune already keeps the written set
//! inside the budget, so nothing an agent loads is past it while the warning
//! stands.
//!
//! The rule is about a projection read against the corpus, so it creates no
//! instance, and it enters the findings list for the reason
//! [`crate::lock_stale`] does. It is never cached: the reading is taken on
//! every run. Its inputs are the source documents, which the census already
//! puts in the read set, and the lock, which keys the cache.

use crate::finding::{Finding, Severity};
use crate::scope::Scope;

pub const RULE: &str = "projection.budget.exceeded";

/// The grain of [`RULE`]: a fact about a projection read against the corpus,
/// so it creates no instance and accounts nothing against the census.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of [`findings`] reached a verdict.
///
/// Raise the rule version when a rule's behavior changes.
pub const VERSION: u32 = 1;

/// Empty: a front-matter schema has no instance to hold this against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// One rule file past `per_file`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Over {
    /// The path the rule file would have, relative to the root.
    pub path: String,
    /// The source document the rule points at.
    pub source: String,
    /// The size of the rendered rule file, in bytes.
    pub bytes: u64,
}

/// One `agent_rules` projection that declares a budget, measured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// The position of the projection in the taxonomy's `projections` list.
    pub index: usize,
    /// The declared `output`.
    pub output: String,
    pub per_file: u64,
    pub aggregate: u64,
    /// Every rule file past `per_file`, in path order.
    pub over: Vec<Over>,
    /// The bytes of every rule file that is not past `per_file`, together.
    pub total: u64,
    /// How many of those the prune withholds to fit `aggregate`.
    pub pruned: usize,
}

/// The findings of every reading. `source` is the path a projection finding
/// carries, which is the lock for a run of the verb.
pub fn findings(readings: &[Reading], source: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for reading in readings {
        let at = format!("projections.{}", reading.index);
        for over in &reading.over {
            out.push(finding(
                over.path.clone(),
                format!(
                    "`{}` is {} bytes, and `{at}.budget.per_file` is {}, so `headwater \
                     generate` does not write it. The layer to trim is the `summary` of `{}`, \
                     or `{at}.budget.per_file`",
                    over.path, over.bytes, reading.per_file, over.source
                ),
                format!(
                    "shorten the `summary` of `{}`, or raise `{at}.budget.per_file`, then run \
                     `headwater generate`",
                    over.source
                ),
            ));
        }
        if reading.total > reading.aggregate {
            out.push(finding(
                source.to_string(),
                format!(
                    "`{at}` (`agent_rules`, `{}`) holds rules of {} bytes together, and its \
                     `budget.aggregate` is {}, so the prune withholds {} {} from the satellite \
                     end of the reading order",
                    reading.output,
                    reading.total,
                    reading.aggregate,
                    reading.pruned,
                    match reading.pruned {
                        1 => "rule",
                        _ => "rules",
                    }
                ),
                format!(
                    "narrow the `for` or the `governs` edges of `{at}`, shorten the summaries \
                     of its sources, or raise `{at}.budget.aggregate`, then run `headwater \
                     generate`"
                ),
            ));
        }
    }
    out
}

fn finding(path: String, message: String, remediation: String) -> Finding {
    Finding {
        rule: RULE,
        severity: Severity::Warn,
        obligation: None,
        path,
        line: 0,
        column: 0,
        message,
        remediation,
        patch: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(total: u64, aggregate: u64) -> Reading {
        Reading {
            index: 2,
            output: ".github/instructions/{name}.instructions.md".to_string(),
            per_file: 600,
            aggregate,
            over: Vec::new(),
            total,
            pruned: 1,
        }
    }

    /// The aggregate gate in three directions: under, at and over the figure.
    /// At the figure is inside the budget, because the figure is what the
    /// projection may hold.
    #[test]
    fn an_aggregate_finding_is_owed_only_past_the_figure() {
        let lock = ".headwater/taxonomy.lock";
        assert!(findings(&[reading(999, 1000)], lock).is_empty());
        assert!(findings(&[reading(1000, 1000)], lock).is_empty());
        let over = findings(&[reading(1001, 1000)], lock);
        assert_eq!(over.len(), 1, "{over:?}");
        assert_eq!(over[0].path, lock);
        assert!(over[0].message.contains("`projections.2`"), "{over:?}");
        assert!(over[0].message.contains("1001 bytes"), "{over:?}");
        assert!(over[0].message.contains("withholds 1 rule "), "{over:?}");
    }

    #[test]
    fn a_per_file_finding_is_against_the_rule_path_and_names_the_summary() {
        let mut held = reading(0, 1000);
        held.over.push(Over {
            path: ".github/instructions/a.instructions.md".to_string(),
            source: "docs/a.md".to_string(),
            bytes: 700,
        });
        let found = findings(&[held], ".headwater/taxonomy.lock");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].path, ".github/instructions/a.instructions.md");
        assert!(found[0].message.contains("700 bytes"), "{found:?}");
        assert!(
            found[0].message.contains("the `summary` of `docs/a.md`"),
            "{found:?}"
        );
        assert!(
            found[0]
                .message
                .contains("`projections.2.budget.per_file` is 600"),
            "{found:?}"
        );
    }
}
