// SPDX-License-Identifier: Apache-2.0
//! `taxonomy validate`, over a resolved taxonomy.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema) lists
//! what the verb checks. Two of those items are decidable over one source and
//! `headwater-meta` runs them before anything merges. Three decide a merge and
//! [`crate::resolve`] runs them as it merges. The rest read the *result*, and
//! this module is the rest.
//!
//! # The accounting is the deliverable, and not the rules alone
//!
//! [`RULES`] holds every item of spec 2's list with where it runs, and
//! [`Ran::Partly`] is the entry that earns the table. Several rules are decided
//! in part here and in part nowhere, because the part that is missing needs a
//! declaration that the language does not have. A validator that reported those
//! as passes would be the silent pass that
//! [spec 4](../../../../docs/spec/04-assurance-model.md) exists to remove, and
//! one that reported them as failures would refuse every taxonomy there is.
//! So each one states what it decided and what it did not, and
//! `headwater taxonomy validate` prints the statement.
//!
//! # A finding here names an address and no line
//!
//! A merged tree carries the span of each node and not the file it came from,
//! so a caret drawn from one would point into whichever source happened to
//! declare it. Every finding here therefore names the declaration by address —
//! `kinds.evaluation.purpose` — which is the coordinate that survives a merge.
//! The address is also what an overlay writes, so a reader can go from a finding
//! to the operation that caused it.

use crate::error::{ResolveError, ResolveErrorKind};
use headwater_meta::identifier::Template;
use headwater_meta::Pattern;
use headwater_yaml::{Mapping, Span, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Where one rule of `taxonomy validate` runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ran {
    /// Decided over one source by the meta-schema, before anything merged.
    Source(&'static str),
    /// Decided by the resolver, as part of the merge.
    Merge(&'static str),
    /// Decided here, over the resolved taxonomy.
    Resolved(&'static str),
    /// Decided here in part. The rest needs something that does not exist.
    Partly {
        decides: &'static str,
        waits: &'static str,
    },
}

/// The identity of the rule set that [`RULES`] and [`check`] carry, for
/// [`headwater_lock`](../../../../engine/crates/lock/src/lib.rs) to bind a lock
/// to.
///
/// [`headwater_lock::read`] checks a lock's format and its digest, and neither
/// says whether the 23 rules that validated it are the 23 rules this build of
/// the engine would run. A lock is written once, at `taxonomy resolve` time,
/// and a rule can gain or lose ground between that write and any later read —
/// this crate's own history, not a hypothetical: `#219` widened lifecycle
/// soundness from "reachable within the regime that names a state" to
/// "reachable in some lifecycle regime", which took a lock that a resolve had
/// already validated and made it accept taxonomies the same rule now refuses.
/// A hand-maintained number is what stands in for "this is what validated it",
/// because [`env!("CARGO_PKG_VERSION")`] is the wrong grain: it moves on a
/// change to this crate that touches no rule, and it stays put on a rule moved
/// out of this crate into another one this crate depends on.
///
/// **Any edit to a function this array reaches, or to [`check`] itself, that
/// changes what a taxonomy it accepted before now refuses, or the reverse,
/// must increment this number.** That is every one of the 23 rules spec 2
/// lists: the 18 called from [`check`] below, plus the five that gate earlier
/// in the pipeline and never reach this file — structural conformance and
/// reference well-formedness at [`Source::validate`](crate::Source::validate),
/// core satisfiability, edge provenance and overlay confluence in
/// [`crate::resolve`] and [`crate::confluence`]. A rule whose wording changed
/// with no change to what it accepts or refuses does not need the bump; a rule
/// whose *verdict* over some taxonomy changed does.
pub const RULE_SET: u32 = 1;

/// Every rule that [spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema)
/// lists for `taxonomy validate`, in spec 2's own order, and where each runs.
///
/// The list is data because a caller reports what it ran. It is also the answer
/// to a question that cost this issue an afternoon: spec 2 lists twenty-three
/// items and the engine used to say twenty-one, because four of them appeared in
/// neither the run list nor the waiting list. A rule in neither list is the
/// exact silence these constants exist to prevent.
pub const RULES: [(&str, Ran); 23] = [
    (
        "structural conformance",
        Ran::Source("the declarations, their members, every scalar type and every closed set"),
    ),
    (
        "reference well-formedness",
        Ran::Source("every address and every `$`-reference parses, and stands where one may"),
    ),
    (
        "referential integrity",
        Ran::Resolved(
            "every name a declaration reads is declared, every relation endpoint is a kind or an \
             anchor kind, and every control discharges a declared obligation. The `$`-reference \
             half runs at merge time, because a reference that reads nothing stops the merge",
        ),
    ),
    (
        "anchor integrity",
        Ran::Resolved("one resolver per anchor kind, and no two anchor kinds share a namespace"),
    ),
    (
        "identifier integrity",
        Ran::Resolved(
            "every scheme carries a namespace, every pattern reads as a template, and no two \
             schemes admit one string. The last is decided out of the parsed segments and not \
             out of the pattern text: a `{slug}` is terminal, so a template is a fixed run of \
             literal and digit positions with an optional free tail, and whether two such runs \
             meet is a comparison rather than an estimate",
        ),
    ),
    (
        "coverage",
        Ran::Resolved(
            "every shelf reaches a kind, every concrete kind is reached by a shelf, and an \
             abstract kind is reached by none",
        ),
    ),
    (
        "kind inheritance",
        Ran::Resolved(
            "`is_a` names a declared abstract kind, the chain terminates and holds no cycle, and \
             no child un-requires or forbids what a parent requires",
        ),
    ),
    (
        "purpose completeness",
        Ran::Resolved(
            "every concrete kind has a purpose by declaration or inheritance, and every declared \
             purpose is served",
        ),
    ),
    (
        "determinism",
        Ran::Resolved(
            "no two shelf patterns can match one path at equal specificity. The overlap is \
             decided over the two patterns, so no corpus has to exist for the rule to fire",
        ),
    ),
    (
        "role uniqueness",
        Ran::Resolved("at most one facet claims each role of the closed registry"),
    ),
    (
        "lifecycle soundness",
        Ran::Resolved(
            "every state is a value of the state vocabulary, every state a regime names is \
             reachable from that regime's initial state, every state of the vocabulary is \
             reachable in at least one regime, and every state with no exit is declared terminal",
        ),
    ),
    (
        "relation coherence",
        Ran::Partly {
            decides: "a nucleus–satellite relation names its nucleus and a multinuclear one \
                      names none, and `inherits` names facets that both ends carry",
            waits: "\"a family's default is not contradicted without explicit override\" states \
                    no rule that a validator can fail. A declared nuclearity is the explicit \
                    override, and spec 2 gives the count to `taxonomy audit`",
        },
    ),
    (
        "expectation well-formedness",
        Ran::Resolved(
            "every expectation names a declared relation or a declared inverse, endpoints that \
             the relation admits, a reachable target kind, and an origin facet that the \
             declaring kind requires",
        ),
    ),
    (
        "core satisfiability",
        Ran::Merge("checked on the result, naming the operation that removed the last satisfier"),
    ),
    (
        "facet canons",
        Ran::Partly {
            decides: "relevance, ascertainability and permanence, which spec 2 states are \
                      decidable from the schema alone",
            waits: "differentiation and orthogonality need a corpus, and spec 2 gives both to \
                    `taxonomy audit`",
        },
    ),
    (
        "kind rigidity",
        Ran::Resolved(
            "no kind name is a value of the state vocabulary, and none is a bare phase adjective",
        ),
    ),
    (
        "edge provenance",
        Ran::Merge(
            "the meta-schema requires `created_by` on every relation, and the resolver validates \
             the merged result against the meta-schema as well as each source. So a `remove` that \
             takes the member out is refused before these rules run, and a rule here would be \
             unreachable",
        ),
    ),
    (
        "attribute well-formedness",
        Ran::Resolved(
            "every instance attribute names a value space and an owning end, and no attribute is \
             called `to`",
        ),
    ),
    (
        "warrant integrity",
        Ran::Partly {
            decides: "every transcription projection pins a declared anchor kind, and that \
                      anchor kind names one resolver",
            waits: "whether a transcription writes over an authored path is a question about a \
                    corpus, and it belongs to the same rule as every other projection target",
        },
    ),
    (
        "context safety",
        Ran::Partly {
            decides: "every facet that carries the freshness role declares a staleness policy",
            waits: "the size budget half. Spec 5 requires one on every agent-facing kind and \
                    projection, and no declaration in the language carries a budget or marks a \
                    kind agent-facing",
        },
    ),
    (
        "overlay confluence",
        Ran::Merge("pairwise over the leaves each operation writes, before anything merges"),
    ),
    (
        "mapping integrity",
        Ran::Partly {
            decides: "every kind and every facet value that a mapping names on this side is \
                      declared here",
            waits: "the far side. A mapping names kinds in another taxonomy, and nothing fetches \
                    or resolves one",
        },
    ),
    (
        "projection targets",
        Ran::Partly {
            decides: "every projection writes a relative path that stays inside the corpus root",
            waits: "the collision with an authored path, which needs the census. `generate \
                    --check` is where that lands",
        },
    ),
];

/// What no phase decides yet, and why. One item, and it is a sub-rule rather
/// than an item of the list above.
pub const WAITING: [(&str, &str); 1] = [(
    "the blame half of a dependent-key `remove`",
    "spec 2 fails a `remove` when a surviving declaration still references the removed key. \
     Referential integrity over the result now catches every such orphan, including the bare \
     strings that no reference names, so the rule holds. What is missing is the message: the \
     finding names the declaration that reads a name nobody declares, and not the operation that \
     removed it. Spec 13 carries it",
)];

/// The accounting as a report: every rule, where it ran, and what it did not
/// decide.
///
/// `headwater taxonomy validate` prints this beside its verdict, because a
/// verdict with no statement of what was checked is the pass that
/// [spec 4](../../../../docs/spec/04-assurance-model.md) refuses.
pub fn render() -> String {
    let mut out = String::new();
    for (rule, ran) in RULES {
        match ran {
            Ran::Source(how) => out.push_str(&format!("  {rule}\n    over one source: {how}\n")),
            Ran::Merge(how) => out.push_str(&format!("  {rule}\n    at merge time: {how}\n")),
            Ran::Resolved(how) => out.push_str(&format!("  {rule}\n    on the result: {how}\n")),
            Ran::Partly { decides, waits } => out.push_str(&format!(
                "  {rule} (in part)\n    on the result: {decides}\n    not decided: {waits}\n"
            )),
        }
    }
    // The heading carries its own count, so a list that empties says so rather
    // than looking like a list nobody printed.
    out.push_str(&format!("\nnot decided anywhere: {}\n", WAITING.len()));
    for (rule, why) in WAITING {
        out.push_str(&format!("  {rule}\n    {why}\n"));
    }
    out
}

/// Every shelf that declares no display name, in declaration order.
///
/// A shelf key is written for a machine and read by a person the moment an
/// emitter prints it: `spec_series` was the label of a navigation group, the
/// `<title>` of a served page and the text of a search result, on the ten
/// shelves of this repository, until `shelves.<s>.title` was declared. So a
/// shelf with no display name is a shelf whose key a reader will meet.
pub fn undisplayed(taxonomy: &Mapping) -> Vec<&str> {
    View::new(taxonomy)
        .members("shelves")
        .into_iter()
        .filter(|(_, body)| text(body, "title").is_none())
        .map(|(shelf, _)| shelf)
        .collect()
}

/// [`undisplayed`] as the block `taxonomy validate` prints.
///
/// **It refuses nothing, and that is clause 1 of
/// [#538](https://github.com/headwater-ai/headwater/issues/538): a shelf that
/// declares no display name is a decision rather than a defect.** A taxonomy
/// whose keys read well in the corpus that wrote them is a taxonomy that needs
/// no display name, and the emitter falls through to the key. What a validator
/// owes such a corpus is the reading, not a refusal, because whether
/// `evaluations` is a label or an identifier is a judgment about an audience
/// that no rule here can take.
///
/// The heading carries its own count and prints at zero, on the precedent
/// [`render`] sets for `WAITING` and [`crate::Resolution::foundings`] sets for
/// a founding: a list that empties says so rather than looking like a list
/// nobody printed. The trailing note prints only beside a name, and it is the
/// one place the fall-through is written down for a reader of this verb.
pub fn display_names(taxonomy: &Mapping) -> String {
    let bare = undisplayed(taxonomy);
    let mut out = format!("shelves that print their key for want of a display name: {}\n", bare.len());
    for shelf in &bare {
        out.push_str(&format!("  shelves.{shelf}\n"));
    }
    if !bare.is_empty() {
        out.push_str(
            "  Nothing here refuses. `shelves.<s>.title` is optional, and an emitter that \
             prints a shelf falls through to the key when none is declared. Declare one \
             wherever the key above is a name a reader would not have chosen.\n",
        );
    }
    out
}

/// The default nuclearity of each relation family, from spec 2's own table. A
/// family with no default has a blank cell there, and it is absent here.
///
/// It is public because `taxonomy audit` reports every relation that
/// contradicts the default its family gives ([spec
/// 2](../../../../docs/spec/02-taxonomy-model.md#nuclearity)), and that
/// reading is the same table read from the other side. A second copy of it in
/// the audit crate could disagree with the one a validator runs, and the two
/// would then name different relations as overrides.
pub const FAMILY_NUCLEARITY: [(&str, &str); 4] = [
    ("succession", "multinuclear"),
    ("derivation", "nucleus-satellite"),
    ("composition", "nucleus-satellite"),
    ("association", "multinuclear"),
];

/// The phase adjectives that no kind may be named with. Spec 2 states the list.
const PHASE_ADJECTIVES: [&str; 7] = [
    "draft",
    "pending",
    "proposed",
    "deprecated",
    "legacy",
    "temporary",
    "obsolete",
];

/// The engine-significant facet roles. The registry is closed and spec 2 states
/// it in the role-uniqueness rule itself.
const ROLES: [&str; 6] = [
    "state",
    "state_entered",
    "created",
    "freshness",
    "scent",
    "name",
];

/// Run every rule that reads a resolved taxonomy.
pub fn check(taxonomy: &Mapping) -> Vec<ResolveError> {
    let view = View::new(taxonomy);
    let mut out = Vec::new();
    referential_integrity(&view, &mut out);
    anchor_integrity(&view, &mut out);
    identifier_integrity(&view, &mut out);
    coverage(&view, &mut out);
    kind_inheritance(&view, &mut out);
    purpose_completeness(&view, &mut out);
    determinism(&view, &mut out);
    role_uniqueness(&view, &mut out);
    lifecycle_soundness(&view, &mut out);
    relation_coherence(&view, &mut out);
    expectations(&view, &mut out);
    facet_canons(&view, &mut out);
    kind_rigidity(&view, &mut out);
    attributes(&view, &mut out);
    warrant_integrity(&view, &mut out);
    context_safety(&view, &mut out);
    mapping_integrity(&view, &mut out);
    projection_targets(&view, &mut out);
    out
}

// --- the view ---------------------------------------------------------------

/// A resolved taxonomy, read the way the rules ask questions of it.
struct View<'a> {
    root: &'a Mapping,
}

impl<'a> View<'a> {
    fn new(root: &'a Mapping) -> Self {
        Self { root }
    }

    /// The members of a declaration whose value is a mapping, in source order.
    fn members(&self, block: &str) -> Vec<(&'a str, &'a Mapping)> {
        let Some(map) = self.root.get(block).and_then(|node| node.value.as_map()) else {
            return Vec::new();
        };
        map.iter()
            .filter_map(|entry| {
                entry
                    .value
                    .value
                    .as_map()
                    .map(|body| (entry.key.value.as_str(), body))
            })
            .collect()
    }

    /// The items of a declaration whose value is a sequence of mappings.
    fn items(&self, block: &str) -> Vec<&'a Mapping> {
        let Some(items) = self.root.get(block).and_then(|node| node.value.as_seq()) else {
            return Vec::new();
        };
        items
            .iter()
            .filter_map(|item| item.value.as_map())
            .collect()
    }

    fn names(&self, block: &str) -> BTreeSet<&'a str> {
        self.members(block)
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    }

    fn regimes(&self, family: &str) -> BTreeSet<&'a str> {
        let Some(regimes) = self
            .root
            .get("regimes")
            .and_then(|node| node.value.as_map())
        else {
            return BTreeSet::new();
        };
        let Some(map) = regimes.get(family).and_then(|node| node.value.as_map()) else {
            return BTreeSet::new();
        };
        map.iter().map(|entry| entry.key.value.as_str()).collect()
    }

    fn kind(&self, name: &str) -> Option<&'a Mapping> {
        self.members("kinds")
            .into_iter()
            .find(|(found, _)| *found == name)
            .map(|(_, body)| body)
    }

    /// Whether a kind is abstract, through the core schema.
    ///
    /// The reading this replaces compared the text of the scalar to `"true"`.
    /// A package that wrote `abstract: True` passed the meta-schema, which
    /// validates the member with the same function called here, and then had
    /// its one abstract kind read as concrete. Three rules read this answer,
    /// and none of the findings they then reported named `abstract`.
    fn is_abstract(&self, name: &str) -> bool {
        self.kind(name)
            .and_then(|body| headwater_yaml::core_schema::flag(body, "abstract"))
            .unwrap_or(false)
    }

    /// A kind and every kind above it, nearest first, with a cycle cut short.
    fn ancestry(&self, name: &str) -> Vec<&'a str> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = self
            .members("kinds")
            .into_iter()
            .find(|(found, _)| *found == name)
            .map(|(found, _)| found);
        while let Some(step) = current {
            if !seen.insert(step) {
                break;
            }
            chain.push(step);
            current = self
                .kind(step)
                .and_then(|body| text(body, "is_a"))
                .and_then(|parent| {
                    self.members("kinds")
                        .into_iter()
                        .find(|(found, _)| *found == parent)
                        .map(|(found, _)| found)
                });
        }
        chain
    }

    /// Every facet a kind carries, required or optional, through the chain.
    fn facets_of(&self, kind: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for step in self.ancestry(kind) {
            let Some(contract) = self.kind(step).and_then(|body| block(body, "facets")) else {
                continue;
            };
            for key in ["require", "optional"] {
                out.extend(strings(contract, key));
            }
        }
        out
    }

    fn required_facets_of(&self, kind: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for step in self.ancestry(kind) {
            let Some(contract) = self.kind(step).and_then(|body| block(body, "facets")) else {
                continue;
            };
            out.extend(strings(contract, "require"));
        }
        out
    }

    /// The facet that carries an engine role, if one does.
    fn facet_with_role(&self, role: &str) -> Option<&'a str> {
        self.members("facets")
            .into_iter()
            .find(|(_, body)| text(body, "role") == Some(role))
            .map(|(name, _)| name)
    }

    /// The values of the state vocabulary, read through the facet that carries
    /// the state role. The vocabulary is already resolved, so a value is either
    /// a scalar or a mapping with a `value` member.
    fn states(&self) -> Vec<(String, Option<String>)> {
        let Some(facet) = self.facet_with_role("state") else {
            return Vec::new();
        };
        let Some(values) = self
            .members("facets")
            .into_iter()
            .find(|(name, _)| *name == facet)
            .and_then(|(_, body)| body.get("values"))
            .and_then(|node| node.value.as_seq())
        else {
            return Vec::new();
        };
        values
            .iter()
            .filter_map(|item| match &item.value {
                Value::Scalar(scalar) => Some((scalar.text.clone(), None)),
                Value::Map(map) => text(map, "value")
                    .map(|value| (value.to_string(), text(map, "role").map(str::to_string))),
                Value::Seq(_) => None,
            })
            .collect()
    }

    /// Every shelf that reaches a kind, as kind name to shelf name.
    fn shelved_kinds(&self) -> BTreeMap<String, Vec<&'a str>> {
        let mut out: BTreeMap<String, Vec<&'a str>> = BTreeMap::new();
        for (shelf, body) in self.members("shelves") {
            let mut named = strings(body, "kinds");
            if let Some(one) = text(body, "kind") {
                named.push(one.to_string());
            }
            for kind in named {
                out.entry(kind).or_default().push(shelf);
            }
        }
        out
    }
}

// --- the rules --------------------------------------------------------------

/// Every name that a declaration reads is a name that something declares.
///
/// This is the half of referential integrity that no `$`-reference reaches. A
/// shelf writes `kind: decision` as a bare string, and the meta-schema types
/// that position as a string and says nothing about it naming a kind. The
/// resolver could not compute the dependent set of a `remove` for that reason.
/// It does not have to: a `remove` that orphans a name leaves a declaration
/// reading a name that nothing declares, and that is what this rule finds.
///
/// The traversal is [`dangling_names`] and this is the rendering of it. One
/// traversal answers both questions a dangling name raises — the refusal a
/// reader gets, and which other declaration would have supplied the name — so
/// there is no second reading of the resolved taxonomy to drift against this
/// one. See [`crate::selection`], which asks the second question.
fn referential_integrity(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "referential integrity";
    for found in dangling_names(view) {
        out.push(refusal(RULE, &found.at, found.message()));
    }
}

/// A name a declaration reads that nothing in the resolved taxonomy declares.
///
/// The address, the sort of thing the name was read as, and the name itself,
/// kept apart. [`referential_integrity`] renders these three into the sentence
/// a reader gets, and that sentence is the only place they are joined: a
/// consumer of this reading takes the fields, never the prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dangling {
    /// The address of the declaration that reads the name.
    pub at: String,
    /// What the position types the name as: `kind`, `purpose`, `facet`, and so
    /// on. The message says "no {what} of that name is declared".
    pub what: String,
    /// The name that nothing declares.
    pub name: String,
}

impl Dangling {
    /// The sentence `referential_integrity` reports.
    pub fn message(&self) -> String {
        format!(
            "reads `{}`, and no {} of that name is declared",
            self.name, self.what
        )
    }
}

/// Every name a resolved taxonomy reads and does not declare.
///
/// The same traversal `taxonomy validate` refuses on, exposed for a caller that
/// has a question about the names rather than about the verdict.
pub fn dangling(taxonomy: &Mapping) -> Vec<Dangling> {
    dangling_names(&View::new(taxonomy))
}

fn dangling_names(view: &View) -> Vec<Dangling> {
    let mut out: Vec<Dangling> = Vec::new();
    let kinds = view.names("kinds");
    let facets = view.names("facets");
    let purposes = view.names("purposes");
    let anchors = view.names("anchors");
    let relations = view.names("relations");
    let schemes = view.names("identifier_schemes");
    let shelves = view.names("shelves");
    let voice = view.regimes("voice");
    let lifecycle = view.regimes("lifecycle");
    let language = view.regimes("language");

    let mut reads = |at: String, what: &str, name: &str, declared: &BTreeSet<&str>| {
        if !declared.contains(name) {
            out.push(Dangling {
                at,
                what: what.to_string(),
                name: name.to_string(),
            });
        }
    };

    for (kind, body) in view.members("kinds") {
        let at = |member: &str| format!("kinds.{kind}.{member}");
        if let Some(parent) = text(body, "is_a") {
            reads(at("is_a"), "kind", parent, &kinds);
        }
        if let Some(purpose) = text(body, "purpose") {
            reads(at("purpose"), "purpose", purpose, &purposes);
        }
        for (member, declared) in [
            ("voice", &voice),
            ("lifecycle", &lifecycle),
            ("language", &language),
        ] {
            if let Some(regime) = text(body, member) {
                reads(at(member), &format!("{member} regime"), regime, declared);
            }
        }
        if let Some(scheme) = block(body, "identifier").and_then(|node| text(node, "scheme")) {
            reads(
                at("identifier.scheme"),
                "identifier scheme",
                scheme,
                &schemes,
            );
        }
        if let Some(contract) = block(body, "facets") {
            for member in ["require", "optional", "forbid"] {
                for facet in strings(contract, member) {
                    reads(at(&format!("facets.{member}")), "facet", &facet, &facets);
                }
            }
        }
    }

    for (shelf, body) in view.members("shelves") {
        let at = |member: &str| format!("shelves.{shelf}.{member}");
        if let Some(kind) = text(body, "kind") {
            reads(at("kind"), "kind", kind, &kinds);
        }
        for kind in strings(body, "kinds") {
            reads(at("kinds"), "kind", &kind, &kinds);
        }
        for member in ["discriminator", "group_by"] {
            if let Some(facet) = text(body, member) {
                reads(at(member), "facet", facet, &facets);
            }
        }
    }

    // An endpoint is a declared kind or a declared anchor kind, and an abstract
    // kind is a legal endpoint because endpoints resolve through the chain.
    let endpoints: BTreeSet<&str> = kinds.union(&anchors).copied().collect();
    for (relation, body) in view.members("relations") {
        let at = |member: &str| format!("relations.{relation}.{member}");
        for end in ["from", "to"] {
            for name in strings(body, end) {
                reads(at(end), "kind or anchor kind", &name, &endpoints);
            }
        }
        for facet in strings(body, "inherits") {
            reads(at("inherits"), "facet", &facet, &facets);
        }
    }

    for (index, body) in view.items("projections").into_iter().enumerate() {
        let at = |member: &str| format!("projections.{index}.{member}");
        for shelf in strings(body, "for") {
            reads(at("for"), "shelf", &shelf, &shelves);
        }
        if let Some(anchor) = block(body, "from").and_then(|node| text(node, "anchor")) {
            reads(at("from.anchor"), "anchor kind", anchor, &anchors);
        }
    }

    for (index, body) in core_requirements(view).into_iter().enumerate() {
        let at = |member: &str| format!("core.requires.{index}.{member}");
        if let Some(purpose) = text(body, "purpose") {
            reads(at("purpose"), "purpose", purpose, &purposes);
        }
        if let Some(scheme) = text(body, "identifier_scheme") {
            reads(
                at("identifier_scheme"),
                "identifier scheme",
                scheme,
                &schemes,
            );
        }
    }

    // A control names the obligations it discharges, as bare strings that only
    // a rule over the result can check. This is where a `remove` that orphans an
    // obligation lands, and it is the whole of what the taxonomy can decide
    // about the binding: which *rule* a mechanism names is the engine's rule set
    // rather than a declaration, so no rule here reads it.
    let obligations = view.names("obligations");
    for (control, body) in view.members("controls") {
        let at = format!("controls.{control}.discharges");
        for obligation in strings(body, "discharges") {
            reads(at.clone(), "obligation", &obligation, &obligations);
        }
    }

    // An expectation names a relation or the declared inverse of one. The
    // inverse is what a kind at the receiving end has to write, and the section
    // on expectations reads it in that direction.
    let inverses: BTreeSet<&str> = view
        .members("relations")
        .into_iter()
        .filter_map(|(_, body)| text(body, "inverse"))
        .collect();
    let namable: BTreeSet<&str> = relations.union(&inverses).copied().collect();
    for (kind, expectation) in expectations_of(view) {
        let at = format!("kinds.{kind}.relations.expect.{}", id_of(expectation));
        if let Some(relation) = text(expectation, "relation") {
            reads(
                format!("{at}.relation"),
                "relation or declared inverse",
                relation,
                &namable,
            );
        }
        if let Some(target) = text(expectation, "to_kind") {
            reads(format!("{at}.to_kind"), "kind", target, &kinds);
        }
    }

    out
}

/// One resolver per anchor kind, and one anchor kind per resolver namespace.
fn anchor_integrity(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "anchor integrity";
    let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();
    for (anchor, body) in view.members("anchors") {
        let Some(resolver) = text(body, "resolver") else {
            out.push(refusal(
                RULE,
                &format!("anchors.{anchor}"),
                "names no resolver, and an anchor kind names exactly one".to_string(),
            ));
            continue;
        };
        if let Some(other) = claimed.insert(resolver, anchor) {
            out.push(refusal(
                RULE,
                &format!("anchors.{anchor}.resolver"),
                format!(
                    "claims the resolver namespace `{resolver}`, and `anchors.{other}` \
                         already claims it. One string would then resolve two ways"
                ),
            ));
        }
    }
}

/// A namespace on every scheme, a readable pattern on every scheme, and no two
/// schemes that admit one string.
///
/// The disjointness half is [`Template::disjoint`], which is exact over the
/// grammar spec 2 writes. It replaced a comparison of the text before each
/// pattern's first placeholder, which was sound and incomplete: it refused
/// `{namespace}-DR-{seq:04d}` beside `{namespace}-DR-{seq:06d}`, where no string
/// is both exactly four digits and exactly six. The parse that decides it was
/// already in this engine, one crate away, and this rule carrying a second and
/// weaker reading of one grammar is what
/// [#210](https://github.com/headwater-ai/headwater/issues/210) reports.
fn identifier_integrity(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "identifier integrity";
    let mut by_namespace: BTreeMap<&str, Vec<(&str, Template)>> = BTreeMap::new();
    for (scheme, body) in view.members("identifier_schemes") {
        // Three states and not two. The meta-schema lets a package omit
        // `namespace`, so an absent key means the choice is open and an empty
        // one means a consumer answered it with nothing. Collapsing them into
        // one `Option` would report a package defect against a consumer.
        let namespace = match text(body, "namespace") {
            None => {
                out.push(refusal(
                    RULE,
                    &format!("identifier_schemes.{scheme}"),
                    "carries no namespace after resolution. A package leaves the namespace to \
                     the corpus that adopts it, so an overlay of this corpus has to declare one"
                        .to_string(),
                ));
                continue;
            }
            Some("") => {
                out.push(refusal(
                    RULE,
                    &format!("identifier_schemes.{scheme}.namespace"),
                    "is the empty string, which names nobody. The namespace is the part of an \
                     identifier that tells this corpus's records from another corpus's"
                        .to_string(),
                ));
                continue;
            }
            Some(found) => found,
        };
        // A pattern this engine cannot read is a scheme it can say nothing
        // about: not what it admits, and so not whether another scheme admits
        // the same string. The generated check skips such a scheme and
        // `headwater new` refuses to mint under it, and this is the third
        // reader of the same fact, at the one moment where a taxonomy is
        // refused rather than reported around.
        let pattern = text(body, "pattern").unwrap_or_default();
        match Template::parse(pattern, namespace) {
            Ok(template) => by_namespace
                .entry(namespace)
                .or_default()
                .push((scheme, template)),
            Err(why) => out.push(refusal(
                RULE,
                &format!("identifier_schemes.{scheme}.pattern"),
                format!("cannot be read, so nothing can be said about what it admits: {why}"),
            )),
        }
    }
    // Grouping by namespace is a shortcut and never the rule. Two schemes in
    // two namespaces are disjoint because the namespace is a run of literal
    // characters in both patterns, which `disjoint` reads for itself; the
    // grouping only saves the comparison.
    for (namespace, schemes) in by_namespace {
        for (index, (scheme, template)) in schemes.iter().enumerate() {
            for (other, other_template) in &schemes[index + 1..] {
                if !template.disjoint(other_template) {
                    out.push(refusal(
                        RULE,
                        &format!("identifier_schemes.{scheme}.pattern"),
                        format!(
                            "and `identifier_schemes.{other}.pattern` are both in namespace \
                             `{namespace}` and one string satisfies both"
                        ),
                    ));
                }
            }
        }
    }
}

/// Every shelf reaches a kind, every concrete kind is reached, and an abstract
/// kind is reached by nothing.
fn coverage(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "coverage";
    for (shelf, body) in view.members("shelves") {
        if text(body, "kind").is_none() && strings(body, "kinds").is_empty() {
            out.push(refusal(
                RULE,
                &format!("shelves.{shelf}"),
                "resolves to no kind, so every document under it is unclassifiable".to_string(),
            ));
        }
    }
    let shelved = view.shelved_kinds();
    for (kind, _) in view.members("kinds") {
        let reached = shelved.get(kind);
        match (view.is_abstract(kind), reached) {
            (false, None) => out.push(refusal(
                RULE,
                &format!("kinds.{kind}"),
                "is concrete and no shelf reaches it, so no document can ever be one".to_string(),
            )),
            (true, Some(shelves)) => out.push(refusal(
                RULE,
                &format!("kinds.{kind}"),
                format!(
                    "is abstract and `shelves.{}` reaches it. No document resolves to an \
                     abstract kind",
                    shelves[0]
                ),
            )),
            _ => {}
        }
    }
}

/// `is_a` names a declared abstract kind, the chain terminates, and no child
/// weakens what a parent requires.
fn kind_inheritance(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "kind inheritance";
    for (kind, body) in view.members("kinds") {
        let Some(parent) = text(body, "is_a") else {
            continue;
        };
        let at = format!("kinds.{kind}.is_a");
        if view.kind(parent).is_none() {
            continue; // referential integrity reported it
        }
        if !view.is_abstract(parent) {
            out.push(refusal(
                RULE,
                &at,
                format!("names `{parent}`, which is concrete. A parent is an abstract kind"),
            ));
        }
        // The chain is walked with a seen set, so a cycle shortens it rather
        // than hanging. A chain that does not reach a root has a cycle in it.
        let chain = view.ancestry(kind);
        let top = chain.last().copied().unwrap_or(kind);
        if view.kind(top).and_then(|body| text(body, "is_a")).is_some() {
            out.push(refusal(
                RULE,
                &at,
                format!(
                    "is in a cycle: {}. Inheritance is a chain and it terminates",
                    chain.join(" is_a ")
                ),
            ));
            continue;
        }
        let required_above: BTreeSet<String> = chain[1..]
            .iter()
            .flat_map(|step| {
                view.kind(step)
                    .and_then(|body| block(body, "facets"))
                    .map(|contract| strings(contract, "require"))
                    .unwrap_or_default()
            })
            .collect();
        let Some(contract) = block(body, "facets") else {
            continue;
        };
        for facet in strings(contract, "forbid") {
            if required_above.contains(&facet) {
                out.push(refusal(
                    RULE,
                    &format!("kinds.{kind}.facets.forbid"),
                    format!(
                        "forbids `{facet}`, and a kind above it requires the same facet. \
                         A child may not void a contract that a reader of the parent trusts"
                    ),
                ));
            }
        }
    }
}

/// Every concrete kind has a purpose, and every purpose is served.
fn purpose_completeness(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "purpose completeness";
    let mut served: BTreeSet<String> = BTreeSet::new();
    for (kind, _) in view.members("kinds") {
        let purpose = view
            .ancestry(kind)
            .into_iter()
            .find_map(|step| view.kind(step).and_then(|body| text(body, "purpose")));
        match purpose {
            Some(purpose) if !view.is_abstract(kind) => {
                served.insert(purpose.to_string());
            }
            Some(_) => {}
            None if !view.is_abstract(kind) => out.push(refusal(
                RULE,
                &format!("kinds.{kind}"),
                "declares no purpose and inherits none. Every concrete kind serves a reader \
                 intent"
                    .to_string(),
            )),
            None => {}
        }
    }
    for (purpose, _) in view.members("purposes") {
        if !served.contains(purpose) {
            out.push(refusal(
                RULE,
                &format!("purposes.{purpose}"),
                "is declared and no concrete kind serves it, so no document can answer it"
                    .to_string(),
            ));
        }
    }
}

/// No two shelf patterns claim one path at equal specificity.
fn determinism(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "determinism";
    let shelves: Vec<(&str, Pattern)> = view
        .members("shelves")
        .into_iter()
        .filter_map(|(name, body)| text(body, "path").map(|path| (name, Pattern::new(path))))
        .collect();
    for (index, (shelf, pattern)) in shelves.iter().enumerate() {
        for (other, other_pattern) in &shelves[index + 1..] {
            if pattern.overlaps(other_pattern)
                && pattern.specificity() == other_pattern.specificity()
            {
                out.push(refusal(
                    RULE,
                    &format!("shelves.{shelf}.path"),
                    format!(
                        "`{}` and `shelves.{other}.path` `{}` can match one path and neither is \
                         more specific, so the kind of that path is a coin flip",
                        pattern.source(),
                        other_pattern.source()
                    ),
                ));
            }
        }
    }
}

/// At most one facet per engine role.
fn role_uniqueness(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "role uniqueness";
    let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();
    for (facet, body) in view.members("facets") {
        let Some(role) = text(body, "role") else {
            continue;
        };
        if !ROLES.contains(&role) {
            out.push(refusal(
                RULE,
                &format!("facets.{facet}.role"),
                format!(
                    "is `{role}`, which is outside the closed registry ({}). To add a role is a \
                     meta-schema change",
                    ROLES.join(", ")
                ),
            ));
            continue;
        }
        if let Some(other) = claimed.insert(role, facet) {
            out.push(refusal(
                RULE,
                &format!("facets.{facet}.role"),
                format!("claims `{role}`, and `facets.{other}` already claims it"),
            ));
        }
    }
}

/// The role the state vocabulary gives one value, for a message that names it.
///
/// An empty string where the value carries no role, because the one caller
/// reads this only on the arm where a `terminal-` role is what it found.
fn role_of(states: &[(String, Option<String>)], state: &str) -> String {
    states
        .iter()
        .find(|(value, _)| value == state)
        .and_then(|(_, role)| role.clone())
        .unwrap_or_default()
}

/// Connected, with an initial state, and one reading of which states end it.
fn lifecycle_soundness(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "lifecycle soundness";
    let states = view.states();
    let known: BTreeSet<&str> = states.iter().map(|(value, _)| value.as_str()).collect();
    let retained: BTreeSet<&str> = states
        .iter()
        .filter(|(_, role)| role.as_deref().is_some_and(crate::core::role_is_terminal))
        .map(|(value, _)| value.as_str())
        .collect();

    let Some(regimes) = view
        .root
        .get("regimes")
        .and_then(|node| node.value.as_map())
    else {
        return;
    };
    let Some(family) = regimes
        .get("lifecycle")
        .and_then(|node| node.value.as_map())
    else {
        return;
    };
    // Every state any regime of the family reaches, and the initial state each
    // one starts from. Both are read after the loop, because the question they
    // answer is about the family and not about one member of it.
    let mut anywhere: BTreeSet<String> = BTreeSet::new();
    let mut initials: Vec<String> = Vec::new();
    for entry in family {
        let regime = entry.key.value.as_str();
        let Some(body) = entry.value.value.as_map() else {
            continue;
        };
        let mut stranded: BTreeSet<&str> = BTreeSet::new();
        let at = |member: &str| format!("regimes.lifecycle.{regime}.{member}");
        let initial = text(body, "initial").unwrap_or_default();
        initials.push(format!("`{regime}` from `{initial}`"));
        if !known.is_empty() && !known.contains(initial) {
            out.push(refusal(
                RULE,
                &at("initial"),
                format!("is `{initial}`, which the state vocabulary does not hold"),
            ));
        }

        let transitions = block(body, "transitions").cloned().unwrap_or_default();
        let mut exits: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for entry in &transitions {
            let from = entry.key.value.clone();
            let targets: Vec<String> = entry
                .value
                .value
                .as_seq()
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .collect();
            for state in std::iter::once(&from).chain(targets.iter()) {
                if !known.is_empty() && !known.contains(state.as_str()) {
                    out.push(refusal(
                        RULE,
                        &at("transitions"),
                        format!("names `{state}`, which the state vocabulary does not hold"),
                    ));
                }
            }
            exits.insert(from, targets);
        }

        // Connected, and the question is asked twice because two different
        // things can be broken.
        //
        // Within one regime: a state this regime *names* and cannot reach from
        // its own initial state is a machine with a piece that does not join
        // it. That is a defect of this declaration wherever else the state may
        // be reachable.
        //
        // Across the family: a state of the vocabulary that no regime reaches
        // is a state no document can ever be in, which is a declaration that
        // reads as a promise. That reading used to be taken per regime, and it
        // made two regimes over one vocabulary undeclarable — the moment a
        // second regime carried the extra state, the first failed for not
        // carrying it. Per regime is right while one regime exists, and a
        // taxonomy that narrows a state set per kind declares more than one.
        // [#219](https://github.com/headwater-ai/headwater/issues/219).
        let mut reached: BTreeSet<String> = BTreeSet::new();
        let mut frontier = vec![initial.to_string()];
        while let Some(state) = frontier.pop() {
            if !reached.insert(state.clone()) {
                continue;
            }
            frontier.extend(exits.get(&state).cloned().unwrap_or_default());
        }
        for named in exits
            .iter()
            .flat_map(|(from, targets)| std::iter::once(from).chain(targets.iter()))
        {
            if !reached.contains(named) && !stranded.contains(named.as_str()) {
                stranded.insert(named.as_str());
                out.push(refusal(
                    RULE,
                    &at("transitions"),
                    format!(
                        "names `{named}` and does not reach it from `{initial}`, so this regime \
                         holds a state its own documents cannot get to"
                    ),
                ));
            }
        }
        anywhere.extend(reached.iter().cloned());

        // Terminality, said once. The `terminal-` role on the vocabulary value
        // is the declaration and the machine is the derivation, and this engine
        // reads both: `lifecycle.dependency.on_terminal` asks the role, and
        // `lifecycle.deletion.not_permitted` and `lifecycle.transition
        // .not_permitted` ask the machine. A state a regime reaches is
        // therefore terminal to both readings or to neither, and each half of
        // that is refused here.
        //
        // The `terminal` member this rule once read was the third way to say
        // it. It reached no reader but the escape on the first half below, so a
        // state it named was terminal to the machine and not to the role, and
        // the two rules above then answered differently about one state.
        for state in &reached {
            let leaves = exits.get(state).is_some_and(|targets| !targets.is_empty());
            let terminal = retained.contains(state.as_str());
            if leaves != terminal {
                continue;
            }
            let message = match leaves {
                true => format!(
                    "gives `{state}` an exit, and the `{}` role on its value names it terminal. \
                     One rule reads the role and another reads the machine, so a state is \
                     terminal to both or to neither",
                    role_of(&states, state)
                ),
                false => format!(
                    "gives `{state}` no exit and nothing declares it terminal. A terminal state \
                     is named by a `terminal-` role on its value"
                ),
            };
            out.push(refusal(RULE, &at("transitions"), message));
        }
    }

    // The family-level half. It reports at `regimes.lifecycle` rather than at
    // one regime, because no single regime is the one that should have carried
    // the state, and a report that picked one would name a file the author has
    // no reason to open.
    let tried = initials.join(", ");
    for (state, _) in &states {
        if !anywhere.contains(state) {
            out.push(refusal(
                RULE,
                "regimes.lifecycle",
                format!(
                    "leaves `{state}` unreachable in every lifecycle regime ({tried}), so no \
                     document of any kind reaches it"
                ),
            ));
        }
    }
}

/// Nuclearity, its nucleus, and the facets that `inherits` names.
fn relation_coherence(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "relation coherence";
    for (relation, body) in view.members("relations") {
        let at = |member: &str| format!("relations.{relation}.{member}");
        let family = text(body, "family").unwrap_or_default();
        let default = FAMILY_NUCLEARITY
            .iter()
            .find(|(name, _)| *name == family)
            .map(|(_, nuclearity)| *nuclearity);
        let nuclearity = text(body, "nuclearity").or(default);
        match (nuclearity, text(body, "nucleus")) {
            (Some("nucleus-satellite"), None) => out.push(refusal(
                RULE,
                &at("nucleus"),
                format!(
                    "is absent, and this relation is nucleus–satellite{}. The family cannot \
                     supply which end is the nucleus, because that is the direction its author \
                     chose",
                    if text(body, "nuclearity").is_none() {
                        format!(" by the default of the `{family}` family")
                    } else {
                        String::new()
                    }
                ),
            )),
            (Some("multinuclear"), Some(end)) => out.push(refusal(
                RULE,
                &at("nucleus"),
                format!(
                    "names `{end}`, and a multinuclear relation has no nucleus. Both ends \
                         stand alone"
                ),
            )),
            _ => {}
        }

        // `inherits` names facets that exist on both ends.
        let inherits = strings(body, "inherits");
        if inherits.is_empty() {
            continue;
        }
        for end in ["from", "to"] {
            for kind in strings(body, end) {
                if view.kind(&kind).is_none() {
                    continue; // an anchor kind, or already reported
                }
                let carried = view.facets_of(&kind);
                for facet in &inherits {
                    if !carried.contains(facet) {
                        out.push(refusal(
                            RULE,
                            &at("inherits"),
                            format!(
                                "names `{facet}`, and `kinds.{kind}` at the `{end}` end does not \
                                 carry it. A satellite inherits a facet that both ends have"
                            ),
                        ));
                    }
                }
            }
        }
    }
}

/// An expectation names endpoints the relation admits, a reachable target, and
/// an origin facet that the declaring kind requires.
fn expectations(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "expectation well-formedness";
    let shelved = view.shelved_kinds();
    for (kind, expectation) in expectations_of(view) {
        let at = format!("kinds.{kind}.relations.expect.{}", id_of(expectation));
        let target = text(expectation, "to_kind").unwrap_or_default();
        if view.kind(target).is_some() && !shelved.contains_key(target) {
            out.push(refusal(
                RULE,
                &format!("{at}.to_kind"),
                format!(
                    "names `{target}`, which no shelf reaches. An expectation over a kind that \
                     no document can be is a finding that can never clear"
                ),
            ));
        }

        // The named relation may be an inverse, and then the ends are swapped.
        if let Some(named) = text(expectation, "relation") {
            if let Some((from, to)) = endpoints(view, named) {
                if !admits(view, &from, kind) {
                    out.push(refusal(
                        RULE,
                        &format!("{at}.relation"),
                        format!(
                            "names `{named}`, and `kinds.{kind}` is not at its source end. \
                             An expectation is declared on the kind that writes the edge"
                        ),
                    ));
                }
                if !target.is_empty() && view.kind(target).is_some() && !admits(view, &to, target) {
                    out.push(refusal(
                        RULE,
                        &format!("{at}.to_kind"),
                        format!(
                            "names `{target}`, and `{named}` does not admit it at its target end"
                        ),
                    ));
                }
            }
        }

        // The origin facet has to be required on the kind that declares the
        // expectation, or the window silently never starts.
        let since = text(expectation, "since").unwrap_or_default();
        if let Some(facet) = view.facet_with_role(since) {
            if !view.required_facets_of(kind).contains(facet) {
                out.push(refusal(
                    RULE,
                    &format!("{at}.since"),
                    format!(
                        "is `{since}`, whose facet is `{facet}`, and `kinds.{kind}` does not \
                         require it. A window measured from a facet that a document may omit \
                         never starts"
                    ),
                ));
            }
        } else if !since.is_empty() {
            out.push(refusal(
                RULE,
                &format!("{at}.since"),
                format!("is `{since}`, and no facet carries that role"),
            ));
        }

        // A state condition names a facet and a value that facet holds.
        for (facet, value) in conditions(expectation) {
            let declared = view
                .members("facets")
                .into_iter()
                .find(|(name, _)| *name == facet);
            let Some((_, body)) = declared else {
                out.push(refusal(
                    RULE,
                    &format!("{at}.when"),
                    format!("names `{facet}`, and no facet of that name is declared"),
                ));
                continue;
            };
            let values = value_set(body);
            if !values.is_empty() && !values.contains(&value) {
                out.push(refusal(
                    RULE,
                    &format!("{at}.when"),
                    format!("expects `{facet}` to be `{value}`, which is not one of its values"),
                ));
            }
        }
    }
}

/// Relevance, ascertainability and permanence. Spec 2 states that the three are
/// decidable from the schema alone, and gives differentiation and orthogonality
/// to `taxonomy audit`.
fn facet_canons(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "facet canons";
    let read = facets_read(view);
    for (facet, body) in view.members("facets") {
        let at = |member: &str| format!("facets.{facet}.{member}");

        if !read.contains(facet) {
            out.push(refusal(
                RULE,
                &format!("facets.{facet}"),
                "relevance: nothing reads it. No kind requires it, no shelf discriminates on it, \
                 no expectation reads it, no projection filters on it, and it carries no engine \
                 role"
                    .to_string(),
            ));
        }

        let values = value_set(body);
        if !values.is_empty() {
            let guidance = block(body, "guidance").cloned().unwrap_or_default();
            for value in &values {
                if guidance.get(value).is_none() {
                    out.push(refusal(
                        RULE,
                        &at("guidance"),
                        format!(
                            "ascertainability: `{value}` carries no guidance, so nothing states \
                             when an author picks it"
                        ),
                    ));
                }
            }
        }

        match text(body, "volatility") {
            None => out.push(refusal(
                RULE,
                &format!("facets.{facet}"),
                "permanence: declares no `volatility`, so nothing says whether a value of it may \
                 be used where a rename is expensive"
                    .to_string(),
            )),
            Some("mutable") => {
                for (at, text) in mutable_positions(view) {
                    if placeholders(&text).contains(facet) {
                        out.push(refusal(
                            RULE,
                            &at,
                            format!(
                                "permanence: reads `{facet}`, which is `mutable`. A value that \
                                 changes may not appear in an identifier, a path or a shelf \
                                 pattern, because every reference to it dies at the change"
                            ),
                        ));
                    }
                }
            }
            Some(_) => {}
        }
    }
}

/// No kind is a state, and none is a bare phase adjective.
fn kind_rigidity(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "kind rigidity";
    let states: BTreeSet<String> = view.states().into_iter().map(|(value, _)| value).collect();
    for (kind, _) in view.members("kinds") {
        if states.contains(kind) {
            out.push(refusal(
                RULE,
                &format!("kinds.{kind}"),
                "collides with a value of the state vocabulary. A kind is rigid and a state is \
                 not, so a document would change identity as it matured"
                    .to_string(),
            ));
        }
        if PHASE_ADJECTIVES.contains(&kind) {
            out.push(refusal(
                RULE,
                &format!("kinds.{kind}"),
                "is a bare phase adjective, which names a state rather than a species of \
                 document"
                    .to_string(),
            ));
        }
    }
}

/// An instance attribute names a value space and an owning end.
fn attributes(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "attribute well-formedness";
    for (relation, body) in view.members("relations") {
        let Some(declared) = block(body, "attributes") else {
            continue;
        };
        for entry in declared {
            let name = entry.key.value.as_str();
            let at = format!("relations.{relation}.attributes.{name}");
            if name == "to" {
                out.push(refusal(
                    RULE,
                    &at,
                    "is called `to`, which is the key that names the other end of the edge"
                        .to_string(),
                ));
            }
            let Some(attribute) = entry.value.value.as_map() else {
                continue;
            };
            // The owning end is a required member of the shape, so the
            // meta-schema decides it over every source and over the merged
            // result. The value space is not, so it is decided here.
            if text(attribute, "type").is_none() && strings(attribute, "values").is_empty() {
                out.push(refusal(
                    RULE,
                    &at,
                    "names no value space. An attribute takes a type or a value set, and it \
                     takes it from the facet set rather than inventing one"
                        .to_string(),
                ));
            }
        }
    }
}

/// A transcription pins an anchor kind that names one resolver.
fn warrant_integrity(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "warrant integrity";
    for (index, body) in view.items("projections").into_iter().enumerate() {
        if text(body, "kind") != Some("transcription") {
            continue;
        }
        let at = format!("projections.{index}");
        let Some(anchor) = block(body, "from").and_then(|node| text(node, "anchor")) else {
            out.push(refusal(
                RULE,
                &at,
                "is a transcription and pins nothing. A transcribed document carries the \
                 identity of the snapshot it copies"
                    .to_string(),
            ));
            continue;
        };
        let declared = view
            .members("anchors")
            .into_iter()
            .find(|(name, _)| *name == anchor);
        if declared.is_some_and(|(_, body)| text(body, "resolver").is_none()) {
            out.push(refusal(
                RULE,
                &format!("{at}.from.anchor"),
                format!("pins `{anchor}`, which names no resolver, so the pin resolves nowhere"),
            ));
        }
    }
}

/// The half of context safety that a declaration exists for.
fn context_safety(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "context safety";
    let Some(facet) = view.facet_with_role("freshness") else {
        return;
    };
    let declared = view
        .members("facets")
        .into_iter()
        .find(|(name, _)| *name == facet)
        .map(|(_, body)| text(body, "stale_after_days").is_some());
    if declared == Some(false) {
        out.push(refusal(
            RULE,
            &format!("facets.{facet}"),
            "carries the freshness role and declares no staleness policy, so nothing decides \
             when a verification has expired"
                .to_string(),
        ));
    }
}

/// The half of a mapping that this taxonomy can decide.
fn mapping_integrity(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "mapping integrity";
    let kinds = view.names("kinds");
    for (index, body) in view.items("mappings").into_iter().enumerate() {
        let at = |member: &str| format!("mappings.{index}.{member}");
        if let Some(named) = block(body, "kinds") {
            for entry in named {
                let name = entry.key.value.as_str();
                if !kinds.contains(name) {
                    out.push(refusal(
                        RULE,
                        &at("kinds"),
                        format!("maps `{name}`, and no kind of that name is declared here"),
                    ));
                }
            }
        }
        if let Some(named) = block(body, "facet_values") {
            for entry in named {
                let key = entry.key.value.as_str();
                let Some((facet, value)) = key.split_once('.') else {
                    out.push(refusal(
                        RULE,
                        &at("facet_values"),
                        format!(
                            "keys `{key}`, and the key is a facet and one of its values \
                                 joined by a dot"
                        ),
                    ));
                    continue;
                };
                let declared = view
                    .members("facets")
                    .into_iter()
                    .find(|(name, _)| *name == facet);
                match declared {
                    None => out.push(refusal(
                        RULE,
                        &at("facet_values"),
                        format!("maps `{facet}`, and no facet of that name is declared here"),
                    )),
                    Some((_, body)) => {
                        let values = value_set(body);
                        if !values.is_empty() && !values.iter().any(|held| held == value) {
                            out.push(refusal(
                                RULE,
                                &at("facet_values"),
                                format!(
                                    "maps `{facet}.{value}`, and `{facet}` holds no such value"
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// A projection writes a relative path inside the corpus.
fn projection_targets(view: &View, out: &mut Vec<ResolveError>) {
    const RULE: &str = "projection targets";
    for (index, body) in view.items("projections").into_iter().enumerate() {
        let Some(output) = text(body, "output") else {
            continue;
        };
        let at = format!("projections.{index}.output");
        if output.starts_with('/') {
            out.push(refusal(
                RULE,
                &at,
                format!("is `{output}`, which is absolute. A projection writes inside the corpus"),
            ));
        }
        if output.split('/').any(|segment| segment == "..") {
            out.push(refusal(
                RULE,
                &at,
                format!("is `{output}`, which climbs above the corpus root"),
            ));
        }
    }
}

// --- shared readings --------------------------------------------------------

/// Every facet that something in the taxonomy reads. This is the relevance
/// canon's denominator, and it is deliberately generous: a facet that any
/// declaration touches is read, because the canon asks whether the facet does
/// work and not whether a check exists for it yet.
fn facets_read(view: &View) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (facet, body) in view.members("facets") {
        if text(body, "role").is_some() {
            out.insert(facet.to_string());
        }
    }
    for (kind, _) in view.members("kinds") {
        out.extend(view.facets_of(kind));
        if let Some(contract) = view.kind(kind).and_then(|body| block(body, "facets")) {
            out.extend(strings(contract, "forbid"));
        }
    }
    for (_, body) in view.members("shelves") {
        for member in ["discriminator", "group_by"] {
            if let Some(facet) = text(body, member) {
                out.insert(facet.to_string());
            }
        }
        for member in ["path", "layout"] {
            if let Some(text) = text(body, member) {
                out.extend(placeholders(text));
            }
        }
    }
    for (_, body) in view.members("relations") {
        out.extend(strings(body, "inherits"));
        if let Some(condition) = block(body, "invalid_when").and_then(|node| block(node, "both")) {
            out.extend(condition.iter().map(|entry| entry.key.value.clone()));
        }
    }
    for (_, expectation) in expectations_of(view) {
        out.extend(conditions(expectation).into_iter().map(|(facet, _)| facet));
    }
    for body in view.items("projections") {
        if let Some(filter) = block(body, "filter") {
            for member in ["include", "exclude"] {
                if let Some(set) = block(filter, member) {
                    out.extend(set.iter().map(|entry| entry.key.value.clone()));
                }
            }
        }
        if let Some(output) = text(body, "output") {
            out.extend(placeholders(output));
        }
    }
    out
}

/// Every position where a facet value would be written into a name that
/// something else points at, with the address that declares it.
fn mutable_positions(view: &View) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (scheme, body) in view.members("identifier_schemes") {
        if let Some(pattern) = text(body, "pattern") {
            out.push((
                format!("identifier_schemes.{scheme}.pattern"),
                pattern.to_string(),
            ));
        }
    }
    for (shelf, body) in view.members("shelves") {
        for member in ["path", "layout"] {
            if let Some(text) = text(body, member) {
                out.push((format!("shelves.{shelf}.{member}"), text.to_string()));
            }
        }
    }
    out
}

/// Every `{name}` in a pattern, with any format specifier dropped, because
/// `{sequence:02d}` reads the facet `sequence`.
fn placeholders(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        let inner = &rest[open + 1..open + close];
        let name = inner.split(':').next().unwrap_or(inner);
        if !name.is_empty() {
            out.insert(name.to_string());
        }
        rest = &rest[open + close + 1..];
    }
    out
}

/// Every declared value of a facet, whether the value set was written out or
/// arrived through a resolved reference.
fn value_set(facet: &Mapping) -> Vec<String> {
    let Some(values) = facet.get("values").and_then(|node| node.value.as_seq()) else {
        return Vec::new();
    };
    values
        .iter()
        .filter_map(|item| match &item.value {
            Value::Scalar(scalar) => Some(scalar.text.clone()),
            Value::Map(map) => text(map, "value").map(str::to_string),
            Value::Seq(_) => None,
        })
        .collect()
}

/// Every expectation in the taxonomy, with the kind that declares it.
fn expectations_of<'a>(view: &View<'a>) -> Vec<(&'a str, &'a Mapping)> {
    let mut out = Vec::new();
    for (kind, body) in view.members("kinds") {
        let Some(expect) = block(body, "relations")
            .and_then(|node| node.get("expect"))
            .and_then(|node| node.value.as_seq())
        else {
            continue;
        };
        for item in expect {
            if let Some(map) = item.value.as_map() {
                out.push((kind, map));
            }
        }
    }
    out
}

/// The `id` of an expectation, which is what a message calls it by.
fn id_of(expectation: &Mapping) -> String {
    text(expectation, "id").unwrap_or("<unnamed>").to_string()
}

/// The `when` condition of an expectation, as facet and value pairs.
fn conditions(expectation: &Mapping) -> Vec<(String, String)> {
    let Some(when) = block(expectation, "when") else {
        return Vec::new();
    };
    when.iter()
        .filter_map(|entry| {
            entry
                .value
                .value
                .as_scalar()
                .map(|scalar| (entry.key.value.clone(), scalar.text.clone()))
        })
        .collect()
}

/// The source and target ends of a relation named forward or by its inverse.
fn endpoints(view: &View, named: &str) -> Option<(Vec<String>, Vec<String>)> {
    for (relation, body) in view.members("relations") {
        let from = strings(body, "from");
        let to = strings(body, "to");
        if relation == named {
            return Some((from, to));
        }
        if text(body, "inverse") == Some(named) {
            return Some((to, from));
        }
    }
    None
}

/// Whether an endpoint list admits a kind, through the inheritance chain.
fn admits(view: &View, end: &[String], kind: &str) -> bool {
    view.ancestry(kind)
        .into_iter()
        .any(|step| end.iter().any(|admitted| admitted == step))
}

fn core_requirements<'a>(view: &View<'a>) -> Vec<&'a Mapping> {
    view.root
        .get("core")
        .and_then(|node| node.value.as_map())
        .and_then(|core| core.get("requires"))
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_map())
                .collect()
        })
        .unwrap_or_default()
}

fn block<'a>(map: &'a Mapping, key: &str) -> Option<&'a Mapping> {
    map.get(key).and_then(|node| node.value.as_map())
}

fn text<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.as_str())
}

fn strings(map: &Mapping, key: &str) -> Vec<String> {
    map.get(key)
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn refusal(rule: &'static str, at: &str, message: String) -> ResolveError {
    ResolveError::new(
        ResolveErrorKind::Invalid { rule, message },
        "the resolved taxonomy",
        at,
        Span::default(),
    )
}
