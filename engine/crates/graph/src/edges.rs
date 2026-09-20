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
use headwater_meta::pattern::Pattern;
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

/// The instance attribute that carries the routing cue of an edge.
///
/// [Q20](../../../../docs/decisions/0020-where-scent-lives.md) puts a cue on
/// the relation instance rather than on the relation type, so it is a key of
/// [`Edge::attributes`] and never a facet. The name is here because two
/// components read it: a traversal serves the cue where one exists, and
/// `taxonomy audit` counts the halves that carry one.
pub const CUE: &str = "cue";

/// The instance attribute that carries the pinned revision an imported edge was
/// checked against.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one)
/// declares it by this name, owned by the edge. It is here beside [`CUE`] for
/// the reason [`CUE`] is here: two crates read it and neither may depend on the
/// other. `headwater_import::write` writes it, `headwater_check::suspect`
/// compares it against the revision the resolver answered with, and
/// `headwater-import` already depends on `headwater-check`, so the name has to
/// sit under both of them.
pub const VERIFIED_REVISION: &str = "verified_revision";

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
        /// The identity of the anchor node: the normalized patterns, sorted,
        /// and joined by `, ` where there is more than one
        /// ([HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)).
        /// A value with no wildcard is one pattern, so this is that one
        /// normalized string, exactly as before a pattern language reached
        /// this anchor kind.
        normalized: String,
        /// The corpus exclusion that claims the target, when one does. Only
        /// ever set for a single, literal pattern: a list, or a pattern with a
        /// wildcard, drops an excluded hit from its matched set instead of
        /// naming one exclusion for the whole anchor. See
        /// [`crate::anchors::Binding::Resolved`].
        excluded_by: Option<String>,
        /// What the resolver's source says the target is at now, carried
        /// through from [`crate::anchors::Binding::Resolved`]. `None` for every
        /// resolver but a committed snapshot's, and for every anchor that
        /// holds more than one pattern: a snapshot resolver names one item at
        /// one revision, and this ruling admits a list only where the
        /// resolver is `source-tree`.
        revision: Option<String>,
        /// One entry per pattern the anchor holds, in the identity order
        /// above. A value with no wildcard is one anchor with one member here.
        /// `headwater explain` reads each member's own matched count, and the
        /// union of every member's matched set is what a cached verdict has to
        /// hold in its key ([HW-OBL-0117](../../../../docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md)):
        /// [`Target::resolution`] is the derived `Debug` of this whole
        /// variant, so this field reaching the key needs no separate wiring.
        patterns: Vec<PatternMember>,
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

/// One pattern of an anchor, with what it reached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternMember {
    /// The pattern as the resolver normalized it.
    pub pattern: String,
    /// The tree entries this one pattern matched, sorted and with no
    /// duplicate, straight from [`crate::anchors::Binding::Resolved::matched`].
    pub matched: Vec<String>,
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

    /// Whether this anchor's pattern set reaches `path`: some pattern it holds
    /// matches it.
    ///
    /// [HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md):
    /// "A query about a path matches the path against every pattern of every
    /// anchor and never against the string." This recompiles each pattern
    /// rather than reading [`PatternMember::matched`], so it answers for a
    /// path this run never resolved an edge onto, and not only for the ones a
    /// resolver already walked.
    pub fn reaches(&self, path: &str) -> bool {
        match self {
            Target::Anchor { patterns, .. } => patterns
                .iter()
                .any(|member| Pattern::new(&member.pattern).matches(path)),
            _ => false,
        }
    }

    /// How many tree entries this anchor reaches, in total and per pattern —
    /// the denominator [HW-OBL-0104](../../../../docs/obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md)
    /// records as absent, and `headwater explain` is what reports it.
    pub fn reach(&self) -> Option<Reach> {
        let Target::Anchor { patterns, .. } = self else {
            return None;
        };
        let mut total: Vec<&str> = patterns
            .iter()
            .flat_map(|member| member.matched.iter().map(String::as_str))
            .collect();
        total.sort_unstable();
        total.dedup();
        Some(Reach {
            total: total.len(),
            members: patterns
                .iter()
                .map(|member| (member.pattern.clone(), member.matched.len()))
                .collect(),
        })
    }

    /// The patterns of an anchor, sorted and joined by `, `, for a reader
    /// rather than for identity. `None` for every other variant.
    ///
    /// `Target::Anchor.normalized` is not this: for two or more patterns it
    /// is an encoding chosen so that two different lists can never share one
    /// value (see `crate::edges::encode_list_identity`), and that guarantee
    /// is worth nothing to a person reading a rendered edge. A caller that
    /// prints an anchor for a human — `headwater explain`, `headwater route`,
    /// the MCP tools built on both — reaches for this instead. A caller that
    /// needs the identity a cache key or a duplicate-edge check can trust
    /// reaches for `normalized`, unchanged.
    pub fn anchor_display(&self) -> Option<String> {
        let Target::Anchor { patterns, .. } = self else {
            return None;
        };
        Some(
            patterns
                .iter()
                .map(|member| member.pattern.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
        )
    }
}

/// How many entries an anchor reaches, in total and per pattern it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reach {
    /// The size of the union across every pattern the anchor holds.
    pub total: usize,
    /// One pair per pattern, in the anchor's own order: the pattern, and how
    /// many entries it alone matched.
    pub members: Vec<(String, usize)>,
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

    // Whether every kind this entry may reach is an anchor kind. A list is the
    // pattern language's own "or a list of them"
    // ([HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)),
    // and a document target is one identifier, never a set, so a list is
    // admitted only where nothing here could resolve to a document.
    let anchor_only = permitted
        .iter()
        .all(|kind| declarations.anchor(kind).is_some());

    for item in items {
        let (raws, attributes, span): (Vec<String>, Vec<Entry>, Span) = match &item.value {
            Value::Scalar(scalar) => (vec![scalar.text.clone()], Vec::new(), item.span),
            Value::Map(map) => match target_of(map, anchor_only) {
                Some((raws, span)) => (raws, instance_attributes(map), span),
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
            // A list in the place of the string: one anchor over several
            // patterns, admitted only where the endpoint is an anchor kind. A
            // document target stays one identifier, so a nested list there is
            // refused exactly as it always was.
            Value::Seq(members) if anchor_only => {
                let mut raws = Vec::new();
                let mut every_member_a_scalar = true;
                for member in members {
                    match member.value.as_scalar() {
                        Some(scalar) => raws.push(scalar.text.clone()),
                        None => {
                            every_member_a_scalar = false;
                            break;
                        }
                    }
                }
                if !every_member_a_scalar || raws.is_empty() {
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
                (raws, Vec::new(), item.span)
            }
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

        let raw_target = raws.join(", ");
        let target = bind(&raws, permitted, index, declarations, resolvers);
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

/// One member of a claimed anchor's [`Binding::Resolved`], carried as a named
/// type rather than a tuple so that a reader — and clippy's
/// `type_complexity` lint — sees what each position means.
struct Resolved {
    normalized: String,
    excluded_by: Option<String>,
    revision: Option<String>,
    matched: Vec<String>,
}

/// An identity for two or more sorted patterns, injective over their content.
///
/// A plain joined string is not: `["a", "b, c"]` and `["a, b", "c"]` both
/// sort and join on `, ` to `"a, b, c"`, so two different lists would share
/// one node, one `RepeatedTriple` count and one `anchor_nodes()` entry. Each
/// pattern here is prefixed with its own byte length instead, so the one
/// place a delimiter could appear is inside a length prefix's own digits,
/// which a colon closes before any pattern byte is read — the boundary
/// between two members never depends on what either member's bytes are.
///
/// This is `format!("{}:{pattern}", pattern.len())` per member, concatenated
/// with nothing between them: a caller that wants to reconstruct the list
/// reads the digits up to the next `:`, takes that many bytes as one pattern,
/// and repeats.
///
/// It is not proven injective against every literal, single-pattern
/// identity: a real path could in principle spell a valid encoding of some
/// other list (a file named `1:a1:b`, matching the encoding of `["a", "b"]`,
/// is legal on this engine's target filesystems). No pattern in this
/// repository does, encoding a list is the one case this function is for,
/// and a single pattern never reaches it — see the `[member]` arm beside
/// every call site.
fn encode_list_identity(patterns: &[&str]) -> String {
    patterns
        .iter()
        .map(|pattern| format!("{}:{pattern}", pattern.len()))
        .collect()
}

/// Bind one target: a single string, or the several patterns of a list anchor
/// admitted where the endpoint is anchor-only
/// ([HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)).
/// `raws` is never empty. A single entry is bound in the order the module
/// comment states, unchanged from before a list existed; a list is bound only
/// against an anchor kind, and only where every pattern it holds binds under
/// the same one.
fn bind(
    raws: &[String],
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

    if let [raw] = raws {
        if admits_a_document {
            if let Some(node) = index.node(raw) {
                return Target::Document {
                    id: node.id.clone(),
                    path: node.path.clone(),
                    kind: node.kind.clone().unwrap_or_default(),
                };
            }
        }
    }

    // Every anchor kind the declaration admits, asked in declaration order. A
    // string (or a list of them) that two anchor kinds both claim has two
    // identities, and identity is the one thing an anchor carries.
    let mut claimed: Vec<(&str, Vec<Binding>)> = Vec::new();
    let mut refusals: Vec<Unbound> = Vec::new();
    for anchor in &anchor_kinds {
        let Some(resolver) = resolvers.get(&anchor.resolver) else {
            refusals.push(Unbound::NoResolver {
                anchor_kind: anchor.name.clone(),
                resolver: anchor.resolver.clone(),
            });
            continue;
        };
        let mut bindings = Vec::with_capacity(raws.len());
        let mut dead: Option<Unbound> = None;
        for raw in raws {
            match resolver.resolve(raw) {
                binding @ (Binding::Resolved { .. } | Binding::Withheld { .. }) => {
                    bindings.push(binding);
                }
                // A list with one dead member is reported for that member,
                // and the first dead member found stops the search: the rest
                // add no information a repair of this one does not also need.
                Binding::Unresolved(why) => {
                    dead = Some(Unbound::AnchorUnresolved {
                        anchor_kind: anchor.name.clone(),
                        why: match raws.len() {
                            1 => why,
                            _ => format!("`{raw}` {why}"),
                        },
                    });
                    break;
                }
            }
        }
        match dead {
            Some(unbound) => refusals.push(unbound),
            None => claimed.push((&anchor.name, bindings)),
        }
    }

    if claimed.len() > 1 {
        return Target::Unbound(Unbound::AmbiguousAnchor {
            anchor_kinds: claimed.iter().map(|(name, _)| name.to_string()).collect(),
        });
    }
    if let Some((anchor_kind, bindings)) = claimed.pop() {
        // A withheld member withholds the whole anchor: withholding is a
        // per-item decision no export filter states over a set yet (M6), so
        // one withheld pattern cannot be silently dropped from the union.
        if let Some(profile) = bindings.iter().find_map(|binding| match binding {
            Binding::Withheld { profile } => Some(profile.clone()),
            _ => None,
        }) {
            return Target::Withheld {
                anchor_kind: anchor_kind.to_string(),
                profile,
            };
        }

        // Every remaining binding is `Resolved`: `Withheld` was just handled,
        // and a `Binding::Unresolved` never reaches `claimed` in the first
        // place — the loop above turns the first one into a refusal instead.
        let mut resolved: Vec<Resolved> = bindings
            .into_iter()
            .map(|binding| match binding {
                Binding::Resolved {
                    normalized,
                    excluded_by,
                    revision,
                    matched,
                } => Resolved {
                    normalized,
                    excluded_by,
                    revision,
                    matched,
                },
                _ => unreachable!("withheld handled above, and unresolved never reaches `claimed`"),
            })
            .collect();
        // HW-DR-0074: "the identity of an anchor node is its normalized
        // patterns, sorted." One list written in two orders is one node.
        resolved.sort_by(|a, b| a.normalized.cmp(&b.normalized));

        // A single pattern's identity is the pattern itself, unchanged: every
        // edge this corpus already declares keeps the identity it has. A list
        // needs an identity that tells two different lists apart, which a
        // plain joined string cannot promise — see `encode_list_identity`.
        // `Target::anchor_display` is the human-facing join, kept separate on
        // purpose, and it is what a renderer should reach for instead of this
        // field.
        let normalized = match resolved.as_slice() {
            [member] => member.normalized.clone(),
            members => encode_list_identity(
                &members
                    .iter()
                    .map(|member| member.normalized.as_str())
                    .collect::<Vec<&str>>(),
            ),
        };
        // A single, literal pattern keeps the one exclusion note it carried
        // before a list existed. A list, or a wildcard pattern, drops an
        // excluded hit from the matched count instead of naming one exclusion
        // for the whole set — see `PatternMember` and `Binding::Resolved`.
        let (excluded_by, revision) = match resolved.as_slice() {
            [member] => (member.excluded_by.clone(), member.revision.clone()),
            _ => (None, None),
        };
        let patterns = resolved
            .into_iter()
            .map(|member| PatternMember {
                pattern: member.normalized,
                matched: member.matched,
            })
            .collect();

        return Target::Anchor {
            anchor_kind: anchor_kind.to_string(),
            resolver: declarations
                .anchor(anchor_kind)
                .map(|anchor| anchor.resolver.clone())
                .unwrap_or_default(),
            normalized,
            excluded_by,
            revision,
            patterns,
        };
    }

    // Nothing bound it. Whose defect that is depends on whether a document
    // carries the identifier and the census gave it no kind. A list never
    // reaches this branch as a document: the caller admits one only where
    // every permitted kind is an anchor, so `admits_a_document` is always
    // false there.
    if admits_a_document {
        let raw = raws.first().map(String::as_str).unwrap_or_default();
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
fn target_of(map: &Mapping, anchor_only: bool) -> Option<(Vec<String>, Span)> {
    let to = map.get("to")?;
    if let Some(scalar) = to.value.as_scalar() {
        return Some((vec![scalar.text.clone()], to.span));
    }
    // A list in the place of the string, admitted only where the endpoint is
    // an anchor kind — see the caller's `anchor_only` and HW-DR-0074.
    if anchor_only {
        if let Value::Seq(members) = &to.value {
            let raws: Option<Vec<String>> = members
                .iter()
                .map(|member| member.value.as_scalar().map(|scalar| scalar.text.clone()))
                .collect();
            if let Some(raws) = raws {
                if !raws.is_empty() {
                    return Some((raws, to.span));
                }
            }
        }
    }
    None
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
                id: "HW-SPEC-ai-integration".to_string(),
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
            revision: None,
            patterns: vec![PatternMember {
                pattern: ".claude/hooks/lib.sh".to_string(),
                matched: vec![".claude/hooks/lib.sh".to_string()],
            }],
        }
    }

    /// The same anchor, at a revision. A snapshot resolver answers with one and
    /// the source tree never does, so this is the second shape of one binding.
    fn pinned(revision: &str) -> Target {
        Target::Anchor {
            anchor_kind: "ado_work_item".to_string(),
            resolver: "ado-snapshot".to_string(),
            normalized: "12345".to_string(),
            excluded_by: None,
            revision: Some(revision.to_string()),
            patterns: vec![PatternMember {
                pattern: "12345".to_string(),
                matched: vec!["12345".to_string()],
            }],
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
            revision: None,
            patterns: vec![PatternMember {
                pattern: ".claude/hooks/lib.sh".to_string(),
                matched: vec![".claude/hooks/lib.sh".to_string()],
            }],
        };
        let others = [
            excluded,
            gone(),
            // One identity at two revisions. Nothing else about these two
            // differs: same anchor kind, same resolver, same normalized
            // string. A rendering that dropped the revision would give them one
            // key, and `relation.target.suspect` would then be served its own
            // stale verdict on every run after the first.
            pinned("7"),
            pinned("8"),
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
                id: "HW-SPEC-check-layer".to_string(),
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

    /// The collision a plain `, `-joined identity admits: two different
    /// lists, sorted, that join to one identical string because one member
    /// holds the separator text. `encode_list_identity` has to tell them
    /// apart, which `, `-joining them never could.
    #[test]
    fn two_different_lists_never_share_one_identity_even_when_a_member_holds_the_join_text() {
        let left = encode_list_identity(&["a", "b, c"]);
        let right = encode_list_identity(&["a, b", "c"]);
        assert_ne!(
            left, right,
            "both lists sort and join to \"a, b, c\" under a plain join"
        );
    }

    /// The general property behind the case above: any two different sorted
    /// pattern lists encode to two different identities.
    #[test]
    fn encode_list_identity_is_injective_over_a_table_of_adversarial_lists() {
        let lists: [&[&str]; 6] = [
            &["a", "b"],
            &["a", "b, c"],
            &["a, b", "c"],
            &["a", "b", "c"],
            &[",", ":"],
            &["1:a", "b"],
        ];
        let mut seen = std::collections::BTreeSet::new();
        for list in lists {
            let encoded = encode_list_identity(list);
            assert!(
                seen.insert(encoded.clone()),
                "two different lists share one identity: {encoded}"
            );
        }
    }
}
