// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check, generated from the relation names a taxonomy
//! declares: a relation written as a top-level front-matter key, outside the
//! block the graph reads edges from.
//!
//! [Spec 9, Q4](../../../../docs/spec/09-decisions.md) puts every edge half
//! under one key of the front matter, `relations:` unless the taxonomy
//! configures another, and the graph reads that key and no other. So a
//! document that writes `conflicts_with: [X]` at the top level declares no
//! edge. Before this rule nothing said so: the key was a facet that no kind
//! declares, every rule passed it, and the author believed an edge was there
//! ([#1639](https://github.com/headwater-ai/headwater/issues/1639)). One trial
//! the issue cites scripted the move for 58 records by hand.
//!
//! Nothing below names a relation. The population is every declared relation
//! name and every declared inverse name, so a taxonomy that declares one more
//! relation gets one more key read and this file does not change.
//!
//! # A name that is also a facet draws no finding
//!
//! A taxonomy can declare a facet and a relation with one name. A top-level
//! key of that name is then a facet the author meant, as far as anything here
//! can tell, and a move under `relations:` would take a value away from a
//! facet that reads it. The two readings cannot be told apart without
//! judgment, so the rule reads the name as the facet and reports nothing.
//!
//! The configured key itself is never read as a relation, for the same reason
//! from the other side: it is the block.
//!
//! # Error, with a patch
//!
//! The remedy is mechanical and total: delete the top-level key and write each
//! of its entries as a half of the same name under the block. That is the one
//! correct outcome, so the finding carries a [`Patch::Relocate`], and a
//! mechanical and total remedy is an error by `CLAUDE.md`'s rule. The patch is
//! offered for a value that is an identifier or a list of identifiers. Any
//! other shape, a mapping or a list that holds one, is reported with no patch,
//! because which member is the target and which are attributes is a reading
//! the writer would have to guess.
//!
//! The writer for the halves is
//! [`headwater_scaffold::write::splice`](../../scaffold/src/write.rs), which
//! writes under `relations:` and under no other key. So where a taxonomy
//! configures another key, the finding names that key in its remediation and
//! carries no patch: a patch that moved the key into a block the graph does
//! not read would be a second silent drop.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::patch::Patch;
use crate::scope::{DocumentCheck, DocumentView, ExportTargets};
use crate::shape::Shape;
use headwater_graph::Declarations;
use headwater_yaml::value::Value;

pub const RULE: &str = "relation.key.outside_block";

/// The one block name the splice writes. See the module comment.
const SPLICED: &str = "relations";

/// The check, generated from the relation declarations.
pub struct OutsideBlock {
    /// Every declared relation name and inverse name that no facet shares, in
    /// declaration order and each once.
    names: Vec<String>,
    /// The key the graph reads edges from.
    block: String,
}

impl OutsideBlock {
    /// The generation step, in full.
    pub fn over(declarations: &Declarations, shape: &Shape, block: &str) -> Self {
        let mut names: Vec<String> = Vec::new();
        for relation in &declarations.relations {
            for name in std::iter::once(&relation.name).chain(relation.inverse.as_ref()) {
                let facet = shape.facets.iter().any(|facet| &facet.name == name);
                if facet || name == block || names.contains(name) {
                    continue;
                }
                names.push(name.clone());
            }
        }
        OutsideBlock {
            names,
            block: block.to_string(),
        }
    }

    /// The identifiers a value names, in source order, and nothing for a shape
    /// the splice cannot express as bare halves.
    fn identifiers(value: &Value) -> Option<Vec<String>> {
        let named = |value: &Value| {
            value
                .as_scalar()
                .filter(|scalar| !headwater_yaml::core_schema::as_null(scalar))
                .filter(|scalar| !scalar.text.trim().is_empty())
                .map(|scalar| scalar.text.clone())
        };
        match value {
            Value::Seq(items) => items
                .iter()
                .map(|item| named(&item.value))
                .collect::<Option<Vec<String>>>()
                .filter(|ids| !ids.is_empty()),
            other => named(other).map(|id| vec![id]),
        }
    }
}

impl DocumentCheck for OutsideBlock {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 1;
    /// No emitter target. A JSON Schema could forbid the keys, but not move
    /// them, and the finding's patch is half of what the rule is.
    const EXPORTABLE_AS: ExportTargets = &[];

    /// Every kind, when the taxonomy declares a relation name at all. A key is
    /// a relation name whatever kind writes it, and a kind that may sit at no
    /// end of the relation still dropped the edge it meant: that the edge is
    /// then also an endpoint defect is `relation.endpoint` reading the moved
    /// half.
    fn instantiates(&self, _kind: &str) -> bool {
        !self.names.is_empty()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let block = &self.block;
        let findings = view
            .facets()
            .iter()
            .filter(|entry| self.names.contains(&entry.key.value))
            .map(|entry| {
                let key = &entry.key.value;
                let ids = Self::identifiers(&entry.value.value);
                let patch = match (&ids, block.as_str() == SPLICED) {
                    (Some(ids), true) => Some(Patch::Relocate {
                        path: view.path().to_string(),
                        key: key.clone(),
                        ids: ids.clone(),
                    }),
                    _ => None,
                };
                let (line, column) = at(Some(entry.key.span));
                Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    line,
                    column,
                    message: format!(
                        "`{key}` is a relation, and it is written as a top-level key outside \
                         `{block}:`, so the graph read no edge from it"
                    ),
                    remediation: match ids {
                        Some(_) => format!(
                            "move `{key}` and its entries under `{block}:` in {}",
                            view.path()
                        ),
                        None => format!(
                            "write each target of `{key}` as an entry of `{key}` under \
                             `{block}:` in {}, and remove the top-level key",
                            view.path()
                        ),
                    },
                    patch,
                }
            })
            .collect();
        Outcome::failed(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(items: &[&str]) -> Value {
        let parsed = headwater_yaml::load(&format!("k: [{}]\n", items.join(", ")))
            .expect("the fixture parses");
        parsed
            .value
            .as_map()
            .and_then(|map| map.get("k"))
            .map(|value| value.value.clone())
            .expect("the key is there")
    }

    /// The shapes a value takes, and which of them the splice can carry.
    #[test]
    fn only_an_identifier_or_a_list_of_identifiers_is_movable() {
        assert_eq!(
            OutsideBlock::identifiers(&seq(&["A", "B"])),
            Some(vec!["A".to_string(), "B".to_string()])
        );
        assert_eq!(OutsideBlock::identifiers(&seq(&[])), None);
        assert_eq!(OutsideBlock::identifiers(&seq(&["{to: A}"])), None);
    }
}
