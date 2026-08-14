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
//! [OBL-repo-0119](../../../../docs/obligations/0119-an-audit-reading-has-no-declared-bar-so-a-distribution-cannot-become-a-finding.md)
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

/// One reading that this verb does not take, and what it waits on.
#[derive(Clone, Copy, Debug)]
pub struct Waiting {
    pub reading: &'static str,
    pub waits_on: &'static str,
}

/// Every measurement spec 6 names that this verb does not take.
///
/// The list is here rather than in the CLI because it is a statement about what
/// the readings above cover, and a reader of the crate needs it beside them.
/// Spec 6's grammar rule holds a declared name to running or to a named wait,
/// and a measurement inside a verb answers to the same rule.
pub const WAITING: [Waiting; 3] = [
    Waiting {
        reading: "transition continuity",
        waits_on: "a record of a transition. A document states the state it is in and the date it entered, and nothing states the state it left, so no run can tell one entry into `current` from a second one",
    },
    Waiting {
        reading: "scent quality",
        waits_on: "authored cues. Q20 puts a cue on a relation instance, and OBL-repo-0023 records that no corpus has authored enough of them to grade",
    },
    Waiting {
        reading: "promotion rate",
        waits_on: "a population and an input. No document of this corpus carries `warrant: asserted`, and promotions per change reads a history of changes that no crate of this engine opens",
    },
];

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
    pub dwell: Vec<DwellReading>,
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
        dwell: dwell(&classified, shape, now),
        freshness,
    }
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
