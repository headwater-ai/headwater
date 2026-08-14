// SPDX-License-Identifier: Apache-2.0
//! Edges: a `relations:` block resolved against the index and the resolvers.
//!
//! # The identity of an edge
//!
//! [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage) fixes it:
//! the source identifier, the relation name, and the normalized target. Two
//! consequences follow from that sentence and both are enforced here. List
//! order carries no meaning, so nothing downstream may read it. A repeated
//! triple in one document is an error, because the second one adds no edge and
//! the author meant something by writing it.
//!
//! # The authored form, and the sugar
//!
//! An entry is a target reference, or a mapping with `to:` and instance
//! attributes. The scalar is sugar for the mapping, and both produce the same
//! edge ([spec 1](../../../../docs/spec/01-conceptual-model.md#the-authored-form)).
//! A single entry and a sequence of one are also the same thing: `supersedes:
//! DR-0031` and `supersedes: [DR-0031]` differ in YAML and not in the corpus.
//!
//! # Either half may be the one that is written
//!
//! A relation declares an inverse, and an author may write either name. This
//! corpus writes both: `cites_evidence` on the citing document and `cited_by`
//! on the cited one, 81 times each. So the *authored* direction and the
//! *declared* direction are two different things, and an edge carries both.
//! The two halves of one reciprocal pair normalize to one triple through
//! [`Edge::declared_triple`], which is what lets a reciprocity check find the
//! half that is missing without knowing which name an author reached for.
//!
//! # What decides whether a target is a document or an anchor
//!
//! The declaration does. [Spec 1](../../../../docs/spec/01-conceptual-model.md#external-anchor)
//! says a relation endpoint is a declared kind or a declared anchor kind and
//! never a bare string, so the endpoint set at the authored target end says
//! which. Where that set holds both — `traces_to` declares `to: [governed_document,
//! code_path]` — the index is consulted first and the resolvers after it. That
//! order is an engine decision and no document states it, so
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries it. Two anchor kinds that both claim one string are reported as a
//! tie and bound to neither, on the precedent that
//! [`headwater_census::resolve`] set for two shelves of equal specificity.

use crate::anchors::{Binding, Resolvers};
use crate::declarations::{Declarations, Direction};
use crate::index::Index;
use crate::Config;
use headwater_census::census::Census;
use headwater_yaml::{Entry, Mapping, Span, Value};

/// A declared edge, resolved.
#[derive(Clone, Debug)]
pub struct Edge {
    /// The document that declared it.
    pub source: Source,
    /// The relation name as the author wrote it, which may be the inverse.
    pub name: String,
    /// The relation type that name resolves to.
    pub declared: String,
    pub direction: Direction,
    /// The target as written, before anything normalized it.
    pub raw_target: String,
    pub target: Target,
    /// Instance attributes: every key of the entry except `to`.
    pub attributes: Vec<Entry>,
    /// What the relation type declares about who pays for an edge of it, copied
    /// here from the declaration so that a reader of one edge does not have to
    /// hold the taxonomy as well.
    ///
    /// [Q4](../../../../docs/decisions/0004-relation-storage.md) puts the value
    /// on the relation type, so two edges of one relation carry one value here
    /// whatever wrote them. A scaffolded `supersedes` and a hand-typed one are
    /// one string on disk, and this field says so rather than implying a
    /// per-edge provenance that nothing records. What it does separate is a
    /// relation an importer may write from one it may not, and
    /// `headwater_import` refuses every relation whose value is not `import`.
    pub created_by: Option<String>,
    /// The entry that declared this edge, which is where a finding about it
    /// anchors. It is the list element rather than the relation name, so a
    /// document with eight targets under one relation reports at the one that
    /// is wrong.
    pub span: Span,
}

/// The declaring end of an edge.
#[derive(Clone, Debug)]
pub struct Source {
    pub id: String,
    pub path: String,
    pub kind: String,
    /// The span of the facet that chose the kind, from the census's own
    /// derivation. An endpoint-violation finding anchors here rather than at
    /// the top of the file.
    pub kind_span: Option<Span>,
}

/// What the target end resolved to.
#[derive(Clone, Debug)]
pub enum Target {
    /// A document of the corpus, and so a node of the graph.
    Document {
        id: String,
        path: String,
        kind: String,
    },
    /// An external anchor, normalized by the resolver that owns its kind.
    Anchor {
        anchor_kind: String,
        resolver: String,
        normalized: String,
        /// The corpus exclusion that claims the target, when one does.
        excluded_by: Option<String>,
    },
    /// The source withheld the target under a declared export filter. Never a
    /// defect, and never counted as one.
    Withheld {
        anchor_kind: String,
        profile: String,
    },
    /// Nothing bound it, and the variant says whose defect that is.
    Unbound(Unbound),
}

/// Why a target bound to nothing. The set is closed, and two of its members
/// send an author to two different files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unbound {
    /// No document and no anchor carries this string. Fix this link.
    ///
    /// `also_tried` carries what each anchor kind said, because a relation that
    /// admits both a document and an anchor consulted both, and a report that
    /// named only one of them would read as though the other was never asked.
    NoSuchTarget { also_tried: Vec<String> },
    /// A document carries it, and the census gave that document no kind. Fix
    /// that document: only a typed document can be an edge endpoint, because
    /// both ends of a declared relation are kinds.
    NotTyped {
        path: String,
        class: &'static str,
        detail: String,
    },
    /// The anchor kind is declared, and its resolver is not one this run has.
    /// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#behavior-at-the-limits):
    /// an anchor that no resolver claims is a finding.
    NoResolver {
        anchor_kind: String,
        resolver: String,
    },
    /// A resolver ran and reported a defect in the anchor.
    AnchorUnresolved { anchor_kind: String, why: String },
    /// Two anchor kinds claim one string, so the target has two identities.
    AmbiguousAnchor { anchor_kinds: Vec<String> },
}

impl std::fmt::Display for Unbound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unbound::NoSuchTarget { also_tried } if also_tried.is_empty() => {
                write!(f, "resolves to nothing at all")
            }
            Unbound::NoSuchTarget { also_tried } => write!(
                f,
                "resolves to nothing at all, and {}",
                also_tried.join("; ")
            ),
            Unbound::NotTyped {
                path,
                class,
                detail,
            } => write!(f, "names {path}, which is {class}: {detail}"),
            Unbound::NoResolver {
                anchor_kind,
                resolver,
            } => write!(
                f,
                "`{anchor_kind}` names the resolver `{resolver}`, which this run does not have"
            ),
            Unbound::AnchorUnresolved { anchor_kind, why } => write!(f, "`{anchor_kind}`: {why}"),
            Unbound::AmbiguousAnchor { anchor_kinds } => write!(
                f,
                "{} both claim this string, so it has two identities",
                anchor_kinds.join(" and ")
            ),
        }
    }
}

/// What a `relations:` block declared that the graph could not use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The block is not a mapping of relation names.
    BlockNotAMapping(&'static str),
    /// A key of the block is neither a relation nor any relation's inverse.
    UnknownRelation { name: String },
    /// An entry is neither a target reference nor a mapping with `to`.
    EntryNotUsable {
        relation: String,
        found: &'static str,
    },
    /// A mapping entry with no `to`, so it names no target.
    NoTarget { relation: String },
    /// The same triple twice in one document. The second adds no edge, and Q4
    /// makes it an error rather than a duplicate to drop quietly.
    RepeatedTriple { relation: String, target: String },
    /// The declaring document has no identifier, so no edge of it has a source.
    SourceHasNoIdentifier,
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::BlockNotAMapping(found) => {
                write!(f, "`relations` is {found}, and it names relation types")
            }
            Problem::UnknownRelation { name } => {
                write!(f, "`{name}` is neither a declared relation nor a declared inverse")
            }
            Problem::EntryNotUsable { relation, found } => write!(
                f,
                "an entry of `{relation}` is {found}, and an entry is a target or a mapping with `to`"
            ),
            Problem::NoTarget { relation } => {
                write!(f, "an entry of `{relation}` declares no `to`, so it names no target")
            }
            Problem::RepeatedTriple { relation, target } => write!(
                f,
                "`{relation}` names {target} twice, and the second declares no second edge"
            ),
            Problem::SourceHasNoIdentifier => write!(
                f,
                "this document declares edges and no identifier, so none of them has a source"
            ),
        }
    }
}

/// One problem, and the document it is about.
#[derive(Clone, Debug)]
pub struct Reported {
    pub path: String,
    pub span: Option<Span>,
    pub problem: Problem,
}

impl Edge {
    /// The identity Q4 gives an edge: source, relation name, normalized target.
    pub fn triple(&self) -> (&str, &str, String) {
        (&self.source.id, &self.name, self.normalized_target())
    }

    /// The same edge in the direction its relation declares.
    ///
    /// Two reciprocal halves written from their two ends produce one value
    /// here, which is how a later check finds the half nobody wrote.
    pub fn declared_triple(&self) -> Option<(String, String, String)> {
        let Target::Document { id, .. } = &self.target else {
            return None;
        };
        Some(match self.direction {
            Direction::AsDeclared => (self.source.id.clone(), self.declared.clone(), id.clone()),
            Direction::Inverse => (id.clone(), self.declared.clone(), self.source.id.clone()),
        })
    }

    /// The target after the thing that owns its identity normalized it.
    ///
    /// An identifier is its own normal form: spec 3 mints it once and never
    /// reuses it. An anchor string is whatever its resolver made of it.
    pub fn normalized_target(&self) -> String {
        match &self.target {
            Target::Document { id, .. } => id.clone(),
            Target::Anchor { normalized, .. } => normalized.clone(),
            Target::Withheld { .. } | Target::Unbound(_) => self.raw_target.clone(),
        }
    }

    pub fn is_bound(&self) -> bool {
        !matches!(self.target, Target::Unbound(_))
    }
}

impl Target {
    /// What binding this target string reached, in full, as one line.
    ///
    /// [`Edge::normalized_target`] is the identity of an edge and this is not
    /// that. The identity of an anchor edge is the string the resolver made of
    /// the target, and it is the **same string** whether the resolver bound it
    /// or refused it: an unbound target falls back to the raw text, and a path
    /// anchor normalizes to that text. So two runs over two trees, one holding
    /// the file and one not, produce one identity and two verdicts.
    ///
    /// A cache key needs the second value rather than the first.
    /// [Spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
    /// puts "the content hash of every document **and edge** that an instance
    /// read" in a read set, and a read set is a list of corpus paths. An anchor
    /// names something outside the corpus, so no path in that list moves when
    /// the anchor's target moves, and this value is the only thing in a key
    /// that names it.
    ///
    /// **It is the derived `Debug`, and that is the point.** The value has to
    /// be total over the type: a variant or a field that this rendering omits
    /// is an input that a key omits, which is the correctness bug spec 12 names
    /// and the one this method exists for. A hand-written arm per variant is a
    /// second statement of the shape of [`Target`], and the next variant added
    /// beside it would key nothing new while compiling clean. The compiler
    /// writes this one, so it cannot fall behind the type.
    ///
    /// The format is not a stable one, and it does not need to be. Nothing
    /// reads this back: it is hashed into a key, and a rustc that renders it
    /// differently costs one uncached run, which is the direction
    /// [`headwater_check`'s cache](../../../../docs/spec/12-check-layer.md#determinism-concretely)
    /// resolves every doubtful case in.
    pub fn resolution(&self) -> String {
        format!("{self:?}")
    }
}

/// Resolve every `relations:` block of the census into edges.
///
/// The input set is the rows that resolved a kind, which is
/// [`headwater_census::census::Outcome::node`]'s answer rather than a second
/// reading here. A document the census gave no kind is not an endpoint, so its
/// `relations:` block declares no edge, and walking the tree again to find one
/// would be a second denominator.
///
/// A generated document that declared an identity is on that list, so it can be
/// either end of an edge. Spec 6 excuses it from checks and not from identity,
/// and an edge it declares is a fact its emitter wrote from the corpus rather
/// than a claim its author made.
pub fn build(
    census: &Census,
    index: &Index,
    declarations: &Declarations,
    resolvers: &Resolvers,
    config: &Config,
) -> (Vec<Edge>, Vec<Reported>) {
    let mut edges = Vec::new();
    let mut problems = Vec::new();

    for row in &census.rows {
        let Some((kind, derivation)) = row.outcome.node() else {
            continue;
        };
        let Some(document) = &row.document else {
            continue;
        };
        let Some(block) = document.facets.get(&config.relations_facet) else {
            continue;
        };

        let Some(relations) = block.value.as_map() else {
            problems.push(Reported {
                path: row.path.clone(),
                span: Some(block.span),
                problem: Problem::BlockNotAMapping(block.value.kind_name()),
            });
            continue;
        };

        // An edge is identified by its source, so a document with no
        // identifier declares nothing that can be one. Reported once, against
        // the document, rather than once per target.
        let Some(node) = index.by_path(&row.path).and_then(|entry| entry.id.as_ref()) else {
            problems.push(Reported {
                path: row.path.clone(),
                span: Some(document.block),
                problem: Problem::SourceHasNoIdentifier,
            });
            continue;
        };

        let source = Source {
            id: node.clone(),
            path: row.path.clone(),
            kind: kind.to_string(),
            kind_span: derivation.and_then(|derivation| derivation.span),
        };

        let mut declared_here: Vec<(String, String)> = Vec::new();
        for entry in relations {
            read_relation_entry(
                &source,
                entry,
                index,
                declarations,
                resolvers,
                &mut declared_here,
                &mut edges,
                &mut problems,
            );
        }
    }

    (edges, problems)
}

#[allow(clippy::too_many_arguments)]
fn read_relation_entry(
    source: &Source,
    entry: &Entry,
    index: &Index,
    declarations: &Declarations,
    resolvers: &Resolvers,
    declared_here: &mut Vec<(String, String)>,
    edges: &mut Vec<Edge>,
    problems: &mut Vec<Reported>,
) {
    let name = entry.key.value.clone();
    let Some(named) = declarations.named(&name) else {
        problems.push(Reported {
            path: source.path.clone(),
            span: Some(entry.key.span),
            problem: Problem::UnknownRelation { name },
        });
        return;
    };

    // Which end of the relation the *target* sits at. A document that writes
    // the inverse name is the target of the relation it names, so its own
    // targets are sources of that relation, and the permitted set is `from`.
    let permitted = match named.direction {
        Direction::AsDeclared => &named.relation.to,
        Direction::Inverse => &named.relation.from,
    };

    // One entry, or a sequence of them. Both mean the same thing, and list
    // order carries no meaning either way.
    let items: Vec<&headwater_yaml::Spanned<Value>> = match &entry.value.value {
        Value::Seq(items) => items.iter().collect(),
        _ => vec![&entry.value],
    };

    for item in items {
        let (raw_target, attributes, span) = match &item.value {
            Value::Scalar(scalar) => (scalar.text.clone(), Vec::new(), item.span),
            Value::Map(map) => match target_of(map) {
                Some((target, span)) => (target, instance_attributes(map), span),
                None => {
                    problems.push(Reported {
                        path: source.path.clone(),
                        span: Some(item.span),
                        problem: Problem::NoTarget {
                            relation: name.clone(),
                        },
                    });
                    continue;
                }
            },
            Value::Seq(_) => {
                problems.push(Reported {
                    path: source.path.clone(),
                    span: Some(item.span),
                    problem: Problem::EntryNotUsable {
                        relation: name.clone(),
                        found: item.value.kind_name(),
                    },
                });
                continue;
            }
        };

        let target = bind(&raw_target, permitted, index, declarations, resolvers);
        let edge = Edge {
            source: source.clone(),
            name: name.clone(),
            declared: named.relation.name.clone(),
            direction: named.direction,
            raw_target,
            target,
            attributes,
            created_by: named.relation.created_by.clone(),
            span,
        };

        // Q4's identity, checked inside the one document that can repeat it.
        let key = (edge.name.clone(), edge.normalized_target());
        if declared_here.contains(&key) {
            problems.push(Reported {
                path: source.path.clone(),
                span: Some(span),
                problem: Problem::RepeatedTriple {
                    relation: key.0,
                    target: key.1,
                },
            });
            continue;
        }
        declared_here.push(key);
        edges.push(edge);
    }
}

/// Bind one target string, in the order the module comment states.
fn bind(
    raw: &str,
    permitted: &[String],
    index: &Index,
    declarations: &Declarations,
    resolvers: &Resolvers,
) -> Target {
    let anchor_kinds: Vec<&crate::declarations::AnchorKind> = permitted
        .iter()
        .filter_map(|name| declarations.anchor(name))
        .collect();
    let admits_a_document = permitted.len() > anchor_kinds.len();

    if admits_a_document {
        if let Some(node) = index.node(raw) {
            return Target::Document {
                id: node.id.clone(),
                path: node.path.clone(),
                kind: node.kind.clone().unwrap_or_default(),
            };
        }
    }

    // Every anchor kind the declaration admits, asked in declaration order. A
    // string that two of them claim has two identities, and identity is the
    // one thing an anchor carries.
    let mut claimed: Vec<(&str, Binding)> = Vec::new();
    let mut refusals: Vec<Unbound> = Vec::new();
    for anchor in &anchor_kinds {
        let Some(resolver) = resolvers.get(&anchor.resolver) else {
            refusals.push(Unbound::NoResolver {
                anchor_kind: anchor.name.clone(),
                resolver: anchor.resolver.clone(),
            });
            continue;
        };
        match resolver.resolve(raw) {
            binding @ (Binding::Resolved { .. } | Binding::Withheld { .. }) => {
                claimed.push((&anchor.name, binding))
            }
            Binding::Unresolved(why) => refusals.push(Unbound::AnchorUnresolved {
                anchor_kind: anchor.name.clone(),
                why,
            }),
        }
    }

    if claimed.len() > 1 {
        return Target::Unbound(Unbound::AmbiguousAnchor {
            anchor_kinds: claimed.iter().map(|(name, _)| name.to_string()).collect(),
        });
    }
    if let Some((anchor_kind, binding)) = claimed.pop() {
        return match binding {
            Binding::Resolved {
                normalized,
                excluded_by,
            } => Target::Anchor {
                anchor_kind: anchor_kind.to_string(),
                resolver: declarations
                    .anchor(anchor_kind)
                    .map(|anchor| anchor.resolver.clone())
                    .unwrap_or_default(),
                normalized,
                excluded_by,
            },
            Binding::Withheld { profile } => Target::Withheld {
                anchor_kind: anchor_kind.to_string(),
                profile,
            },
            Binding::Unresolved(_) => unreachable!("an unresolved binding never reaches `claimed`"),
        };
    }

    // Nothing bound it. Whose defect that is depends on whether a document
    // carries the identifier and the census gave it no kind.
    if admits_a_document {
        if let Some(near) = index.near_miss(raw) {
            if let Some(entry) = index.by_path(&near.path) {
                return Target::Unbound(Unbound::NotTyped {
                    path: near.path.clone(),
                    class: entry.class,
                    detail: "the census gave it no kind, and an endpoint is a kind".to_string(),
                });
            }
        }
        // The index was consulted and it missed, which is the fact that names
        // the defect. What the resolvers said is carried beside it rather than
        // in place of it.
        return Target::Unbound(Unbound::NoSuchTarget {
            also_tried: refusals.iter().map(|refusal| refusal.to_string()).collect(),
        });
    }
    refusals
        .into_iter()
        .next()
        .map(Target::Unbound)
        .unwrap_or(Target::Unbound(Unbound::NoSuchTarget {
            also_tried: Vec::new(),
        }))
}

/// The `to` of a mapping entry, with the span a finding anchors to.
fn target_of(map: &Mapping) -> Option<(String, Span)> {
    let to = map.get("to")?;
    let scalar = to.value.as_scalar()?;
    Some((scalar.text.clone(), to.span))
}

/// Every key of the entry except `to`.
///
/// Nothing here decides whether the relation type permits the attribute. That
/// is a Graph check, and an undeclared attribute is its finding
/// ([Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage)).
fn instance_attributes(map: &Mapping) -> Vec<Entry> {
    map.iter()
        .filter(|entry| entry.key.value != "to")
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(target: Target) -> Edge {
        Edge {
            source: Source {
                id: "SPEC-HW-ai-integration".to_string(),
                path: "docs/spec/05-ai-integration.md".to_string(),
                kind: "design_spec".to_string(),
                kind_span: None,
            },
            name: "governs".to_string(),
            declared: "governs".to_string(),
            direction: Direction::AsDeclared,
            raw_target: ".claude/hooks/lib.sh".to_string(),
            target,
            attributes: Vec::new(),
            created_by: None,
            span: Span::default(),
        }
    }

    fn bound() -> Target {
        Target::Anchor {
            anchor_kind: "code_path".to_string(),
            resolver: "source-tree".to_string(),
            normalized: ".claude/hooks/lib.sh".to_string(),
            excluded_by: None,
        }
    }

    fn gone() -> Target {
        Target::Unbound(Unbound::AnchorUnresolved {
            anchor_kind: "code_path".to_string(),
            why: "no `.claude/hooks/lib.sh` in the source tree".to_string(),
        })
    }

    /// The defect [#160](https://github.com/headwater-ai/headwater/issues/160)
    /// is about, stated as two values that a key can tell apart and an identity
    /// cannot.
    ///
    /// One `governs` edge, one anchor string, and the file behind it removed.
    /// The identity is the same on both sides, because a path anchor normalizes
    /// to the text an author wrote and an unbound target falls back to that
    /// same text. So a cache key over the identity serves the verdict of the
    /// tree that still held the file.
    #[test]
    fn a_removed_anchor_target_keeps_its_identity_and_changes_its_resolution() {
        assert_eq!(
            edge(bound()).triple(),
            edge(gone()).triple(),
            "the identity told the two trees apart, so this test proves nothing"
        );
        assert_ne!(
            bound().resolution(),
            gone().resolution(),
            "a key over this value serves a verdict the tree falsifies"
        );
    }

    /// Every field a resolver decides is in the value, so no two bindings that
    /// differ share one.
    ///
    /// The rendering is the derived `Debug`, so this is a test of that choice
    /// rather than of a hand-written table: an exclusion that claims the target
    /// and a profile that withholds it are both a resolver's answer, and a key
    /// that dropped either would serve a verdict across the change that makes
    /// it.
    #[test]
    fn two_bindings_that_differ_anywhere_render_two_resolutions() {
        let excluded = Target::Anchor {
            anchor_kind: "code_path".to_string(),
            resolver: "source-tree".to_string(),
            normalized: ".claude/hooks/lib.sh".to_string(),
            excluded_by: Some("engine/**".to_string()),
        };
        let others = [
            excluded,
            gone(),
            Target::Withheld {
                anchor_kind: "code_path".to_string(),
                profile: "public".to_string(),
            },
            Target::Unbound(Unbound::NoResolver {
                anchor_kind: "code_path".to_string(),
                resolver: "source-tree".to_string(),
            }),
            Target::Unbound(Unbound::NoSuchTarget {
                also_tried: Vec::new(),
            }),
            Target::Document {
                id: "SPEC-HW-check-layer".to_string(),
                path: "docs/spec/12-check-layer.md".to_string(),
                kind: "design_spec".to_string(),
            },
        ];
        let mut seen = vec![bound().resolution()];
        for other in others {
            let text = other.resolution();
            assert!(
                !seen.contains(&text),
                "two bindings share one value: {text}"
            );
            seen.push(text);
        }
    }
}
