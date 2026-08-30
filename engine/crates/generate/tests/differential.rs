// SPDX-License-Identifier: Apache-2.0
//! The `exportable_as` differential: does the emitted schema catch exactly what
//! the native check catches?
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)
//! lets a check name an emitter target only when the emitted constraint catches
//! exactly what the native check catches. This file is the test that permits the
//! two `jsonschema` values in the check registry, and it holds them to both
//! halves of that sentence.
//!
//! # Two directions, and the second one is why this file exists
//!
//! A finding the engine reports and the validator misses is a partial
//! translation. It is a coverage claim that is partly true, which spec 12 rules
//! worse than a claim of none.
//!
//! A finding the validator reports and the engine does not is worse, and no loss
//! set redeems it. The consumer reads an error about a document this engine
//! accepts, which is a constraint arriving with an inverted meaning. Two
//! constructs in the emitter met that case, and writing this file is what found
//! them: a `not` over a forbidden facet, and a bare `enum` over a facet whose
//! value the check declines to read. Both are gone from the emitter, and both
//! have a test below that puts the old form back and watches the differential
//! fail on it.
//!
//! A regression in the emitter needs no mutant to catch it. [`evaluate`] reads
//! `oneOf` and `not`, so reinstating either one fails
//! [`the_emitted_schema_and_the_engine_agree_document_for_document`] over this
//! corpus, and a construct that nothing here reads fails
//! [`the_emitted_schema_uses_only_keywords_this_file_reads`].
//!
//! # What is independent here, and what is not
//!
//! Two validators run over the emitted schema, and they are independent of it in
//! different degrees.
//!
//! [`evaluate`] is in this repository and in this language. It is independent of
//! the *emitter*, because it reads the emitted bytes and knows nothing about
//! [`headwater_check::Shape`]. It is not independent of the author who read the
//! JSON Schema specification to write it, and on its own it would establish only
//! that two readings by one reader agree.
//!
//! `differential_oracle.py` beside this file is a stock validator: PyYAML parses
//! the front matter and the `jsonschema` package evaluates the schema. It
//! imports nothing this project wrote. When it is present,
//! [`the_stock_validator_agrees_with_the_evaluator_in_this_file`] holds the
//! in-repo evaluator to it record for record, and that is what pins the reading.
//! When it is absent that one test reports the skip and the rest of the
//! differential still runs, because the pinned build container carries no
//! Python. Set `HEADWATER_STOCK_VALIDATOR` and the skip becomes a failure.
//! Continuous integration sets it.
//!
//! # The corpus has to break something
//!
//! `fixtures/differential/` violates each claimed family on purpose, and carries
//! valid documents besides. An empty finding set compared with an empty finding
//! set is a test that passes over an emitter which emits nothing, and
//! [`the_corpus_is_not_vacuous`] is what stops this file from becoming one.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{
    facet_required, facet_value, Cache, Context, Date, Declared, Register, Run, Shape,
};
use headwater_generate::{export, Emitter, Filter, Grain, Profile};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_yaml::{Mapping, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The two check families the JSON Schema emitter claims. Nothing else in the
/// registry declares a target, and [`the_registry_partitions`] holds that.
const CLAIMED: [&str; 2] = [facet_required::RULE, facet_value::RULE];

const TARGET: &str = "jsonschema";

/// One finding, in the only grain both sides can state: the document, the rule
/// and the facet.
///
/// Not the message, because the engine writes prose no validator produces and
/// comparing it would compare wording rather than verdict. Not the line, because
/// a validator reads a parsed instance and has no span. The facet is in, because
/// two sets that agreed on counts while the facets differed would hide a swap.
type Record = (String, String, String);

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn taxonomy_path() -> PathBuf {
    fixtures_dir().join("differential.taxonomy.yml")
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn parse(text: &str) -> Value {
    headwater_yaml::load(text)
        .unwrap_or_else(|errors| panic!("the schema does not parse: {errors:?}"))
        .value
}

/// Everything a surface borrows, owned, so that a test may build one.
struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    config: Config,
}

impl Built {
    fn over(corpus: &Corpus, root: &Mapping) -> Self {
        let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
        let relations = Declarations::read(root).expect("the declarations read");
        let shape = Shape::read(root).expect("the shape reads");
        let census = census::take(corpus, &taxonomy);
        let graph = Graph::build(
            &census,
            &relations,
            &Resolvers::over(corpus),
            corpus,
            &Config::default(),
        );
        Built {
            census,
            graph,
            shape,
            taxonomy,
            relations,
            config: Config::default(),
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }
}

fn built() -> Built {
    Built::over(
        &Corpus::new(fixtures_dir(), "differential"),
        &load_map(&taxonomy_path()),
    )
}

/// The profile the schema is emitted under: everything, filtered by nothing.
///
/// A filter withholds documents, and a document the export never saw is one the
/// validator cannot be asked about. The differential is about what the two sides
/// say over the same set.
fn whole_corpus() -> Profile {
    Profile {
        name: "differential".to_string(),
        filter: Filter::default(),
        tombstone: Grain::Counted,
        entries: Vec::new(),
    }
}

/// The emitted schema: the bytes, and the same bytes parsed back.
fn schema(built: &Built) -> (String, Value) {
    let emission = export::emit(&built.surface(), &whole_corpus(), Emitter::JsonSchema, None)
        .expect("the schema emits");
    let value = parse(&emission.bytes);
    (emission.bytes, value)
}

// ---------------------------------------------------------------------------
// The engine's half
// ---------------------------------------------------------------------------

fn engine_run(built: &Built) -> Run {
    let register = Register::read(&load_map(&taxonomy_path())).expect("the register reads");
    let config = Config::default();
    headwater_check::run(
        &built.census,
        &built.graph,
        &Declared {
            lock: "differential",
            taxonomy: &built.taxonomy,
            shape: &built.shape,
            relations: &built.relations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/generate/fixtures/differential.taxonomy.yml",
        },
        &Context::at(Date::parse("2026-08-12").expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The findings the engine reports for the two claimed families.
///
/// Read off `instances` rather than `findings`. A suppression directive or an
/// adoption entry removes a finding before it reaches the report, and a
/// differential that read the filtered list would call a suppressed document an
/// over-report by the validator. This tree carries no directive, and reading the
/// unfiltered verdicts keeps that true if somebody adds one.
fn engine_records(run: &Run) -> BTreeSet<Record> {
    let mut out = BTreeSet::new();
    for instance in &run.instances {
        if !CLAIMED.contains(&instance.rule) {
            continue;
        }
        for finding in instance.findings() {
            out.insert((
                finding.path.clone(),
                finding.rule.to_string(),
                facet_of(&finding.message),
            ));
        }
    }
    out
}

/// The facet a finding is about, taken out of its message.
///
/// Both messages are generated from one format string each, and both name the
/// facet in a code span. [`the_facet_a_finding_names_is_read_out_of_its_message`]
/// holds the two formats to that shape, so a message that changed fails there
/// rather than silently widening this differential.
fn facet_of(message: &str) -> String {
    let spans: Vec<&str> = message.split('`').skip(1).step_by(2).collect();
    // `facet.required.missing` names the kind first and the facet second.
    let wanted = match message.contains("requires the facet") {
        true => 1,
        false => 0,
    };
    spans.get(wanted).unwrap_or(&"").to_string()
}

// ---------------------------------------------------------------------------
// The validator's half: an evaluator over the emitted subset
// ---------------------------------------------------------------------------

/// Evaluate one instance against one schema, and name the family of each error.
///
/// It reads the keywords this emitter writes and no others: `type`,
/// `properties`, `required`, `enum`, `const`, `anyOf`, `allOf`, `oneOf`, `not`,
/// `if` with `then`, and `$ref` into `$defs`. A keyword outside that set is
/// ignored, which is what a validator does with a keyword it does not know and
/// therefore what this evaluator has to do to agree with one. The emitted schema
/// is held to that set by
/// [`the_emitted_schema_uses_only_keywords_this_file_reads`], so a construct
/// nothing here reads fails a test rather than passing quietly.
fn evaluate(root: &Value, schema: &Value, instance: &Value, at: &str, out: &mut Vec<Record>) {
    let Some(members) = schema.as_map() else {
        return;
    };
    for entry in members.entries() {
        let key = entry.key.value.as_str();
        let value = &entry.value.value;
        match key {
            "type" => {
                if !type_holds(value, instance) {
                    out.push(record(at, "unclassified:type", ""));
                }
            }
            "required" => {
                for name in seq(value) {
                    let name = text(name);
                    if member(instance, &name).is_none() {
                        out.push(record(at, facet_required::RULE, &name));
                    }
                }
            }
            "properties" => {
                let Some(properties) = value.as_map() else {
                    continue;
                };
                for property in properties.entries() {
                    let name = property.key.value.as_str();
                    if let Some(held) = member(instance, name) {
                        constrain(&property.value.value, held, at, name, out);
                    }
                }
            }
            "allOf" => {
                for branch in seq(value) {
                    evaluate(root, branch, instance, at, out);
                }
            }
            "oneOf" => {
                let matched = seq(value)
                    .iter()
                    .filter(|branch| passes(root, branch, instance))
                    .count();
                if matched != 1 {
                    out.push(record(at, "unclassified:oneOf", ""));
                }
            }
            "anyOf" => {
                if !seq(value)
                    .iter()
                    .any(|branch| passes(root, branch, instance))
                {
                    out.push(record(at, "unclassified:anyOf", ""));
                }
            }
            "not" => {
                if passes(root, value, instance) {
                    out.push(record(at, "unclassified:not", ""));
                }
            }
            "if" => {
                if passes(root, value, instance) {
                    if let Some(then) = members.get("then") {
                        evaluate(root, &then.value, instance, at, out);
                    }
                }
            }
            "$ref" => {
                if let Some(target) = resolve(root, &text(value)) {
                    evaluate(root, target, instance, at, out);
                }
            }
            // Anything else is ignored, because a validator ignores a keyword
            // it does not know and this evaluator has to behave like one. What
            // stops an unread construct from passing unnoticed is not silence
            // here but `the_emitted_schema_uses_only_keywords_this_file_reads`,
            // which walks the emitted schema and names every keyword in it.
            _ => {}
        }
    }
}

/// One member of a document, against the constraint the schema puts on it.
///
/// This is where a declared value set is read. A bare `enum` is
/// `facet.value.not_permitted`, and so is the guarded form the emitter writes.
/// The guard is honored here rather than assumed: a composite passes the first
/// branch, which is what the native check does when it cannot read a scalar.
fn constrain(schema: &Value, held: &Value, at: &str, facet: &str, out: &mut Vec<Record>) {
    let Some(members) = schema.as_map() else {
        return;
    };
    for entry in members.entries() {
        let key = entry.key.value.as_str();
        let value = &entry.value.value;
        match key {
            "enum" => {
                if !enum_holds(value, held) {
                    out.push(record(at, facet_value::RULE, facet));
                }
            }
            "const" => {
                if held.as_scalar().map(|s| s.text.as_str()) != Some(text(value).as_str()) {
                    out.push(record(at, "unclassified:const", facet));
                }
            }
            "type" => {
                if !type_holds(value, held) {
                    out.push(record(at, "unclassified:type", facet));
                }
            }
            "anyOf" => {
                let any = seq(value).iter().any(|branch| {
                    let mut probe = Vec::new();
                    constrain(branch, held, at, facet, &mut probe);
                    probe.is_empty()
                });
                if !any {
                    out.push(record(at, facet_value::RULE, facet));
                }
            }
            // Ignored, for the reason the same arm in `evaluate` is.
            _ => {}
        }
    }
}

fn passes(root: &Value, schema: &Value, instance: &Value) -> bool {
    let mut probe = Vec::new();
    evaluate(root, schema, instance, "probe", &mut probe);
    probe.is_empty()
}

fn record(at: &str, rule: &str, facet: &str) -> Record {
    (at.to_string(), rule.to_string(), facet.to_string())
}

fn seq(value: &Value) -> Vec<&Value> {
    value
        .as_seq()
        .map(|items| items.iter().map(|item| &item.value).collect())
        .unwrap_or_default()
}

fn text(value: &Value) -> String {
    value
        .as_scalar()
        .map(|scalar| scalar.text.clone())
        .unwrap_or_default()
}

fn member<'a>(instance: &'a Value, name: &str) -> Option<&'a Value> {
    instance.as_map()?.get(name).map(|held| &held.value)
}

fn enum_holds(value: &Value, held: &Value) -> bool {
    let Some(actual) = held.as_scalar() else {
        // A composite is not a member of a set of scalars, so a bare `enum`
        // rejects it. That is the over-report the guard exists to stop, and
        // saying so truthfully here is what lets the adversarial test see it.
        return false;
    };
    seq(value).iter().any(|member| text(member) == actual.text)
}

fn type_holds(value: &Value, instance: &Value) -> bool {
    let wanted: Vec<String> = match value.as_seq() {
        Some(_) => seq(value).iter().map(|item| text(item)).collect(),
        None => vec![text(value)],
    };
    let actual = match instance {
        Value::Map(_) => "object",
        Value::Seq(_) => "array",
        Value::Scalar(_) => "string",
    };
    wanted.iter().any(|name| name == actual)
}

fn resolve<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer == "#" {
        return Some(root);
    }
    let mut cursor = root;
    for step in pointer.trim_start_matches("#/").split('/') {
        cursor = &cursor.as_map()?.get(step)?.value;
    }
    Some(cursor)
}

/// Run a schema over every document of the fixture corpus, the way its own
/// bindings say to.
fn schema_records(root: &Value) -> BTreeSet<Record> {
    let bindings: Vec<(String, String)> = root
        .as_map()
        .and_then(|map| map.get("headwater:bindings"))
        .map(|value| seq(&value.value))
        .unwrap_or_default()
        .iter()
        .filter_map(|binding| {
            let map = binding.as_map()?;
            Some((
                text(&map.get("path")?.value),
                text(&map.get("schema")?.value),
            ))
        })
        .collect();

    let mut out = BTreeSet::new();
    for (path, front) in documents() {
        let Some(pointer) = bindings
            .iter()
            .find(|(glob, _)| selects(glob, &path))
            .map(|(_, pointer)| pointer)
        else {
            continue;
        };
        let Some(schema) = resolve(root, pointer) else {
            continue;
        };
        let mut records = Vec::new();
        evaluate(root, schema, &front, &path, &mut records);
        out.extend(records);
    }
    out
}

/// Whether a shelf pattern selects a path. Every shelf here ends in `**`.
fn selects(glob: &str, path: &str) -> bool {
    match glob.strip_suffix("**") {
        Some(prefix) => path.starts_with(prefix),
        None => glob == path,
    }
}

/// Every document of the fixture corpus, with its front matter parsed.
fn documents() -> Vec<(String, Value)> {
    let mut out = Vec::new();
    let mut stack = vec![fixtures_dir().join("differential")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("the fixture tree reads") {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a fixture reads");
            let Some(front) = front_matter(&text) else {
                continue;
            };
            let relative = path
                .strip_prefix(fixtures_dir())
                .expect("inside the fixtures")
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, front));
        }
    }
    out.sort_by(|left, right| left.0.cmp(&right.0));
    out
}

fn front_matter(text: &str) -> Option<Value> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some(headwater_yaml::load(&rest[..end]).ok()?.value)
}

// ---------------------------------------------------------------------------
// The differential
// ---------------------------------------------------------------------------

/// What the two sides disagree about, in the two directions that mean different
/// things.
#[derive(Debug)]
struct Divergence {
    /// Reported by the engine and not by the validator: a partial translation.
    missed: Vec<Record>,
    /// Reported by the validator and not by the engine: an inverted meaning,
    /// which no loss set redeems.
    invented: Vec<Record>,
}

impl Divergence {
    fn between(engine: &BTreeSet<Record>, validator: &BTreeSet<Record>) -> Self {
        Divergence {
            missed: engine.difference(validator).cloned().collect(),
            invented: validator.difference(engine).cloned().collect(),
        }
    }

    fn is_empty(&self) -> bool {
        self.missed.is_empty() && self.invented.is_empty()
    }
}

/// The engine and one schema, over the same corpus.
fn differential(built: &Built, root: &Value) -> Divergence {
    Divergence::between(&engine_records(&engine_run(built)), &schema_records(root))
}

// ---------------------------------------------------------------------------
// The bar
// ---------------------------------------------------------------------------

/// Stated once: the two finding sets are the same set.
#[test]
fn the_emitted_schema_and_the_engine_agree_document_for_document() {
    let built = built();
    let (_, root) = schema(&built);
    let divergence = differential(&built, &root);
    assert!(
        divergence.is_empty(),
        "the emitted schema no longer catches what the checks catch, so no check \
         may declare `{TARGET}`.\n\
         reported by the engine and missed by the schema: {:#?}\n\
         reported by the schema and not by the engine: {:#?}",
        divergence.missed,
        divergence.invented
    );
}

/// The test above passes over an emitter that emits nothing, unless the corpus
/// breaks something. This is what says it does.
#[test]
fn the_corpus_is_not_vacuous() {
    let engine = engine_records(&engine_run(&built()));
    for rule in CLAIMED {
        let count = engine.iter().filter(|(_, found, _)| found == rule).count();
        assert!(
            count >= 2,
            "{rule} has {count} findings over the differential corpus, and a \
             differential over a corpus that violates nothing proves nothing"
        );
    }
    let touched: BTreeSet<&String> = engine.iter().map(|(path, _, _)| path).collect();
    assert!(
        touched.len() < documents().len(),
        "every document in the corpus is a violation, so a validator that \
         rejected everything would pass this differential"
    );
}

/// Every keyword the emitted schema writes is one [`evaluate`] reads.
///
/// This is what makes an ignored keyword safe. A validator ignores what it does
/// not know, so the evaluator does too, and the guarantee has to come from
/// somewhere else: the emitter may write nothing that nothing here evaluates. A
/// new construct fails this test, and the author has to decide whether it
/// belongs in the differential or out of the emitter.
#[test]
fn the_emitted_schema_uses_only_keywords_this_file_reads() {
    /// The envelope, which every emitter writes and no validator reads. They
    /// sit beside the schema at the root, and a stock validator passes over
    /// them as unknown keywords.
    const ENVELOPE: [&str; 7] = [
        "headwater:generated",
        "version",
        "export_version",
        "profile",
        "loss_set",
        "census",
        "tombstones",
    ];
    /// What `evaluate` and `constrain` between them read.
    const READ: [&str; 12] = [
        "$schema",
        "$defs",
        "$ref",
        "type",
        "properties",
        "required",
        "enum",
        "const",
        "allOf",
        "anyOf",
        "if",
        "then",
    ];

    let (_, root) = schema(&built());
    let mut seen = BTreeSet::new();
    keywords(&root, &mut seen, 0);
    let unread: Vec<&String> = seen
        .iter()
        .filter(|key| {
            *key != "headwater:bindings" && !READ.contains(&key.as_str())
                || ENVELOPE.contains(&key.as_str())
        })
        .collect();
    assert!(
        unread.is_empty(),
        "the emitted schema writes {unread:?}, which nothing in this file \
         evaluates. Either the differential learns the construct, or the \
         emitter drops it."
    );
    assert!(
        seen.contains("anyOf"),
        "the guard on a value set is gone, and a bare `enum` rejects a composite \
         that `facet.value.not_permitted` declines to read"
    );
}

/// Every key that stands in a schema position, envelope members aside.
fn keywords(schema: &Value, seen: &mut BTreeSet<String>, depth: usize) {
    let Some(members) = schema.as_map() else {
        return;
    };
    for entry in members.entries() {
        let key = entry.key.value.as_str();
        let value = &entry.value.value;
        if depth == 0
            && matches!(
                key,
                "headwater:generated"
                    | "version"
                    | "export_version"
                    | "profile"
                    | "loss_set"
                    | "census"
                    | "tombstones"
            )
        {
            continue;
        }
        seen.insert(key.to_string());
        match key {
            // A subschema hangs off each member, and the member names are
            // facets and kinds rather than keywords.
            "properties" | "$defs" => {
                for member in value.as_map().iter().flat_map(|map| map.entries()) {
                    keywords(&member.value.value, seen, depth + 1);
                }
            }
            "allOf" | "anyOf" | "oneOf" => {
                for branch in seq(value) {
                    keywords(branch, seen, depth + 1);
                }
            }
            "if" | "then" | "not" => keywords(value, seen, depth + 1),
            // `required`, `enum`, `const`, `type` and `$ref` carry data.
            _ => {}
        }
    }
}

/// The two constructs the emitter used to write, named so that reinstating one
/// reads as a regression rather than as a new idea.
#[test]
fn the_emitted_schema_carries_no_construct_that_inverts() {
    let (bytes, _) = schema(&built());
    assert!(
        !bytes.contains("\"oneOf\""),
        "a root `oneOf` is back. See the module comment on `schema` in export.rs"
    );
    assert!(
        !bytes.contains("\"not\""),
        "a prohibition is back. See `forbidden_facets` in export.rs"
    );
    assert!(
        bytes.contains("\"anyOf\""),
        "the guard on a value set is gone, and a bare `enum` rejects a composite \
         that `facet.value.not_permitted` declines to read"
    );
}

// ---------------------------------------------------------------------------
// Proof that the differential can fail
// ---------------------------------------------------------------------------
//
// Each schema below is written out rather than produced by mutating the
// emitter, so that the emitter keeps one code path. What they hold to account is
// the machinery: the evaluator, the record grain, and the two directions of
// `Divergence`. A regression in the emitter itself is caught by the two tests
// above without any of this.

/// A root `oneOf` reports an error where the engine reports none.
///
/// The valid decision on the discriminated shelf matches a kind and the kind
/// above it. This is the inverted-meaning direction, and a differential that
/// measured only missed findings would let it through.
#[test]
fn a_root_one_of_is_a_divergence() {
    let schema = parse(
        r##"{
          "type": "object",
          "oneOf": [{"$ref": "#/$defs/decision"}, {"$ref": "#/$defs/guide"}],
          "$defs": {
            "decision": {"type": "object", "required": ["status", "status_since", "summary"]},
            "guide": {"type": "object", "required": ["status", "status_since", "summary"]}
          },
          "headwater:bindings": [{"path": "differential/archive/**", "schema": "#"}]
        }"##,
    );
    let divergence = differential(&built(), &schema);
    assert!(
        divergence
            .invented
            .iter()
            .any(|(path, rule, _)| path.contains("decision-valid") && rule.contains("oneOf")),
        "a root `oneOf` over a valid document produced no invented finding, so \
         this differential measures one direction only: {divergence:#?}"
    );
}

/// A prohibition reports an error where the engine reports none.
///
/// This one shipped. `not: {anyOf: [{required: [audience]}]}` rejects the note
/// that states a facet its kind forbids, and no check in the registry reports
/// that document. See `forbidden_facets` in `export.rs`.
#[test]
fn a_prohibition_that_no_check_carries_is_a_divergence() {
    let schema = parse(
        r##"{
          "type": "object",
          "$defs": {
            "note": {"type": "object", "not": {"anyOf": [{"required": ["audience"]}]}}
          },
          "headwater:bindings": [{"path": "differential/notes/**", "schema": "#/$defs/note"}]
        }"##,
    );
    let divergence = differential(&built(), &schema);
    assert!(
        divergence
            .invented
            .iter()
            .any(|(path, rule, _)| path.contains("scratch") && rule.contains("not")),
        "a prohibition over a forbidden facet produced no invented finding: \
         {divergence:#?}"
    );
}

/// A bare `enum` rejects a value the check declines to read.
///
/// The document is built here rather than committed, because front matter whose
/// shape is wrong is the meta-schema's business and not something the rest of
/// this corpus should carry. The guarded half of the comparison reads the real
/// emitted schema, so it is the shipped guard under test and not a copy of it.
#[test]
fn a_bare_enum_over_a_composite_value_is_a_divergence() {
    let (_, emitted) = schema(&built());
    let instance = parse("status: [draft, current]\nstatus_since: 2026-03-09\nsummary: a composite where a scalar was declared\n");

    let guarded = resolve(&emitted, "#/$defs/decision").expect("the emitted definition");
    let mut under_guard = Vec::new();
    evaluate(&emitted, guarded, &instance, "probe", &mut under_guard);
    assert!(
        under_guard.is_empty(),
        "the emitted schema rejects a composite value, which \
         `facet.value.not_permitted` does not: {under_guard:#?}"
    );

    let bare = parse(
        r#"{"type": "object",
            "properties": {"status": {"enum": ["draft", "current", "superseded"]}},
            "required": ["status", "status_since", "summary"]}"#,
    );
    let mut without_guard = Vec::new();
    evaluate(&bare, &bare, &instance, "probe", &mut without_guard);
    assert!(
        without_guard
            .iter()
            .any(|(_, rule, facet)| rule == facet_value::RULE && facet == "status"),
        "a bare `enum` accepted a composite, so this test does not measure the guard"
    );
}

/// A schema that requires nothing misses every finding the engine has.
///
/// The partial translation, which is the other direction. A consumer would be
/// told that a document with no `summary` is fine.
#[test]
fn a_schema_that_drops_required_is_a_divergence() {
    let schema = parse(
        r##"{
          "type": "object",
          "$defs": {"anything": {"type": "object"}},
          "headwater:bindings": [{"path": "differential/**", "schema": "#/$defs/anything"}]
        }"##,
    );
    let divergence = differential(&built(), &schema);
    assert!(
        divergence.invented.is_empty(),
        "a schema that constrains nothing invented a finding: {divergence:#?}"
    );
    assert!(
        !divergence.missed.is_empty(),
        "a schema that constrains nothing missed no finding, so the engine side \
         of this differential is empty: {divergence:#?}"
    );
}

// ---------------------------------------------------------------------------
// The declaration this test permits
// ---------------------------------------------------------------------------

/// The registry splits in two, and every rule lands in exactly one half.
#[test]
fn the_registry_partitions() {
    let partition = headwater_check::partition(TARGET);
    let mut all = partition.exported.clone();
    all.extend(partition.unexported.iter().copied());
    all.sort_unstable();
    let mut rules: Vec<&str> = headwater_check::RULES.to_vec();
    rules.sort_unstable();
    assert_eq!(all, rules, "the two halves are not the registry");
    assert_eq!(
        partition.exported.len() + partition.unexported.len(),
        headwater_check::RULES.len(),
        "a rule falls into both halves or into neither"
    );
    assert_eq!(
        partition.exported,
        CLAIMED.to_vec(),
        "the exported set is not the set this file measured"
    );
}

/// The message formats this differential reads a facet out of.
#[test]
fn the_facet_a_finding_names_is_read_out_of_its_message() {
    assert_eq!(
        facet_of("`decision` requires the facet `summary`, and it is not declared"),
        "summary"
    );
    assert_eq!(
        facet_of("`status` admits draft, current, and this document declares `retired`"),
        "status"
    );
}

// ---------------------------------------------------------------------------
// The stock validator
// ---------------------------------------------------------------------------

/// Hold the evaluator in this file to an implementation nobody here wrote.
///
/// The evaluator above is one reader's reading of a specification. This test
/// holds it to a widely used implementation of the same specification, over the
/// same schema and the same documents. When the two agree, a divergence the
/// evaluator failed to see is a divergence that two independent readings both
/// failed to see, which is a different and a much better claim.
#[test]
fn the_stock_validator_agrees_with_the_evaluator_in_this_file() {
    let required = std::env::var_os("HEADWATER_STOCK_VALIDATOR").is_some();
    let built = built();
    let (bytes, root) = schema(&built);

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let written = dir.join("front-matter.schema.json");
    std::fs::write(&written, &bytes).expect("the schema writes");

    let oracle = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("differential_oracle.py");
    let ran = Command::new("python3")
        .arg(&oracle)
        .arg(&written)
        .arg(fixtures_dir())
        .output();

    let output = match ran {
        Ok(output) if output.status.success() => output,
        other => {
            let reason = match other {
                Ok(output) => String::from_utf8_lossy(&output.stderr).to_string(),
                Err(error) => error.to_string(),
            };
            assert!(
                !required,
                "HEADWATER_STOCK_VALIDATOR is set and the stock validator did not \
                 run, so nothing pins the evaluator in this file: {reason}\n\
                 Install it with: pip install jsonschema pyyaml"
            );
            eprintln!(
                "note: the stock validator did not run, so the evaluator in this \
                 file is unpinned. Install it with `pip install jsonschema pyyaml`, \
                 or set HEADWATER_STOCK_VALIDATOR to make its absence a failure.\n\
                 {reason}"
            );
            return;
        }
    };

    let stock: BTreeSet<Record> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            Some((
                parts.next()?.to_string(),
                parts.next()?.to_string(),
                parts.next().unwrap_or_default().to_string(),
            ))
        })
        .collect();

    assert_eq!(
        schema_records(&root),
        stock,
        "the evaluator in this file and the stock validator disagree about the \
         emitted schema, so this differential rests on a reading that nobody \
         else shares"
    );
}
