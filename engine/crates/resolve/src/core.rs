// SPDX-License-Identifier: Apache-2.0
//! The immutable core, checked on the result and never on an operation.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the-immutable-core):
//! "Core satisfaction is checked **after** overlay resolution, against the
//! resolved taxonomy. The check does not forbid particular operations. An
//! overlay is rejected when the *result* fails to satisfy a core requirement.
//! The error names the requirement and the operation that removed its last
//! satisfier."
//!
//! Both halves of that sentence are load-bearing in opposite directions. The
//! check reads the result, so an overlay may rename every shelf, relocate every
//! directory and replace the whole state vocabulary and still pass. The message
//! reads the operations, because an author who is told only that the result
//! fails has to find which of thirty lines did it.
//!
//! # The core is semantic, so a satisfier is a role and never a name
//!
//! Spec 2: "It constrains *roles and purposes*, never names or paths." So a
//! `facet_role` requirement is satisfied by whichever facet carries the role,
//! whatever it is called, and a `purpose` requirement is satisfied by whichever
//! concrete kind serves it, through inheritance if that is how the kind gets
//! it. Spec 2's own worked overlay renames every lifecycle state and stays
//! conformant, and only dropping the `terminal-retained` role would fail.
//!
//! # `lifecycle_sensitive` is read as an effect on the target's state
//!
//! The base writes `relation_family: succession` with `lifecycle_sensitive:
//! true`, and the relation that satisfies it is `supersedes`, which declares
//! `on_target: {set_state: superseded}`. Spec 2 states the requirement in prose
//! — "lineage remains expressible and lifecycle-sensitive" — and names no
//! field. `on_target` is the only declaration in the language that makes a
//! relation act on a state, so it is what the check reads. A relation family
//! whose members declare no `on_target` is expressible lineage that no
//! lifecycle follows, which is the case the requirement exists to refuse.

/// Whether a role on a state value names a terminal state.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the-immutable-core)
/// writes the role as `terminal-retained`, and the hyphen carries what happens
/// to the document rather than whether the state ends the machine. A taxonomy
/// that declares a second terminal role writes a second `terminal-` name, so
/// the prefix is the reading and the suffix is that taxonomy's business.
///
/// One function, because three components ask the question: `lifecycle
/// soundness` asks it of a declaration, [`satisfiers`] states it in prose as
/// the role a conformant overlay may not drop, and the check layer asks it of
/// the state a document stands in.
pub fn role_is_terminal(role: &str) -> bool {
    role.starts_with("terminal")
}

use crate::error::{ResolveError, ResolveErrorKind};
use crate::merge;
use crate::operation::{OpKind, Operation};
use headwater_yaml::{Mapping, Value};

/// One entry of `core.requires`.
#[derive(Clone, Debug)]
pub struct Requirement {
    /// The requirement as the package wrote it, for the message.
    pub text: String,
    pub kind: Kind,
}

#[derive(Clone, Debug)]
pub enum Kind {
    FacetRole(String),
    Purpose(String),
    RelationFamily {
        family: String,
        lifecycle_sensitive: bool,
    },
    IdentifierScheme(String),
    /// A requirement shape this engine does not know. It is carried rather than
    /// dropped, because a core requirement that silently does not run is the
    /// silent pass the core exists to prevent.
    Unknown,
}

/// Read `core.requires` out of a taxonomy.
pub fn read(tree: &Mapping) -> Vec<Requirement> {
    let Some(items) = merge::lookup(tree, &["core".into(), "requires".into()])
        .and_then(|node| node.value.as_seq())
    else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| item.value.as_map())
        .map(|entry| {
            let field = |name: &str| {
                entry
                    .get(name)
                    .and_then(|value| value.value.as_scalar())
                    .map(|scalar| scalar.text.clone())
            };
            // A boolean is read through the core schema and never off the
            // text of the scalar. The meta-schema declares this member
            // `boolean` and validates it with the same function, so `True` and
            // `TRUE` arrive here as declarations that already passed
            // validation. A comparison against the literal `"true"` read both
            // of them as false, which turns a stated requirement into an
            // unstated one, and it read the quoted string `"true"` — which the
            // meta-schema refuses — as the boolean.
            let flag = |name: &str| {
                entry
                    .get(name)
                    .and_then(|value| value.value.as_scalar())
                    .and_then(headwater_yaml::core_schema::as_bool)
                    .unwrap_or(false)
            };
            let kind = if let Some(role) = field("facet_role") {
                Kind::FacetRole(role)
            } else if let Some(purpose) = field("purpose") {
                Kind::Purpose(purpose)
            } else if let Some(family) = field("relation_family") {
                Kind::RelationFamily {
                    family,
                    lifecycle_sensitive: flag("lifecycle_sensitive"),
                }
            } else if let Some(scheme) = field("identifier_scheme") {
                Kind::IdentifierScheme(scheme)
            } else {
                Kind::Unknown
            };
            Requirement {
                text: describe(entry),
                kind,
            }
        })
        .collect()
}

/// Every node that satisfies a requirement, by path.
///
/// The paths are what the message needs. A requirement that fails on the result
/// and held in the base lost a specific declaration, and the operation to blame
/// is the one whose address covers it.
pub fn satisfiers(tree: &Mapping, requirement: &Requirement) -> Vec<Vec<String>> {
    match &requirement.kind {
        Kind::FacetRole(role) => named_with(tree, "facets", |body| {
            scalar(body, "role").as_deref() == Some(role.as_str())
        }),
        Kind::Purpose(purpose) => {
            let declared = merge::lookup(tree, &["purposes".into(), purpose.clone()]).is_some();
            if !declared {
                return Vec::new();
            }
            let mut found = vec![vec!["purposes".to_string(), purpose.clone()]];
            found.extend(named_with(tree, "kinds", |body| {
                scalar(body, "abstract").as_deref() != Some("true")
                    && serves(tree, body) == Some(purpose.clone())
            }));
            // A purpose that no concrete kind serves is not served at all, and
            // the declaration on its own is not a satisfier.
            if found.len() == 1 {
                return Vec::new();
            }
            found
        }
        Kind::RelationFamily {
            family,
            lifecycle_sensitive,
        } => named_with(tree, "relations", |body| {
            scalar(body, "family").as_deref() == Some(family.as_str())
                && (!lifecycle_sensitive
                    || body
                        .get("on_target")
                        .and_then(|node| node.value.as_map())
                        .and_then(|node| node.get("set_state"))
                        .is_some())
        }),
        Kind::IdentifierScheme(scheme) => {
            match merge::lookup(tree, &["identifier_schemes".into(), scheme.clone()]) {
                Some(_) => vec![vec!["identifier_schemes".to_string(), scheme.clone()]],
                None => Vec::new(),
            }
        }
        Kind::Unknown => Vec::new(),
    }
}

/// Check the core on the result, and blame the operation that emptied it.
pub fn check(
    base: &Mapping,
    resolved: &Mapping,
    operations: &[Operation],
    sources: &[String],
) -> Vec<ResolveError> {
    let mut out = Vec::new();
    for requirement in read(resolved) {
        if !satisfiers(resolved, &requirement).is_empty() {
            continue;
        }
        let before = satisfiers(base, &requirement);
        let blame = before
            .iter()
            .find_map(|lost| culprit(operations, sources, lost));
        out.push(ResolveError::new(
            ResolveErrorKind::CoreUnsatisfied {
                requirement: requirement.text.clone(),
                blame,
            },
            "",
            "core.requires",
            headwater_yaml::Span::default(),
        ));
    }
    out
}

/// The operation that covers a lost satisfier, deletions first.
fn culprit(operations: &[Operation], sources: &[String], lost: &[String]) -> Option<String> {
    let covers = |operation: &&Operation| {
        let address = operation.address.segments();
        lost.starts_with(address) || address.starts_with(lost)
    };
    let name =
        |operation: &Operation| format!("`{}` in {}", operation.at(), sources[operation.source]);
    operations
        .iter()
        .filter(covers)
        .find(|operation| operation.kind == OpKind::Remove)
        .or_else(|| operations.iter().rfind(covers))
        .map(name)
}

/// The purpose a kind serves, its own or the one it inherits.
fn serves(tree: &Mapping, body: &Mapping) -> Option<String> {
    let mut body = body.clone();
    // The chain terminates because it is bounded by the number of kinds, and a
    // cycle in `is_a` is `taxonomy validate`'s finding rather than a hang here.
    for _ in 0..64 {
        if let Some(purpose) = scalar(&body, "purpose") {
            return Some(purpose);
        }
        let parent = scalar(&body, "is_a")?;
        body = merge::lookup(tree, &["kinds".into(), parent])?
            .value
            .as_map()?
            .clone();
    }
    None
}

/// The names in `block` whose body satisfies a test, as paths.
fn named_with(tree: &Mapping, block: &str, test: impl Fn(&Mapping) -> bool) -> Vec<Vec<String>> {
    let Some(map) = merge::lookup(tree, &[block.to_string()]).and_then(|node| node.value.as_map())
    else {
        return Vec::new();
    };
    map.iter()
        .filter(|entry| entry.value.value.as_map().is_some_and(&test))
        .map(|entry| vec![block.to_string(), entry.key.value.clone()])
        .collect()
}

fn scalar(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

/// A requirement written back the way the package wrote it.
fn describe(entry: &Mapping) -> String {
    entry
        .iter()
        .map(|item| match &item.value.value {
            Value::Scalar(scalar) => format!("{}: {}", item.key.value, scalar.text),
            other => format!("{}: {}", item.key.value, other.kind_name()),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "\
purposes:
  rationale: {intent: why}
facets:
  status: {role: state}
kinds:
  governed_document: {abstract: true}
  decision: {is_a: governed_document, purpose: rationale}
relations:
  supersedes: {family: succession, on_target: {set_state: superseded}}
core:
  requires:
    - facet_role: state
    - purpose: rationale
    - relation_family: succession
      lifecycle_sensitive: true
";

    fn tree(source: &str) -> Mapping {
        headwater_yaml::load(source)
            .expect("the fixture loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone()
    }

    #[test]
    fn the_base_satisfies_its_own_core() {
        let base = tree(BASE);
        assert!(check(&base, &base, &[], &[]).is_empty());
    }

    /// The whole point of a semantic core: rename everything, keep the roles.
    #[test]
    fn a_renamed_facet_that_keeps_the_role_still_satisfies_the_core() {
        let base = tree(BASE);
        let renamed = tree(&BASE.replace("  status: {role: state}", "  phase: {role: state}"));
        assert!(check(&base, &renamed, &[], &[]).is_empty());
    }

    #[test]
    fn a_purpose_that_only_an_abstract_kind_serves_is_not_served() {
        let base = tree(BASE);
        let abstracted = tree(&BASE.replace(
            "  decision: {is_a: governed_document, purpose: rationale}",
            "  decision: {is_a: governed_document}",
        ));
        let found = check(&base, &abstracted, &[], &[]);
        assert_eq!(found.len(), 1);
        assert!(found[0].to_string().contains("purpose: rationale"));
    }

    #[test]
    fn a_succession_relation_that_touches_no_state_does_not_satisfy_the_core() {
        let base = tree(BASE);
        let inert = tree(&BASE.replace(
            "  supersedes: {family: succession, on_target: {set_state: superseded}}",
            "  supersedes: {family: succession}",
        ));
        assert_eq!(check(&base, &inert, &[], &[]).len(), 1);
    }

    /// `True` is the boolean the core schema resolves, so a requirement that
    /// spells it that way is the same requirement. The reading here was a
    /// comparison against the text `true`, which read the two other spellings
    /// of the boolean as false and so stopped asking for the thing the
    /// requirement asks for. The failure is silent by construction: a weaker
    /// requirement is satisfied by more taxonomies, not by fewer.
    #[test]
    fn a_boolean_is_read_as_the_core_schema_resolves_it_and_never_as_its_text() {
        let base = tree(BASE);
        for spelling in ["true", "True", "TRUE"] {
            let stated = BASE.replace("lifecycle_sensitive: true", &format!("lifecycle_sensitive: {spelling}"));
            let inert = tree(&stated.replace(
                "  supersedes: {family: succession, on_target: {set_state: superseded}}",
                "  supersedes: {family: succession}",
            ));
            assert_eq!(
                check(&base, &inert, &[], &[]).len(),
                1,
                "`lifecycle_sensitive: {spelling}` states the requirement"
            );
        }
        // The quoted form is a string and the meta-schema refuses it, so it
        // states no requirement here either.
        let quoted = tree(
            &BASE
                .replace("lifecycle_sensitive: true", "lifecycle_sensitive: \"true\"")
                .replace(
                    "  supersedes: {family: succession, on_target: {set_state: superseded}}",
                    "  supersedes: {family: succession}",
                ),
        );
        assert!(check(&base, &quoted, &[], &[]).is_empty());
    }

    #[test]
    fn the_message_names_the_operation_that_removed_the_last_satisfier() {
        let base = tree(BASE);
        let after = tree(&BASE.replace("  status: {role: state}\n", ""));
        let root = headwater_yaml::load("remove:\n  - facets.status\n").expect("loads");
        let operations = crate::operation::read(0, "overlay", &root).expect("operations");
        let found = check(&base, &after, &operations, &["overlay.yml".to_string()]);
        assert_eq!(found.len(), 1);
        assert!(
            found[0]
                .to_string()
                .contains("`remove.facets.status` in overlay.yml removed its last satisfier"),
            "{}",
            found[0]
        );
    }
}
