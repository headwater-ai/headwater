// SPDX-License-Identifier: Apache-2.0
//! The relation view: a traceability matrix that follows an ordered chain of
//! relations from every document of one shelf.
//!
//! [HW-DR-0114](../../../../docs/decisions/0114-a-relation-view-follows-an-ordered-chain-of-relations-from-every-document-of-one-shelf.md)
//! states the six points of the owner's ruling of 2026-11-22 and the four
//! choices the ruling left open. This module is those choices and nothing
//! more, so each comment below names the point it implements.
//!
//! # The row the matrix exists for is the empty one
//!
//! Point 2: every document of the starting shelf is a row, and a missing link
//! is an empty cell. A matrix that listed only the requirements something
//! verifies would read as complete and would not be. So the walk starts from
//! the shelf and never from the edges, and a document that reaches nothing is
//! one row whose later cells are all empty.
//!
//! # One row per path
//!
//! A requirement that three criteria verify cannot hold three criteria in one
//! cell and still say which verification proves which criterion. So a row is
//! one path through the chain, and the start document repeats on each of its
//! rows. The rows sort by the start path, then by the path in each later cell,
//! so two runs over one tree write the same bytes.
//!
//! # What a cell states
//!
//! Point 3, read for each kind of edge, because a document-to-document edge
//! holds no revision. A cell names the document, its lifecycle status, a state
//! and the snapshot entry that names the document:
//!
//! - A `verification` reads its freshness off `.headwater/observations.yml`
//!   against the document of the cell before it, with the comparison that
//!   `headwater check` reads ([`headwater_check::verification::compare`]):
//!   `declared`, `observed`, `suspect` or `unknown`.
//! - A code path that a `source-tree` anchor reaches compares the edge's
//!   `verified_revision` with the digest the anchor reaches now: `stamped`,
//!   `baseline`, `suspect` or `unstamped`.
//! - Every other edge holds no stamp, and its cell says `no stamp`.
//!
//! Nothing in a cell reads the clock.
//!
//! # A step that reaches code
//!
//! Point 4 names `traces_to` and `governs`. This module reads it as any step
//! whose `to` names an anchor kind, so the rule is the declaration's rather
//! than a list of two names. A target that is an anchor ends its path in that
//! cell, and a step whose `to` names anchor kinds alone is the last step,
//! which [`read`] holds.

use crate::{shelf_index, shelf_of, Declaration, Kind, Output, Plan, Unwritten};
use headwater_census::census::Census;
use headwater_check::lifecycle_state::{StateFacet, Stood};
use headwater_check::observation::Recorded;
use headwater_check::verification::{compare, Freshness};
use headwater_check::Observations;
use headwater_graph::declarations::Direction;
use headwater_graph::Target;
use headwater_query::Surface;
use headwater_yaml::json::Json;
use headwater_yaml::value::{Mapping, Value};

/// The placeholder an output path may carry: the start shelf's directory, as
/// [`crate::shelf_index`] reads it.
const SHELF: &str = "{shelf}";

/// The kind whose cell reads an observation, which is the kind
/// `headwater check` reads one for.
const VERIFICATION: &str = headwater_check::verification::TARGET_KIND;

/// What a `relation_view` declares beyond what every projection declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    /// The chain, in order. One column for each step after the start column.
    pub relations: Vec<String>,
    pub format: Format,
}

/// The format a view writes. Point 6: Markdown by default, then CSV and JSON.
///
/// Not an export [`crate::Emitter`]: those serialize the graph, and a view
/// serializes its own rows. A `relation_view` never joins an export profile's
/// emitter set for that reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Csv,
    Json,
}

impl Format {
    fn parse(text: &str) -> Option<Format> {
        match text {
            "markdown" => Some(Format::Markdown),
            "csv" => Some(Format::Csv),
            "json" => Some(Format::Json),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Format::Markdown => "markdown",
            Format::Csv => "csv",
            Format::Json => "json",
        }
    }

    /// The output extensions this format writes, so that the census reads the
    /// marker in the syntax the bytes carry.
    fn extensions(self) -> &'static [&'static str] {
        match self {
            Format::Markdown => &["md", "markdown"],
            Format::Csv => &["csv"],
            Format::Json => &["json"],
        }
    }
}

/// Read the members of one entry that only a `relation_view` takes.
///
/// `Ok(None)` for every other kind that states none of them. Every refusal
/// clause 2 of #2018 names is here, so that `generate` and `taxonomy publish`
/// refuse one declaration with one sentence:
///
/// - `relations` on any other kind;
/// - a `relation_view` with no `relations`, or with no single `for` shelf;
/// - a step that names no declared relation;
/// - a step that reaches code and is not the last step;
/// - two steps that do not join, and a start shelf that does not join the
///   first step;
/// - a `format` outside the three, or an output whose extension is not the
///   format's.
pub(crate) fn read(
    root: &Mapping,
    body: &Mapping,
    kind: Kind,
    index: usize,
    shelves: &[String],
    output: &str,
) -> Result<Option<View>, String> {
    let at = format!("projections.{index}");
    if kind != Kind::RelationView {
        if body.get("relations").is_some() {
            return Err(format!(
                "`{at}` is a `{}` and states `relations`, and only a `relation_view` takes \
                 `relations`",
                kind.name()
            ));
        }
        if extension(output) == Some("csv") {
            return Err(format!(
                "`{at}.output` is `{output}`, and only a `relation_view` writes a `.csv` file"
            ));
        }
        return Ok(None);
    }

    let relations = scalars(body, "relations");
    if relations.is_empty() {
        return Err(format!(
            "`{at}` is a `relation_view` and states no `relations`, so nothing says which \
             relations its columns follow"
        ));
    }
    let [shelf] = shelves else {
        return Err(format!(
            "`{at}` is a `relation_view` and names {} shelves in `for`, and a view starts from \
             exactly one shelf",
            shelves.len()
        ));
    };

    let format = match body.get("format").and_then(|node| node.value.as_scalar()) {
        None => Format::Markdown,
        Some(scalar) => Format::parse(&scalar.text).ok_or_else(|| {
            format!(
                "`{at}.format` is `{}`, and a `relation_view` writes `markdown`, `csv` or `json`",
                scalar.text
            )
        })?,
    };
    if !extension(output).is_some_and(|found| format.extensions().contains(&found)) {
        return Err(format!(
            "`{at}.output` is `{output}`, and a `relation_view` in the format `{}` writes a file \
             ending in {}",
            format.name(),
            format
                .extensions()
                .iter()
                .map(|one| format!("`.{one}`"))
                .collect::<Vec<_>>()
                .join(" or ")
        ));
    }

    let declared = Endpoints::read(root);
    let mut ends: Vec<(&str, Vec<String>, Vec<String>)> = Vec::new();
    for name in &relations {
        let Some((from, to)) = declared.relation(name) else {
            return Err(format!(
                "`{at}.relations` names `{name}`, and no relation of that name is declared"
            ));
        };
        ends.push((name, from, to));
    }
    for (step, (name, _, to)) in ends.iter().enumerate() {
        let last = step + 1 == ends.len();
        if !last && to.iter().all(|end| declared.is_anchor(end)) {
            return Err(format!(
                "`{at}.relations` names `{name}` at step {}, and it reaches only code, so no \
                 later step can follow it. A step that reaches code is the last step",
                step + 1
            ));
        }
    }
    if let Some(start) = declared.shelf_kind(shelf) {
        let (name, from, _) = &ends[0];
        if !from.iter().any(|end| declared.joins(&start, end)) {
            return Err(format!(
                "`{at}` starts from the shelf `{shelf}`, which holds `{start}`, and its first \
                 step `{name}` runs from {}, so no edge of it starts on that shelf",
                listed(from)
            ));
        }
    }
    for pair in ends.windows(2) {
        let (before, _, to) = &pair[0];
        let (after, from, _) = &pair[1];
        let joined = to
            .iter()
            .filter(|end| !declared.is_anchor(end))
            .any(|end| from.iter().any(|start| declared.joins(end, start)));
        if !joined {
            return Err(format!(
                "`{at}.relations` names `{before}` and then `{after}`, and they do not join: \
                 `{before}` reaches {}, and `{after}` runs from {}",
                listed(to),
                listed(from)
            ));
        }
    }
    Ok(Some(View { relations, format }))
}

fn listed(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn extension(path: &str) -> Option<&str> {
    std::path::Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
}

fn scalars(body: &Mapping, key: &str) -> Vec<String> {
    match body.get(key).map(|node| &node.value) {
        Some(Value::Seq(items)) => items
            .iter()
            .filter_map(|item| item.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .collect(),
        _ => Vec::new(),
    }
}

/// The relation endpoints, the kind lineage and the anchor kinds of a resolved
/// taxonomy, read off its root, so that a join is decided by the declarations
/// and an abstract endpoint joins every kind below it.
struct Endpoints<'a> {
    root: &'a Mapping,
}

impl<'a> Endpoints<'a> {
    fn read(root: &'a Mapping) -> Self {
        Endpoints { root }
    }

    fn block(&self, key: &str) -> Option<&'a Mapping> {
        self.root.get(key).and_then(|node| node.value.as_map())
    }

    fn relation(&self, name: &str) -> Option<(Vec<String>, Vec<String>)> {
        let body = self
            .block("relations")?
            .get(name)
            .and_then(|node| node.value.as_map())?;
        Some((scalars(body, "from"), scalars(body, "to")))
    }

    fn is_anchor(&self, name: &str) -> bool {
        self.block("anchors")
            .is_some_and(|anchors| anchors.get(name).is_some())
    }

    fn shelf_kind(&self, shelf: &str) -> Option<String> {
        self.block("shelves")?
            .get(shelf)
            .and_then(|node| node.value.as_map())?
            .get("kind")
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
    }

    /// The kind and every kind it inherits from, nearest first.
    fn lineage(&self, kind: &str) -> Vec<String> {
        let mut out = vec![kind.to_string()];
        let kinds = self.block("kinds");
        while let Some(parent) = kinds
            .and_then(|kinds| kinds.get(out.last().expect("never empty")))
            .and_then(|node| node.value.as_map())
            .and_then(|body| body.get("is_a"))
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
        {
            // A cycle is `taxonomy validate`'s to report, and it ends the walk.
            if out.contains(&parent) {
                break;
            }
            out.push(parent);
        }
        out
    }

    /// Whether a document at one end may also sit at the other: one of the two
    /// kinds is, or inherits from, the other. An abstract `to` such as
    /// `governed_document` joins a concrete `from`, because an edge of it can
    /// land on that kind, and the reverse holds for the same reason.
    fn joins(&self, one: &str, other: &str) -> bool {
        self.lineage(one).iter().any(|kind| kind == other)
            || self.lineage(other).iter().any(|kind| kind == one)
    }
}

/// One cell of a row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Cell {
    /// The path ended before this step, or this step reached nothing.
    Empty,
    Document {
        path: String,
        id: Option<String>,
        status: Option<String>,
        /// `None` on the start cell of a row, which no edge of the chain
        /// reached, unless it is a verification.
        state: Option<&'static str>,
        /// The commit of the snapshot entry that names this document, and
        /// `unknown` where the snapshot or the entry did not read.
        observation: Option<String>,
    },
    /// The code paths one anchor reaches, sorted, and the stamp of the edge.
    Code {
        paths: Vec<String>,
        state: &'static str,
    },
    /// A target that bound to nothing, as written. `headwater check` reports
    /// why, and the view keeps the row rather than dropping it.
    Unresolved { target: String },
}

impl Cell {
    /// What a row sorts on: the path in the cell.
    fn key(&self) -> String {
        match self {
            Cell::Empty => String::new(),
            Cell::Document { path, .. } => path.clone(),
            Cell::Code { paths, .. } => paths.join(", "),
            Cell::Unresolved { target } => target.clone(),
        }
    }
}

/// Where a path stands after a step: on a document it can walk on from, or at
/// an end.
enum Stand {
    On(String),
    Ended,
}

pub(crate) fn emit(
    surface: &Surface<'_>,
    census: &Census,
    declaration: &Declaration,
    observations: &Observations,
    plan: &mut Plan,
) {
    let Some(view) = declaration.view.as_ref() else {
        return;
    };
    let decline = |plan: &mut Plan, at: String, reason: String| {
        plan.unwritten.push(Unwritten {
            at,
            kind: Kind::RelationView,
            reason,
        });
    };
    let [shelf_name] = declaration.shelves.as_slice() else {
        return;
    };
    let Some(shelf) = surface
        .taxonomy()
        .shelves
        .iter()
        .find(|shelf| &shelf.name == shelf_name)
    else {
        decline(
            plan,
            declaration.output.clone(),
            format!("names the shelf `{shelf_name}`, which this taxonomy does not declare"),
        );
        return;
    };
    let path = declaration
        .output
        .replace(SHELF, &shelf_index::directory_of(shelf.pattern.source()));

    let mut starts: Vec<_> = surface
        .documents()
        .into_iter()
        .filter(|document| {
            shelf_of(document.path, surface).is_some_and(|found| &found.name == shelf_name)
        })
        .collect();
    starts.sort_by(|a, b| a.path.cmp(b.path));

    let walker = Walker {
        surface,
        census,
        observations,
        state: StateFacet::of(surface.shape()),
    };
    let mut rows: Vec<Vec<Cell>> = Vec::new();
    for start in starts {
        let first = walker.document(start.path, None);
        let mut frontier = vec![(vec![first], Stand::On(start.path.to_string()))];
        for relation in &view.relations {
            let mut next = Vec::new();
            for (cells, stand) in frontier {
                let Stand::On(at) = stand else {
                    let mut cells = cells;
                    cells.push(Cell::Empty);
                    next.push((cells, Stand::Ended));
                    continue;
                };
                let reached = walker.step(relation, &at);
                if reached.is_empty() {
                    let mut cells = cells;
                    cells.push(Cell::Empty);
                    next.push((cells, Stand::Ended));
                    continue;
                }
                for (cell, stand) in reached {
                    let mut cells = cells.clone();
                    cells.push(cell);
                    next.push((cells, stand));
                }
            }
            frontier = next;
        }
        let mut paths: Vec<Vec<Cell>> = frontier.into_iter().map(|(cells, _)| cells).collect();
        paths.sort_by_key(|cells| cells.iter().map(Cell::key).collect::<Vec<_>>());
        rows.extend(paths);
    }

    let bytes = match view.format {
        Format::Markdown => markdown(&path, shelf, &view.relations, &rows),
        Format::Csv => csv(&path, shelf_name, &view.relations, &rows),
        Format::Json => json(shelf_name, &view.relations, &rows),
    };
    plan.outputs.push(Output {
        bytes,
        path,
        kind: Kind::RelationView,
        committed: true,
    });
}

struct Walker<'s, 'a> {
    surface: &'s Surface<'a>,
    census: &'s Census,
    observations: &'s Observations,
    state: StateFacet,
}

impl Walker<'_, '_> {
    /// Every far end of `relation` from the document at `at`, read from the
    /// end the relation runs from, whichever end wrote the edge. Both halves of
    /// one pair are one far end.
    fn step(&self, relation: &str, at: &str) -> Vec<(Cell, Stand)> {
        let mut documents: Vec<String> = Vec::new();
        let mut out: Vec<(Cell, Stand)> = Vec::new();
        for edge in &self.surface.graph().edges {
            if edge.declared != relation {
                continue;
            }
            match (&edge.direction, &edge.target) {
                (Direction::AsDeclared, target) if edge.source.path == at => match target {
                    Target::Document { path, .. } => {
                        if !documents.contains(path) {
                            documents.push(path.clone());
                        }
                    }
                    Target::Anchor {
                        resolver,
                        revision,
                        patterns,
                        ..
                    } => {
                        let mut paths: Vec<String> = patterns
                            .iter()
                            .flat_map(|member| match member.matched.is_empty() {
                                true => vec![member.pattern.clone()],
                                false => member.matched.clone(),
                            })
                            .collect();
                        paths.sort();
                        paths.dedup();
                        let state = match (resolver.as_str(), revision.get()) {
                            ("source-tree", Some(current)) => match edge.verified_revision() {
                                None => "unstamped",
                                Some(recorded) if recorded != current => "suspect",
                                Some(_) if edge.verified_as_baseline() => "baseline",
                                Some(_) => "stamped",
                            },
                            _ => "no stamp",
                        };
                        let cell = Cell::Code { paths, state };
                        if !out.iter().any(|(held, _)| held == &cell) {
                            out.push((cell, Stand::Ended));
                        }
                    }
                    Target::Unbound(_) => out.push((
                        Cell::Unresolved {
                            target: edge.raw_target.clone(),
                        },
                        Stand::Ended,
                    )),
                    // An export filter withheld it. A view reads the corpus,
                    // and nothing here is filtered.
                    Target::Withheld { .. } => {}
                },
                (Direction::Inverse, Target::Document { path, .. }) if path == at => {
                    if !documents.contains(&edge.source.path) {
                        documents.push(edge.source.path.clone());
                    }
                }
                _ => {}
            }
        }
        for path in documents {
            let cell = self.document(&path, Some(at));
            out.push((cell, Stand::On(path)));
        }
        out
    }

    /// The cell of the document at `path`, reached from the document at
    /// `before`, or the start cell where `before` is `None`.
    fn document(&self, path: &str, before: Option<&str>) -> Cell {
        let found = self.surface.find(path);
        let id = found.and_then(|document| document.id).map(str::to_string);
        let kind = found.map(|document| document.kind);
        let status = found.and_then(|document| match self.state.stood(document.facets) {
            Stood::At(state) | Stood::NotAState(state) => Some(state.to_string()),
            Stood::Undeclared => None,
        });
        let recorded = id
            .as_deref()
            .map(|id| self.observations.recorded(id))
            .unwrap_or(Recorded::Absent);
        let observation = match recorded {
            Recorded::Absent => None,
            Recorded::Unread(_) => Some("unknown".to_string()),
            Recorded::Entry { commit, .. } => Some(commit.to_string()),
        };
        let is_verification =
            kind.is_some_and(|kind| self.surface.shape().descends_from(kind, VERIFICATION));
        let state = match (is_verification, id.as_deref(), before) {
            (true, Some(id), Some(before)) => Some(match digest_at(self.census, before) {
                Some(digest) => match compare(self.observations, id, digest) {
                    Freshness::Declared => "declared",
                    Freshness::Observed { .. } => "observed",
                    Freshness::Suspect { .. } => "suspect",
                    Freshness::Unknown { .. } => "unknown",
                },
                None => "unknown",
            }),
            // A verification that starts a row has no criterion before it to
            // compare against, so it states whether a snapshot names it.
            (true, _, None) => Some(match recorded {
                Recorded::Absent => "declared",
                Recorded::Unread(_) => "unknown",
                Recorded::Entry { .. } => "observed",
            }),
            // The start of a row: no edge of the chain reached it.
            (false, _, None) => None,
            _ => Some("no stamp"),
        };
        Cell::Document {
            path: path.to_string(),
            id,
            status,
            state,
            observation,
        }
    }
}

/// The census digest of the file at a path, which is the digest
/// `headwater observe` records for a criterion.
fn digest_at<'c>(census: &'c Census, path: &str) -> Option<&'c str> {
    census
        .rows
        .iter()
        .find(|row| row.path == path)
        .and_then(|row| row.digest.as_deref())
}

/// The heading of each column: the start shelf, then each relation.
fn headings(start: &str, relations: &[String]) -> Vec<String> {
    std::iter::once(start.to_string())
        .chain(relations.iter().cloned())
        .collect()
}

fn markdown(
    output: &str,
    shelf: &headwater_census::shelves::Shelf,
    relations: &[String],
    rows: &[Vec<Cell>],
) -> String {
    let base = shelf_index::parent_of(output);
    let mut out = headwater_mark::marker(Kind::RelationView.name(), output)
        .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
    let title = shelf.title.clone().unwrap_or_else(|| shelf.name.clone());
    out.push_str(&format!("\n\n# {title}: {}\n\n", relations.join(", ")));
    out.push_str(&format!(
        "One row for each path from each document of the shelf `{}` through the chain {}. A \
         document that reaches nothing at a step has an empty cell there, and that cell is what \
         this file is for.\n\n",
        shelf.name,
        relations
            .iter()
            .map(|relation| format!("`{relation}`"))
            .collect::<Vec<_>>()
            .join(", then ")
    ));
    out.push_str(
        "A cell names the document, its lifecycle status, its state and the snapshot entry of \
         `.headwater/observations.yml` that names it. A verification is `declared`, `observed`, \
         `suspect` or `unknown`. A code path is `stamped`, `baseline`, `suspect` or \
         `unstamped`. Any other edge holds `no stamp`.\n\n",
    );
    let headings = headings(&title, relations);
    out.push_str(&format!("| {} |\n", headings.join(" | ")));
    out.push_str(&format!("|{}\n", "---|".repeat(headings.len())));
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .map(|cell| match cell {
                Cell::Empty => String::new(),
                Cell::Document {
                    path,
                    id,
                    status,
                    state,
                    observation,
                } => {
                    let label = id.clone().unwrap_or_else(|| path.clone());
                    let observation = match observation {
                        Some(commit) if commit == "unknown" => "snapshot unread".to_string(),
                        Some(commit) => format!("observed at `{commit}`"),
                        None => "not observed".to_string(),
                    };
                    let state = state.map(|state| format!(" · {state}")).unwrap_or_default();
                    format!(
                        "[{label}]({}) · {}{state} · {observation}",
                        shelf_index::relative(&base, path),
                        status.as_deref().unwrap_or("no status"),
                    )
                }
                Cell::Code { paths, state } => format!(
                    "{} · {state}",
                    paths
                        .iter()
                        .map(|path| format!("`{path}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Cell::Unresolved { target } => format!("`{target}` · unresolved"),
            })
            .map(|cell| cell.replace('|', "\\|"))
            .collect();
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
    }
    out
}

/// One field of a CSV record, quoted where RFC 4180 requires it.
fn field(text: &str) -> String {
    match text.contains([',', '"', '\n', '\r']) {
        true => format!("\"{}\"", text.replace('"', "\"\"")),
        false => text.to_string(),
    }
}

/// Four fields for each cell, so a reader filters on one of them: the
/// document or code paths, the status, the state and the observation.
fn fields(cell: &Cell) -> [String; 4] {
    match cell {
        Cell::Empty => Default::default(),
        Cell::Document {
            path,
            id,
            status,
            state,
            observation,
        } => [
            id.clone().unwrap_or_else(|| path.clone()),
            status.clone().unwrap_or_default(),
            state.map(str::to_string).unwrap_or_default(),
            observation.clone().unwrap_or_default(),
        ],
        Cell::Code { paths, state } => [
            paths.join(", "),
            String::new(),
            state.to_string(),
            String::new(),
        ],
        Cell::Unresolved { target } => [
            target.clone(),
            String::new(),
            "unresolved".to_string(),
            String::new(),
        ],
    }
}

fn csv(output: &str, shelf: &str, relations: &[String], rows: &[Vec<Cell>]) -> String {
    let mut out = headwater_mark::marker(Kind::RelationView.name(), output)
        .unwrap_or_else(|| format!("# {}", headwater_mark::MARKER));
    out.push('\n');
    let header: Vec<String> = headings(shelf, relations)
        .iter()
        .flat_map(|heading| {
            [
                heading.clone(),
                format!("{heading} status"),
                format!("{heading} state"),
                format!("{heading} observation"),
            ]
        })
        .map(|heading| field(&heading))
        .collect();
    out.push_str(&header.join(","));
    out.push('\n');
    for row in rows {
        let record: Vec<String> = row.iter().flat_map(fields).map(|one| field(&one)).collect();
        out.push_str(&record.join(","));
        out.push('\n');
    }
    out
}

fn json(shelf: &str, relations: &[String], rows: &[Vec<Cell>]) -> String {
    let null = || Json::Raw("null".to_string());
    let optional = |value: &Option<String>| match value {
        Some(text) => Json::string(text.as_str()),
        None => null(),
    };
    let rows = rows
        .iter()
        .map(|row| {
            Json::Array(
                row.iter()
                    .map(|cell| match cell {
                        Cell::Empty => null(),
                        Cell::Document {
                            path,
                            id,
                            status,
                            state,
                            observation,
                        } => Json::object([
                            ("document", optional(id)),
                            ("path", Json::string(path.as_str())),
                            ("status", optional(status)),
                            ("state", optional(&state.map(str::to_string))),
                            ("observation", optional(observation)),
                        ]),
                        Cell::Code { paths, state } => Json::object([
                            (
                                "code",
                                Json::Array(
                                    paths
                                        .iter()
                                        .map(|path| Json::string(path.as_str()))
                                        .collect(),
                                ),
                            ),
                            ("state", Json::string(*state)),
                        ]),
                        Cell::Unresolved { target } => Json::object([
                            ("unresolved", Json::string(target.as_str())),
                            ("state", Json::string("unresolved")),
                        ]),
                    })
                    .collect(),
            )
        })
        .collect();
    Json::object([
        (
            headwater_mark::MARKER,
            Json::string(headwater_mark::marker_text(Kind::RelationView.name())),
        ),
        ("for", Json::string(shelf)),
        (
            "relations",
            Json::Array(
                relations
                    .iter()
                    .map(|one| Json::string(one.as_str()))
                    .collect(),
            ),
        ),
        ("rows", Json::Array(rows)),
    ])
    .render_pretty()
}
