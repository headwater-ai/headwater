// SPDX-License-Identifier: Apache-2.0
//! Structural conformance: a source read against the meta-schema.
//!
//! This is the first bullet of what
//! [`headwater taxonomy validate`](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema)
//! checks, plus reference well-formedness, and it is every bullet that a single
//! source can decide on its own. The rest of the list reads a *resolved* tree
//! and waits for the resolver. [`skipped`] names each one, so a caller can say
//! which checks it did not run rather than report a pass it did not earn.

use crate::error::{MetaError, MetaErrorKind};
use crate::schema::{MetaSchema, Position};
use crate::shape::{Form, ScalarType, Shape};
use headwater_ref::{classify, Address, Scalar as RefScalar};
use headwater_yaml::{core_schema, Mapping, Span, Spanned, Value};

/// The rules of the meta-schema that need a resolved taxonomy, and why.
///
/// Spec 2 lists twenty-one things that `taxonomy validate` checks. The four
/// below are the ones this crate can run over one source, and everything else
/// is here. The order is spec 2's own.
pub const SKIPPED: [(&str, &str); 17] = [
    ("referential integrity", "a reference resolves over the merged tree, so a name that an overlay supplies is not there yet"),
    ("anchor integrity", "a resolver namespace collides across the base and every bundle, and one source holds only its own"),
    ("identifier integrity", "two schemes collide across sources, and this repository's overlay mints four against a base that has one"),
    ("coverage", "a shelf reaches a kind that a bundle may declare"),
    ("kind inheritance", "`is_a` names a kind that the base declares and the bundle extends"),
    ("purpose completeness", "a purpose is declared in one source and served by a kind in another"),
    ("determinism", "two shelf patterns collide across sources"),
    ("role uniqueness", "two facets claim one role across sources"),
    ("lifecycle soundness", "an overlay may override the state vocabulary that the machine runs over"),
    ("relation coherence", "`inherits` names facets that exist on both ends, and an end may be a kind from another source"),
    ("expectation well-formedness", "an expectation names a relation and kinds that another source may declare"),
    ("core satisfiability", "the core is checked on the result, and never on an operation"),
    ("facet canons", "relevance asks whether any check reads the facet, over the resolved taxonomy"),
    ("kind rigidity", "a kind name collides with a lifecycle-state value, and an overlay may replace the vocabulary"),
    ("edge provenance", "`created_by` is required here, and whether the resolved relation set is author-free is the audit's"),
    ("overlay confluence", "an overlay set is more than one source by definition"),
    ("mapping integrity", "a mapping names kinds in two taxonomies, and neither is resolved here"),
];

/// What a caller reports as unrun. See [`SKIPPED`].
pub fn skipped() -> &'static [(&'static str, &'static str)] {
    &SKIPPED
}

/// Validate a taxonomy source: the thirteen declarations and their shapes.
pub fn taxonomy(schema: &MetaSchema, root: &Spanned<Value>) -> Vec<MetaError> {
    let mut out = Vec::new();
    let Some(map) = root.value.as_map() else {
        out.push(MetaError::new(
            MetaErrorKind::WrongForm {
                expected: "a mapping of declarations",
                found: root.value.kind_name(),
            },
            "",
            root.span,
        ));
        return out;
    };
    closed_root(schema, map, schema.declarations(), &mut out, false);
    out
}

/// Validate an overlay source: operations at addresses, and the value each one
/// writes read against the shape at the node it names.
pub fn overlay(schema: &MetaSchema, root: &Spanned<Value>) -> Vec<MetaError> {
    let mut out = Vec::new();
    let Some(map) = root.value.as_map() else {
        out.push(MetaError::new(
            MetaErrorKind::WrongForm {
                expected: "a mapping of operations",
                found: root.value.kind_name(),
            },
            "",
            root.span,
        ));
        return out;
    };
    closed_root(schema, map, schema.operations(), &mut out, true);

    for (operation, entry) in map.iter().filter_map(|entry| {
        matches!(
            entry.key.value.as_str(),
            "add" | "override" | "add_to" | "remove_from"
        )
        .then_some((entry.key.value.as_str(), entry))
    }) {
        let Some(operations) = entry.value.value.as_map() else {
            continue; // the shape check above already reported it
        };
        for operation_entry in operations {
            let address = match parse_address(
                &operation_entry.key.value,
                operation,
                &operation_entry.key.span,
                &mut out,
            ) {
                Some(address) => address,
                None => continue,
            };
            let at = format!("{operation}.{address}");
            let shape = match position(schema, &address, &at, operation_entry.key.span, &mut out) {
                Some(shape) => shape,
                None => continue,
            };
            // `add: {relations.forbids: $package.optional.forbids}` enables a
            // relation by reference. So an operation's whole value may be a
            // reference wherever the operation itself is legal, which is the
            // one reference position that the addressed shape does not decide.
            if let Value::Scalar(scalar) = &operation_entry.value.value {
                if let Ok(RefScalar::Reference(_)) = classify(&scalar.text) {
                    continue;
                }
            }
            if matches!(operation, "add_to" | "remove_from")
                && !matches!(schema.resolve(shape).form, Form::Seq(_))
            {
                out.push(MetaError::new(
                    MetaErrorKind::NotAList(address.to_string()),
                    &at,
                    operation_entry.key.span,
                ));
                continue;
            }
            check(schema, &operation_entry.value, shape, &at, &mut out);
        }
    }

    if let Some(entry) = map.get("remove") {
        if let Some(items) = entry.value.as_seq() {
            for item in items {
                let Some(scalar) = item.value.as_scalar() else {
                    continue;
                };
                if let Some(address) = parse_address(&scalar.text, "remove", &item.span, &mut out) {
                    let at = format!("remove.{address}");
                    position(schema, &address, &at, item.span, &mut out);
                }
            }
        }
    }
    out
}

fn parse_address(
    text: &str,
    operation: &str,
    span: &Span,
    out: &mut Vec<MetaError>,
) -> Option<Address> {
    match Address::parse(text) {
        Ok(address) => Some(address),
        Err(error) => {
            out.push(MetaError::new(
                MetaErrorKind::MalformedAddress(format!("`{text}` is not an address: {error}")),
                operation,
                *span,
            ));
            None
        }
    }
}

/// Where an address lands, reported as the shape there or as a rejection.
fn position<'a>(
    schema: &'a MetaSchema,
    address: &Address,
    at: &str,
    span: Span,
    out: &mut Vec<MetaError>,
) -> Option<&'a Shape> {
    if address.block() == "package" {
        out.push(MetaError::new(
            MetaErrorKind::ReservedRootAddress(address.to_string()),
            at,
            span,
        ));
        return None;
    }
    match schema.at(address) {
        Position::At(shape) => Some(shape),
        Position::Undeclared { stopped_at } => {
            out.push(MetaError::new(
                MetaErrorKind::UndeclaredAddress {
                    address: address.to_string(),
                    stopped_at,
                },
                at,
                span,
            ));
            None
        }
        Position::InsideList { list } => {
            out.push(MetaError::new(
                MetaErrorKind::AddressIntoList {
                    address: address.to_string(),
                    list,
                },
                at,
                span,
            ));
            None
        }
        Position::BelowScalar { scalar } => {
            out.push(MetaError::new(
                MetaErrorKind::AddressBelowScalar {
                    address: address.to_string(),
                    scalar,
                },
                at,
                span,
            ));
            None
        }
    }
}

/// The root of either source: a closed key set, with `package` refused by name.
fn closed_root(
    schema: &MetaSchema,
    map: &Mapping,
    declared: &[crate::shape::Member],
    out: &mut Vec<MetaError>,
    overlay: bool,
) {
    for entry in map {
        let key = entry.key.value.as_str();
        if key == "package" {
            out.push(MetaError::new(
                MetaErrorKind::ReservedRoot,
                key,
                entry.key.span,
            ));
            continue;
        }
        let Some(member) = declared.iter().find(|member| member.name == key) else {
            let kind = if overlay {
                MetaErrorKind::UnknownOperation(key.to_string())
            } else {
                MetaErrorKind::UnknownDeclaration(key.to_string())
            };
            out.push(MetaError::new(kind, key, entry.key.span));
            continue;
        };
        check(schema, &entry.value, &member.shape, key, out);
    }
    for member in declared.iter().filter(|member| member.required) {
        if map.get(&member.name).is_none() {
            out.push(MetaError::new(
                MetaErrorKind::MissingMember(member.name.clone()),
                "",
                Span::default(),
            ));
        }
    }
}

/// One node against one shape.
fn check(
    schema: &MetaSchema,
    node: &Spanned<Value>,
    shape: &Shape,
    at: &str,
    out: &mut Vec<MetaError>,
) {
    let shape = schema.resolve(shape);

    // A reference is decided before the form is, because a reference stands
    // where a literal of any form would: `values:` takes a list or a reference
    // to one, and the scalar that carries the reference is neither.
    if let Value::Scalar(scalar) = &node.value {
        match classify(&scalar.text) {
            Ok(RefScalar::Reference(reference)) => {
                if !shape.reference {
                    out.push(MetaError::new(
                        MetaErrorKind::ReferenceNotAdmitted(reference.to_string()),
                        at,
                        node.span,
                    ));
                }
                return;
            }
            Ok(RefScalar::Literal(_)) => {}
            Err(error) => {
                out.push(MetaError::new(
                    MetaErrorKind::MalformedReference(format!(
                        "`{}` opens with the reference sigil and {error}",
                        scalar.text
                    )),
                    at,
                    node.span,
                ));
                return;
            }
        }
    }

    match &shape.form {
        Form::Free(_) => {}
        Form::Scalar(declared) => {
            let Some(scalar) = node.value.as_scalar() else {
                return wrong(out, at, node, "a scalar");
            };
            if !is_of_type(scalar, *declared) {
                out.push(MetaError::new(
                    MetaErrorKind::NotOfType {
                        declared: declared.name(),
                        found: scalar.text.clone(),
                    },
                    at,
                    node.span,
                ));
            }
        }
        Form::Enum(values) => {
            let Some(scalar) = node.value.as_scalar() else {
                return wrong(out, at, node, "a scalar");
            };
            if !values.contains(&scalar.text) {
                out.push(MetaError::new(
                    MetaErrorKind::NotInSet {
                        found: scalar.text.clone(),
                        allowed: values.clone(),
                    },
                    at,
                    node.span,
                ));
            }
        }
        Form::Seq(inner) => {
            let Some(items) = node.value.as_seq() else {
                return wrong(out, at, node, "a sequence");
            };
            for (index, item) in items.iter().enumerate() {
                check(schema, item, inner, &format!("{at}.{index}"), out);
            }
        }
        Form::Map(inner) => {
            let Some(items) = node.value.as_map() else {
                return wrong(out, at, node, "a mapping");
            };
            for item in items {
                if shape.addressable {
                    if let Some(why) = not_a_segment(&item.key.value) {
                        out.push(MetaError::new(
                            MetaErrorKind::KeyIsNotASegment {
                                key: item.key.value.clone(),
                                why,
                            },
                            at,
                            item.key.span,
                        ));
                    }
                }
                check(
                    schema,
                    &item.value,
                    inner,
                    &format!("{at}.{}", item.key.value),
                    out,
                );
            }
        }
        Form::Members(members) => {
            let Some(items) = node.value.as_map() else {
                return wrong(out, at, node, "a mapping");
            };
            for item in items {
                match members.iter().find(|member| member.name == item.key.value) {
                    Some(member) => check(
                        schema,
                        &item.value,
                        &member.shape,
                        &format!("{at}.{}", member.name),
                        out,
                    ),
                    None => out.push(MetaError::new(
                        MetaErrorKind::UnknownMember(item.key.value.clone()),
                        at,
                        item.key.span,
                    )),
                }
            }
            for member in members.iter().filter(|member| member.required) {
                if items.get(&member.name).is_none() {
                    out.push(MetaError::new(
                        MetaErrorKind::MissingMember(member.name.clone()),
                        at,
                        node.span,
                    ));
                }
            }
        }
        Form::OneOf(alternatives) => {
            for alternative in alternatives {
                let mut attempt = Vec::new();
                check(schema, node, alternative, at, &mut attempt);
                if attempt.is_empty() {
                    return;
                }
            }
            out.push(MetaError::new(MetaErrorKind::NoAlternative, at, node.span));
        }
        // `resolve` followed every `block:`, and a `block:` that names nothing
        // is refused when the meta-schema loads.
        Form::Block(_) => {}
    }
}

fn wrong(out: &mut Vec<MetaError>, at: &str, node: &Spanned<Value>, expected: &'static str) {
    out.push(MetaError::new(
        MetaErrorKind::WrongForm {
            expected,
            found: node.value.kind_name(),
        },
        at,
        node.span,
    ));
}

/// Why a declared key cannot be a segment of an address, or `None`.
fn not_a_segment(key: &str) -> Option<String> {
    match Address::parse(key) {
        Ok(address) if address.segments().len() == 1 => None,
        Ok(_) => Some(
            "it holds a `.`, which separates two segments, so an overlay that named it \
             would be naming two nodes"
                .to_string(),
        ),
        Err(error) => Some(error.to_string()),
    }
}

/// Whether a scalar's text is of the declared type.
///
/// Q2 puts this decision here rather than in the loader, so every rule about
/// what `no` or `013` means lives in one place: the core schema offers a
/// reading, and a declared type is what asks for it.
fn is_of_type(scalar: &headwater_yaml::Scalar, declared: ScalarType) -> bool {
    match declared {
        // A key written with nothing after it is not a string. It is the
        // author leaving the value out, and a required member that is there
        // and empty would otherwise pass.
        ScalarType::String => !core_schema::as_null(scalar),
        ScalarType::Integer => core_schema::as_int(scalar).is_some(),
        ScalarType::Boolean => core_schema::as_bool(scalar).is_some(),
        ScalarType::Date => is_date(&scalar.text),
        ScalarType::Duration => is_duration(&scalar.text),
        ScalarType::Null => core_schema::as_null(scalar),
    }
}

/// `YYYY-MM-DD`, which is the one form [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md)
/// writes and the one form a date facet holds in this repository's corpus.
fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && [0, 1, 2, 3, 5, 6, 8, 9]
            .iter()
            .all(|index| bytes[*index].is_ascii_digit())
}

/// A count and a unit, as `within: 90d` writes it.
///
/// The unit set is a guess in the direction Q2 settles: `d`, `w`, `m` and `y`
/// are what a participation window is written in, and nothing states the set.
fn is_duration(text: &str) -> bool {
    let Some(unit) = text.chars().last() else {
        return false;
    };
    let count = &text[..text.len() - unit.len_utf8()];
    matches!(unit, 'd' | 'w' | 'm' | 'y')
        && !count.is_empty()
        && count.bytes().all(|byte| byte.is_ascii_digit())
}
