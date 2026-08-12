// SPDX-License-Identifier: Apache-2.0
//! The M1 stand-in for overlay resolution. **Not a resolver.**
//!
//! The consumer declaration names a package, a bundle selection and an overlay,
//! and resolving those three into one taxonomy is
//! [#50](https://github.com/headwater-ai/headwater/issues/50). Until it exists,
//! every reader of this repository's taxonomy needs the three files put
//! together from somewhere, and this is the somewhere.
//!
//! It applies `add` operations at dotted addresses and stops. It runs no
//! confluence check, it resolves no `$`-reference, it refuses no `override`,
//! and it validates nothing. Each of those is the difference between a
//! stand-in and a resolver. `tools/abox-check.py` is the same stand-in in
//! Python, for the same reason, and both go away together.
//!
//! It lives in a crate rather than in a test file because two readers now need
//! it — the census and the graph build — and two copies of a stand-in can
//! disagree about the taxonomy while each one passes its own fixtures. Where a
//! binary gets its taxonomy from is a live question and this module does not
//! answer it ([#46](https://github.com/headwater-ai/headwater/issues/46)).
//!
//! # What it does not carry
//!
//! An address deeper than a member of a block — `kinds.design_spec.identifier`,
//! `regimes.language.ste_house` — reaches inside something a previous operation
//! declared, and merging is exactly the part a resolver owns. Those addresses
//! are skipped, and a caller that needs one has found the point where the
//! stand-in stops being enough.

use crate::walk::{Corpus, Exclusion};
use headwater_yaml::{Entry, Mapping, Span, Spanned, Value};
use std::path::Path;

/// The base package, committed as a fenced block inside an evaluation and
/// nowhere else.
pub const BASE: &str = "docs/evaluations/default-taxonomy-first-run.md";
pub const BUNDLE: &str = "docs/taxonomies/design-spec/bundle.yml";
pub const OVERLAY: &str = ".headwater/overlay.yml";
pub const CONSUMER: &str = ".headwater/taxonomy.yml";

/// The base package, the bundle and the overlay, put together.
///
/// The result is the root mapping of a taxonomy: `shelves`, `kinds`,
/// `relations`, `anchors` and whatever else the three files declare, each one
/// in declaration order.
pub fn resolved(root: &Path) -> Mapping {
    let base = base_package(root);
    let mut blocks: Vec<(String, Vec<Entry>)> = base
        .iter()
        .filter_map(|entry| {
            let map = entry.value.value.as_map()?;
            Some((entry.key.value.clone(), map.iter().cloned().collect()))
        })
        .collect();

    for source in [BUNDLE, OVERLAY] {
        let overlay = load_map(&root.join(source));
        let Some(adds) = overlay.get("add").and_then(|value| value.value.as_map()) else {
            continue;
        };
        for entry in adds {
            let Some((block, member)) = entry.key.value.split_once('.') else {
                continue;
            };
            if member.contains('.') {
                continue; // deeper than a member; see the module comment
            }
            let added = Entry {
                key: Spanned::new(member.to_string(), entry.key.span),
                value: entry.value.clone(),
            };
            match blocks.iter_mut().find(|(name, _)| name == block) {
                Some((_, entries)) => entries.push(added),
                None => blocks.push((block.to_string(), vec![added])),
            }
        }
    }

    Mapping::new(
        blocks
            .into_iter()
            .map(|(name, entries)| named(&name, Value::Map(Mapping::new(entries))))
            .collect(),
    )
}

/// The corpus root and the exclusions, from the consumer declaration.
pub fn corpus(root: &Path) -> Corpus {
    let consumer = load_map(&root.join(CONSUMER));
    let block = consumer
        .get("corpus")
        .and_then(|value| value.value.as_map())
        .expect("the consumer declaration names a corpus");
    let corpus_root = block
        .get("root")
        .and_then(|value| value.value.as_scalar())
        .expect("the corpus names a root")
        .text
        .clone();

    let exclusions = block
        .get("exclude")
        .and_then(|value| value.value.as_seq())
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let item = item.value.as_map().expect("an exclusion is a mapping");
                    let path = item
                        .get("path")
                        .and_then(|value| value.value.as_scalar())
                        .expect("an exclusion names a path");
                    // The reason is not optional. An exclusion with no reason is
                    // a silent pass with a configuration file in front of it.
                    let reason = item
                        .get("reason")
                        .and_then(|value| value.value.as_scalar())
                        .expect("an exclusion states a reason");
                    Exclusion::new(&path.text, &reason.text)
                })
                .collect()
        })
        .unwrap_or_default();

    Corpus::new(root, &corpus_root).excluding(exclusions)
}

/// The base package out of the walkthrough's fenced block.
///
/// The tidy part of an otherwise untidy arrangement: the block is code in a
/// Markdown body, and `headwater_doc` already reports the body's code blocks.
pub fn base_package(root: &Path) -> Mapping {
    let source = std::fs::read_to_string(root.join(BASE)).expect("the first-run walkthrough");
    let document = headwater_doc::parse(&source).expect("the walkthrough parses");
    for block in &document.body.blocks {
        let text = block.text();
        if !text.contains("package: headwater/standard") {
            continue;
        }
        return headwater_yaml::load(&text)
            .expect("the base package loads")
            .value
            .as_map()
            .expect("the base package is a mapping")
            .clone();
    }
    panic!("no base package in {BASE}");
}

pub fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {:?}", path.display(), errors))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn named(key: &str, value: Value) -> Entry {
    Entry {
        key: Spanned::new(key.to_string(), Span::default()),
        value: Spanned::new(value, Span::default()),
    }
}
