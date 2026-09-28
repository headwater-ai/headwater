// SPDX-License-Identifier: Apache-2.0
//! A corpus check that a link label naming a record names its target record.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use headwater_graph::index::Index;
use headwater_graph::links::{Binding, Link};
use std::collections::HashSet;

pub const RULE: &str = "link.identifier.mismatch";

const NO_LINKS: &str = "the view carries no bound prose links for this corpus";
const NO_LINK_LABELS: &str = "the view carries no source label for a bound prose link";
const NO_IDENTIFIER_INDEX: &str = "the view carries no corpus identifier index";

pub struct IdentifierLinks;

impl CorpusCheck for IdentifierLinks {
    const RULE: &'static str = self::RULE;
    const VERSION: u32 = 2;
    const NEEDS_LINKS: bool = true;
    const NEEDS_LINK_LABELS: bool = true;
    const NEEDS_IDENTIFIER_INDEX: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(links) = view.links() else {
            return Outcome::Skipped(NO_LINKS.to_string());
        };
        let Some(index) = view.identifier_index() else {
            return Outcome::Skipped(NO_IDENTIFIER_INDEX.to_string());
        };

        let identifiers = identifiers(index);
        let mut missing_labels = false;
        let findings = links
            .iter()
            .filter_map(|link| match view.link_text(link) {
                Some(text) => finding(link, text, &identifiers),
                None => {
                    missing_labels = true;
                    None
                }
            })
            .collect();
        if missing_labels {
            return Outcome::Skipped(NO_LINK_LABELS.to_string());
        }
        Outcome::failed(findings)
    }
}

fn identifiers(index: &Index) -> HashSet<&str> {
    index
        .typed
        .iter()
        .chain(&index.untyped)
        .map(|node| node.id.as_str())
        .collect()
}

fn finding(link: &Link, link_text: &str, identifiers: &HashSet<&str>) -> Option<Finding> {
    let linked_identifier = link_text.trim();
    if !identifiers.contains(linked_identifier) {
        return None;
    }

    let Binding::Corpus {
        path,
        id: Some(target_identifier),
        ..
    } = &link.binding
    else {
        return None;
    };
    if linked_identifier == target_identifier {
        return None;
    }

    Some(Finding {
        rule: self::RULE,
        severity: Severity::Warn,
        obligation: None,
        path: link.source_path.clone(),
        line: link.span.start.line,
        column: link.span.start.col,
        message: format!(
            "link text names `{linked_identifier}`, but `{path}` has identifier `{target_identifier}`"
        ),
        remediation: format!(
            "write `{target_identifier}` as the link text, or point it at the document with identifier `{linked_identifier}`"
        ),
        patch: None,
    })
}
