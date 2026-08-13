// SPDX-License-Identifier: Apache-2.0
//! `headwater generate`, and `--check` over what it wrote last time.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#projections) gives
//! the contract in four lines. A projection is a generated artifact, the
//! taxonomy declares each one, `generate` writes it, and `generate --check`
//! fails when a committed output differs from what this corpus and this lock
//! produce now.
//!
//! # The kinds are two value sets, not one
//!
//! Spec 6 names the kinds the engine implements, and
//! [Q19](../../../../docs/spec/09-decisions.md) adds `transcription`. Collecting
//! them for the meta-schema showed that they do not form one set. Seven are
//! declarable: a taxonomy names the kind and the output path, and
//! [principle 1](../../../../docs/spec/00-vision-and-scope.md#design-principles)
//! makes the path a schema decision. Two are engine-defined. Spec 4 makes the
//! register "engine-defined and non-optional", and
//! [Q20](../../../../docs/spec/09-decisions.md) fixes the corpus descriptor at
//! `.headwater/corpus.json`, both for the same reason: a reader who must consult
//! the taxonomy to find an artifact already knows what the artifact would tell
//! them. A declaration of either would put a second copy of one artifact at a
//! path the engine did not fix.
//!
//! So [`Kind`] carries all nine and [`Kind::declarable`] separates them. The
//! meta-schema's enum holds the seven.
//!
//! # The marker is the record of the previous run
//!
//! Spec 6: "A projection carries a generated-file marker. The engine refuses to
//! overwrite a file that lacks the marker and did not come from a previous run."
//! Read literally that is two permissions, and the second one needs the engine
//! to remember what it wrote. It does not need to. The marker in the file *is*
//! the record, and a manifest beside the file would be a second statement of one
//! fact, which is the drift
//! [principle 2](../../../../docs/spec/00-vision-and-scope.md#design-principles)
//! rules against. So this engine reads one permission from the bytes on disk:
//!
//! - the path holds nothing — write it,
//! - the path holds a file whose first line carries the marker — overwrite it,
//! - the path holds anything else — refuse, and say which path.
//!
//! An authored document is never destroyed, which is the property spec 6 asks
//! the marker for.
//!
//! # No output states when it was generated
//!
//! `--check` compares bytes. A timestamp inside a generated file makes every run
//! differ from the last one, so the gate reports drift on a corpus nobody
//! touched. Every marker this module writes is therefore a function of the kind
//! alone. Spec 6 asks a *filtered export* to state "when it was generated", and
//! that sentence and this one cannot both hold for an artifact that `--check`
//! covers. The contradiction is recorded in
//! [spec 13](../../../../docs/spec/13-open-obligations.md) against the export
//! verb, which is where it lands.

use headwater_census::census::Census;
use headwater_census::resolve::{shelf_for, ShelfMatch};
use headwater_census::shelves::DeclarationError;
use headwater_query::{Document, Pointer, Surface};
use headwater_yaml::value::{Mapping, Value};
use std::path::Path;

mod shelf_index;

/// The word that marks a file as this engine's output.
pub const MARKER: &str = "headwater:generated";

/// A projection kind.
///
/// Nine of them. Seven a taxonomy declares, and two the engine defines. See the
/// module header for why that split exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ShelfIndex,
    RelationView,
    AgentRules,
    SiteNav,
    GraphExport,
    Template,
    Transcription,
    /// Spec 4: the register, engine-defined and non-optional.
    CoverageReport,
    /// Q20: `.headwater/corpus.json`, engine-defined and non-optional.
    CorpusDescriptor,
}

impl Kind {
    /// The name as a taxonomy writes it, and as a report prints it.
    pub fn name(self) -> &'static str {
        match self {
            Kind::ShelfIndex => "shelf_index",
            Kind::RelationView => "relation_view",
            Kind::AgentRules => "agent_rules",
            Kind::SiteNav => "site_nav",
            Kind::GraphExport => "graph_export",
            Kind::Template => "template",
            Kind::Transcription => "transcription",
            Kind::CoverageReport => "coverage_report",
            Kind::CorpusDescriptor => "corpus_descriptor",
        }
    }

    /// Whether a taxonomy may declare this kind.
    ///
    /// The two that it may not are the two whose output path the engine fixes.
    pub fn declarable(self) -> bool {
        !matches!(self, Kind::CoverageReport | Kind::CorpusDescriptor)
    }

    /// The declarable kinds, in the order the meta-schema lists them.
    pub const DECLARABLE: [Kind; 7] = [
        Kind::ShelfIndex,
        Kind::RelationView,
        Kind::AgentRules,
        Kind::SiteNav,
        Kind::GraphExport,
        Kind::Template,
        Kind::Transcription,
    ];

    fn parse(text: &str) -> Option<Kind> {
        Kind::DECLARABLE
            .into_iter()
            .find(|kind| kind.name() == text)
    }
}

/// One entry of the `projections` block.
#[derive(Clone, Debug)]
pub struct Declaration {
    pub kind: Kind,
    /// The shelves this projection covers. Empty means every shelf, which is
    /// what a declaration with no `for` says.
    pub shelves: Vec<String>,
    pub output: String,
}

/// The `projections` block of a resolved taxonomy.
#[derive(Clone, Debug, Default)]
pub struct Projections {
    pub declared: Vec<Declaration>,
}

impl Projections {
    /// Read `projections` from the root of a resolved taxonomy.
    ///
    /// Optional at this layer, like `obligations`. A taxonomy that declares none
    /// is a corpus that generates nothing, and that is a true report of a
    /// package which has declared no derived artifact.
    ///
    /// A kind outside the value set is refused here as well as by the
    /// meta-schema. The meta-schema runs over sources and this runs over a lock,
    /// and a lock written before the value set closed would otherwise reach the
    /// emitter with a name no emitter answers to.
    pub fn read(root: &Mapping) -> Result<Self, Vec<DeclarationError>> {
        let mut errors = Vec::new();
        let mut out = Projections::default();
        let Some(node) = root.get("projections") else {
            return Ok(out);
        };
        let Value::Seq(items) = &node.value else {
            return Err(vec![DeclarationError {
                message: format!(
                    "`projections` is {}, and it lists projections",
                    node.value.kind_name()
                ),
                span: node.span,
            }]);
        };
        for (index, item) in items.iter().enumerate() {
            let Value::Map(body) = &item.value else {
                errors.push(DeclarationError {
                    message: format!(
                        "`projections.{index}` is {}, and a projection is a block",
                        item.value.kind_name()
                    ),
                    span: item.span,
                });
                continue;
            };
            let kind = match body.get("kind").and_then(|node| node.value.as_scalar()) {
                Some(scalar) => match Kind::parse(&scalar.text) {
                    Some(kind) => kind,
                    None => {
                        errors.push(DeclarationError {
                            message: format!(
                                "`projections.{index}.kind` is `{}`, which is not a kind a \
                                 taxonomy may declare. The declarable kinds are {}",
                                scalar.text,
                                names(&Kind::DECLARABLE)
                            ),
                            span: item.span,
                        });
                        continue;
                    }
                },
                None => {
                    errors.push(DeclarationError {
                        message: format!("`projections.{index}` states no `kind`"),
                        span: item.span,
                    });
                    continue;
                }
            };
            let Some(output) = body.get("output").and_then(|node| node.value.as_scalar()) else {
                errors.push(DeclarationError {
                    message: format!("`projections.{index}` states no `output`"),
                    span: item.span,
                });
                continue;
            };
            let mut shelves = Vec::new();
            if let Some(node) = body.get("for") {
                if let Value::Seq(items) = &node.value {
                    for item in items {
                        if let Some(scalar) = item.value.as_scalar() {
                            shelves.push(scalar.text.clone());
                        }
                    }
                }
            }
            out.declared.push(Declaration {
                kind,
                shelves,
                output: output.text.clone(),
            });
        }
        match errors.is_empty() {
            true => Ok(out),
            false => Err(errors),
        }
    }
}

fn names(kinds: &[Kind]) -> String {
    kinds
        .iter()
        .map(|kind| format!("`{}`", kind.name()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// One file this run would write.
#[derive(Clone, Debug)]
pub struct Output {
    /// Relative to the repository root, in the form the declaration wrote it.
    pub path: String,
    pub kind: Kind,
    /// The whole file, marker included.
    pub bytes: String,
}

/// A projection that produced no file, and why.
///
/// Reported rather than dropped. A generator that silently emitted nothing for
/// half its declarations is the silent pass that
/// [spec 4](../../../../docs/spec/04-assurance-model.md) removes everywhere
/// else, and the reason is the part a reader acts on.
#[derive(Clone, Debug)]
pub struct Unwritten {
    /// What the declaration or the engine named, for a reader to find it.
    pub at: String,
    pub kind: Kind,
    pub reason: String,
}

/// Everything one run would write, and everything it would not.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub outputs: Vec<Output>,
    pub unwritten: Vec<Unwritten>,
}

/// What the engine writes with no declaration at all.
///
/// Two kinds are engine-defined and neither is emitted yet. They are listed
/// here, with the reason, so that a run states the whole of what a projection
/// layer owes rather than the part that is built. Spec 4 calls the register "a
/// projection like any other" and asks this verb to hold it to regeneration;
/// building the rest of the verb is what showed why it cannot yet.
fn engine_defined() -> Vec<Unwritten> {
    vec![
        Unwritten {
            at: "the register".to_string(),
            kind: Kind::CoverageReport,
            reason: "its content is a function of the clock as well as of the corpus and the \
                     lock, because a migration task lapses and a suppression expires on a date. \
                     A committed copy would fail this check on a morning when nothing changed. \
                     Spec 13 carries it"
                .to_string(),
        },
        Unwritten {
            at: ".headwater/corpus.json".to_string(),
            kind: Kind::CorpusDescriptor,
            reason: "nothing computes a descriptor yet. Issue #64 owns it, and it writes through \
                     this verb when it lands"
                .to_string(),
        },
    ]
}

/// Build the plan: what every declaration and the engine itself would write.
pub fn plan(surface: &Surface<'_>, census: &Census, projections: &Projections) -> Plan {
    let mut plan = Plan::default();
    for declaration in &projections.declared {
        match declaration.kind {
            Kind::ShelfIndex => shelf_index::emit(surface, census, declaration, &mut plan),
            other => plan.unwritten.push(Unwritten {
                at: declaration.output.clone(),
                kind: other,
                reason: unbuilt(other).to_string(),
            }),
        }
    }
    plan.unwritten.extend(engine_defined());
    plan
}

/// Why a declarable kind produces nothing yet, and who owns it.
///
/// Named rather than ignored, and each one names the issue that owns it. This is
/// the posture the binary already takes over `headwater query`: a surface the
/// specification lists and this engine does not implement is reported as that,
/// and never as an empty result.
fn unbuilt(kind: Kind) -> &'static str {
    match kind {
        Kind::GraphExport => {
            "the emitters, the loss set and the projection census are issue #65, and an export \
             that declared no loss set would be the untrusted projector spec 6 built the census \
             to catch"
        }
        Kind::RelationView => {
            "a relation view needs the traceability grain that the obligation and control \
             register holds, and no document states which relations a view covers"
        }
        Kind::AgentRules | Kind::SiteNav => {
            "spec 5 and Q16 name the artifact and no document states its form, so an emitter here \
             would be this engine inventing a schema for somebody else's consumer"
        }
        Kind::Template => {
            "a template is the permitted relations, facets and sections of a kind, which \
             `headwater explain` already prints. What no document states is the file it goes in"
        }
        Kind::Transcription => {
            "a transcription needs a resolver that reads text from a pinned snapshot, and Q19 \
             leaves whether it ships at all to the first adopter who asks"
        }
        Kind::ShelfIndex | Kind::CoverageReport | Kind::CorpusDescriptor => "this engine emits it",
    }
}

/// What happened to one output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Written, and nothing was there before.
    Written,
    /// Written over this engine's own earlier output.
    Rewritten,
    /// Already exactly these bytes.
    Unchanged,
    /// A file is there, it carries no marker, and this engine will not destroy
    /// it.
    Occupied,
    /// `--check` only: committed bytes differ from what this corpus produces.
    Differs,
    /// `--check` only: nothing is committed at the path.
    Missing,
    /// The write failed, and this is what the operating system said.
    Failed(String),
}

impl Verdict {
    /// Whether this verdict fails the run.
    ///
    /// `Occupied` fails a write and a check alike. A projection whose path an
    /// authored document holds is a schema defect, and it is the one the marker
    /// rule exists to surface rather than to work around.
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Verdict::Occupied | Verdict::Differs | Verdict::Missing | Verdict::Failed(_)
        )
    }

    fn line(&self) -> String {
        match self {
            Verdict::Written => "written".to_string(),
            Verdict::Rewritten => "rewritten".to_string(),
            Verdict::Unchanged => "unchanged".to_string(),
            Verdict::Occupied => {
                "a file is there and it carries no generated-file marker, so this engine will \
                 not overwrite it"
                    .to_string()
            }
            Verdict::Differs => {
                "committed, and it is not what this corpus and this lock produce".to_string()
            }
            Verdict::Missing => "not committed, and this run would write it".to_string(),
            Verdict::Failed(error) => format!("not written: {error}"),
        }
    }
}

/// One line of the report.
#[derive(Clone, Debug)]
pub struct Wrote {
    pub path: String,
    pub kind: Kind,
    pub verdict: Verdict,
}

/// What a run of the verb did.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub wrote: Vec<Wrote>,
    pub unwritten: Vec<Unwritten>,
    /// Whether this run wrote anything, or only compared.
    pub checked: bool,
}

impl Report {
    pub fn has_errors(&self) -> bool {
        self.wrote.iter().any(|wrote| wrote.verdict.is_error())
    }

    /// The report, which states what it did not do as well as what it did.
    pub fn render(&self) -> String {
        let mut out = String::new();
        match self.checked {
            true => out.push_str("projections, held to regeneration\n"),
            false => out.push_str("projections\n"),
        }
        match self.wrote.is_empty() {
            true => out.push_str("  no declaration produced a file\n"),
            false => {
                for wrote in &self.wrote {
                    out.push_str(&format!("  {} {}\n", wrote.kind.name(), wrote.path));
                    out.push_str(&format!("    {}\n", wrote.verdict.line()));
                }
            }
        }
        if !self.unwritten.is_empty() {
            out.push_str("\nwhat this verb does not write, and why\n");
            for unwritten in &self.unwritten {
                out.push_str(&format!("  {} {}\n", unwritten.kind.name(), unwritten.at));
                out.push_str(&format!("    {}\n", unwritten.reason));
            }
        }
        out
    }
}

/// The comment syntax a path's format admits, which is where its marker goes.
enum Comment {
    /// Markdown, so an HTML comment.
    Html,
    /// YAML and anything else line-oriented.
    Hash,
    /// A format with no comment syntax. JSON is the one that reaches here.
    None,
}

fn comment_for(path: &str) -> Comment {
    match path.rsplit('.').next() {
        Some("md") | Some("markdown") => Comment::Html,
        Some("yml") | Some("yaml") | Some("toml") => Comment::Hash,
        Some("json") => Comment::None,
        _ => Comment::Hash,
    }
}

/// The marker line for a kind at a path, or `None` when the format carries no
/// comment.
///
/// A caller that gets `None` has an artifact this engine will not write into the
/// corpus, and the reason travels with the refusal rather than being discovered
/// at the moment the file is clobbered.
pub fn marker(kind: Kind, path: &str) -> Option<String> {
    let body = format!(
        "{MARKER} {}. `headwater generate` writes this file, and `headwater generate --check` \
         holds it. Edit the corpus, not this file.",
        kind.name()
    );
    match comment_for(path) {
        Comment::Html => Some(format!("<!-- {body} -->")),
        Comment::Hash => Some(format!("# {body}")),
        Comment::None => None,
    }
}

/// Whether a file's first line marks it as this engine's output.
///
/// The first line, and not anywhere in the file. A document that quotes the
/// marker while discussing it is an authored document, and this repository's own
/// specification is exactly such a document.
pub fn carries_marker(text: &str) -> bool {
    text.lines()
        .next()
        .is_some_and(|line| line.contains(MARKER))
}

/// Write the plan.
pub fn write(root: &Path, plan: &Plan) -> Report {
    run(root, plan, false)
}

/// Compare the plan against what is committed, and write nothing.
///
/// This is the same comparison `headwater taxonomy resolve --check` performs
/// over the lock: build what the sources produce, read what is committed, and
/// report the difference rather than repair it. The difference between the two
/// verbs is what they read. `resolve --check` reads the taxonomy sources, so it
/// answers "is this lock current". This reads the corpus *through* the lock, so
/// it answers "is this artifact current". A corpus edit moves this one and never
/// that one, which is why they are two flags and not one.
pub fn check(root: &Path, plan: &Plan) -> Report {
    run(root, plan, true)
}

fn run(root: &Path, plan: &Plan, checking: bool) -> Report {
    let mut report = Report {
        checked: checking,
        unwritten: plan.unwritten.clone(),
        ..Report::default()
    };
    for output in &plan.outputs {
        let path = root.join(&output.path);
        let committed = std::fs::read_to_string(&path).ok();
        let verdict = match (&committed, checking) {
            (Some(text), _) if text == &output.bytes => Verdict::Unchanged,
            // The marker decides before the difference does. A file that is
            // there and unmarked is authored, and reporting it as drift would
            // tell a reader to run the verb that destroys it.
            (Some(text), _) if !carries_marker(text) => Verdict::Occupied,
            (Some(_), true) => Verdict::Differs,
            (None, true) => Verdict::Missing,
            (Some(_), false) => put(&path, &output.bytes, Verdict::Rewritten),
            (None, false) => put(&path, &output.bytes, Verdict::Written),
        };
        report.wrote.push(Wrote {
            path: output.path.clone(),
            kind: output.kind,
            verdict,
        });
    }
    report
}

fn put(path: &Path, bytes: &str, ok: Verdict) -> Verdict {
    if let Some(parent) = path.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return Verdict::Failed(format!("cannot create {}: {error}", parent.display()));
        }
    }
    match std::fs::write(path, bytes) {
        Ok(()) => ok,
        Err(error) => Verdict::Failed(format!("cannot write {}: {error}", path.display())),
    }
}

/// The shelf that claimed a path, by the function that claimed it.
///
/// The census recorded this when it classified the row. Asking the same function
/// again is not a second implementation of the rule, and it keeps this crate
/// from destructuring a derivation whose shape belongs to the census.
pub(crate) fn shelf_of<'a>(
    path: &str,
    surface: &Surface<'a>,
) -> Option<&'a headwater_census::shelves::Shelf> {
    match shelf_for(path, surface.taxonomy()) {
        ShelfMatch::Matched { shelf, .. } => Some(shelf),
        ShelfMatch::Stopped(_) => None,
    }
}

/// A pointer per document, which is the form the reading order sorts.
pub(crate) fn pointers(surface: &Surface<'_>, documents: &[Document<'_>]) -> Vec<Pointer> {
    documents
        .iter()
        .map(|document| surface.pointer(document))
        .collect()
}
