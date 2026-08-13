// SPDX-License-Identifier: Apache-2.0
//! `headwater export`: the emitters, the loss set, and the projection census.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)
//! settles the shape of this module in three sentences. "A graph export is a
//! projection like the others." "Every emitter declares a **loss set**: the node
//! classes, edge classes, and attributes that its target cannot carry, each with
//! a reason." "Every node and every edge in the graph is either present in the
//! output, or accounted for by a declared loss reason. An omission that no
//! reason covers is a projector defect, and it fails the run."
//!
//! # Why the census is the whole point
//!
//! The [SHACL evaluation](../../../../docs/evaluations/shacl-worked-example.md)
//! found that the projector was the component everything downstream trusted and
//! nothing could check. A round trip catches a lossless emitter that dropped
//! something. It catches nothing at all in a lossy one, because a lossy emitter
//! is *supposed* to drop things, and an emitter that drops one node too many
//! looks exactly like an emitter working correctly. The census is what closes
//! that: the emitter states in advance which classes it cannot carry, and the
//! engine then holds every node and every edge against that statement. A drop
//! the loss set did not predict fails the run.
//!
//! So the loss set is a claim and the census is the audit of it, and neither
//! alone is worth much. That is the coverage doctrine of
//! [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! applied one layer out, which is what spec 6 says it is.
//!
//! # The denominator is the corpus, not the identifier index
//!
//! A typed document with no identifier is not a node of
//! [`headwater_graph::Index`], and it is still content that leaves a corpus or
//! does not. Counting only indexed nodes would let an emitter drop every
//! unidentified document and report a complete census, which is the silent pass
//! this module exists to remove. So the node set here is every classified
//! document plus every external anchor the edges reach, and the identifier is a
//! member of a node rather than the thing that makes one.
//!
//! # Every scalar leaves as a string, and that is what makes the round trip
//! # exact
//!
//! JSON has numbers and YAML has a core schema that resolves them
//! ([Q2](../../../../docs/spec/09-decisions.md#q2--schema-format)). An emitter
//! that resolved a scalar into a JSON number would export `1.10` as `1.1`, and
//! the native export claims no loss. The text a document wrote is what travels,
//! and a consumer that wants a type reads the facet declaration that the same
//! export carries. A vocabulary is a better place to learn that a value is a
//! date than a guess by a serializer is.

use crate::profile::{Admission, Emitter, Grain, Profile};
use crate::Kind;
use headwater_graph::{Edge, Graph, Target};
use headwater_query::json::Json;
use headwater_query::{Document, Surface};
use headwater_yaml::value::Value;

/// The export format's own version, as `Major.Minor`.
///
/// The rule the corpus descriptor states about its version holds here for the
/// same reason: a version with no client behavior attached is a string. A major
/// above what a reader understands is a hard failure, a minor mismatch is a
/// warning, and a member added later moves the minor.
pub const VERSION: &str = "1.0";

/// What a loss is about.
///
/// The three classes spec 6 names, and no fourth. An attribute is the class that
/// is easy to forget, and it is where a cue and an edge's instance data live.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Node,
    Edge,
    Attribute,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Node => "node",
            Class::Edge => "edge",
            Class::Attribute => "attribute",
        }
    }
}

/// One entry of an emitter's loss set: a class its target cannot carry, and the
/// reason.
#[derive(Clone, Debug)]
pub struct Loss {
    pub class: Class,
    /// The class as the taxonomy names it: a kind, an anchor kind, a relation,
    /// or an attribute key.
    pub name: String,
    pub reason: String,
}

/// One class the output does not carry, with the count the census measured.
#[derive(Clone, Debug)]
pub struct Accounted {
    pub class: Class,
    pub name: String,
    pub reason: String,
    pub count: usize,
}

/// One node or edge the output does not carry and no reason covers.
///
/// This is the projector defect. It has a class and a name so that a reader can
/// find the emitter arm that dropped it, and it names the instance so that a
/// reader can reproduce it.
#[derive(Clone, Debug)]
pub struct Unaccounted {
    pub class: Class,
    pub name: String,
    pub at: String,
}

/// How many of one thing the graph held, and what became of each.
#[derive(Clone, Copy, Debug, Default)]
pub struct Tally {
    pub in_graph: usize,
    pub carried: usize,
    pub accounted: usize,
    pub unaccounted: usize,
}

/// The projection census: every node and every edge, carried or accounted for.
#[derive(Clone, Debug, Default)]
pub struct Census {
    pub nodes: Tally,
    pub edges: Tally,
    pub accounted: Vec<Accounted>,
    pub unaccounted: Vec<Unaccounted>,
}

impl Census {
    /// Whether this census fails the run.
    ///
    /// One rule, and it is spec 6's: an omission that no reason covers is a
    /// projector defect.
    pub fn is_defective(&self) -> bool {
        !self.unaccounted.is_empty()
    }
}

/// An export, and the census that says it is honest.
#[derive(Clone, Debug)]
pub struct Emission {
    pub bytes: String,
    pub census: Census,
}

/// Why an export produced nothing.
///
/// An exporter fails closed ([spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-profile-carries-a-filter)):
/// "An exporter that cannot evaluate its filter emits nothing and fails the run.
/// It never emits an unfiltered artifact, and it never emits a partly filtered
/// one."
#[derive(Clone, Debug)]
pub enum Refusal {
    /// The target is one of spec 6's seven and this engine does not emit it.
    NotBuilt(Emitter),
    /// A filter clause names a facet this taxonomy does not declare, so the
    /// exporter cannot tell whether a document satisfies it.
    Unevaluable { facet: String, clause: String },
}

impl Refusal {
    pub fn reason(&self) -> String {
        match self {
            Refusal::NotBuilt(emitter) => format!(
                "the `{}` emitter waits on a named external consumer. Q13 stages the six \
                 emitters and puts every one after the second behind a consumer who asks for \
                 it, and none has. `json` and `jsonschema` ship",
                emitter.name()
            ),
            Refusal::Unevaluable { facet, clause } => format!(
                "the filter clause `{clause}` names the facet `{facet}`, and this taxonomy \
                 declares no such facet. An exporter that cannot evaluate its filter emits \
                 nothing: a clause over a facet that does not exist withholds nothing, and an \
                 unfiltered artifact for a filtered audience is the one failure this verb may \
                 not have"
            ),
        }
    }
}

/// A node of the graph, as the census counts them.
struct Node<'a> {
    /// The key that distinguishes this node from every other.
    key: String,
    /// The class: a kind for a document, an anchor kind for an anchor.
    class: String,
    document: Option<Document<'a>>,
}

/// Every node the census holds against the output.
fn nodes<'a>(surface: &Surface<'a>, graph: &Graph) -> Vec<Node<'a>> {
    let mut out: Vec<Node<'a>> = surface
        .documents()
        .into_iter()
        .map(|document| Node {
            key: document.path.to_string(),
            class: document.kind.to_string(),
            document: Some(document),
        })
        .collect();
    for anchor in graph.anchor_nodes() {
        out.push(Node {
            key: format!("{}:{}", anchor.anchor_kind, anchor.normalized),
            class: anchor.anchor_kind.clone(),
            document: None,
        });
    }
    out
}

/// What one emitter produced, before the census and the envelope reach it.
struct Body {
    value: Vec<(String, Json)>,
    /// The key of every node the output carries.
    carried_nodes: Vec<String>,
    /// The index in `graph.edges` of every edge the output carries.
    carried_edges: Vec<usize>,
    losses: Vec<Loss>,
}

/// Emit one profile through one target.
///
/// The order is fixed and it matters. The filter runs first, because a
/// withholding is a loss reason and an emitter must never see a document the
/// profile withholds. The emitter runs second and reports what it carried. The
/// census runs last, over the graph rather than over the emitter's own account
/// of itself.
pub fn emit(
    surface: &Surface<'_>,
    profile: &Profile,
    emitter: Emitter,
    generated_at: Option<&str>,
) -> Result<Emission, Refusal> {
    if !emitter.is_built() {
        return Err(Refusal::NotBuilt(emitter));
    }
    let graph = surface.graph();
    let withheld = withhold(surface, profile)?;
    let all = nodes(surface, graph);

    let body = match emitter {
        Emitter::JsonSchema => schema(surface, &all),
        _ => native(surface, graph, &all, &withheld),
    };

    let census = audit(
        surface,
        &withheld,
        &body.carried_nodes,
        &body.carried_edges,
        &body.losses,
    );
    let value = envelope(profile, emitter, generated_at, &withheld, &census, body);
    Ok(Emission {
        bytes: value.render_pretty(),
        census,
    })
}

/// What the profile's filter withheld, by node key, with the rule that withheld
/// each one.
///
/// An empty filter withholds nothing and the map is empty, which is the first
/// release's one profile. An anchor is never withheld here: a filter is over
/// facet values, an anchor states no facets, and a filter that reached one would
/// be filtering on a document that the corpus does not hold.
fn withhold(surface: &Surface<'_>, profile: &Profile) -> Result<Vec<(String, String)>, Refusal> {
    if profile.filter.is_empty() {
        return Ok(Vec::new());
    }
    let shape = surface.shape();
    for (direction, clauses) in [
        ("include", &profile.filter.include),
        ("exclude", &profile.filter.exclude),
    ] {
        for clause in clauses {
            if shape.facet(&clause.facet).is_none() {
                return Err(Refusal::Unevaluable {
                    facet: clause.facet.clone(),
                    clause: format!("{}.{direction}.{}", profile.name, clause.facet),
                });
            }
        }
    }
    let mut out = Vec::new();
    for document in surface.documents() {
        if let Admission::Withheld(rule) = profile.filter.admits(&profile.name, document.facets) {
            out.push((document.path.to_string(), rule));
        }
    }
    Ok(out)
}

fn rule_for(withheld: &[(String, String)], key: &str) -> Option<String> {
    withheld
        .iter()
        .find(|(path, _)| path == key)
        .map(|(_, rule)| rule.clone())
}

/// The native export: the property graph, with no loss.
///
/// [Q13](../../../../docs/spec/09-decisions.md#q13--linkml-and-shacl-as-substrate)
/// puts it second in the staging order and gives the reason it ships without an
/// external consumer: the solution corpus of Q9 reads it, and so does any local
/// tool. Its loss set is empty, and the round-trip test is what proves that
/// rather than asserts it.
fn native(
    surface: &Surface<'_>,
    graph: &Graph,
    all: &[Node<'_>],
    withheld: &[(String, String)],
) -> Body {
    let mut documents = Vec::new();
    let mut carried_nodes = Vec::new();
    for node in all {
        let Some(document) = &node.document else {
            continue;
        };
        if rule_for(withheld, &node.key).is_some() {
            continue;
        }
        let mut members = vec![
            ("path".to_string(), Json::string(document.path)),
            ("kind".to_string(), Json::string(document.kind)),
        ];
        if let Some(id) = document.id {
            members.push(("id".to_string(), Json::string(id)));
        }
        // Spec 6: "Every node carries a warrant, and the native export carries
        // it with no loss." It is a member of the provenance block, and it
        // travels twice for a reason: once inside `facets` as the document
        // wrote it, and once here where a consumer that reads no taxonomy still
        // finds it. A tier that harvests this file decides what to trust from
        // this member.
        if let Some(warrant) = surface.warrant(document) {
            members.push(("warrant".to_string(), Json::string(warrant)));
        }
        members.push(("facets".to_string(), of_value_map(document.facets)));
        carried_nodes.push(node.key.clone());
        documents.push(Json::Object(members));
    }

    let mut anchors = Vec::new();
    for anchor in graph.anchor_nodes() {
        let key = format!("{}:{}", anchor.anchor_kind, anchor.normalized);
        let mut members = vec![
            ("anchor_kind".to_string(), Json::string(&anchor.anchor_kind)),
            ("id".to_string(), Json::string(&anchor.normalized)),
            ("resolver".to_string(), Json::string(&anchor.resolver)),
        ];
        if let Some(pattern) = &anchor.excluded_by {
            members.push(("excluded_by".to_string(), Json::string(pattern)));
        }
        carried_nodes.push(key);
        anchors.push(Json::Object(members));
    }

    let mut edges = Vec::new();
    let mut carried_edges = Vec::new();
    for (at, edge) in graph.edges.iter().enumerate() {
        if withheld_edge(edge, withheld).is_some() {
            continue;
        }
        carried_edges.push(at);
        edges.push(of_edge(edge));
    }

    Body {
        value: vec![(
            "graph".to_string(),
            Json::Object(vec![
                ("documents".to_string(), Json::Array(documents)),
                ("anchors".to_string(), Json::Array(anchors)),
                ("edges".to_string(), Json::Array(edges)),
            ]),
        )],
        carried_nodes,
        carried_edges,
        // Spec 6: "The native graph export carries the property graph with no
        // loss, and that includes the instance attributes on edges." An empty
        // loss set is a claim, and the census below is what audits it: a node
        // or an edge this emitter dropped has no reason to fall under and fails
        // the run.
        losses: Vec::new(),
    }
}

/// The rule that withheld an edge, which is the rule that withheld either end.
///
/// Spec 6's claim about a filtered export is that it "contains no document that
/// its declared filter withholds, and no artifact inside the profile derives
/// from one". An edge names both of its ends, so an edge into a withheld
/// document states that the document exists and what it is called.
fn withheld_edge(edge: &Edge, withheld: &[(String, String)]) -> Option<String> {
    if let Some(rule) = rule_for(withheld, &edge.source.path) {
        return Some(rule);
    }
    match &edge.target {
        Target::Document { path, .. } => rule_for(withheld, path),
        _ => None,
    }
}

fn of_edge(edge: &Edge) -> Json {
    let mut members = vec![
        ("source".to_string(), Json::string(&edge.source.path)),
        ("relation".to_string(), Json::string(&edge.declared)),
        // The name as the author wrote it, which may be the inverse of the
        // relation it resolved to. Both travel: a consumer that re-renders the
        // corpus needs the written name, and one that reasons over relations
        // needs the canonical one.
        ("written_as".to_string(), Json::string(&edge.name)),
        ("target".to_string(), of_target(&edge.target)),
    ];
    if !edge.attributes.is_empty() {
        members.push((
            "attributes".to_string(),
            Json::Object(
                edge.attributes
                    .iter()
                    .map(|entry| (entry.key.value.clone(), of_value(&entry.value.value)))
                    .collect(),
            ),
        ));
    }
    Json::Object(members)
}

fn of_target(target: &Target) -> Json {
    match target {
        Target::Document { id, path, kind } => Json::object([
            ("bound", Json::string("document")),
            ("id", Json::string(id.as_str())),
            ("path", Json::string(path.as_str())),
            ("kind", Json::string(kind.as_str())),
        ]),
        Target::Anchor {
            anchor_kind,
            resolver,
            normalized,
            ..
        } => Json::object([
            ("bound", Json::string("anchor")),
            ("anchor_kind", Json::string(anchor_kind.as_str())),
            ("id", Json::string(normalized.as_str())),
            ("resolver", Json::string(resolver.as_str())),
        ]),
        Target::Withheld {
            anchor_kind,
            profile,
        } => Json::object([
            ("bound", Json::string("withheld")),
            ("anchor_kind", Json::string(anchor_kind.as_str())),
            ("rule", Json::string(profile.as_str())),
        ]),
        // An unbound target is a fact about the corpus and not a defect of this
        // emitter. It travels, because a consumer that reads an export and finds
        // no unbound edge would conclude that every relation resolved.
        Target::Unbound(unbound) => Json::object([
            ("bound", Json::string("nothing")),
            ("reason", Json::string(unbound.to_string())),
        ]),
    }
}

fn of_value_map(map: &headwater_yaml::Mapping) -> Json {
    Json::Object(
        map.iter()
            .map(|entry| (entry.key.value.clone(), of_value(&entry.value.value)))
            .collect(),
    )
}

fn of_value(value: &Value) -> Json {
    match value {
        Value::Scalar(scalar) => Json::string(scalar.text.as_str()),
        Value::Seq(items) => Json::Array(items.iter().map(|item| of_value(&item.value)).collect()),
        Value::Map(map) => of_value_map(map),
    }
}

/// The JSON Schema emitter: the shape of front matter, and nothing else.
///
/// # What it constrains, and why the list is short
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)
/// admits a target only when "the emitted constraint catches exactly what the
/// native check catches". Two check families clear that bar and no third one
/// does.
///
/// - A required facet becomes `required`. The native check reports a document
///   of a kind that states no value for a facet the kind requires, and so does
///   the keyword.
/// - A facet with a declared value set becomes `enum` under a guard that lets a
///   mapping or a list through, because the check declines one too. Same set,
///   same verdict.
/// - A forbidden facet becomes nothing, and the loss set says so. See
///   [`forbidden_facets`]: no check reports the document a prohibition would
///   reject, so carrying it would state more than the engine enforces.
///
/// Everything else is graph-scoped, corpus-scoped, temporal, or about a body,
/// and a validator that holds one instance in its hand reaches none of them.
///
/// # The kind is selected by a discriminator or by a path, and never by `oneOf`
///
/// The obvious construction is a root `oneOf` over one schema per kind. It is
/// wrong, and it is wrong in the direction that matters: an abstract kind and
/// the kind that descends from it accept the same front matter, so a valid
/// document matches two branches and `oneOf` reports an error where the native
/// check has none. That is a constraint arriving with an inverted meaning
/// rather than a constraint that is missing, and a loss set does not record it.
///
/// So this emitter selects the way the corpus does. A heterogeneous shelf
/// declares a discriminator facet, and the schema applies the kind's shape under
/// `if`/`then` on that facet's value, which imposes nothing on a document of
/// another kind. A homogeneous shelf carries the kind in the directory, and no
/// front-matter member states it, so the schema cannot select at all. The
/// binding list below tells the editor which schema to apply to which glob, and
/// the loss set records that a binding is the editor's configuration rather than
/// something the artifact enforces.
fn schema(surface: &Surface<'_>, all: &[Node<'_>]) -> Body {
    let shape = surface.shape();
    let taxonomy = surface.taxonomy();

    let mut defs = Vec::new();
    let mut forbids = false;
    for kind in &shape.kinds {
        let required = shape.required_facets(&kind.name);
        let mut properties = Vec::new();
        for name in &required {
            properties.push((name.clone(), constraint(shape, name)));
        }
        for facet in &shape.facets {
            if !required.contains(&facet.name) && !facet.values.is_empty() {
                properties.push((facet.name.clone(), constraint(shape, &facet.name)));
            }
        }
        let mut members = vec![
            ("type".to_string(), Json::string("object")),
            ("properties".to_string(), Json::Object(properties)),
        ];
        if !required.is_empty() {
            members.push((
                "required".to_string(),
                Json::Array(
                    required
                        .iter()
                        .map(|name| Json::string(name.as_str()))
                        .collect(),
                ),
            ));
        }
        // A prohibition is not emitted. See `forbidden_facets` for the reason,
        // and for the loss that records the drop.
        if !forbidden_facets(shape, &kind.name).is_empty() {
            forbids = true;
        }
        defs.push((kind.name.clone(), Json::Object(members)));
    }

    let mut conditionals = Vec::new();
    let mut bindings = Vec::new();
    for shelf in &taxonomy.shelves {
        match &shelf.body {
            headwater_census::shelves::ShelfBody::Homogeneous { kind } => {
                bindings.push(Json::object([
                    ("path", Json::string(shelf.pattern.source())),
                    ("schema", Json::string(format!("#/$defs/{kind}"))),
                ]));
            }
            headwater_census::shelves::ShelfBody::Heterogeneous {
                discriminator,
                kinds,
            } => {
                bindings.push(Json::object([
                    ("path", Json::string(shelf.pattern.source())),
                    ("schema", Json::string("#")),
                ]));
                for kind in kinds {
                    conditionals.push(Json::object([
                        (
                            "if",
                            Json::object([
                                (
                                    "properties",
                                    Json::Object(vec![(
                                        discriminator.clone(),
                                        Json::object([("const", Json::string(kind.as_str()))]),
                                    )]),
                                ),
                                (
                                    "required",
                                    Json::Array(vec![Json::string(discriminator.as_str())]),
                                ),
                            ]),
                        ),
                        (
                            "then",
                            Json::object([("$ref", Json::string(format!("#/$defs/{kind}")))]),
                        ),
                    ]));
                }
            }
        }
    }

    let mut losses = Vec::new();
    for node in all {
        if losses
            .iter()
            .any(|loss: &Loss| loss.class == Class::Node && loss.name == node.class)
        {
            continue;
        }
        // An anchor is not a document and has no front matter, so it takes a
        // reason of its own. One reason for both classes would say something
        // false about half the nodes it covers, and a loss set that a reader
        // cannot trust is worse than no loss set.
        let reason = match node.document.is_some() {
            true => {
                "a JSON Schema states the shape that front matter of this class must satisfy, \
                 and carries no instance of it. What travels is the constraint, and the \
                 documents stay in the corpus"
            }
            false => {
                "an anchor is an external node with no front matter of its own. A schema over \
                 front matter has nothing to say about one, and the resolver that normalizes \
                 it stays inside this engine"
            }
        };
        losses.push(Loss {
            class: Class::Node,
            name: node.class.clone(),
            reason: reason.to_string(),
        });
    }
    for edge in &surface.graph().edges {
        if losses
            .iter()
            .any(|loss| loss.class == Class::Edge && loss.name == edge.declared)
        {
            continue;
        }
        losses.push(Loss {
            class: Class::Edge,
            name: edge.declared.clone(),
            reason: "an edge is a resolved relation between two documents. A validator holds \
                     one instance and has no second document to reach, so endpoint legality, \
                     reciprocity and cardinality stay native. The schema constrains the shape \
                     of the `relations` block and never what it resolves to"
                .to_string(),
        });
    }
    losses.push(Loss {
        class: Class::Attribute,
        name: "*".to_string(),
        reason: "an instance attribute is a member of a `relations` entry, and it travels only \
                 as far as the edge it hangs on, which is to say not at all"
            .to_string(),
    });
    if forbids {
        losses.push(Loss {
            class: Class::Attribute,
            name: "facets.forbid".to_string(),
            reason: "a kind that forbids a facet generates no value check over it, and nothing \
                     reports a document that states one. A schema keyword that rejected such a \
                     document would carry more force than the check it claims, so the \
                     prohibition is dropped rather than translated"
                .to_string(),
        });
    }
    losses.push(Loss {
        class: Class::Attribute,
        name: "the binding".to_string(),
        reason: "a homogeneous shelf carries its kind in the directory and no front-matter \
                 member states it, so this schema cannot select the kind for one. The bindings \
                 below say which glob takes which definition, and an editor that applies them \
                 is configuration rather than a constraint this artifact enforces"
            .to_string(),
    });

    let mut value = vec![
        (
            "$schema".to_string(),
            Json::string("https://json-schema.org/draft/2020-12/schema"),
        ),
        ("type".to_string(), Json::string("object")),
    ];
    if !conditionals.is_empty() {
        value.push(("allOf".to_string(), Json::Array(conditionals)));
    }
    value.push(("$defs".to_string(), Json::Object(defs)));
    value.push(("headwater:bindings".to_string(), Json::Array(bindings)));

    Body {
        value,
        carried_nodes: Vec::new(),
        carried_edges: Vec::new(),
        losses,
    }
}

/// The constraint one facet contributes, and the guard that keeps it equivalent.
///
/// A facet with no declared value set gets no constraint at all, and the
/// omission is deliberate. The taxonomy declares whether a facet is required and
/// what values it admits, and it declares no type. A `type: string` here would
/// be this emitter inventing a constraint that no native check enforces, which
/// is the resemblance spec 12 refuses.
///
/// A declared value set becomes `enum` over the same members. The guard is the
/// part the differential forced. `facet.value.not_permitted` produces nothing
/// for a value it cannot read as a scalar, because a mapping where the taxonomy
/// declared a set of scalars is a shape defect and the meta-schema owns shape.
/// A bare `enum` rejects that document, which is a constraint arriving with
/// more force than the check it claims to carry. So the emitted form admits a
/// composite and enumerates a scalar, which is what the check does.
fn constraint(shape: &headwater_check::Shape, name: &str) -> Json {
    match shape.facet(name) {
        Some(facet) if !facet.values.is_empty() => Json::object([(
            "anyOf",
            Json::Array(vec![
                Json::object([(
                    "type",
                    Json::Array(vec![Json::string("object"), Json::string("array")]),
                )]),
                Json::object([(
                    "enum",
                    Json::Array(
                        facet
                            .values
                            .iter()
                            .map(|value| Json::string(value.as_str()))
                            .collect(),
                    ),
                )]),
            ]),
        )]),
        _ => Json::Object(Vec::new()),
    }
}

/// The facets a kind forbids, including the ones it inherits.
///
/// The schema emitter reads this to learn whether a taxonomy forbids anything,
/// and then drops the prohibition rather than carrying it. `not: {anyOf: [...]}`
/// rejects a document that states a forbidden facet, and no check in the
/// registry reports that document: `facet.value.not_permitted` generates no
/// instance over a kind that forbids the facet, which is the whole of what the
/// prohibition does natively. A schema keyword that rejected it would report an
/// error the engine does not, and
/// [spec 12](../../../../docs/spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)
/// rules that an emitter omits a construct whose meaning inverts. The loss set
/// records the drop, which is a different statement: less coverage, not a
/// wrong answer.
fn forbidden_facets(shape: &headwater_check::Shape, kind: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for ancestor in shape.ancestry(kind) {
        for name in &ancestor.forbid {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
    }
    out
}

/// Hold an output against the graph: the projection census.
///
/// Neither the emitter's word nor the loss set is trusted here. The census walks
/// the graph, asks of each node and each edge whether the output carries it, and
/// where it does not, looks for a reason that covers its class. The order of the
/// two lookups matters: a withholding is checked first, because a document the
/// filter removed is accounted for by the profile and not by the emitter.
///
/// It is public and it takes three lists rather than an emitter, which is what
/// makes it testable against a projector that dropped something. An emitter that
/// audited itself would be the untrusted projector all over again, one layer up.
///
/// `carried_nodes` holds the key of each node the output carries: the path of a
/// document, or `anchor_kind:id` for an anchor. `carried_edges` holds the index
/// in the graph's own edge list. `withheld` is what the profile's filter
/// removed, as `(path, rule)`.
pub fn audit(
    surface: &Surface<'_>,
    withheld: &[(String, String)],
    carried_nodes: &[String],
    carried_edges: &[usize],
    losses: &[Loss],
) -> Census {
    let graph = surface.graph();
    let all = nodes(surface, graph);
    let all = &all;
    let mut census = Census::default();

    for node in all {
        census.nodes.in_graph += 1;
        if carried_nodes.contains(&node.key) {
            census.nodes.carried += 1;
            continue;
        }
        if let Some(rule) = rule_for(withheld, &node.key) {
            census.nodes.accounted += 1;
            note(
                &mut census.accounted,
                Class::Node,
                &node.class,
                &withholding(&rule),
            );
            continue;
        }
        match losses
            .iter()
            .find(|loss| loss.class == Class::Node && loss.name == node.class)
        {
            Some(loss) => {
                census.nodes.accounted += 1;
                note(
                    &mut census.accounted,
                    Class::Node,
                    &node.class,
                    &loss.reason,
                );
            }
            None => {
                census.nodes.unaccounted += 1;
                census.unaccounted.push(Unaccounted {
                    class: Class::Node,
                    name: node.class.clone(),
                    at: node.key.clone(),
                });
            }
        }
    }

    for (at, edge) in graph.edges.iter().enumerate() {
        census.edges.in_graph += 1;
        if carried_edges.contains(&at) {
            census.edges.carried += 1;
            continue;
        }
        if let Some(rule) = withheld_edge(edge, withheld) {
            census.edges.accounted += 1;
            note(
                &mut census.accounted,
                Class::Edge,
                &edge.declared,
                &withholding(&rule),
            );
            continue;
        }
        match losses
            .iter()
            .find(|loss| loss.class == Class::Edge && loss.name == edge.declared)
        {
            Some(loss) => {
                census.edges.accounted += 1;
                note(
                    &mut census.accounted,
                    Class::Edge,
                    &edge.declared,
                    &loss.reason,
                );
            }
            None => {
                census.edges.unaccounted += 1;
                census.unaccounted.push(Unaccounted {
                    class: Class::Edge,
                    name: edge.declared.clone(),
                    at: format!("{} -> {}", edge.source.path, edge.raw_target),
                });
            }
        }
    }

    census
}

/// One more of a class the output does not carry, under a reason already seen or
/// a new one.
fn note(accounted: &mut Vec<Accounted>, class: Class, name: &str, reason: &str) {
    match accounted
        .iter_mut()
        .find(|entry| entry.class == class && entry.name == name && entry.reason == reason)
    {
        Some(entry) => entry.count += 1,
        None => accounted.push(Accounted {
            class,
            name: name.to_string(),
            reason: reason.to_string(),
            count: 1,
        }),
    }
}

/// The reason a withheld node or edge carries.
///
/// The rule identifier and nothing else. Spec 6: "A withholding reason comes
/// from a closed set that the taxonomy declares. Free prose in a tombstone is a
/// channel, and a reason that quotes the document is a leak wearing a label."
fn withholding(rule: &str) -> String {
    format!("withheld by the rule `{rule}`")
}

/// The artifact: the marker, what this export is, what it dropped, and the body.
///
/// The declaration travels with the artifact ([spec 12](../../../../docs/spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)),
/// so the loss set and the census are members of the file rather than lines of a
/// report. A consumer who copies the export copies its limits with it, and one
/// who reads a copy that arrived without the report is the reader this rule is
/// written for.
fn envelope(
    profile: &Profile,
    emitter: Emitter,
    generated_at: Option<&str>,
    withheld: &[(String, String)],
    census: &Census,
    body: Body,
) -> Json {
    let filtered = !profile.filter.is_empty();
    let mut declaration = vec![
        ("name".to_string(), Json::string(profile.name.as_str())),
        ("target".to_string(), Json::string(emitter.name())),
        // The invariant, stated in the artifact under both grains. Spec 6: "No
        // profile may produce a view that presents as total." Under `sealed` a
        // reader still learns to stop drawing conclusions from absence, which
        // is the harm the rule exists to prevent.
        ("filtered".to_string(), Json::Bool(filtered)),
    ];
    if filtered {
        declaration.push((
            "tombstone".to_string(),
            Json::string(profile.tombstone.name()),
        ));
    }
    // Spec 6 asks a filtered export to state when it was generated, so that a
    // reader can compute the revocation lag. A clock read inside the emitter
    // would make every run differ from the last and turn `--check` into a coin
    // toss, so the time is injected and never read: `headwater export --at`
    // supplies it for the artifact that leaves the repository, and `generate`
    // supplies none for the artifact that is committed and held to
    // regeneration.
    if let Some(at) = generated_at {
        declaration.push(("generated_at".to_string(), Json::string(at)));
    }

    let mut members = vec![
        (
            headwater_mark::MARKER.to_string(),
            Json::string(headwater_mark::marker_text(Kind::GraphExport.name())),
        ),
        ("export_version".to_string(), Json::string(VERSION)),
        ("profile".to_string(), Json::Object(declaration)),
        (
            "loss_set".to_string(),
            Json::Array(
                body.losses
                    .iter()
                    .map(|loss| {
                        Json::object([
                            ("class", Json::string(loss.class.name())),
                            ("name", Json::string(loss.name.as_str())),
                            ("reason", Json::string(loss.reason.as_str())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("census".to_string(), of_census(census)),
    ];

    if filtered && profile.tombstone == Grain::Counted {
        let mut stones: Vec<(String, usize)> = Vec::new();
        for (_, rule) in withheld {
            match stones.iter_mut().find(|(name, _)| name == rule) {
                Some(entry) => entry.1 += 1,
                None => stones.push((rule.clone(), 1)),
            }
        }
        members.push((
            "tombstones".to_string(),
            Json::Array(
                stones
                    .into_iter()
                    .map(|(rule, count)| {
                        Json::object([
                            ("rule", Json::string(rule)),
                            ("documents", Json::Raw(count.to_string())),
                        ])
                    })
                    .collect(),
            ),
        ));
    }

    members.extend(body.value);
    Json::Object(members)
}

fn of_census(census: &Census) -> Json {
    let tally = |tally: &Tally| {
        Json::object([
            ("in the graph", Json::Raw(tally.in_graph.to_string())),
            ("carried", Json::Raw(tally.carried.to_string())),
            ("accounted for", Json::Raw(tally.accounted.to_string())),
            ("unaccounted for", Json::Raw(tally.unaccounted.to_string())),
        ])
    };
    Json::object([
        ("nodes", tally(&census.nodes)),
        ("edges", tally(&census.edges)),
        (
            "accounted for",
            Json::Array(
                census
                    .accounted
                    .iter()
                    .map(|entry| {
                        Json::object([
                            ("class", Json::string(entry.class.name())),
                            ("name", Json::string(entry.name.as_str())),
                            ("count", Json::Raw(entry.count.to_string())),
                            ("reason", Json::string(entry.reason.as_str())),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "unaccounted for",
            Json::Array(
                census
                    .unaccounted
                    .iter()
                    .map(|entry| {
                        Json::object([
                            ("class", Json::string(entry.class.name())),
                            ("name", Json::string(entry.name.as_str())),
                            ("at", Json::string(entry.at.as_str())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

/// The census as a report a person reads.
pub fn render(census: &Census) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} nodes: {} carried, {} accounted for",
        census.nodes.in_graph, census.nodes.carried, census.nodes.accounted
    );
    let _ = writeln!(
        out,
        "{} edges: {} carried, {} accounted for",
        census.edges.in_graph, census.edges.carried, census.edges.accounted
    );
    if !census.accounted.is_empty() {
        out.push_str("\nwhat the output does not carry, and why\n");
        for entry in &census.accounted {
            let _ = writeln!(
                out,
                "  {:5} {} `{}`\n    {}",
                entry.count,
                entry.class.name(),
                entry.name,
                entry.reason
            );
        }
    }
    if !census.unaccounted.is_empty() {
        out.push_str("\nomitted, and no declared reason covers it\n");
        for entry in &census.unaccounted {
            let _ = writeln!(
                out,
                "  {} `{}` at {}",
                entry.class.name(),
                entry.name,
                entry.at
            );
        }
    }
    out
}
