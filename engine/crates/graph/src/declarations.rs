// SPDX-License-Identifier: Apache-2.0
//! The two declarations the graph build reads: `relations` and `anchors`.
//!
//! The same posture as [`headwater_census::shelves`], for the same reason.
//! Nothing here validates a taxonomy. The meta-schema owns shape and it is
//! [M2](https://github.com/headwater-ai/headwater/milestone/2). This module
//! reads what edge resolution needs and refuses only what it cannot use.
//!
//! # What edge resolution needs, and why each field is here
//!
//! **`to`** decides what a target *is*. [Spec 1](../../../../docs/spec/01-conceptual-model.md#external-anchor)
//! says a relation endpoint is a declared kind or a declared anchor kind and
//! never a bare string, so the declared target set is what tells the resolver
//! whether to look in the identifier index or to hand the string to an anchor
//! resolver. A relation that declares no `to` is refused here: every one of its
//! targets would be unresolvable, and reporting that once against the
//! declaration is more use than reporting it once per edge.
//!
//! **`inverse`** is needed because a document may declare either half. This
//! corpus does: `cited_by` appears in front matter 81 times and
//! `cites_evidence` the same 81, and the two are one edge seen from its two
//! ends. Resolution has to recognize the inverse name, or every reciprocal half
//! in the corpus reads as an undeclared relation.
//!
//! **`from`** is carried and never read here. Whether the two endpoint kinds
//! are permitted is a Graph check
//! ([#56](https://github.com/headwater-ai/headwater/issues/56)), and an edge
//! carries the kinds at both ends so that the check has them.
//!
//! **`reciprocal`** is carried and never read here either, for the reason that
//! makes it worth carrying. A reciprocity check is *generated* from the
//! relation declaration rather than written
//! ([spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)),
//! so the declared value is the whole input to it. Resolution never consults
//! it, because a half that nobody wrote is a missing edge rather than a
//! mis-resolved one.

use headwater_census::shelves::DeclarationError;
use headwater_yaml::{Mapping, Span, Value};

/// The relation types and the anchor kinds of a resolved taxonomy.
#[derive(Clone, Debug, Default)]
pub struct Declarations {
    /// In declaration order, because a report is read by a person.
    pub relations: Vec<Relation>,
    pub anchors: Vec<AnchorKind>,
}

#[derive(Clone, Debug)]
pub struct Relation {
    pub name: String,
    /// The kinds that may sit at the source end.
    pub from: Vec<String>,
    /// The kinds and anchor kinds that may sit at the target end.
    pub to: Vec<String>,
    /// The name of the other half, when the relation declares one.
    pub inverse: Option<String>,
    /// What the relation says about the half nobody wrote.
    pub reciprocal: Reciprocal,
    /// The relation family, which is one of spec 2's six. Read as written: the
    /// meta-schema owns the value set, and a family this engine does not know
    /// derives no reading order rather than an invented one.
    pub family: Option<String>,
    /// `nuclearity`, and `nucleus` beside it: which end stands alone.
    pub nuclearity: Option<String>,
    pub nucleus: Option<String>,
    /// The span of the relation's name, which a finding about the
    /// *declaration* points at.
    pub span: Span,
}

/// Which end of an edge governs the reading of the pair.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#reading-precedence-is-derived):
/// "When two documents are linked, no declaration states which one governs the
/// reading. Properties already declared entail it." So this is derived here and
/// never read off a member, and the cut `dominance` field is the thing this
/// function replaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Governs {
    /// The declaring end governs.
    Source,
    /// The far end governs.
    Target,
    /// Spec 2's fourth clause: "Everything else carries no reading order."
    Neither,
}

impl Relation {
    /// The end that governs the reading, from what this relation declares.
    ///
    /// Spec 2's four clauses, in its own order, and they are total over the six
    /// families. A nucleus–satellite relation that names no `nucleus` derives
    /// nothing: `taxonomy validate` refuses that declaration, and a reader that
    /// guessed an end would order a pair by a coin toss.
    pub fn governs(&self) -> Governs {
        // The nucleus clause, read off `nucleus` rather than off `nuclearity`.
        // A family carries a default nuclearity that `taxonomy validate`
        // applies, so a relation can be nucleus–satellite with no `nuclearity`
        // line at all. `nucleus` is the member that means nothing under any
        // other nuclearity, and reading it here keeps that default table in the
        // one place that owns it.
        match self.nucleus.as_deref() {
            Some("from") => return Governs::Source,
            Some("to") => return Governs::Target,
            _ => {}
        }
        match self.family.as_deref() {
            // "On succession, the successor governs" — the declaring end, since
            // `supersedes` is written from the successor.
            Some("succession") => Governs::Source,
            // "On governance between two documents, the source governs."
            Some("governance") => Governs::Source,
            _ => Governs::Neither,
        }
    }
}

/// What a relation declares about its other half.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#behavior-at-the-limits)
/// gives the vocabulary. The set is closed here, and a value outside it reads
/// as [`Reciprocal::Unknown`] rather than as an error, because the meta-schema
/// owns the value set and it is
/// [M2](https://github.com/headwater-ai/headwater/milestone/2). A generated
/// check reads `Required` and nothing else, so an unrecognized value produces
/// no instance instead of a guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reciprocal {
    /// The relation declares nothing, so the half nobody wrote is not missing.
    Absent,
    /// Both documents declare the pair, and a half that nobody wrote is a
    /// finding.
    Required,
    /// The relation is its own inverse.
    Symmetric,
    /// A word this engine does not know, kept as written so that a report can
    /// name it.
    Unknown(String),
}

/// A non-document node type that a relation may target.
#[derive(Clone, Debug)]
pub struct AnchorKind {
    pub name: String,
    /// The single component that owns identity for this anchor type
    /// ([spec 2](../../../../docs/spec/02-taxonomy-model.md#behavior-at-the-limits)).
    pub resolver: String,
    pub span: Span,
}

/// Which way round a document declared an edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    /// The document declared the relation under its own name, so the document
    /// is the source of the edge the relation declares.
    AsDeclared,
    /// The document declared the relation under its inverse name, so the
    /// document is the *target* of the edge the relation declares.
    Inverse,
}

/// A relation name as an author wrote it, resolved against the declarations.
#[derive(Clone, Copy, Debug)]
pub struct Named<'a> {
    pub relation: &'a Relation,
    pub direction: Direction,
}

impl Declarations {
    /// Read `relations` and `anchors` from the root of a resolved taxonomy.
    ///
    /// Both are optional. A taxonomy that declares no relations builds a graph
    /// of nodes and no edges, which is a true report of a corpus that declares
    /// no edges either.
    pub fn read(root: &Mapping) -> Result<Self, Vec<DeclarationError>> {
        let mut errors = Vec::new();
        let mut declarations = Declarations::default();

        if let Some(relations) = root.get("relations") {
            match &relations.value {
                Value::Map(map) => {
                    for entry in map {
                        match read_relation(&entry.key.value, &entry.value.value, entry.key.span) {
                            Ok(relation) => declarations.relations.push(relation),
                            Err(error) => errors.push(error),
                        }
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!(
                        "`relations` is {}, and it names relation types",
                        other.kind_name()
                    ),
                    span: relations.span,
                }),
            }
        }

        if let Some(anchors) = root.get("anchors") {
            match &anchors.value {
                Value::Map(map) => {
                    for entry in map {
                        match read_anchor(&entry.key.value, &entry.value.value, entry.key.span) {
                            Ok(anchor) => declarations.anchors.push(anchor),
                            Err(error) => errors.push(error),
                        }
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!(
                        "`anchors` is {}, and it names anchor kinds",
                        other.kind_name()
                    ),
                    span: anchors.span,
                }),
            }
        }

        if errors.is_empty() {
            Ok(declarations)
        } else {
            Err(errors)
        }
    }

    /// Resolve a relation name as an author wrote it.
    ///
    /// A name matches a relation directly, or it matches the inverse a relation
    /// declares. Nothing else is a relation, and a front-matter key that
    /// reaches neither is reported against the document rather than guessed at.
    pub fn named(&self, name: &str) -> Option<Named<'_>> {
        if let Some(relation) = self.relations.iter().find(|r| r.name == name) {
            return Some(Named {
                relation,
                direction: Direction::AsDeclared,
            });
        }
        self.relations
            .iter()
            .find(|r| r.inverse.as_deref() == Some(name))
            .map(|relation| Named {
                relation,
                direction: Direction::Inverse,
            })
    }

    pub fn anchor(&self, name: &str) -> Option<&AnchorKind> {
        self.anchors.iter().find(|anchor| anchor.name == name)
    }
}

fn read_relation(name: &str, value: &Value, span: Span) -> Result<Relation, DeclarationError> {
    let map = value.as_map().ok_or_else(|| DeclarationError {
        message: format!(
            "relation `{name}` is {}, and a relation type is a mapping",
            value.kind_name()
        ),
        span,
    })?;

    let to = sequence(map, "to").ok_or_else(|| DeclarationError {
        message: format!(
            "relation `{name}` declares no `to`, so no target of it can resolve to anything"
        ),
        span,
    })?;

    let scalar = |key: &str| {
        map.get(key)
            .and_then(|value| value.value.as_scalar())
            .map(|scalar| scalar.text.clone())
    };

    Ok(Relation {
        name: name.to_string(),
        from: sequence(map, "from").unwrap_or_default(),
        to,
        family: scalar("family"),
        nuclearity: scalar("nuclearity"),
        nucleus: scalar("nucleus"),
        inverse: map
            .get("inverse")
            .and_then(|value| value.value.as_scalar())
            .map(|scalar| scalar.text.clone()),
        reciprocal: match map
            .get("reciprocal")
            .and_then(|value| value.value.as_scalar())
            .map(|scalar| scalar.text.as_str())
        {
            None => Reciprocal::Absent,
            Some("required") => Reciprocal::Required,
            Some("symmetric") => Reciprocal::Symmetric,
            Some(other) => Reciprocal::Unknown(other.to_string()),
        },
        span,
    })
}

fn read_anchor(name: &str, value: &Value, span: Span) -> Result<AnchorKind, DeclarationError> {
    let map = value.as_map().ok_or_else(|| DeclarationError {
        message: format!(
            "anchor kind `{name}` is {}, and an anchor kind is a mapping",
            value.kind_name()
        ),
        span,
    })?;

    // Spec 2 requires exactly one resolver per anchor kind, and the requirement
    // is about identity rather than about tidiness: the resolver is what
    // normalizes two spellings of one target into one node. An anchor kind with
    // no resolver has no identity, so it is refused here rather than carried.
    let resolver = map
        .get("resolver")
        .and_then(|value| value.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .ok_or_else(|| DeclarationError {
            message: format!(
                "anchor kind `{name}` names no `resolver`, so nothing owns its identity"
            ),
            span,
        })?;

    Ok(AnchorKind {
        name: name.to_string(),
        resolver,
        span,
    })
}

/// A sequence of scalars, or a single scalar read as a sequence of one.
///
/// The scalar form is the same sugar the authored relation entry uses, and a
/// declaration that writes `to: evaluation` means the same thing as
/// `to: [evaluation]`.
fn sequence(map: &Mapping, key: &str) -> Option<Vec<String>> {
    let value = map.get(key)?;
    if let Some(scalar) = value.value.as_scalar() {
        return Some(vec![scalar.text.clone()]);
    }
    let items = value.value.as_seq()?;
    Some(
        items
            .iter()
            .filter_map(|item| item.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "\
relations:
  cites_evidence:
    family: evidence
    from: [design_spec, decision_register]
    to:   [evaluation]
    inverse: cited_by
    reciprocal: required
  governs:
    family: governance
    from: [design_spec]
    to:   [code_path]
anchors:
  code_path: {resolver: source-tree}
";

    fn read(source: &str) -> Result<Declarations, Vec<DeclarationError>> {
        let root = headwater_yaml::load(source).expect("the fixture loads");
        Declarations::read(root.value.as_map().expect("a mapping"))
    }

    #[test]
    fn a_relation_reads_with_its_endpoints_and_its_inverse() {
        let declarations = read(SOURCE).expect("reads");
        let cites = &declarations.relations[0];
        assert_eq!(cites.to, ["evaluation"]);
        assert_eq!(cites.inverse.as_deref(), Some("cited_by"));
        assert_eq!(
            declarations.anchor("code_path").unwrap().resolver,
            "source-tree"
        );
    }

    /// The generated reciprocity check reads this and nothing else, so a value
    /// outside the closed set may not read as `required` and may not vanish.
    #[test]
    fn the_reciprocal_declaration_reads_as_a_closed_set_with_a_home_for_the_rest() {
        let declarations = read(SOURCE).expect("reads");
        assert_eq!(declarations.relations[0].reciprocal, Reciprocal::Required);
        assert_eq!(declarations.relations[1].reciprocal, Reciprocal::Absent);

        let invented =
            read("relations:\n  r:\n    to: [x]\n    reciprocal: mutual\n").expect("reads");
        assert_eq!(
            invented.relations[0].reciprocal,
            Reciprocal::Unknown("mutual".into())
        );
    }

    #[test]
    fn an_inverse_name_resolves_to_the_relation_that_declares_it() {
        let declarations = read(SOURCE).expect("reads");
        let named = declarations.named("cited_by").expect("an inverse half");
        assert_eq!(named.relation.name, "cites_evidence");
        assert_eq!(named.direction, Direction::Inverse);
        assert_eq!(
            declarations.named("cites_evidence").unwrap().direction,
            Direction::AsDeclared
        );
        assert!(declarations.named("invented").is_none());
    }

    #[test]
    fn a_relation_with_no_target_set_is_refused_at_its_own_line() {
        let errors = read("relations:\n  broken:\n    from: [decision]\n").expect_err("refused");
        assert_eq!(errors[0].span.start.line, 2);
        assert!(
            errors[0].to_string().contains("declares no `to`"),
            "{}",
            errors[0]
        );
    }

    #[test]
    fn an_anchor_kind_with_no_resolver_has_no_identity_and_is_refused() {
        let errors = read("anchors:\n  code_path: {}\n").expect_err("refused");
        assert!(
            errors[0].to_string().contains("names no `resolver`"),
            "{}",
            errors[0]
        );
    }

    /// Spec 2's four clauses, one per relation, and the fourth is the one that
    /// has to stay silent. A declaration that carried a reading order for
    /// `is_alternative_to` would misstate what the relation asserts.
    #[test]
    fn reading_precedence_is_derived_from_the_family_and_the_nucleus() {
        let declarations = read(
            "relations:\n  \
             supersedes: {family: succession, to: [d], inverse: superseded_by}\n  \
             governs: {family: governance, to: [d]}\n  \
             refines: {family: derivation, to: [d], nuclearity: nucleus-satellite, nucleus: to}\n  \
             contains: {family: composition, to: [d], nucleus: from}\n  \
             is_alternative_to: {family: association, to: [d]}\n  \
             traces_to: {family: evidence, to: [d]}\n",
        )
        .expect("the declarations read");
        let governs = |name: &str| {
            declarations
                .named(name)
                .expect("declared")
                .relation
                .governs()
        };
        assert_eq!(governs("supersedes"), Governs::Source, "the successor");
        assert_eq!(governs("governs"), Governs::Source, "the constraint");
        assert_eq!(governs("refines"), Governs::Target, "the nucleus");
        assert_eq!(governs("contains"), Governs::Source, "the nucleus");
        assert_eq!(governs("is_alternative_to"), Governs::Neither);
        assert_eq!(governs("traces_to"), Governs::Neither);
    }

    /// A family this engine does not know derives no order, rather than the
    /// order of whichever clause the match arm happened to reach.
    #[test]
    fn a_family_outside_the_six_derives_no_reading_order() {
        let declarations =
            read("relations:\n  invented: {family: astrology, to: [d]}\n").expect("it reads");
        assert_eq!(declarations.relations[0].governs(), Governs::Neither);
    }
}
