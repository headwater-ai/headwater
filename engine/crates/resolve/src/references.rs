// SPDX-License-Identifier: Apache-2.0
//! `$`-references, resolved over the merged tree and never before it.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the--reference-sublanguage):
//! "A reference resolves last, over the merged tree… The order is thus: apply
//! every overlay, resolve every reference, validate the result." The reason is
//! spec 2's own worked overlay. It overrides `vocabularies.lifecycle_state`,
//! and the facet that reads `$vocabularies.lifecycle_state` is expected to
//! follow it. An early resolution freezes the base list before any overlay is
//! read, and the override then changes nothing that a check can see.
//!
//! # Every site is read before any site is written
//!
//! The substitutions are collected against the merged tree and applied
//! afterwards, so no reference can read a value that another reference put
//! there. That is the same rule as "a reference points at a value and never at
//! a second reference", enforced by construction rather than by an order.
//!
//! # Where a reference may stand is the meta-schema's answer
//!
//! A scalar is read as a reference only where the shape at its position carries
//! `reference: allowed`. Everywhere else `$` is an ordinary character, which is
//! what lets an identifier scheme write `"^DR-[A-Z]{2,6}-[0-9]{4}$"` without an
//! escape. In a position that does admit one, `$$` stands for a single literal
//! `$`, and this is where that escape is spent.

use crate::error::{ResolveError, ResolveErrorKind};
use crate::merge;
use headwater_meta::{MetaSchema, Position};
use headwater_ref::{classify, Address, Reference, Root, Scalar as RefScalar};
use headwater_yaml::{Mapping, Scalar, Span, Spanned, Value};

/// What `$package` would read, and why nothing can supply it yet.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the--reference-sublanguage)
/// says outright that what `optional` means under the `package` root "is not
/// fixed here", that spec 7 declares no such block in the package manifest, and
/// that [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
/// carries the gap. So the root parses, the resolver knows the name, and there
/// is nothing for it to read. A refusal that says that is better than a
/// dangling-reference message that sends an author to look for a typing
/// mistake.
const PACKAGE_ROOT_GAP: &str = "the `package` root reads the publisher's library of defined but \
                                unenabled declarations, and no package file gives that library a \
                                shape. Spec 2 leaves it open and spec 13 carries the gap";

/// Resolve every reference in the merged tree.
pub fn resolve(tree: &Mapping, schema: &MetaSchema) -> Result<Mapping, Vec<ResolveError>> {
    let mut sites = Vec::new();
    collect(tree, schema, &[], &mut sites);

    let mut errors = Vec::new();
    let mut writes: Vec<(Vec<String>, Spanned<Value>)> = Vec::new();
    for site in sites {
        if let Some(value) = read(tree, &site, &mut errors) {
            writes.push((site.path, value));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut out = tree.clone();
    for (path, value) in writes {
        let full = path.join(".");
        match merge::graft(&out, &path, &value, merge::Mode::Override, &full) {
            Ok(next) => out = next,
            Err(kind) => errors.push(ResolveError::new(kind, "", &full, value.span)),
        }
    }
    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors)
    }
}

/// Every reference in a tree that reads nothing, with the path it wanted.
///
/// A `remove` "deletes a key and everything that depends on it — and the
/// resolver **fails** if a declaration that survives still references the
/// removed key" ([spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)).
/// This is the half of that rule the language can decide: a `$`-reference names
/// the node it reads, so a reader knows what it lost. The other half is a
/// declaration that names another by a bare string, and [`crate::WAITING`]
/// states why nothing here can find those.
pub fn dangling(tree: &Mapping, schema: &MetaSchema) -> Vec<Dangling> {
    let mut sites = Vec::new();
    collect(tree, schema, &[], &mut sites);
    sites
        .into_iter()
        .filter_map(|site| {
            let reference = site.reference.as_ref()?;
            let wanted = target(reference)?;
            merge::lookup(tree, &wanted).is_none().then(|| Dangling {
                at: site.path.join("."),
                reference: reference.to_string(),
                wanted,
                span: site.span,
            })
        })
        .collect()
}

/// A reference with nothing to read.
#[derive(Clone, Debug)]
pub struct Dangling {
    /// The position that holds the reference.
    pub at: String,
    pub reference: String,
    /// The path the reference wanted, as segments.
    pub wanted: Vec<String>,
    pub span: Span,
}

/// One scalar in a position that admits a reference.
struct Site {
    path: Vec<String>,
    text: String,
    style: headwater_yaml::Style,
    span: Span,
    /// The reference the text holds, when it holds one rather than an escape.
    reference: Option<Reference>,
}

fn collect(map: &Mapping, schema: &MetaSchema, walked: &[String], out: &mut Vec<Site>) {
    for entry in map {
        let mut path = walked.to_vec();
        path.push(entry.key.value.clone());
        match &entry.value.value {
            Value::Map(inner) => collect(inner, schema, &path, out),
            Value::Scalar(scalar) => {
                if !admits_a_reference(schema, &path) {
                    continue;
                }
                let reference = match classify(&scalar.text) {
                    Ok(RefScalar::Reference(reference)) => Some(reference),
                    // A `$$` escape is a literal that still has to be rewritten,
                    // and a plain literal is not a site at all.
                    Ok(RefScalar::Literal(literal)) if literal != scalar.text => None,
                    _ => continue,
                };
                out.push(Site {
                    path,
                    text: scalar.text.clone(),
                    style: scalar.style,
                    span: entry.value.span,
                    reference,
                });
            }
            // A list has no addressable interior, so no position inside one is
            // a position the meta-schema can name.
            Value::Seq(_) => {}
        }
    }
}

fn admits_a_reference(schema: &MetaSchema, path: &[String]) -> bool {
    let Ok(address) = Address::parse(&path.join(".")) else {
        return false;
    };
    match schema.at(&address) {
        Position::At(shape) => schema.resolve(shape).reference,
        _ => false,
    }
}

/// The value a site resolves to, or `None` with the refusal recorded.
fn read(tree: &Mapping, site: &Site, errors: &mut Vec<ResolveError>) -> Option<Spanned<Value>> {
    let at = site.path.join(".");
    let mut refuse = |kind| {
        errors.push(ResolveError::new(kind, "", &at, site.span));
        None
    };

    let Some(reference) = &site.reference else {
        // The escape: one `$` comes off, and the rest is the literal.
        return Some(Spanned::new(
            Value::Scalar(Scalar {
                text: site.text[1..].to_string(),
                style: site.style,
            }),
            site.span,
        ));
    };

    let Some(path) = target(reference) else {
        return refuse(ResolveErrorKind::ReferenceRootUnavailable {
            reference: reference.to_string(),
            why: PACKAGE_ROOT_GAP.to_string(),
        });
    };

    let Some(found) = merge::lookup(tree, &path) else {
        return refuse(ResolveErrorKind::ReferenceUnresolved(reference.to_string()));
    };

    if let Some(scalar) = found.value.as_scalar() {
        if let Ok(RefScalar::Reference(second)) = classify(&scalar.text) {
            return refuse(ResolveErrorKind::ReferenceChain {
                reference: reference.to_string(),
                reads: second.to_string(),
            });
        }
    }

    Some(found.clone())
}

/// The path a reference reads, or `None` when its root supplies nothing.
fn target(reference: &Reference) -> Option<Vec<String>> {
    match reference.root() {
        Root::Vocabularies => {
            let mut path = vec!["vocabularies".to_string()];
            path.extend(reference.address().segments().iter().cloned());
            Some(path)
        }
        Root::Package => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(source: &str) -> Mapping {
        headwater_yaml::load(source)
            .expect("the fixture loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone()
    }

    fn schema() -> MetaSchema {
        MetaSchema::shipped().expect("the shipped meta-schema")
    }

    const BASE: &str = "\
taxonomy: test
version: 1.0.0
vocabularies:
  lifecycle_state:
    - {value: draft, role: initial}
    - {value: current, role: live}
facets:
  status: {role: state, values: $vocabularies.lifecycle_state}
";

    #[test]
    fn a_vocabulary_reference_becomes_the_value_set_it_reads() {
        let resolved = resolve(&tree(BASE), &schema()).expect("resolves");
        let values = merge::lookup(
            &resolved,
            &["facets".into(), "status".into(), "values".into()],
        )
        .expect("a value set");
        assert_eq!(values.value.as_seq().expect("a list").len(), 2);
    }

    /// The whole reason the order is apply, then resolve. An overlay that
    /// replaced the vocabulary has to reach the facet that reads it.
    #[test]
    fn a_reference_reads_the_merged_tree_and_not_the_base() {
        let overridden = tree(&BASE.replace(
            "    - {value: current, role: live}\n",
            "    - {value: current, role: live}\n    - {value: retired, role: terminal-retained}\n",
        ));
        let resolved = resolve(&overridden, &schema()).expect("resolves");
        let values = merge::lookup(
            &resolved,
            &["facets".into(), "status".into(), "values".into()],
        )
        .expect("a value set");
        assert_eq!(values.value.as_seq().expect("a list").len(), 3);
    }

    #[test]
    fn a_reference_that_reads_nothing_is_refused_by_name() {
        let source = BASE.replace("lifecycle_state:", "other_state:");
        let errors = resolve(&tree(&source), &schema()).expect_err("unresolved");
        assert!(matches!(
            errors[0].kind,
            ResolveErrorKind::ReferenceUnresolved(_)
        ));
    }

    #[test]
    fn a_reference_that_reads_a_second_reference_is_refused() {
        let source = "\
taxonomy: test
version: 1.0.0
vocabularies:
  a: $vocabularies.b
  b: [one]
facets:
  status: {values: $vocabularies.a}
";
        let errors = resolve(&tree(source), &schema()).expect_err("a chain");
        assert!(matches!(
            errors[0].kind,
            ResolveErrorKind::ReferenceChain { .. }
        ));
    }

    #[test]
    fn the_package_root_says_what_is_missing_rather_than_that_a_name_is_wrong() {
        let source = BASE.replace("$vocabularies.lifecycle_state", "$package.optional.states");
        let errors = resolve(&tree(&source), &schema()).expect_err("no package library");
        assert!(matches!(
            errors[0].kind,
            ResolveErrorKind::ReferenceRootUnavailable { .. }
        ));
    }

    /// Outside a reference position `$` is an ordinary character, so an
    /// identifier pattern is not a malformed reference.
    #[test]
    fn a_dollar_outside_a_reference_position_is_left_alone() {
        let source = "\
taxonomy: test
version: 1.0.0
identifier_schemes:
  decision_id: {pattern: \"^DR-[A-Z]{2,6}$\", namespace: repo, allocation: minted-once}
";
        let resolved = resolve(&tree(source), &schema()).expect("resolves");
        let pattern = merge::lookup(
            &resolved,
            &[
                "identifier_schemes".into(),
                "decision_id".into(),
                "pattern".into(),
            ],
        )
        .expect("a pattern");
        assert_eq!(pattern.value.as_scalar().unwrap().text, "^DR-[A-Z]{2,6}$");
    }

    #[test]
    fn the_escape_is_spent_where_a_reference_could_have_stood() {
        let source = BASE.replace("$vocabularies.lifecycle_state", "$$literal");
        let resolved = resolve(&tree(&source), &schema()).expect("resolves");
        let values = merge::lookup(
            &resolved,
            &["facets".into(), "status".into(), "values".into()],
        )
        .expect("a value");
        assert_eq!(values.value.as_scalar().unwrap().text, "$literal");
    }
}
