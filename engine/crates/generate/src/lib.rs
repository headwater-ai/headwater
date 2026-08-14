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
//! Spec 6 names the kinds the engine implements,
//! [Q19](../../../../docs/spec/09-decisions.md) adds `transcription`, and
//! [spec 5](../../../../docs/spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document)
//! adds `probe_result`. Collecting them for the meta-schema showed that they do
//! not form one set. Nine are declarable: a taxonomy names the kind and the
//! output path, and
//! [principle 1](../../../../docs/spec/00-vision-and-scope.md#design-principles)
//! makes the path a schema decision. Two are engine-defined. Spec 4 makes the
//! register "engine-defined and non-optional", and
//! [Q14](../../../../docs/spec/09-decisions.md#q14--discovery-surface) fixes the
//! corpus descriptor at `.headwater/corpus.json`. Both for the same reason: a
//! reader who must consult the taxonomy to find an artifact already knows what
//! the artifact would tell them. A declaration of either would put a second copy
//! of one artifact at a path the engine did not fix.
//!
//! So [`Kind`] carries all eleven and [`Kind::declarable`] separates them. The
//! meta-schema's enum holds the nine.
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
//! - the path holds a file that carries the marker — overwrite it,
//! - the path holds anything else — refuse, and say which path.
//!
//! An authored document is never destroyed, which is the property spec 6 asks
//! the marker for.
//!
//! **Where the marker sits is the format's business, and it is found either
//! way.** A commented format carries it on the first line. JSON has no comment,
//! so the descriptor carries the same sentence in a top-level member, and
//! [`headwater_mark::carries_marker`] reads whichever of the two a path admits.
//! The alternative was to rule that a path the engine fixes needs no marker, and
//! that rule ends with this engine overwriting a file an adopter wrote by hand.
//!
//! The wording and the two rules live in `headwater-mark` rather than here,
//! because the census reads a marker too and it cannot depend on this crate.
//! That crate's header says why.
//!
//! # A marked file that no declaration writes
//!
//! The marker is a claim, and this verb is what tests it. A file inside the
//! corpus root that carries the marker is a file the census excuses from every
//! document check, so the line would otherwise be a self-service exemption: add
//! it to an authored document and nothing reports that document again. So a run
//! reads the census beside the plan, and a marked file that no output claims is
//! [`Orphaned`] — a stale artifact of a declaration that was removed or
//! repointed, or a marker somebody wrote by hand. Both are errors, and the
//! remedy for both is to delete the file or to restore the declaration.
//!
//! # No output states when it was generated, and the export verb is where the
//! # exception lives
//!
//! `--check` compares bytes. A timestamp inside a generated file makes every run
//! differ from the last one, so the gate reports drift on a corpus nobody
//! touched. Every marker this module writes is therefore a function of the kind
//! alone. Spec 6 asks a *filtered export* to state "when it was generated", and
//! that sentence and this one cannot both hold for an artifact that `--check`
//! covers.
//!
//! They hold for different artifacts, which is what settles it. **The time is
//! injected and never read**, exactly as
//! [spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
//! injects the clock into a check rather than let one call a syscall. A
//! committed export is held to regeneration and gets no time, so `plan` passes
//! none. An export that leaves the repository is the artifact spec 6 is talking
//! about, and `headwater export --at <date>` supplies its time. Same corpus,
//! same lock, same injected clock, byte-identical output, in both cases.

use headwater_census::census::Census;
use headwater_census::resolve::{shelf_for, ShelfMatch};
use headwater_census::shelves::DeclarationError;
use headwater_query::{Document, Pointer, Surface};
use headwater_yaml::value::{Mapping, Value};
use std::path::Path;

pub mod descriptor;
pub mod export;
pub mod identity;
mod probe_result;
pub mod profile;
mod shelf_index;
mod shelf_sections;

pub use profile::{Admission, Clause, Emitter, Filter, Grain, Profile};

/// What the descriptor states that neither the graph nor the census holds.
///
/// Plain strings, and never the lock or the consumer declaration as types. What
/// this crate needs from a resolution is four strings and a list of pairs, and
/// taking the types would give the generator a dependency on the resolver for
/// nothing. [`headwater_census::walk::Corpus::declared`] takes the same posture
/// over the same pairs, for the same reason.
#[derive(Clone, Debug, Default)]
pub struct Identity {
    /// The corpus root, as the consumer declaration wrote it.
    pub corpus_root: String,
    /// Each declared exclusion, with the reason it states.
    pub exclusions: Vec<(String, String)>,
    /// The taxonomy package the consumer took, and the version it pinned.
    pub package: String,
    pub version: String,
    /// The digest of the canonical taxonomy text: spec 6's "taxonomy lock
    /// hash".
    pub lock: String,
}

/// What a probe result is generated from, supplied by the caller for the reason
/// [`Identity`] is supplied: this module opens no file of its own.
///
/// The two members are two of the three inputs
/// [spec 5](../../../../docs/spec/05-ai-integration.md#a-probe-result-is-citable-because-each-of-its-three-inputs-is-a-committed-artifact)
/// makes a result a function of. The third is the grader version, which is the
/// version of the crate that evaluates them and is nothing a caller passes.
///
/// A caller that supplies neither gets a plan that says so. That is the whole
/// posture of [`Unwritten`]: a declaration that produced no file states why,
/// rather than being dropped.
#[derive(Clone, Debug, Default)]
pub struct Runs {
    /// Why nothing composed a selection to grade against, where a plan gave a
    /// reason.
    ///
    /// A plan that refuses returns from inside the loop that composes its
    /// selection, so it holds the probes it had read and none of the rest. A
    /// caller that read that list and dropped this one would grade over the
    /// probes a planner reached before it stopped, which is a denominator no
    /// document declares and which moves with the order the paths sort in.
    /// [`Runs::graded_against`] is what keeps the two apart.
    ///
    /// Private, with the two below, so that [`Runs::graded_against`] is the
    /// only writer of any of them. The defect this type was introduced to fix
    /// was a caller assigning `plan.selected` here and dropping the refusal
    /// beside it, and a private field is what makes that assignment
    /// unwritable rather than merely wrong.
    refusal: Option<headwater_probe::plan::Refusal>,
    /// The probes of this corpus, as `headwater probe plan` composed them.
    ///
    /// From the plan and never from the transcript, because a selection read
    /// out of a recorded run would let a transcript name its own denominator.
    /// It is the whole of what a plan composed or it is empty, and never a
    /// part: see [`Runs::graded_against`].
    selected: Vec<headwater_probe::plan::Selected>,
    /// The digest over the identifiers of [`Runs::selected`], as the plan
    /// computed it.
    ///
    /// Carried rather than recomputed here, because a second derivation of one
    /// value is two values that can disagree, and the one they would disagree
    /// about is the one a transcript is compared against.
    selection: String,
    /// The bytes of every committed transcript.
    ///
    /// A census row carries what it parsed rather than the source it parsed,
    /// and the intake reads fenced blocks out of the source. So the caller that
    /// walked the tree supplies them.
    pub transcripts: Vec<Transcript>,
}

impl Runs {
    /// Hold these transcripts to the selection one plan composed.
    ///
    /// The plan carries a selection and a refusal at once, and this is where
    /// the two stop being carried at once. A refusal that
    /// [`headwater_probe::plan::Refusal::stops_a_grade`] names takes the
    /// selection with it, so this type never holds the part of a selection a
    /// planner managed before it gave up. Every caller that grades goes through
    /// here, so there is one place the rule lives.
    pub fn graded_against(&mut self, plan: &headwater_probe::Plan) {
        match plan.gradable() {
            Err(refusal) => self.refusal = Some(refusal.clone()),
            Ok(selected) => {
                self.selected = selected.to_vec();
                self.selection = plan.selection.clone();
            }
        }
    }

    /// Why nothing is graded here, where a plan gave a reason.
    pub fn refusal(&self) -> Option<&headwater_probe::plan::Refusal> {
        self.refusal.as_ref()
    }

    /// The whole selection a plan composed, or nothing at all.
    pub fn selected(&self) -> &[headwater_probe::plan::Selected] {
        &self.selected
    }
}

/// One committed transcript: where it is, and the bytes the caller read.
#[derive(Clone, Debug)]
pub struct Transcript {
    /// Relative to the repository root, in the form the census wrote it.
    pub path: String,
    pub source: String,
}

/// A projection kind.
///
/// Eleven of them. Nine a taxonomy declares, and two the engine defines. See
/// the module header for why that split exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ShelfIndex,
    /// One heading, and therefore one anchor, for each document on a shelf.
    ShelfSections,
    RelationView,
    AgentRules,
    SiteNav,
    GraphExport,
    Template,
    Transcription,
    /// Spec 5: the verdicts the grader returned over one committed transcript,
    /// written as a document of the corpus.
    ProbeResult,
    /// Spec 4: the register, engine-defined and non-optional.
    CoverageReport,
    /// Q14: `.headwater/corpus.json`, engine-defined and non-optional.
    CorpusDescriptor,
}

impl Kind {
    /// The name as a taxonomy writes it, and as a report prints it.
    pub fn name(self) -> &'static str {
        match self {
            Kind::ShelfIndex => "shelf_index",
            Kind::ShelfSections => "shelf_sections",
            Kind::RelationView => "relation_view",
            Kind::AgentRules => "agent_rules",
            Kind::SiteNav => "site_nav",
            Kind::GraphExport => "graph_export",
            Kind::Template => "template",
            Kind::Transcription => "transcription",
            Kind::ProbeResult => "probe_result",
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
    pub const DECLARABLE: [Kind; 9] = [
        Kind::ShelfIndex,
        Kind::ShelfSections,
        Kind::RelationView,
        Kind::AgentRules,
        Kind::SiteNav,
        Kind::GraphExport,
        Kind::Template,
        Kind::Transcription,
        Kind::ProbeResult,
    ];

    fn parse(text: &str) -> Option<Kind> {
        Kind::DECLARABLE
            .into_iter()
            .find(|kind| kind.name() == text)
    }
}

/// What a generated document declares about itself: the `identity` block.
///
/// Two scalars, and the block is closed at two. See
/// [`crate::identity`] for the argument, and the meta-schema for the same
/// argument in the place a taxonomy author reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredIdentity {
    /// The identifier the generated document carries, which is the identifier
    /// every edge into it already names.
    pub id: String,
    /// The kind the generated document resolves to. A kind, and never a facet:
    /// what the front matter needs in order to resolve it is the shelf's
    /// business and this engine reads it from the shelf.
    pub kind: String,
}

/// One entry of the `projections` block.
#[derive(Clone, Debug)]
pub struct Declaration {
    pub kind: Kind,
    /// The shelves this projection covers. Empty means every shelf, which is
    /// what a declaration with no `for` says.
    pub shelves: Vec<String>,
    pub output: String,
    /// The profile this entry belongs to, and the half of it this entry states.
    pub membership: profile::Membership,
    /// What the written file is, for a projection whose output is a document of
    /// the corpus. `None` for a projection that writes a list beside the corpus,
    /// which is every declaration that shipped before this member existed.
    pub identity: Option<DeclaredIdentity>,
}

impl Declaration {
    /// The emitter target this entry writes through.
    ///
    /// An entry that names no `format` takes the native export. Spec 6 makes
    /// the native graph JSON the one an adopter gets with no external consumer
    /// at all, so it is the target a declaration falls back to rather than an
    /// error a declaration has to avoid.
    pub fn emitter(&self) -> Emitter {
        self.membership.emitter.unwrap_or(Emitter::Json)
    }
}

/// The `projections` block of a resolved taxonomy.
#[derive(Clone, Debug, Default)]
pub struct Projections {
    pub declared: Vec<Declaration>,
    /// Every declared export profile, in the order the entries first name each
    /// one. Assembled by [`profile::group`], which refuses a profile that two
    /// entries describe differently.
    pub profiles: Vec<Profile>,
}

impl Projections {
    /// The profile of a name, which is what `--profile` selects.
    pub fn profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|one| one.name == name)
    }
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
            let membership = profile::Membership::read(body, index, item.span, &mut errors);
            let identity = identity::read(body, kind, index, item.span, &mut errors);
            out.declared.push(Declaration {
                kind,
                shelves,
                output: output.text.clone(),
                membership,
                identity,
            });
        }
        // The grouping runs over what read, so a taxonomy with one bad entry
        // reports that entry rather than a disagreement between it and the rest.
        if errors.is_empty() {
            let memberships: Vec<profile::Membership> = out
                .declared
                .iter()
                .map(|declaration| declaration.membership.clone())
                .collect();
            match profile::group(&memberships) {
                Ok(profiles) => out.profiles = profiles,
                Err(found) => errors.extend(found),
            }
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

/// A file inside the corpus root that carries the marker and that no
/// declaration writes.
///
/// The marker is a claim that this engine wrote the file, and the census
/// believes it: a marked file is excused from every document check, because the
/// content of a generated file is a function of its emitter rather than of its
/// author. So the claim has to be tested somewhere, and this is the only place
/// that can test it, because testing it needs the plan.
///
/// Two things reach here and the remedy differs. A **stale artifact** is the
/// common one: a declaration was removed or its output path was repointed, and
/// the file it used to write stayed behind. Nothing regenerates it, so it says
/// whatever the corpus said on the day it was last written. A **hand-written
/// marker** is the other, and it is the one that matters more: the line is one
/// an author could otherwise add to any document to exempt it from every check,
/// silently and permanently. Reported, it exempts nothing.
#[derive(Clone, Debug)]
pub struct Orphaned {
    pub path: String,
    /// The kind the marker claims, when it claims one.
    pub kind: Option<String>,
    /// Why the declaration that writes this path produced nothing on this run,
    /// where one names the path and declined it.
    ///
    /// The two states need two remedies and printing one of them for both is
    /// how a reader is told to destroy evidence. A file no declaration writes
    /// is deleted. A file a declaration declined to rewrite is the last thing
    /// an earlier run derived, and the remedy is whatever the reason names.
    pub declined: Option<String>,
}

impl Orphaned {
    fn line(&self) -> String {
        let claim = match &self.kind {
            Some(kind) => format!("carries a `{kind}` generated-file marker"),
            None => "carries a generated-file marker that names no kind".to_string(),
        };
        match &self.declined {
            Some(reason) => format!(
                "{claim}, and the declaration that writes it produced nothing on this run: \
                 {reason}. So these bytes are what an earlier run derived and this corpus no \
                 longer derives them. Fix what the reason names rather than delete the file"
            ),
            None => format!(
                "{claim}, and no declaration writes this path. Nothing regenerates this file and \
                 no check reads it. Delete it, or restore the declaration that wrote it"
            ),
        }
    }
}

/// Everything one run would write, and everything it would not.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub outputs: Vec<Output>,
    pub unwritten: Vec<Unwritten>,
    /// Marked files inside the corpus root that no output claims. Empty for a
    /// plan that covers a subset of the declarations, because a subset cannot
    /// tell a file it does not write from a file nobody writes: see
    /// [`export_plan`].
    pub orphaned: Vec<Orphaned>,
}

/// What the engine writes with no declaration at all.
///
/// One of the two engine-defined kinds is emitted and one is not. The register
/// is listed here with its reason, so that a run states the whole of what a
/// projection layer owes rather than the part that is built. Spec 4 calls the
/// register "a projection like any other" and asks this verb to hold it to
/// regeneration; building the rest of the verb is what showed why it cannot
/// yet. The descriptor had the same entry until it acquired an emitter, and the
/// difference between the two is the clock.
fn engine_defined() -> Vec<Unwritten> {
    vec![Unwritten {
        at: "the register".to_string(),
        kind: Kind::CoverageReport,
        reason: "its content is a function of the clock as well as of the corpus and the lock, \
                 because a migration task lapses and a suppression expires on a date. A committed \
                 copy would fail this check on a morning when nothing changed. Spec 13 carries it"
            .to_string(),
    }]
}

/// Build the plan: what every declaration and the engine itself would write.
///
/// The declarations come from the lock and the [`Identity`] from the lock and
/// the consumer declaration. Both are inputs rather than reads, so that one
/// plan is a function of its arguments and two plans over one tree hold the
/// same bytes.
pub fn plan(
    surface: &Surface<'_>,
    census: &Census,
    projections: &Projections,
    identity: &Identity,
    runs: &Runs,
) -> Plan {
    let mut plan = Plan::default();
    for declaration in &projections.declared {
        match declaration.kind {
            Kind::ShelfIndex => shelf_index::emit(surface, census, declaration, &mut plan),
            Kind::ShelfSections => shelf_sections::emit(surface, census, declaration, &mut plan),
            Kind::GraphExport => graph_export(surface, projections, declaration, &mut plan),
            Kind::ProbeResult => {
                probe_result::emit(surface, census, declaration, runs, identity, &mut plan)
            }
            other => plan.unwritten.push(Unwritten {
                at: declaration.output.clone(),
                kind: other,
                reason: unbuilt(other).to_string(),
            }),
        }
    }
    descriptor::emit(surface, identity, projections, &mut plan);
    plan.unwritten.extend(engine_defined());
    // Every declaration has had its turn, so the output set is complete and a
    // marked file outside it is a marked file nothing writes.
    plan.orphaned = orphaned(census, &plan.outputs, &plan.unwritten);
    plan
}

/// Marked files the census found that no output in this plan claims.
///
/// The census is the input rather than a second walk, for the reason every
/// phase after it reads it: two walks of one tree can disagree, and this one
/// would disagree by reporting a file as unwritten that the other never saw.
fn orphaned(census: &Census, outputs: &[Output], unwritten: &[Unwritten]) -> Vec<Orphaned> {
    census
        .rows
        .iter()
        .filter_map(|row| match &row.outcome {
            headwater_census::census::Outcome::Generated { projection, .. }
                if !outputs.iter().any(|output| output.path == row.path) =>
            {
                Some(Orphaned {
                    path: row.path.clone(),
                    kind: projection.clone(),
                    declined: unwritten
                        .iter()
                        .find(|unwritten| unwritten.at == row.path)
                        .map(|unwritten| unwritten.reason.clone()),
                })
            }
            _ => None,
        })
        .collect()
}

/// The plan for `headwater export`: the declared exports, and nothing else.
///
/// The same emitter, the same census and the same marker rule that `plan` uses
/// for a `graph_export`, over the subset one profile names. `selected` is what
/// `--profile` supplies; `None` takes every declared profile, which is spec 6's
/// rule that "with no profile named, the engine writes every declared profile,
/// so a filtered audience is never omitted by accident".
///
/// A profile the taxonomy does not declare is an error rather than an empty
/// plan. An empty plan reports success over nothing, and a caller who mistyped
/// a profile name would read that as an export.
pub fn export_plan(
    surface: &Surface<'_>,
    projections: &Projections,
    selected: Option<&str>,
) -> Result<Plan, String> {
    if let Some(name) = selected {
        if projections.profile(name).is_none() {
            return Err(match projections.profiles.is_empty() {
                true => format!("no profile is called `{name}`, and this taxonomy declares none"),
                false => format!(
                    "no profile is called `{name}`. This taxonomy declares {}",
                    projections
                        .profiles
                        .iter()
                        .map(|profile| format!("`{}`", profile.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }
    }
    let mut plan = Plan::default();
    for declaration in &projections.declared {
        if declaration.kind != Kind::GraphExport {
            continue;
        }
        if selected.is_some_and(|name| name != declaration.membership.name) {
            continue;
        }
        graph_export(surface, projections, declaration, &mut plan);
    }
    Ok(plan)
}

/// One declared `graph_export`, written through the emitter its profile names.
///
/// Spec 6: "A graph export is a projection like the others. The taxonomy
/// declares its output path, so whether an export is committed is a schema
/// decision and not an engine default. A declared export is held to regeneration
/// by `generate --check`, exactly as a shelf index is."
///
/// Two things stop a declaration from producing a file, and both are reported
/// rather than dropped. An emitter that this release does not build is one. A
/// census with an omission that no loss reason covers is the other, and that one
/// is a defect in this engine rather than in the taxonomy. Writing the file
/// anyway would commit the artifact whose trustworthiness the census exists to
/// establish.
///
/// No generation time is injected here. An artifact that `--check` compares by
/// byte cannot carry a clock reading, and `headwater export --at` is where the
/// artifact that leaves the repository gets one.
fn graph_export(
    surface: &Surface<'_>,
    projections: &Projections,
    declaration: &Declaration,
    plan: &mut Plan,
) {
    let name = declaration.membership.name.clone();
    let Some(profile) = projections.profile(&name) else {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::GraphExport,
            reason: format!("names the profile `{name}`, and no entry assembled one"),
        });
        return;
    };
    match export::emit(surface, profile, declaration.emitter(), None) {
        Ok(emission) if emission.census.is_defective() => plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::GraphExport,
            reason: format!(
                "the projection census found {} node and {} edge omissions that no declared \
                 loss reason covers, which is a defect in this emitter. Run `headwater export \
                 --profile {name}` to read them",
                emission.census.nodes.unaccounted, emission.census.edges.unaccounted
            ),
        }),
        Ok(emission) => plan.outputs.push(Output {
            path: declaration.output.clone(),
            kind: Kind::GraphExport,
            bytes: emission.bytes,
        }),
        Err(refusal) => plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::GraphExport,
            reason: refusal.reason(),
        }),
    }
}

/// Why a declarable kind produces nothing yet, and who owns it.
///
/// Named rather than ignored, and each one names the issue that owns it. This is
/// the posture the binary already takes over `headwater query`: a surface the
/// specification lists and this engine does not implement is reported as that,
/// and never as an empty result.
fn unbuilt(kind: Kind) -> &'static str {
    match kind {
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
        Kind::ShelfIndex
        | Kind::ShelfSections
        | Kind::GraphExport
        | Kind::ProbeResult
        | Kind::CoverageReport
        | Kind::CorpusDescriptor => "this engine emits it",
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
    /// Marked files no declaration writes. An error in both directions of the
    /// verb: `generate` does not delete files, so writing the plan does not
    /// clear one.
    pub orphaned: Vec<Orphaned>,
    /// Whether this run wrote anything, or only compared.
    pub checked: bool,
}

impl Report {
    pub fn has_errors(&self) -> bool {
        self.wrote.iter().any(|wrote| wrote.verdict.is_error()) || !self.orphaned.is_empty()
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
        if !self.orphaned.is_empty() {
            out.push_str("\nmarked, and written by no declaration\n");
            for orphaned in &self.orphaned {
                out.push_str(&format!("  {}\n", orphaned.path));
                out.push_str(&format!("    {}\n", orphaned.line()));
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
        orphaned: plan.orphaned.clone(),
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
            (Some(text), _) if !headwater_mark::carries_marker(&output.path, text) => {
                Verdict::Occupied
            }
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
