// SPDX-License-Identifier: Apache-2.0
//! `taxonomy audit`: the schema measured against a corpus.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit)
//! separates this from `taxonomy validate` on the standard TBox/ABox line.
//! `validate` decides the schema alone and it gates. This measures the schema
//! *against documents*, its findings are about the taxonomy rather than about
//! any document, and it gates nothing.
//!
//! # One declared bar, and every other reading is a distribution
//!
//! Spec 6 says the findings here are "advisory by construction, because a young
//! or small corpus fails differentiation for reasons that are not defects". The
//! sharper statement this crate found while building the readings is that
//! almost none of them has a bar at all. Nothing in a taxonomy says how narrow
//! a facet may get before it separates nothing, or how low a capture rate may
//! fall before a relation is unmaintained. A number invented here would be a
//! verdict this engine derived from nothing a corpus declared, which is the one
//! thing [spec 12](../../../../docs/spec/12-check-layer.md) rules out of a
//! check.
//!
//! So the split is by what is declared. `stale_after_days` on the facet in the
//! `freshness` role is a number the taxonomy states, and it is the only one, so
//! it is the only reading here that produces a finding. Every other reading is
//! a distribution printed with its population beside it, and the report says
//! that no declaration turns it into a verdict.
//! [OBL-repo-0119](../../../../docs/obligations/0119-an-audit-reading-carries-no-declared-bar-so-a-distribution-cannot-become-a-finding.md)
//! holds the gap.
//!
//! # The grain of a creator reading is a relation
//!
//! [Q4](../../../../docs/decisions/0004-relation-storage.md) keeps `created_by`
//! on the relation type. A scaffolded `supersedes` and a hand-typed one are one
//! string on disk, so a reading presented per edge instance would imply a
//! per-edge provenance that nothing records. Every row of
//! [`Creators`] is therefore a relation, and an aggregate over a creator names
//! the relations it aggregates.
//!
//! # What an edge's freshness is read from, and why that is not a proxy
//!
//! An edge lives in the front matter of the document that declares it. The
//! facet in the `freshness` role records when somebody last verified that
//! document, front matter included, so the last verification of a half is the
//! last verification of the document that carries it. The reading is stated in
//! those words — "halves on a document whose declared freshness has expired" —
//! rather than as an edge-decay figure, because the second one would claim a
//! per-edge verification record that no corpus keeps.

use headwater_census::census::{Census, Outcome, Row};
use headwater_census::resolve::ShelfMatch;
use headwater_census::shelves::{ShelfBody, Taxonomy};
use headwater_check::context::Date;
use headwater_check::Shape;
use headwater_graph::declarations::{Declarations, Direction, Relation};
use headwater_graph::Graph;
use headwater_yaml::Mapping;
use std::collections::{BTreeMap, BTreeSet};

pub mod render;

/// Spec 2's closed set of edge creators, in the order that document lists them.
///
/// The set is here rather than derived from the relations a taxonomy declares,
/// and that is the whole point of the creator reading. A creator that no
/// relation declares is an arm of a comparison that this corpus cannot supply,
/// and a report built only from the values in use would omit exactly the fact a
/// reader needs.
pub const CREATORS: [&str; 6] = ["author", "scaffold", "generator", "hook", "agent", "import"];

/// What the audit ran against, printed before any number it produced.
#[derive(Clone, Debug)]
pub struct Subject {
    pub package: String,
    pub version: String,
    pub lock: String,
    pub now: Date,
}

/// One relation, measured against the corpus.
#[derive(Clone, Debug)]
pub struct RelationReading {
    pub name: String,
    pub family: Option<String>,
    pub created_by: Option<String>,
    /// Edge halves whose relation type is this one, in either direction.
    pub halves: usize,
    /// Classified documents whose kind may sit at the declaring end.
    pub eligible: usize,
    /// Eligible documents that declared at least one half under this relation's
    /// own name. The inverse name is excluded on purpose: a document that
    /// declares `superseded_by` is at the far end of a `supersedes` edge, and
    /// counting it as participation would report the target's authoring as the
    /// source's.
    pub participating: usize,
    /// Halves whose declaring document is past the declared freshness window.
    pub expired: usize,
    /// Halves whose declaring document states no readable freshness date, so
    /// no window applies to them and they are counted nowhere else.
    pub undated: usize,
}

impl RelationReading {
    /// Participation as a percentage, and `None` where nothing is eligible.
    ///
    /// A rate over an empty population is not a rate, and a zero printed in its
    /// place would read as a relation nobody maintains.
    pub fn capture(&self) -> Option<f64> {
        match self.eligible {
            0 => None,
            eligible => Some(100.0 * self.participating as f64 / eligible as f64),
        }
    }
}

/// The creator reading: every value of spec 2's closed set, in use or not.
#[derive(Clone, Debug)]
pub struct Creators {
    pub by_creator: Vec<(String, Vec<RelationReading>)>,
    /// Relations that declare no creator at all. The meta-schema requires one,
    /// so this is empty for any taxonomy `taxonomy validate` accepted, and it
    /// is carried rather than assumed away.
    pub undeclared: Vec<RelationReading>,
}

impl Creators {
    /// The creators of the closed set that no relation of this taxonomy
    /// declares.
    pub fn absent(&self) -> Vec<&'static str> {
        CREATORS
            .iter()
            .copied()
            .filter(|creator| {
                !self
                    .by_creator
                    .iter()
                    .any(|(name, readings)| name == creator && !readings.is_empty())
            })
            .collect()
    }
}

/// One relation family, and what the corpus put under it.
#[derive(Clone, Debug)]
pub struct FamilyReading {
    pub family: String,
    pub relations: usize,
    pub halves: usize,
    /// Relations that declare a nuclearity the family's default contradicts,
    /// as `(relation, declared, default)`.
    pub overrides: Vec<(String, String, String)>,
    /// Relations that declare the nuclearity the family already gives.
    pub restatements: Vec<String>,
}

/// One facet, measured over the documents that carry it.
#[derive(Clone, Debug)]
pub struct FacetReading {
    pub name: String,
    pub role: Option<String>,
    pub required: bool,
    /// Values the declaration lists, and 0 where it lists none.
    pub declared_values: usize,
    /// Classified documents that carry the facet.
    pub carried_by: usize,
    /// Distinct values in use.
    pub distinct: usize,
    /// The most common value and how many documents carry it.
    pub commonest: Option<(String, usize)>,
}

impl FacetReading {
    /// The share of the carrying documents that the commonest value takes.
    pub fn concentration(&self) -> Option<f64> {
        let (_, count) = self.commonest.as_ref()?;
        match self.carried_by {
            0 => None,
            carried => Some(100.0 * *count as f64 / carried as f64),
        }
    }
}

/// A pair of facets where the value of one fixes the value of the other.
///
/// Orthogonality read as a one-way functional dependency rather than as a
/// correlation. Two facets are orthogonal when each combination of their values
/// is reachable, and the observable failure is the one where knowing the first
/// value leaves no choice about the second: the second facet then carries no
/// information the first does not already carry.
#[derive(Clone, Debug)]
pub struct Dependence {
    pub determines: String,
    pub determined: String,
    /// Documents carrying both facets.
    pub over: usize,
}

/// A shelf that admits several kinds, and how its documents divided.
#[derive(Clone, Debug)]
pub struct ShelfReading {
    pub shelf: String,
    pub discriminator: String,
    pub documents: usize,
    /// Declared kinds and the count of each, including the ones at zero.
    pub kinds: Vec<(String, usize)>,
}

/// One shelf that declares a layout, measured against the files on it.
///
/// [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#templates-and-scaffolding)
/// makes a layout a convention at birth: `headwater new` writes the name and no
/// check reads it afterwards, so a rename answers to nothing
/// ([OBL-repo-0106](../../../../docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md)).
/// This reading is what says how far the convention has drifted, and it is here
/// rather than in that record because a figure a document holds is a figure
/// nobody re-derives.
///
/// The name is rendered through [`headwater_scaffold::render_layout`], which is
/// the function `headwater new` writes a file name with. A second renderer here
/// would report adherence to a template that the scaffolder does not follow.
#[derive(Clone, Debug)]
pub struct LayoutReading {
    pub shelf: String,
    pub layout: String,
    /// Documents the census placed on this shelf and resolved to a kind.
    pub identified: usize,
    /// Of those, the ones every placeholder of the layout could be filled for.
    /// A document the layout cannot be rendered for is measured nowhere, and it
    /// never joins the denominator below.
    pub measured: usize,
    /// Of the measured, the ones whose file name is the name the layout renders.
    pub renders: usize,
    /// What this run could make of the shelf. See [`Supply`]: an arm of this
    /// population that cannot be measured has a location, and folding it into
    /// the denominator would report a schema gap as an authoring failure.
    pub adherence: Supply,
}

/// Time spent in the current state, by state.
#[derive(Clone, Debug)]
pub struct DwellReading {
    pub state: String,
    pub documents: usize,
    /// Days from the state-entry date to the audit's date, sorted.
    pub days: Vec<i64>,
    /// Documents in this state that state no readable entry date.
    pub undated: usize,
}

impl DwellReading {
    pub fn median(&self) -> Option<i64> {
        match self.days.len() {
            0 => None,
            n => Some(self.days[n / 2]),
        }
    }
}

/// The closed warrant set of [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires),
/// in the order that document's table lists it.
///
/// Written out here rather than derived from the values in use, for the reason
/// [`CREATORS`] is written out. A warrant that no document of a corpus carries
/// is an arm of the promotion reading that this corpus cannot fill, and a
/// report built from the values in use omits exactly the arm a reader needs.
pub const WARRANTS: [&str; 4] = ["accepted", "regenerated", "transcribed", "asserted"];

/// The two warrants the engine derives from the generated-file marker.
///
/// Spec 3: "The engine derives `regenerated` from the marker, and never from a
/// declaration." A generated document that declares one would state a fact a
/// hand edit can falsify. So a zero on either row is a property of this engine
/// rather than of the authoring, and the report says which.
pub const DERIVED: [&str; 2] = ["regenerated", "transcribed"];

/// The role a facet would take to record the state a document left.
///
/// No taxonomy declares it, because spec 2's closed role registry holds no such
/// role. The name is here so that the transition-continuity wait names what
/// would end it, rather than describing an absence.
pub const STATE_LEFT: &str = "state_left";

/// One warrant value, and the documents that state it.
#[derive(Clone, Debug)]
pub struct WarrantReading {
    pub value: String,
    /// Classified documents whose provenance block states this value.
    pub stated: usize,
    /// Whether the closed set of spec 3 holds this value. A row where this is
    /// false is a value a corpus invented, and no check reads a warrant, so
    /// this reading is the only place one is reported.
    pub closed_set: bool,
    /// Whether the engine derives this value from the generated-file marker
    /// rather than reading it from a declaration.
    pub derived: bool,
}

/// The warrant distribution: the closed set in full, and what a corpus wrote.
#[derive(Clone, Debug)]
pub struct Warrants {
    /// Every value of the closed set in spec 3's order, then every value a
    /// document stated that the set does not hold.
    pub readings: Vec<WarrantReading>,
    /// Classified documents that state no warrant at all. Spec 3 makes the
    /// value required, so this is a population and never a default.
    pub unstated: usize,
    /// Rows the census reported as generated, whose warrant the engine reads
    /// off the marker rather than out of a declaration.
    pub generated: usize,
}

impl Warrants {
    /// The `asserted` population, which is what a promotion rate divides by.
    pub fn asserted(&self) -> usize {
        self.of("asserted")
    }

    /// The documents that state one named value.
    pub fn of(&self, value: &str) -> usize {
        self.readings
            .iter()
            .find(|reading| reading.value == value)
            .map_or(0, |reading| reading.stated)
    }

    /// Values a document stated that the closed set does not hold.
    pub fn outside(&self) -> Vec<&WarrantReading> {
        self.readings
            .iter()
            .filter(|reading| !reading.closed_set)
            .collect()
    }

    /// Classified documents that state a warrant, whatever it is.
    pub fn stated(&self) -> usize {
        self.readings.iter().map(|reading| reading.stated).sum()
    }
}

/// What one prerequisite of a reading looked like on this run.
///
/// The four arms are four different things to do about it, which is why they
/// are four arms and not a boolean. A declaration ends the second, authoring
/// ends the third, and a decision this engine has not taken ends the fourth.
#[derive(Clone, Debug)]
pub enum Supply {
    /// The corpus supplied it, and the text says what the run counted.
    Supplied(String),
    /// Nothing declares it, so no document of this corpus could carry one. The
    /// location of the absence is the schema.
    Undeclared(String),
    /// Declared, and no document of this corpus carries one. The location of
    /// the absence is the corpus.
    Unauthored(String),
    /// The input is designed and nothing supplies it. A build ends this wait,
    /// and the location is this engine rather than the schema or the corpus.
    ///
    /// It is not the same as an input this engine has decided not to take. The
    /// first draft of this type carried that arm instead, and it was the wrong
    /// one for the only wait that reached it: `needs_prior` is designed in
    /// [spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)
    /// and unimplemented in [`headwater_check::scope`], which is a build rather
    /// than a decision. No wait here reaches an arm that no work of any kind
    /// could end, so this type declares none.
    Unbuilt(String),
}

impl Supply {
    /// Whether this run found what the reading needs.
    pub fn met(&self) -> bool {
        matches!(self, Supply::Supplied(_))
    }

    /// Where the absence lives, in two words, or that it does not.
    pub fn located(&self) -> &'static str {
        match self {
            Supply::Supplied(_) => "supplied",
            Supply::Undeclared(_) => "nothing declares it",
            Supply::Unauthored(_) => "nothing authored one",
            Supply::Unbuilt(_) => "designed and unbuilt",
        }
    }

    /// What the run found, in the words the report prints.
    pub fn says(&self) -> &str {
        match self {
            Supply::Supplied(text)
            | Supply::Undeclared(text)
            | Supply::Unauthored(text)
            | Supply::Unbuilt(text) => text,
        }
    }
}

/// One thing a reading needs before it can be taken.
#[derive(Clone, Debug)]
pub struct Need {
    pub needs: &'static str,
    pub supply: Supply,
}

/// One reading that spec 6 names and this verb does not take.
///
/// Every prerequisite is evaluated against the corpus of the run that prints
/// it. The array this replaced was three string literals, so the verb asserted
/// facts about corpus content at compile time and never measured them. One of
/// the three went false while the corpus moved underneath it and nothing
/// reported that, which is the defect this type exists to make impossible: a
/// wait ends when a corpus supplies what it waits on, and never when somebody
/// edits a string.
#[derive(Clone, Debug)]
pub struct Waiting {
    pub reading: &'static str,
    pub needs: Vec<Need>,
}

impl Waiting {
    /// Whether this reading still waits on anything.
    pub fn waits(&self) -> bool {
        self.needs.iter().any(|need| !need.supply.met())
    }
}

/// Everything one run of the audit measured.
#[derive(Clone, Debug)]
pub struct Audit {
    pub subject: Subject,
    /// Documents the graph holds, which is what every reading below divides by.
    ///
    /// It is the census's classified rows **plus** each generated file that
    /// declared an identity, because both are nodes and an edge may reach
    /// either. That is one more than the `classified` figure a check run
    /// prints, and the header states both so that a reader who re-derives the
    /// number from a check report does not find a discrepancy and no account
    /// of it.
    pub documents: usize,
    pub typed: usize,
    pub generated: usize,
    pub halves: usize,
    /// The declared freshness window, and `None` where no facet declares one.
    pub freshness: Option<(String, i64)>,
    pub creators: Creators,
    pub families: Vec<FamilyReading>,
    pub facets: Vec<FacetReading>,
    pub dependences: Vec<Dependence>,
    pub shelves: Vec<ShelfReading>,
    /// Every shelf that declares a layout, in the taxonomy's shelf order. A
    /// shelf that declares none is absent rather than reported at zero: nothing
    /// names its files, so there is no name to disagree with.
    pub layouts: Vec<LayoutReading>,
    pub dwell: Vec<DwellReading>,
    pub warrants: Warrants,
    /// The readings this verb names and does not take, as this run found their
    /// prerequisites. Never a constant: see [`Waiting`].
    pub waiting: Vec<Waiting>,
}

impl Audit {
    /// The one finding class this verb produces: a relation whose halves sit on
    /// documents past the freshness window the taxonomy declares.
    ///
    /// It is a finding rather than a distribution because the bar is declared.
    /// The subject is the relation, so the message is about the schema, which
    /// is what spec 6 requires of every finding here.
    pub fn findings(&self) -> Vec<&RelationReading> {
        let mut out: Vec<&RelationReading> = self
            .creators
            .by_creator
            .iter()
            .flat_map(|(_, readings)| readings)
            .chain(self.creators.undeclared.iter())
            .filter(|reading| reading.expired > 0)
            .collect();
        out.sort_by(|a, b| b.expired.cmp(&a.expired).then(a.name.cmp(&b.name)));
        out
    }
}

/// Take the audit.
///
/// Every input is already built: the census carries the parsed documents, and
/// the graph carries the edges with the creator each relation declared. Nothing
/// here opens a file, for the reason the graph build states — two passes over
/// one corpus can disagree, and a report that disagreed with the run beside it
/// would be a second account of the same tree.
pub fn take(
    subject: Subject,
    census: &Census,
    graph: &Graph,
    taxonomy: &Taxonomy,
    shape: &Shape,
    relations: &Declarations,
    resolved: &Mapping,
) -> Audit {
    let classified: Vec<&Row> = census
        .rows
        .iter()
        .filter(|row| row.outcome.node().is_some())
        .collect();
    let freshness = freshness_window(shape);
    let now = subject.now;
    let creators = creators(
        &classified,
        graph,
        shape,
        relations,
        freshness.as_ref(),
        now,
    );

    let warrants = warrants(&classified);
    let waiting = waiting(&classified, graph, shape, &warrants);

    Audit {
        subject,
        documents: classified.len(),
        typed: classified
            .iter()
            .filter(|row| matches!(row.outcome, Outcome::Typed { .. }))
            .count(),
        generated: classified
            .iter()
            .filter(|row| matches!(row.outcome, Outcome::Generated { .. }))
            .count(),
        halves: graph.edges.len(),
        creators,
        families: families(graph, relations),
        facets: facets(&classified, shape),
        dependences: dependences(&classified, shape),
        shelves: shelves(&classified, taxonomy),
        layouts: layouts(&classified, graph, taxonomy, shape, resolved),
        dwell: dwell(&classified, shape, now),
        freshness,
        warrants,
        waiting,
    }
}

/// The warrant every classified document states, over the closed set in full.
///
/// The provenance block belongs to the engine, so the value is read through
/// [`headwater_doc::warrant`] rather than by opening the block here. A second
/// reader of it would be a second answer to the question that decides whether a
/// pointer states its warrant out loud.
fn warrants(classified: &[&Row]) -> Warrants {
    let mut stated: BTreeMap<String, usize> = BTreeMap::new();
    let mut unstated = 0;
    for row in classified {
        match row
            .document
            .as_ref()
            .and_then(|document| headwater_doc::warrant(&document.facets))
        {
            Some(value) => *stated.entry(value.to_string()).or_default() += 1,
            None => unstated += 1,
        }
    }

    // The closed set first and whole, so a value nobody wrote still has a row.
    let mut readings: Vec<WarrantReading> = WARRANTS
        .iter()
        .map(|value| WarrantReading {
            value: (*value).to_string(),
            stated: stated.remove(*value).unwrap_or(0),
            closed_set: true,
            derived: DERIVED.contains(value),
        })
        .collect();
    // Then whatever else a document stated. No check reads a warrant, so a
    // value outside the closed set reaches no other report in this engine.
    readings.extend(stated.into_iter().map(|(value, stated)| WarrantReading {
        value,
        stated,
        closed_set: false,
        derived: false,
    }));

    Warrants {
        readings,
        unstated,
        generated: classified
            .iter()
            .filter(|row| matches!(row.outcome, Outcome::Generated { .. }))
            .count(),
    }
}

/// Edge halves that carry an authored cue.
///
/// [Q20](../../../../docs/decisions/0020-where-scent-lives.md) puts the cue on
/// the relation instance, so it is an attribute of the half rather than a facet
/// of either end.
fn cues(graph: &Graph) -> usize {
    graph
        .edges
        .iter()
        .filter(|edge| {
            edge.attributes
                .iter()
                .any(|entry| entry.key.value == headwater_graph::edges::CUE)
        })
        .count()
}

/// What each named reading waits on, derived from the corpus of this run.
fn waiting(classified: &[&Row], graph: &Graph, shape: &Shape, warrants: &Warrants) -> Vec<Waiting> {
    let halves = graph.edges.len();
    vec![
        Waiting {
            reading: "transition continuity",
            needs: vec![Need {
                needs: "a record of the state a document left",
                supply: match shape.facet_in_role(STATE_LEFT) {
                    None => Supply::Undeclared(format!(
                        "no facet of this taxonomy takes the `{STATE_LEFT}` role, and spec 2's \
                         closed role registry holds no such role, so one entry into a state \
                         cannot be told from a second"
                    )),
                    Some(facet) => match values(classified, &facet.name).len() {
                        0 => Supply::Unauthored(format!(
                            "`{}` is declared in the `{STATE_LEFT}` role and no document states it",
                            facet.name
                        )),
                        n => Supply::Supplied(format!(
                            "{n} of {} documents state `{}`",
                            classified.len(),
                            facet.name
                        )),
                    },
                },
            }],
        },
        Waiting {
            reading: "scent quality",
            needs: vec![Need {
                needs: "a cue authored on an edge instance",
                supply: match cues(graph) {
                    0 => Supply::Unauthored(format!(
                        "no half of the {halves} this corpus declares carries a `{}` attribute, \
                         which OBL-repo-0023 records",
                        headwater_graph::edges::CUE
                    )),
                    n => Supply::Supplied(format!(
                        "{n} of {halves} halves carry a `{}` attribute",
                        headwater_graph::edges::CUE
                    )),
                },
            }],
        },
        Waiting {
            reading: "promotion rate",
            needs: vec![
                Need {
                    needs: "an `asserted` population to divide by",
                    supply: match warrants.asserted() {
                        0 => Supply::Unauthored(format!(
                            "no document of the {} classified here states `warrant: asserted`, \
                             so there is nothing to promote from",
                            classified.len()
                        )),
                        n => Supply::Supplied(format!(
                            "{n} of {} classified documents state `warrant: asserted`",
                            classified.len()
                        )),
                    },
                },
                Need {
                    needs: "a promotion to count",
                    supply: Supply::Unbuilt(
                        "a promotion is a lifecycle transition, from the `asserted` warrant to \
                         `accepted`, and spec 12 already designs the input one needs. A check \
                         that declares `needs_prior` receives the previously committed version \
                         of each changed document. No check declares it and nothing implements \
                         it. It could not be read here in any case, because this verb reads one \
                         working tree and the prior version is available only in change-scoped \
                         evaluation"
                            .to_string(),
                    ),
                },
            ],
        },
    ]
}

fn freshness_window(shape: &Shape) -> Option<(String, i64)> {
    let facet = shape.facet_in_role("freshness")?;
    Some((facet.name.clone(), facet.stale_after_days?))
}

/// The kind of a classified row, from the census's own resolution.
fn kind_of(row: &Row) -> Option<&str> {
    match &row.outcome {
        Outcome::Typed { kind, .. } => Some(kind),
        Outcome::Generated { kind, .. } => kind.as_deref(),
        _ => None,
    }
}

/// Whether `kind` is `ancestor`, or descends from it through `is_a`.
///
/// A relation that declares `from: [governed_document]` is eligible on every
/// kind under that abstract kind, so an eligibility count that compared names
/// would report every such relation as eligible on nothing.
fn descends_from(shape: &Shape, kind: &str, ancestor: &str) -> bool {
    let mut current = Some(kind.to_string());
    let mut seen = BTreeSet::new();
    while let Some(name) = current {
        if name == ancestor {
            return true;
        }
        if !seen.insert(name.clone()) {
            return false;
        }
        current = shape
            .kinds
            .iter()
            .find(|k| k.name == name)
            .and_then(|k| k.is_a.clone());
    }
    false
}

/// The freshness date a document declares, when it declares a readable one.
fn verified(row: &Row, facet: &str) -> Option<Date> {
    let document = row.document.as_ref()?;
    let value = document.facets.get(facet)?;
    Date::parse(&value.value.as_scalar()?.text)
}

fn creators(
    classified: &[&Row],
    graph: &Graph,
    shape: &Shape,
    relations: &Declarations,
    freshness: Option<&(String, i64)>,
    now: Date,
) -> Creators {
    let mut by_creator: BTreeMap<String, Vec<RelationReading>> = BTreeMap::new();
    let mut undeclared = Vec::new();

    for relation in &relations.relations {
        let reading = relation_reading(relation, classified, graph, shape, freshness, now);
        match &relation.created_by {
            Some(creator) => by_creator.entry(creator.clone()).or_default().push(reading),
            None => undeclared.push(reading),
        }
    }

    // Spec 2's order, and every value of it, so a creator with no relation
    // still has a row. A map ordered by the values in use would print exactly
    // the arms that exist and omit the ones a comparison needs.
    let mut ordered: Vec<(String, Vec<RelationReading>)> = CREATORS
        .iter()
        .map(|creator| {
            (
                (*creator).to_string(),
                by_creator.remove(*creator).unwrap_or_default(),
            )
        })
        .collect();
    // A value outside the closed set reaches here only from a taxonomy that
    // `taxonomy validate` refused, and it is printed rather than dropped.
    ordered.extend(by_creator);
    Creators {
        by_creator: ordered,
        undeclared,
    }
}

fn relation_reading(
    relation: &Relation,
    classified: &[&Row],
    graph: &Graph,
    shape: &Shape,
    freshness: Option<&(String, i64)>,
    now: Date,
) -> RelationReading {
    let halves: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| edge.declared == relation.name)
        .collect();

    let eligible: Vec<&&Row> = classified
        .iter()
        .filter(|row| {
            kind_of(row).is_some_and(|kind| {
                relation
                    .from
                    .iter()
                    .any(|ancestor| descends_from(shape, kind, ancestor))
            })
        })
        .collect();

    let declaring: BTreeSet<&str> = halves
        .iter()
        .filter(|edge| edge.direction == Direction::AsDeclared)
        .map(|edge| edge.source.path.as_str())
        .collect();
    let participating = eligible
        .iter()
        .filter(|row| declaring.contains(row.path.as_str()))
        .count();

    let mut expired = 0;
    let mut undated = 0;
    for edge in &halves {
        let Some((facet, window)) = freshness else {
            undated += 1;
            continue;
        };
        let row = classified.iter().find(|row| row.path == edge.source.path);
        match row.and_then(|row| verified(row, facet)) {
            Some(date) if now.days_since(date) > *window => expired += 1,
            Some(_) => {}
            None => undated += 1,
        }
    }

    RelationReading {
        name: relation.name.clone(),
        family: relation.family.clone(),
        created_by: relation.created_by.clone(),
        halves: halves.len(),
        eligible: eligible.len(),
        participating,
        expired,
        undated,
    }
}

fn families(graph: &Graph, relations: &Declarations) -> Vec<FamilyReading> {
    let defaults: BTreeMap<&str, &str> = headwater_resolve::rules::FAMILY_NUCLEARITY
        .iter()
        .copied()
        .collect();
    let mut by_family: BTreeMap<String, FamilyReading> = BTreeMap::new();

    for relation in &relations.relations {
        let family = relation
            .family
            .clone()
            .unwrap_or_else(|| "none".to_string());
        let halves = graph
            .edges
            .iter()
            .filter(|edge| edge.declared == relation.name)
            .count();
        let entry = by_family
            .entry(family.clone())
            .or_insert_with(|| FamilyReading {
                family: family.clone(),
                relations: 0,
                halves: 0,
                overrides: Vec::new(),
                restatements: Vec::new(),
            });
        entry.relations += 1;
        entry.halves += halves;
        if let (Some(declared), Some(default)) = (
            relation.nuclearity.as_deref(),
            defaults.get(family.as_str()),
        ) {
            if declared == *default {
                entry.restatements.push(relation.name.clone());
            } else {
                entry.overrides.push((
                    relation.name.clone(),
                    declared.to_string(),
                    (*default).to_string(),
                ));
            }
        }
    }
    by_family.into_values().collect()
}

/// The values one facet takes over the classified documents, in path order.
fn values(classified: &[&Row], facet: &str) -> Vec<(String, String)> {
    classified
        .iter()
        .filter_map(|row| {
            let document = row.document.as_ref()?;
            let value = document.facets.get(facet)?;
            let text = value.value.as_scalar()?.text.clone();
            Some((row.path.clone(), text))
        })
        .collect()
}

fn facets(classified: &[&Row], shape: &Shape) -> Vec<FacetReading> {
    shape
        .facets
        .iter()
        .map(|facet| {
            let held = values(classified, &facet.name);
            let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
            for (_, value) in &held {
                *counts.entry(value.as_str()).or_default() += 1;
            }
            let commonest = counts
                .iter()
                // The largest count wins, and the name breaks a tie, so the
                // report is a function of the corpus and never of map order.
                .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                .map(|(value, count)| ((*value).to_string(), *count));
            FacetReading {
                name: facet.name.clone(),
                role: facet.role.clone(),
                required: facet.required,
                declared_values: facet.values.len(),
                carried_by: held.len(),
                distinct: counts.len(),
                commonest,
            }
        })
        .collect()
}

/// Ordered pairs of facets where the first value fixes the second.
///
/// Only facets with a declared value set take part. A date or a free string
/// takes a value per document, so it determines everything and nothing, and a
/// report that included one would say that `status_since` fixes `status` on
/// every corpus ever written.
fn dependences(classified: &[&Row], shape: &Shape) -> Vec<Dependence> {
    let enumerable: Vec<&str> = shape
        .facets
        .iter()
        .filter(|facet| !facet.values.is_empty())
        .map(|facet| facet.name.as_str())
        .collect();

    let mut out = Vec::new();
    for first in &enumerable {
        for second in &enumerable {
            if first == second {
                continue;
            }
            let mut fixed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            let mut over = 0;
            for row in classified {
                let Some(document) = row.document.as_ref() else {
                    continue;
                };
                let read = |name: &str| {
                    document
                        .facets
                        .get(name)
                        .and_then(|value| value.value.as_scalar())
                        .map(|scalar| scalar.text.clone())
                };
                let (Some(a), Some(b)) = (read(first), read(second)) else {
                    continue;
                };
                over += 1;
                fixed.entry(a).or_default().insert(b);
            }
            // Two documents at least, and every value of the first admitting
            // exactly one value of the second. One document proves nothing, and
            // an empty overlap proves less.
            if over > 1 && !fixed.is_empty() && fixed.values().all(|set| set.len() == 1) {
                out.push(Dependence {
                    determines: (*first).to_string(),
                    determined: (*second).to_string(),
                    over,
                });
            }
        }
    }
    out
}

fn shelves(classified: &[&Row], taxonomy: &Taxonomy) -> Vec<ShelfReading> {
    taxonomy
        .shelves
        .iter()
        .filter_map(|shelf| {
            let ShelfBody::Heterogeneous {
                discriminator,
                kinds,
            } = &shelf.body
            else {
                return None;
            };
            let mut counts: BTreeMap<&str, usize> =
                kinds.iter().map(|kind| (kind.as_str(), 0)).collect();
            let mut documents = 0;
            for row in classified {
                let matched = matches!(
                    headwater_census::resolve::shelf_for(&row.path, taxonomy),
                    ShelfMatch::Matched { shelf: found, .. } if found.name == shelf.name
                );
                if !matched {
                    continue;
                }
                documents += 1;
                if let Some(kind) = kind_of(row) {
                    *counts.entry(kind).or_default() += 1;
                }
            }
            Some(ShelfReading {
                shelf: shelf.name.clone(),
                discriminator: discriminator.clone(),
                documents,
                kinds: counts
                    .into_iter()
                    .map(|(kind, count)| (kind.to_string(), count))
                    .collect(),
            })
        })
        .collect()
}

/// Why one document's file name could not be rendered, and where the absence
/// lives.
///
/// The two arms are the two arms of [`Supply`] that a layout can reach. A
/// declaration ends the first and an authoring pass ends the second, and a
/// reading that reported one as the other would send a reader to the wrong file.
enum Hole {
    /// The schema states no source for the placeholder, so no document of this
    /// kind could carry one.
    Schema(String),
    /// The schema states a source and this document declares no value for it.
    Corpus(String),
}

impl Hole {
    fn says(&self) -> &str {
        match self {
            Hole::Schema(text) | Hole::Corpus(text) => text,
        }
    }
}

/// The last segment of a corpus path, which is the part a layout names.
fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// One scalar facet value, as the document declares it.
fn scalar(document: &headwater_doc::Document, facet: &str) -> Option<String> {
    Some(document.facets.get(facet)?.value.as_scalar()?.text.clone())
}

/// The sequence the identifier of this document carries, under the scheme its
/// kind mints from.
///
/// The identifier comes off the graph's node rather than out of front matter by
/// key, because the graph already read it through the configured facet. A second
/// reader here would be a second answer to which key holds an identifier.
fn sequence(path: &str, kind: &str, graph: &Graph, shape: &Shape) -> Option<u64> {
    let node = graph.index.typed.iter().find(|node| node.path == path)?;
    let scheme = shape.identifier_scheme_of(kind)?;
    headwater_check::identifier::Template::parse(&scheme.pattern, &scheme.namespace)
        .ok()?
        .sequence_of(&node.id)
}

/// Why a placeholder had no value on a document of this kind.
fn hole(shape: &Shape, kind: &str, placeholder: &str) -> Hole {
    let key = placeholder
        .split_once(':')
        .map_or(placeholder, |(key, _)| key);
    if key == "slug" {
        return match shape.facet_in_role("name") {
            None => Hole::Schema(
                "no facet of this taxonomy takes the `name` role, so `{slug}` has no source at all"
                    .to_string(),
            ),
            Some(facet) if !shape.required_facets(kind).contains(&facet.name) => {
                Hole::Schema(format!(
                    "`{kind}` requires no facet in the `name` role, so `{{slug}}` has no declared \
                     source"
                ))
            }
            Some(facet) => Hole::Corpus(format!(
                "a document of kind `{kind}` states no `{}`, so `{{slug}}` has no value",
                facet.name
            )),
        };
    }
    if key == "seq" && shape.identifier_scheme_of(kind).is_none() {
        return Hole::Schema(format!(
            "`{kind}` mints under no identifier scheme, so `{{{placeholder}}}` has no source"
        ));
    }
    if key == "seq" {
        return Hole::Corpus(format!(
            "the identifier of a document of kind `{kind}` carries no sequence its scheme admits, \
             so `{{{placeholder}}}` has no value"
        ));
    }
    match shape.required_facets(kind).iter().any(|facet| facet == key) {
        true => Hole::Corpus(format!(
            "a document of kind `{kind}` states no `{key}`, so `{{{placeholder}}}` has no value"
        )),
        false => Hole::Schema(format!(
            "`{kind}` requires no facet named `{key}`, so `{{{placeholder}}}` has no declared source"
        )),
    }
}

/// The file name this shelf's layout renders for one document, or why it does
/// not render one.
///
/// The three sources are the scaffolder's three, read in the scaffolder's own
/// order: the slug of the facet in the `name` role, a facet the document
/// carries, and the sequence of its identifier. A fourth source here would be a
/// name that `headwater new` does not write.
fn rendered(
    row: &Row,
    kind: &str,
    graph: &Graph,
    shape: &Shape,
    layout: &str,
) -> Result<String, Hole> {
    let Some(document) = row.document.as_ref() else {
        return Err(Hole::Corpus(format!(
            "the front matter of a document of kind `{kind}` did not read, so no placeholder has a \
             value"
        )));
    };
    headwater_scaffold::render_layout(layout, |key| match key {
        "slug" => shape
            .facet_in_role("name")
            .and_then(|facet| scalar(document, &facet.name))
            .map(|title| headwater_scaffold::slugify(&title)),
        _ => match scalar(document, key) {
            Some(value) => Some(value),
            None => match key {
                "seq" => sequence(&row.path, kind, graph, shape).map(|value| value.to_string()),
                _ => None,
            },
        },
    })
    .map_err(|placeholder| hole(shape, kind, &placeholder))
}

/// Every shelf that declares a layout, against the documents standing on it.
///
/// A generated document is excluded, and the exclusion is spec 6's rule at this
/// grain rather than a second decision here: the content of a generated file is
/// a function of its emitter, and so is its name. The projection declaration
/// states an `output`, so holding one to a shelf layout would report the
/// emitter as an authoring failure.
fn layouts(
    classified: &[&Row],
    graph: &Graph,
    taxonomy: &Taxonomy,
    shape: &Shape,
    resolved: &Mapping,
) -> Vec<LayoutReading> {
    taxonomy
        .shelves
        .iter()
        .filter_map(|shelf| {
            let layout = headwater_scaffold::declared::layout(resolved, &shelf.name)?;
            let mut identified = 0;
            let mut measured = 0;
            let mut renders = 0;
            let mut holes: Vec<Hole> = Vec::new();
            for row in classified {
                let matched = matches!(
                    headwater_census::resolve::shelf_for(&row.path, taxonomy),
                    ShelfMatch::Matched { shelf: found, .. } if found.name == shelf.name
                );
                let Outcome::Typed { kind, .. } = &row.outcome else {
                    continue;
                };
                if !matched {
                    continue;
                }
                identified += 1;
                match rendered(row, kind, graph, shape, layout) {
                    Ok(name) => {
                        measured += 1;
                        if file_name(&row.path) == name {
                            renders += 1;
                        }
                    }
                    Err(hole) => {
                        if !holes.iter().any(|known| known.says() == hole.says()) {
                            holes.push(hole);
                        }
                    }
                }
            }

            let unmeasured = identified - measured;
            let reasons: Vec<&str> = holes.iter().map(Hole::says).collect();
            let adherence = match (identified, measured) {
                // A declared shelf that holds nothing. The absence is the
                // corpus's, and a zero printed here would read as a shelf whose
                // files all disagree with their layout.
                (0, _) => Supply::Unauthored(format!(
                    "no document stands on `{}`, so `{layout}` names nothing yet",
                    shelf.name
                )),
                (identified, 0) => {
                    let text = format!(
                        "not one of the {identified} documents on `{}` can be measured: {}",
                        shelf.name,
                        reasons.join("; ")
                    );
                    // Every hole is a missing declaration, so no authoring pass
                    // over this corpus would end the wait. A hole that one
                    // document could fill puts the absence in the corpus, and a
                    // shelf with both is reported as the one somebody can act on.
                    match holes.iter().all(|hole| matches!(hole, Hole::Schema(_))) {
                        true => Supply::Undeclared(text),
                        false => Supply::Unauthored(text),
                    }
                }
                (_, measured) => Supply::Supplied(match unmeasured {
                    0 => format!(
                        "`{layout}` renders the name of {renders} of the {measured} documents on \
                         `{}`",
                        shelf.name
                    ),
                    _ => format!(
                        "`{layout}` renders the name of {renders} of the {measured} measurable \
                         documents on `{}`, and {unmeasured} could not be measured: {}",
                        shelf.name,
                        reasons.join("; ")
                    ),
                }),
            };

            Some(LayoutReading {
                shelf: shelf.name.clone(),
                layout: layout.to_string(),
                identified,
                measured,
                renders,
                adherence,
            })
        })
        .collect()
}

/// Dwell in the current state, per state.
///
/// [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#lifecycle):
/// "dwell in a non-terminal state is observable, not policed". So this reports
/// a distribution and never a verdict, and the state list comes from the facet
/// in the `state` role rather than from the values in use, so a state that no
/// document is in still has a row. That row is the one a reader wants: a
/// lifecycle whose `deprecated` state holds nothing is a lifecycle nobody
/// walks the length of.
///
/// A state-entry date later than the audit's date gives a negative dwell,
/// which is carried rather than clamped. A corpus that dated a document into
/// the future should read as one.
fn dwell(classified: &[&Row], shape: &Shape, now: Date) -> Vec<DwellReading> {
    let Some(state) = shape.facet_in_role("state") else {
        return Vec::new();
    };
    let Some(entered) = shape.facet_in_role("state_entered") else {
        return Vec::new();
    };

    let mut readings: Vec<DwellReading> = state
        .values
        .iter()
        .map(|value| DwellReading {
            state: value.clone(),
            documents: 0,
            days: Vec::new(),
            undated: 0,
        })
        .collect();

    for (path, value) in values(classified, &state.name) {
        let Some(reading) = readings.iter_mut().find(|reading| reading.state == value) else {
            // A value the facet does not declare is `facet.value.not_permitted`
            // in the check layer, and it is that check's news rather than this
            // reading's. Counting it under a state it is not would be worse.
            continue;
        };
        reading.documents += 1;
        let row = classified.iter().find(|row| row.path == path);
        match row.and_then(|row| verified(row, &entered.name)) {
            Some(date) => reading.days.push(now.days_since(date)),
            None => reading.undated += 1,
        }
    }
    for reading in &mut readings {
        reading.days.sort_unstable();
    }
    readings
}
