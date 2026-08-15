// SPDX-License-Identifier: Apache-2.0
//! The scaffolder: what `headwater new <kind>` proposes, and what it refuses.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! names this a correctness root: "Edges marked `created_by: scaffold` are
//! corpus facts that nobody reviews individually. A scaffolder bug manufactures
//! wrong edges at exactly the scale that the assisted-fraction metric
//! celebrates." Three properties follow, and this crate is arranged around
//! them.
//!
//! # 1. The template comes from the taxonomy, and it is not a file
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#templates-and-scaffolding):
//! "Each kind declares a template. The template is not a suggestion file for
//! humans to copy. It is generated from the kind declaration." So nothing here
//! reads a template file, and nothing here carries prose of its own beyond a
//! prompt for a value a human owes.
//!
//! A file would be the second authoring surface that
//! [#130](https://github.com/headwater-ai/headwater/issues/130) refused for a
//! projection, for a reason that reaches a scaffolder unchanged: a taxonomy
//! source sits outside the corpus root, so no census row covers it, no language
//! regime binds it, and no rule reads its links. A template file under a
//! package is prose that the corpus governs, in the one place the corpus cannot
//! see. Every field this crate writes therefore names the declaration it came
//! from, and [`Origin`] is that name.
//!
//! # 2. What it writes is authored, and it is checked
//!
//! A projection's output carries the generated-file marker and no check reads
//! it, "because its content is a function of the emitter and an author cannot
//! repair it in the file"
//! ([spec 6](../../../../docs/spec/06-engine-architecture.md#projections)).
//! Neither clause holds here. A scaffolded document is written once and never
//! written again, so an author repairs it in the file and nothing overwrites
//! the repair. The test between the two is **regeneration**, and it is the
//! whole of the difference. So this crate writes no marker, and the next
//! `headwater check` reads the document as the authored document it is.
//!
//! # 3. Its output goes through the pipeline that already exists
//!
//! Four rules that ship already read what a scaffolder can get wrong, and this
//! crate refuses in front of each rather than writing a document that fails it.
//!
//! | what could go wrong | the rule that would report it | the refusal here |
//! |---|---|---|
//! | an identifier the scheme does not admit | `identifier.pattern.not_met` | [`Refusal::MintRefused`] |
//! | a document a relation may name and that carries no identifier | `identifier.unusable` | [`Refusal::Unnameable`] |
//! | two documents claiming one identifier | `identifier.claimed_twice` | [`Refusal::IdentifierTaken`] |
//! | an edge between kinds the relation forbids | `relation.endpoint.not_permitted` | [`Refusal::EndpointNotPermitted`] |
//! | a target that resolves to nothing | `relation.target.unresolved` | [`Refusal::TargetUnresolved`] |
//! | a facet value outside its set | `facet.value.not_permitted` | [`Refusal::FacetUndeterminable`] |
//! | a missing required section | `section.required.missing` | none needed: the headings are written |
//!
//! The refusals are a closed set for the reason spec 12 gives every closed set
//! in this engine. A test is written against a name, so an invariant that lived
//! in an anonymous arm would be invisible to the suite.
//!
//! # What this crate does not do, and what nothing measures
//!
//! **It writes no facet of another document.** `supersedes` declares
//! `on_target: {set_state: superseded}`, and this crate writes the reciprocal
//! edge half and stops there. A state transition is a lifecycle event, no rule
//! in this engine reads a transition, and a scaffolder that moved a state
//! nothing validates would be manufacturing the fact spec 12 warns about.
//!
//! **Allocation sees the corpus and never the history.** Spec 3 says a deleted
//! document does not free its number. This engine reads a tree, so the highest
//! spent value it can find is the highest one still on disk. [`Minting`]
//! carries that limit beside every identifier it issues.
//!
//! **The assisted fraction is a fact about this run.** [`Assisted`] counts what
//! this invocation supplied against what spec 3 names as the denominator. It is
//! not a property of the corpus, and it cannot become one:
//! [Q4](../../../../docs/decisions/0004-relation-storage.md) keeps `created_by`
//! on the relation type rather than on the edge, so no reader of a committed
//! corpus can tell a scaffolded edge from a hand-typed one. So the run writes
//! the reading down, and [`reading`] is where it lands and what it holds.

pub mod declared;
pub mod fix;
pub mod migrate;
pub mod reading;
pub mod tree;
pub mod write;

use headwater_census::census::Census;
use headwater_census::resolve::{shelf_for, ShelfMatch};
use headwater_census::shelves::{Shelf, ShelfBody, Taxonomy};
use headwater_check::identifier::{Needs, Template};
use headwater_check::shape::Shape;
use headwater_check::Date;
use headwater_graph::declarations::{Declarations, Direction, Reciprocal};
use headwater_graph::index::Index;
use headwater_graph::Config;
use headwater_yaml::Mapping;

/// What a caller asks for.
pub struct Request<'a> {
    pub kind: &'a str,
    pub title: &'a str,
    /// The injected clock. Every date this run writes is this value, so the
    /// same request over the same corpus writes the same bytes.
    pub now: Date,
    /// `(relation as written, target identifier)`, in the order the caller
    /// named them.
    pub relates: &'a [(String, String)],
    /// `(facet, value)` that the caller stated, in the order they named them.
    ///
    /// # Why a caller may state a value at all
    ///
    /// A required facet that declares a closed value set and carries no engine
    /// role refuses this verb, because a prompt is not a member of a set. Two
    /// declarations then meet and neither yields: the facet canons refuse a
    /// facet that no kind requires as unread, and the scaffolder refuses a kind
    /// that requires one. A kind whose category is the author's judgment — the
    /// `probe` kind of the measurement layer is the first — falls between them
    /// and could be written by no route this engine offers.
    ///
    /// So the caller supplies it. The value is validated against the set before
    /// anything is written, and it is recorded as
    /// [`Origin::HandEntry`]: a person typed it, the engine determined nothing,
    /// and the assisted fraction says so. A value here that names a facet the
    /// kind does not require, or one that a declaration already determines, is
    /// refused rather than dropped.
    pub given: &'a [(String, String)],
}

/// Everything the resolved taxonomy and the corpus hand a scaffolder.
pub struct Sources<'a> {
    /// The root of the resolved taxonomy, for the members of [`declared`].
    pub resolved: &'a Mapping,
    pub shape: &'a Shape,
    pub shelves: &'a Taxonomy,
    pub relations: &'a Declarations,
    pub census: &'a Census,
    pub index: &'a Index,
    pub config: &'a Config,
}

/// Where a value came from, which is the whole of the assisted fraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// The taxonomy determines it, and this run wrote it. The string names the
    /// declaration, in the words a person reads in a report.
    Scaffolded(String),
    /// Nothing determines it. The field carries a prompt, and a human owes the
    /// value.
    HandEntry(String),
}

impl Origin {
    pub fn is_scaffolded(&self) -> bool {
        matches!(self, Origin::Scaffolded(_))
    }

    pub fn reason(&self) -> &str {
        match self {
            Origin::Scaffolded(why) | Origin::HandEntry(why) => why,
        }
    }
}

/// One front-matter field of the proposed document.
#[derive(Clone, Debug)]
pub struct Field {
    pub key: String,
    pub value: String,
    /// Whether the value is written as a double-quoted scalar. A prompt and a
    /// free string are; a date, an integer and a value from a closed set are
    /// not.
    pub quoted: bool,
    pub origin: Origin,
}

/// One required section, as a heading and a prompt under it.
#[derive(Clone, Debug)]
pub struct Section {
    pub heading: String,
}

/// One edge this run proposes, with the half at each end.
#[derive(Clone, Debug)]
pub struct Proposed {
    /// The relation name as it is written in the new document's block.
    pub relation: String,
    pub target: String,
    pub target_path: String,
    /// The half written into the target document, for a relation whose
    /// reciprocity is required. `None` where the relation asks for none.
    pub reciprocal: Option<Half>,
    /// The creator the taxonomy assigns, which is `scaffold` on every edge that
    /// reaches this far.
    pub created_by: String,
}

/// The half of an edge that goes into a document somebody else wrote.
#[derive(Clone, Debug)]
pub struct Half {
    pub relation: String,
    pub path: String,
    pub id: String,
    /// Instance attributes the half carries, in the order they are written.
    ///
    /// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one)
    /// lets a relation type declare attributes its instances may take, and an
    /// entry that carries one is a mapping with `to` rather than a bare
    /// identifier. An empty list writes the bare form, which is what every half
    /// this crate proposes for itself takes. `headwater_import` is the caller
    /// that fills it, with the upstream revision an edge was checked against.
    pub attributes: Vec<(String, String)>,
}

/// An identifier, and what its scheme said about issuing one.
#[derive(Clone, Debug)]
pub struct Minting {
    pub id: String,
    pub scheme: String,
    /// `minted-once` or `reconcile-first`, as the scheme declares it, and
    /// nothing where it declares none.
    pub allocation: Option<String>,
    /// The highest value this run found already spent, for a scheme whose
    /// pattern carries a sequence. It is a reading of the tree, and spec 3 asks
    /// for a reading of every value ever allocated. A document that was deleted
    /// is not on the tree, so this number can only be a lower bound and the
    /// report says so.
    pub reconciled_from: Option<u64>,
    /// The value the identifier writes at its `{seq}` segment, and nothing for
    /// a scheme whose pattern carries none. A shelf layout may name it, which
    /// is how a file name carries the number its identifier already holds.
    pub sequence: Option<u64>,
}

/// A relation the new document may declare and this run did not write.
#[derive(Clone, Debug)]
pub struct Expected {
    pub relation: String,
    pub to: Vec<String>,
    pub created_by: String,
}

/// What this run supplied, against what spec 3 names as the denominator.
///
/// "Of the required front matter, sections, identifiers, and relations, how
/// much was scaffolded, derived, or agent-drafted, and how much was
/// hand-entered?" The four terms below are those four, and nothing else is
/// counted. Each unit is one field, one required section heading, the
/// identifier, or one half of one edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Assisted {
    pub fields: (usize, usize),
    pub sections: (usize, usize),
    pub identifier: (usize, usize),
    pub edge_halves: (usize, usize),
}

impl Assisted {
    pub fn supplied(&self) -> usize {
        self.fields.0 + self.sections.0 + self.identifier.0 + self.edge_halves.0
    }

    pub fn total(&self) -> usize {
        self.fields.1 + self.sections.1 + self.identifier.1 + self.edge_halves.1
    }
}

/// What `headwater new` will write, decided before anything is written.
#[derive(Clone, Debug)]
pub struct Plan {
    pub kind: String,
    pub shelf: String,
    /// Relative to the repository root, with `/` separators.
    pub path: String,
    pub minting: Option<Minting>,
    pub fields: Vec<Field>,
    pub sections: Vec<Section>,
    pub edges: Vec<Proposed>,
    pub expected: Vec<Expected>,
    pub title: String,
}

impl Plan {
    /// The count spec 3 asks for, derived from the plan rather than tracked
    /// beside it. A field that changed origin changes this number with it.
    pub fn assisted(&self) -> Assisted {
        let scaffolded = self
            .fields
            .iter()
            .filter(|field| field.origin.is_scaffolded())
            .count();
        Assisted {
            fields: (scaffolded, self.fields.len()),
            sections: (self.sections.len(), self.sections.len()),
            identifier: match self.minting.is_some() {
                true => (1, 1),
                false => (0, 0),
            },
            // Two halves per edge. The author named the target, so the near
            // half is hand entry; the far half follows from it, which is the
            // one edge fact a scaffolder derives rather than receives.
            edge_halves: (
                self.edges.iter().filter(|e| e.reciprocal.is_some()).count(),
                self.edges.len() * 2,
            ),
        }
    }
}

/// Every way this crate declines to write, as a closed set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    KindUnknown {
        kind: String,
        known: Vec<String>,
    },
    KindAbstract {
        kind: String,
    },
    /// No shelf claims the kind, so the document has nowhere to land where the
    /// census would read it back as this kind.
    KindUnshelved {
        kind: String,
    },
    /// Two shelves claim it, and nothing says which one a new document joins.
    KindOnManyShelves {
        kind: String,
        shelves: Vec<String>,
    },
    /// The shelf's path is a glob before its last segment, so no directory
    /// follows from it.
    ShelfPathNotLiteral {
        shelf: String,
        pattern: String,
    },
    /// The shelf's layout names a placeholder that this document holds nothing
    /// for, so the rendered file name would carry a hole where a value belongs.
    LayoutUnresolved {
        shelf: String,
        layout: String,
        placeholder: String,
    },
    /// The title is empty, so no slug and no name follow from it.
    TitleEmpty,
    /// A required facet that no declaration determines and no prompt may stand
    /// in: a closed value set, an integer, or a date.
    FacetUndeterminable {
        facet: String,
        kind: String,
        why: String,
        /// Whether `--facet` is a route out of this one.
        ///
        /// It is not for a facet in an engine role. `--facet` refuses one,
        /// because a role is a declaration that decides the value, and a
        /// message that offered the flag there would send a caller at a second
        /// refusal.
        statable: bool,
    },
    /// The caller stated a value for a facet this kind does not require, so
    /// nothing would have written it and the caller would not have been told.
    FacetNotAsked {
        facet: String,
        kind: String,
        required: Vec<String>,
    },
    /// The caller stated a value for a facet that a declaration decides.
    FacetDetermined {
        facet: String,
        kind: String,
        why: String,
    },
    /// The caller stated a value outside the facet's closed set. The checks
    /// would refuse it, so this run refuses it before it is written.
    FacetNotPermitted {
        facet: String,
        found: String,
        values: Vec<String>,
    },
    /// The kind's scheme declares a pattern this engine cannot read, so it can
    /// check no identifier under it and it issues none.
    SchemeUnreadable {
        scheme: String,
        why: String,
    },
    /// The template refused what this run filled into it. It is a defect in
    /// this crate rather than in a declaration, and it is reported rather than
    /// written.
    MintRefused {
        scheme: String,
        why: String,
    },
    /// A relation may name this kind and no scheme mints for it, so the
    /// document would be neither end of any edge.
    Unnameable {
        kind: String,
        relations: Vec<String>,
    },
    IdentifierTaken {
        id: String,
        path: String,
    },
    PathTaken {
        path: String,
    },
    /// The proposed path does not classify back to the kind it was derived
    /// for. The classifier is the authority, and a disagreement is this crate's
    /// defect.
    PlacementDoesNotResolve {
        path: String,
        kind: String,
        found: String,
    },
    RelationUnknown {
        relation: String,
        known: Vec<String>,
    },
    /// The taxonomy assigns this edge to somebody else, and a scaffolder that
    /// wrote it anyway would be the wrong answer to "who pays for this edge".
    RelationNotScaffolded {
        relation: String,
        created_by: String,
    },
    EndpointNotPermitted {
        relation: String,
        end: &'static str,
        kind: String,
        permitted: Vec<String>,
    },
    TargetUnresolved {
        relation: String,
        target: String,
    },
    /// The reciprocal half would go into a document, and the splice did not
    /// read back. Nothing is written.
    ReciprocalUnwritable {
        path: String,
        why: String,
    },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::KindUnknown { kind, known } => write!(
                f,
                "no kind is named `{kind}`. This taxonomy declares {}",
                list(known)
            ),
            Refusal::KindAbstract { kind } => write!(
                f,
                "`{kind}` is abstract, and an abstract kind is a kind that no document ever is"
            ),
            Refusal::KindUnshelved { kind } => write!(
                f,
                "no shelf claims `{kind}`, so a document of it would land where the census reads \
                 it back as something else. Declare a shelf for the kind first"
            ),
            Refusal::KindOnManyShelves { kind, shelves } => write!(
                f,
                "`{kind}` sits on {}, and nothing states which one a new document joins",
                list(shelves)
            ),
            Refusal::ShelfPathNotLiteral { shelf, pattern } => write!(
                f,
                "the shelf `{shelf}` declares `{pattern}`, and a glob before the last segment \
                 names no directory to write into"
            ),
            Refusal::LayoutUnresolved {
                shelf,
                layout,
                placeholder,
            } => write!(
                f,
                "the shelf `{shelf}` declares `{layout}`, and `{{{placeholder}}}` names neither a \
                 facet this document carries nor the sequence of the identifier it mints. A file \
                 name with a hole in it is not a name this verb writes"
            ),
            Refusal::TitleEmpty => write!(
                f,
                "the title is empty, and the file name and the document's name both come from it"
            ),
            Refusal::FacetUndeterminable {
                facet,
                kind,
                why,
                statable,
            } => match statable {
                true => write!(
                    f,
                    "`{kind}` requires the facet `{facet}`, and {why}. A prompt in that field is \
                     a value the checks refuse, so this run writes nothing. State it yourself \
                     with `--facet {facet}=<value>`"
                ),
                false => write!(
                    f,
                    "`{kind}` requires the facet `{facet}`, and {why}. A prompt in that field is \
                     a value the checks refuse, so this run writes nothing. `--facet` is no route \
                     out: the facet carries a role, and what is missing is the declaration the \
                     role reads"
                ),
            },
            Refusal::FacetNotAsked {
                facet,
                kind,
                required,
            } => write!(
                f,
                "`--facet {facet}=…` names a facet that `{kind}` does not require, so nothing \
                 would have written it. `{kind}` requires: {}",
                required.join(", ")
            ),
            Refusal::FacetDetermined { facet, kind, why } => write!(
                f,
                "`--facet {facet}=…` names a facet of `{kind}` that a declaration decides, and \
                 {why}. A value stated here would be a state nobody chose, written through the \
                 verb that exists to stop that"
            ),
            Refusal::FacetNotPermitted {
                facet,
                found,
                values,
            } => write!(
                f,
                "`--facet {facet}={found}` is outside the value set the taxonomy declares, which \
                 the checks would refuse. The values are: {}",
                values.join(", ")
            ),
            Refusal::SchemeUnreadable { scheme, why } => {
                write!(f, "the scheme `{scheme}` cannot be read: {why}")
            }
            Refusal::MintRefused { scheme, why } => write!(
                f,
                "the identifier this run built is one the scheme `{scheme}` does not admit: \
                 {why}. That is a defect in the scaffolder rather than in the declaration"
            ),
            Refusal::Unnameable { kind, relations } => write!(
                f,
                "`{kind}` names no identifier scheme, and {} may name a document of it. Such a \
                 document is neither end of any edge, which `identifier.unusable` reports on \
                 every run. Declare `kinds.{kind}.identifier` first",
                list(relations)
            ),
            Refusal::IdentifierTaken { id, path } => write!(
                f,
                "`{id}` is already declared on {path}, and an identifier is never reused"
            ),
            Refusal::PathTaken { path } => write!(
                f,
                "{path} is already there, and this verb never overwrites a document"
            ),
            Refusal::PlacementDoesNotResolve { path, kind, found } => write!(
                f,
                "{path} classifies as {found} rather than as `{kind}`, so the placement this run \
                 derived is not the placement the census reads. That is a defect in the \
                 scaffolder"
            ),
            Refusal::RelationUnknown { relation, known } => write!(
                f,
                "no relation is named `{relation}`. This taxonomy declares {}",
                list(known)
            ),
            Refusal::RelationNotScaffolded {
                relation,
                created_by,
            } => write!(
                f,
                "`{relation}` declares `created_by: {created_by}`, and this verb proposes the \
                 edges the taxonomy assigns to a scaffold. Write it by hand, or change what the \
                 declaration says pays for it"
            ),
            Refusal::EndpointNotPermitted {
                relation,
                end,
                kind,
                permitted,
            } => write!(
                f,
                "`{relation}` does not admit `{kind}` at its {end} end, which admits {}",
                list(permitted)
            ),
            Refusal::TargetUnresolved { relation, target } => write!(
                f,
                "`{relation}` names `{target}`, and no document of this corpus carries that \
                 identifier"
            ),
            Refusal::ReciprocalUnwritable { path, why } => write!(
                f,
                "the reciprocal half belongs in {path}, and the result did not read back: {why}. \
                 Nothing was written"
            ),
        }
    }
}

fn list(items: &[String]) -> String {
    match items.is_empty() {
        true => "none".to_string(),
        false => items
            .iter()
            .map(|item| format!("`{item}`"))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Decide everything, and write nothing.
///
/// Every refusal is raised here, so a caller that reaches [`write::apply`] has
/// a plan whose every part already passed. That ordering is the rule the issue
/// asks for: a malformed document is never written to disk and then reported.
pub fn propose(sources: &Sources<'_>, request: &Request<'_>) -> Result<Plan, Refusal> {
    let kind = request.kind;

    let declared_kind = sources.shelves.kind(kind).ok_or_else(|| {
        let mut known: Vec<String> = sources
            .shelves
            .kinds
            .iter()
            .filter(|k| !k.is_abstract)
            .map(|k| k.name.clone())
            .collect();
        known.sort();
        Refusal::KindUnknown {
            kind: kind.to_string(),
            known,
        }
    })?;
    if declared_kind.is_abstract {
        return Err(Refusal::KindAbstract {
            kind: kind.to_string(),
        });
    }

    let title = request.title.trim();
    if title.is_empty() {
        return Err(Refusal::TitleEmpty);
    }
    let slug = slugify(title);
    if slug.is_empty() {
        return Err(Refusal::TitleEmpty);
    }

    let shelf = one_shelf(sources.shelves, kind)?;
    let directory = literal_directory(shelf)?;

    let fields = front_matter(sources, kind, shelf, title, request.now, request.given)?;

    // The identifier is minted before the placement, because a shelf layout may
    // name the sequence the identifier carries. Nothing is written either way,
    // so the only thing this ordering decides is which refusal a request that
    // trips two of them reports first.
    let minting = mint(sources, kind, &slug)?;
    let path = place(&directory, sources, shelf, &fields, &slug, minting.as_ref())?;

    if sources.index.by_path(&path).is_some() {
        return Err(Refusal::PathTaken { path });
    }

    // The classifier is the authority on what a path means. A placement this
    // crate derived and the census reads differently is this crate's defect,
    // and it is reported rather than written.
    match shelf_for(&path, sources.shelves) {
        ShelfMatch::Matched { shelf: found, .. } if found.name == shelf.name => {}
        ShelfMatch::Matched { shelf: found, .. } => {
            return Err(Refusal::PlacementDoesNotResolve {
                path,
                kind: kind.to_string(),
                found: format!("a document of the shelf `{}`", found.name),
            })
        }
        ShelfMatch::Stopped(stop) => {
            return Err(Refusal::PlacementDoesNotResolve {
                path,
                kind: kind.to_string(),
                found: stop.into_resolution().explain(),
            })
        }
    }

    if let Some(minting) = &minting {
        if let Some(node) = sources.index.node(&minting.id) {
            return Err(Refusal::IdentifierTaken {
                id: minting.id.clone(),
                path: node.path.clone(),
            });
        }
    }

    let sections = sources
        .shape
        .required_sections(kind)
        .into_iter()
        .map(|heading| Section { heading })
        .collect();

    let edges = propose_edges(sources, request, kind, minting.as_ref())?;
    let expected = expected_relations(sources, kind, &edges);

    Ok(Plan {
        kind: kind.to_string(),
        shelf: shelf.name.clone(),
        path,
        minting,
        fields,
        sections,
        edges,
        expected,
        title: title.to_string(),
    })
}

/// The one shelf that claims a kind.
fn one_shelf<'a>(shelves: &'a Taxonomy, kind: &str) -> Result<&'a Shelf, Refusal> {
    let claiming: Vec<&Shelf> = shelves
        .shelves
        .iter()
        .filter(|shelf| match &shelf.body {
            ShelfBody::Homogeneous { kind: on } => on == kind,
            ShelfBody::Heterogeneous { kinds, .. } => kinds.iter().any(|on| on == kind),
        })
        .collect();
    match claiming.as_slice() {
        [] => Err(Refusal::KindUnshelved {
            kind: kind.to_string(),
        }),
        [shelf] => Ok(shelf),
        many => Err(Refusal::KindOnManyShelves {
            kind: kind.to_string(),
            shelves: many.iter().map(|shelf| shelf.name.clone()).collect(),
        }),
    }
}

/// The directory a shelf pattern names, which is its run of leading literals.
///
/// `docs/obligations/**` names `docs/obligations`. A pattern with a glob before
/// its last segment names no directory, and this crate refuses rather than
/// picking one of the paths it could mean.
fn literal_directory(shelf: &Shelf) -> Result<String, Refusal> {
    let source = shelf.pattern.source();
    let segments: Vec<&str> = source.split('/').filter(|s| !s.is_empty()).collect();
    let mut directory: Vec<&str> = Vec::new();
    for (position, segment) in segments.iter().enumerate() {
        let last = position + 1 == segments.len();
        let globbed = *segment == "**" || segment.contains(['*', '?']);
        match (globbed, last) {
            (false, _) => directory.push(segment),
            (true, true) => {}
            (true, false) => {
                return Err(Refusal::ShelfPathNotLiteral {
                    shelf: shelf.name.clone(),
                    pattern: source.to_string(),
                })
            }
        }
    }
    match directory.is_empty() {
        true => Err(Refusal::ShelfPathNotLiteral {
            shelf: shelf.name.clone(),
            pattern: source.to_string(),
        }),
        false => Ok(directory.join("/")),
    }
}

/// The path, from the shelf's `layout` where it declares one.
///
/// Three sources fill a placeholder, and they are read in this order.
///
/// | placeholder | value |
/// |---|---|
/// | `{slug}` | the slug of the title |
/// | a facet the document carries | that field's value |
/// | `{seq}` | the sequence of the identifier this run mints |
///
/// The last one is why `place` runs after [`mint`]. A shelf whose kind numbers
/// its identifiers holds that number once, in the identifier, and a file name
/// that carried a second copy of it in a facet would be two writers of one
/// fact. `docs/decisions/**` and `docs/obligations/**` are both that shape.
/// `{namespace}` is the one part of an identifier that no layout may name,
/// because a scheme's namespace is a constant and every file of the shelf would
/// carry the same characters.
///
/// A placeholder that none of the three fills is a refusal rather than an empty
/// string. A layout that renders `-a-title.md` names a document nobody asked
/// for, and the declaration that produced it is the thing to repair.
fn place(
    directory: &str,
    sources: &Sources<'_>,
    shelf: &Shelf,
    fields: &[Field],
    slug: &str,
    minting: Option<&Minting>,
) -> Result<String, Refusal> {
    let Some(layout) = declared::layout(sources.resolved, &shelf.name) else {
        return Ok(format!("{directory}/{slug}.md"));
    };
    match render_layout(layout, |key| match key {
        "slug" => Some(slug.to_string()),
        _ => match fields.iter().find(|field| field.key == key) {
            Some(field) => Some(field.value.clone()),
            None => minting
                .filter(|_| key == "seq")
                .and_then(|minting| minting.sequence)
                .map(|value| value.to_string()),
        },
    }) {
        Ok(name) => Ok(format!("{directory}/{name}")),
        Err(placeholder) => Err(Refusal::LayoutUnresolved {
            shelf: shelf.name.clone(),
            layout: layout.to_string(),
            placeholder,
        }),
    }
}

/// A zero-padding specifier on a value, which is the one form a layout writes.
/// `02d` and `04d` are the two this repository declares, and the width is read
/// rather than assumed.
///
/// [`render_layout`] applies it to every placeholder, where the loop it
/// replaced applied it to a facet and to `{seq}` and never to `{slug}`. The
/// difference is inert: this function returns its argument unchanged for a
/// value that does not parse as a number, and a slug that parses as one is a
/// title of digits alone. No layout of this repository writes a specifier on a
/// slug, and the uniform rule is the one a reader can predict.
fn pad(value: &str, specifier: Option<&str>) -> String {
    let Some(width) = specifier
        .and_then(|s| s.strip_prefix('0'))
        .and_then(|s| s.strip_suffix('d'))
        .and_then(|s| s.parse::<usize>().ok())
    else {
        return value.to_string();
    };
    match value.parse::<u64>() {
        Ok(number) => format!("{number:0width$}"),
        Err(_) => value.to_string(),
    }
}

/// The front matter, one field for each facet the kind requires.
fn front_matter(
    sources: &Sources<'_>,
    kind: &str,
    shelf: &Shelf,
    title: &str,
    now: Date,
    given: &[(String, String)],
) -> Result<Vec<Field>, Refusal> {
    let discriminator = match &shelf.body {
        ShelfBody::Heterogeneous { discriminator, .. } => Some(discriminator.as_str()),
        ShelfBody::Homogeneous { .. } => None,
    };

    let required = sources.shape.required_facets(kind);
    // Every value the caller stated is consumed below, or this run refuses.
    // A value that named a facet nobody reads would be a field silently
    // dropped, and a caller who thought they had set one.
    for (facet, _) in given {
        if !required.iter().any(|name| name == facet) {
            return Err(Refusal::FacetNotAsked {
                facet: facet.clone(),
                kind: kind.to_string(),
                required: required.clone(),
            });
        }
    }

    let mut fields = Vec::new();
    for name in required.iter().cloned() {
        let facet = sources.shape.facet(&name);
        let role = facet.and_then(|facet| facet.role.as_deref());
        let values: &[String] = facet.map(|facet| facet.values.as_slice()).unwrap_or(&[]);
        let declared_type = declared::facet_type(sources.resolved, &name);

        // The discriminator first. A heterogeneous shelf reads the kind out of
        // this facet, so its value is the kind and nothing else.
        if discriminator == Some(name.as_str()) {
            fields.push(Field {
                key: name.clone(),
                value: kind.to_string(),
                quoted: false,
                origin: Origin::Scaffolded(format!(
                    "the shelf `{}` is heterogeneous, and `{name}` is its discriminator",
                    shelf.name
                )),
            });
            continue;
        }

        // What the caller stated, where the engine determines nothing. A facet
        // in a role, and the discriminator above, are decided by a declaration,
        // and a caller who overwrote one would be writing a state nobody chose
        // through the verb that exists to stop that.
        if let Some((_, value)) = given.iter().find(|(facet, _)| facet == &name) {
            if let Some(role) = role {
                return Err(Refusal::FacetDetermined {
                    facet: name,
                    kind: kind.to_string(),
                    why: format!("it carries the `{role}` role, which decides its value"),
                });
            }
            if !values.is_empty() && !values.iter().any(|permitted| permitted == value) {
                return Err(Refusal::FacetNotPermitted {
                    facet: name,
                    found: value.clone(),
                    values: values.to_vec(),
                });
            }
            fields.push(Field {
                key: name.clone(),
                value: value.clone(),
                // A member of a closed set is a bare token, exactly as the
                // state facet writes one. Anything else is prose.
                quoted: values.is_empty(),
                origin: Origin::HandEntry(
                    "no declaration determines it, and the caller stated it on the command line"
                        .to_string(),
                ),
            });
            continue;
        }

        let field =
            match role {
                Some("state") => {
                    let regime = declared::lifecycle_of(sources.resolved, kind);
                    let initial =
                        regime.and_then(|regime| declared::initial_state(sources.resolved, regime));
                    match initial {
                        Some(state) => Field {
                            key: name.clone(),
                            value: state.to_string(),
                            quoted: false,
                            origin: Origin::Scaffolded(format!(
                                "`regimes.lifecycle.{}` opens at `{state}`",
                                regime.unwrap_or_default()
                            )),
                        },
                        None => return Err(Refusal::FacetUndeterminable {
                            facet: name,
                            kind: kind.to_string(),
                            why: "no lifecycle regime that this kind binds declares an `initial` \
                                  state"
                                .to_string(),
                            statable: false,
                        }),
                    }
                }
                Some("state_entered") | Some("freshness") => Field {
                    key: name.clone(),
                    value: now.to_string(),
                    quoted: false,
                    origin: Origin::Scaffolded(format!(
                        "the facet is in the `{}` role, and the run's clock is the date",
                        role.unwrap_or_default()
                    )),
                },
                Some("name") => Field {
                    key: name.clone(),
                    value: title.to_string(),
                    quoted: true,
                    origin: Origin::Scaffolded(
                        "the facet is in the `name` role, and `--title` is the name".to_string(),
                    ),
                },
                _ if !values.is_empty() => {
                    return Err(Refusal::FacetUndeterminable {
                        facet: name,
                        kind: kind.to_string(),
                        why: "it declares a closed value set and it carries no role this engine \
                          derives a value for"
                            .to_string(),
                        statable: true,
                    })
                }
                // An integer the shelf writes into its file names is a position in
                // a series, and the series is on the shelf. Nothing else in a
                // taxonomy says that an integer counts, so this reads the one
                // declaration that does.
                _ if declared_type == Some("integer") && names(sources, shelf, &name) => Field {
                    key: name.clone(),
                    value: next_in_series(sources, shelf, &name).to_string(),
                    quoted: false,
                    origin: Origin::Scaffolded(format!(
                    "the shelf `{}` writes `{name}` into its layout, and this is the next value \
                     on it",
                    shelf.name
                )),
                },
                _ if matches!(declared_type, Some("integer") | Some("date")) => {
                    return Err(Refusal::FacetUndeterminable {
                        facet: name,
                        kind: kind.to_string(),
                        why: format!(
                            "it is declared `type: {}`, and a prompt is not one",
                            declared_type.unwrap_or_default()
                        ),
                        statable: true,
                    })
                }
                _ => Field {
                    key: name.clone(),
                    value: prompt(&name, role),
                    quoted: true,
                    origin: Origin::HandEntry(
                        "no declaration determines it, and this run states the question instead"
                            .to_string(),
                    ),
                },
            };
        fields.push(field);
    }
    Ok(fields)
}

/// Whether a shelf's layout writes this facet into a file name.
fn names(sources: &Sources<'_>, shelf: &Shelf, facet: &str) -> bool {
    let Some(layout) = declared::layout(sources.resolved, &shelf.name) else {
        return false;
    };
    segments(layout)
        .iter()
        .any(|segment| matches!(segment, Segment::Placeholder { key, .. } if *key == facet))
}

/// One past the highest value of a facet over the documents already on a shelf.
///
/// A shelf with no document, and a shelf whose documents all decline the facet,
/// both start the series at 1. That is the same reading the identifier
/// allocator takes, and it inherits the same limit: a document that was deleted
/// is not on the tree, so this is a lower bound on what the series ever held.
fn next_in_series(sources: &Sources<'_>, shelf: &Shelf, facet: &str) -> u64 {
    sources
        .census
        .rows
        .iter()
        .filter(|row| shelf.pattern.matches(&row.path))
        .filter_map(|row| row.document.as_ref())
        .filter_map(|document| document.facets.get(facet))
        .filter_map(|value| value.value.as_scalar())
        .filter_map(|scalar| scalar.text.parse::<u64>().ok())
        .max()
        .map_or(1, |highest| highest + 1)
}

/// The question a field asks when nothing determines its value.
fn prompt(facet: &str, role: Option<&str>) -> String {
    match role {
        Some("scent") => {
            "TODO one sentence: what a reader learns here, which is what routing reads".to_string()
        }
        _ => format!("TODO {facet}"),
    }
}

/// The identifier, minted under the scheme the kind names.
fn mint(sources: &Sources<'_>, kind: &str, slug: &str) -> Result<Option<Minting>, Refusal> {
    let Some(scheme) = sources.shape.identifier_scheme_of(kind) else {
        // A kind that no relation may name needs no identifier, and that is a
        // true statement about a corpus with no edge to it. A kind a relation
        // *may* name and that mints under nothing is the case
        // `identifier.unusable` reports on every run, so this refuses in front
        // of it rather than writing the document that raises it.
        let naming: Vec<String> = sources
            .relations
            .relations
            .iter()
            .filter(|relation| {
                admits(sources, &relation.from, kind) || admits(sources, &relation.to, kind)
            })
            .map(|relation| relation.name.clone())
            .collect();
        return match naming.is_empty() {
            true => Ok(None),
            false => Err(Refusal::Unnameable {
                kind: kind.to_string(),
                relations: naming,
            }),
        };
    };

    let template = Template::parse(&scheme.pattern, &scheme.namespace).map_err(|why| {
        Refusal::SchemeUnreadable {
            scheme: scheme.name.clone(),
            why: why.0,
        }
    })?;

    // Reconcile first: the highest value the corpus already spent, over every
    // identifier this template admits. It reads the tree, so it is a lower
    // bound on every value ever allocated.
    let needs_sequence = template
        .needs()
        .iter()
        .any(|need| matches!(need, Needs::Sequence { .. }));
    let reconciled_from = match needs_sequence {
        false => None,
        true => sources
            .index
            .typed
            .iter()
            .chain(sources.index.untyped.iter())
            .filter_map(|node| template.sequence_of(&node.id))
            .max(),
    };
    let next = reconciled_from
        .map(|highest| highest + 1)
        .or(match needs_sequence {
            true => Some(1),
            false => None,
        });

    let id = template
        .mint(Some(slug), next)
        .map_err(|why| Refusal::MintRefused {
            scheme: scheme.name.clone(),
            why: why.0,
        })?;

    // The minter and the rule read one grammar, and this is the assertion that
    // says so. A failure here is a defect in this crate, reported before a byte
    // is written rather than by `identifier.pattern.not_met` on the next run.
    if !template.admits(&id) {
        return Err(Refusal::MintRefused {
            scheme: scheme.name.clone(),
            why: format!("`{id}` does not match `{}`", template.render()),
        });
    }

    Ok(Some(Minting {
        id,
        scheme: scheme.name.clone(),
        allocation: declared::allocation(sources.resolved, &scheme.name).map(str::to_string),
        reconciled_from,
        sequence: next,
    }))
}

/// Whether an endpoint list admits a kind, following `is_a`.
///
/// An endpoint that names an abstract parent admits every kind under it, which
/// is what makes the base relations reach a bundle's kinds with no edit
/// ([spec 2](../../../../docs/spec/02-taxonomy-model.md#abstract-kinds)).
fn admits(sources: &Sources<'_>, endpoints: &[String], kind: &str) -> bool {
    endpoints
        .iter()
        .any(|endpoint| sources.shape.descends_from(kind, endpoint))
}

fn propose_edges(
    sources: &Sources<'_>,
    request: &Request<'_>,
    kind: &str,
    minting: Option<&Minting>,
) -> Result<Vec<Proposed>, Refusal> {
    let mut proposed = Vec::new();
    for (written, target) in request.relates {
        let named = sources.relations.named(written).ok_or_else(|| {
            let mut known: Vec<String> = sources
                .relations
                .relations
                .iter()
                .filter(|relation| {
                    declared::created_by(sources.resolved, &relation.name) == Some("scaffold")
                })
                .map(|relation| relation.name.clone())
                .collect();
            known.sort();
            Refusal::RelationUnknown {
                relation: written.clone(),
                known,
            }
        })?;
        let relation = named.relation;

        let created_by = declared::created_by(sources.resolved, &relation.name).unwrap_or("");
        if created_by != "scaffold" {
            return Err(Refusal::RelationNotScaffolded {
                relation: relation.name.clone(),
                created_by: created_by.to_string(),
            });
        }

        // Which end this document sits at, from the name the caller wrote.
        let (near, far) = match named.direction {
            Direction::AsDeclared => (&relation.from, &relation.to),
            Direction::Inverse => (&relation.to, &relation.from),
        };
        if !admits(sources, near, kind) {
            return Err(Refusal::EndpointNotPermitted {
                relation: written.clone(),
                end: "source",
                kind: kind.to_string(),
                permitted: near.clone(),
            });
        }

        let node = sources
            .index
            .node(target)
            .ok_or_else(|| Refusal::TargetUnresolved {
                relation: written.clone(),
                target: target.clone(),
            })?;
        let target_kind = node.kind.clone().unwrap_or_default();
        if !admits(sources, far, &target_kind) {
            return Err(Refusal::EndpointNotPermitted {
                relation: written.clone(),
                end: "target",
                kind: target_kind,
                permitted: far.clone(),
            });
        }

        // The far half, for a relation that requires one. It is written under
        // the name the far document reads: the inverse of what this document
        // wrote, and the same name again for a symmetric relation.
        let reciprocal = match (&relation.reciprocal, minting) {
            (Reciprocal::Required, Some(minting)) => {
                let name = match named.direction {
                    Direction::AsDeclared => {
                        relation.inverse.clone().unwrap_or(relation.name.clone())
                    }
                    Direction::Inverse => relation.name.clone(),
                };
                Some(Half {
                    relation: name,
                    path: node.path.clone(),
                    id: minting.id.clone(),
                    attributes: Vec::new(),
                })
            }
            (Reciprocal::Symmetric, Some(minting)) => Some(Half {
                relation: relation.name.clone(),
                path: node.path.clone(),
                id: minting.id.clone(),
                attributes: Vec::new(),
            }),
            _ => None,
        };

        proposed.push(Proposed {
            relation: written.clone(),
            target: target.clone(),
            target_path: node.path.clone(),
            reciprocal,
            created_by: created_by.to_string(),
        });
    }
    Ok(proposed)
}

/// The relations a document of this kind may declare and this run did not.
///
/// Spec 3: `new` "also prints the relations that the new document is expected
/// to declare". Printed and never written: a target is a fact about the world,
/// and a scaffolder that guessed one would be manufacturing the edge that
/// spec 12 warns about.
fn expected_relations(sources: &Sources<'_>, kind: &str, written: &[Proposed]) -> Vec<Expected> {
    sources
        .relations
        .relations
        .iter()
        .filter(|relation| admits(sources, &relation.from, kind))
        .filter(|relation| !written.iter().any(|edge| edge.relation == relation.name))
        .map(|relation| Expected {
            relation: relation.name.clone(),
            to: relation.to.clone(),
            created_by: declared::created_by(sources.resolved, &relation.name)
                .unwrap_or("unstated")
                .to_string(),
        })
        .collect()
}

/// One piece of a shelf layout, as [`segments`] reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Segment<'a> {
    /// Characters the layout writes into every file name on its shelf.
    Literal(&'a str),
    /// A placeholder, with the padding specifier written after the colon.
    Placeholder {
        key: &'a str,
        specifier: Option<&'a str>,
    },
}

/// A layout as its pieces, in the order it writes them.
///
/// One scanner, and every reader of a layout in this engine goes through it:
/// [`render_layout`] fills the placeholders, [`names`] asks whether one of them
/// is a facet, and `taxonomy audit` asks whether the file a document sits in
/// carries the name its shelf's layout renders. Three scanners would be three
/// answers to one question about one string.
///
/// An opening brace that nothing closes is the rest of the layout as a literal.
/// A template somebody mistyped then renders the characters they wrote, which
/// is what [`place`] did before this function existed.
pub fn segments(layout: &str) -> Vec<Segment<'_>> {
    let mut out = Vec::new();
    let mut rest = layout;
    while let Some(open) = rest.find('{') {
        if open > 0 {
            out.push(Segment::Literal(&rest[..open]));
        }
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            out.push(Segment::Literal(&rest[open..]));
            return out;
        };
        let placeholder = &after[..close];
        let (key, specifier) = match placeholder.split_once(':') {
            Some((key, specifier)) => (key, Some(specifier)),
            None => (placeholder, None),
        };
        out.push(Segment::Placeholder { key, specifier });
        rest = &after[close + 1..];
    }
    if !rest.is_empty() {
        out.push(Segment::Literal(rest));
    }
    out
}

/// Render a layout, taking the value of each placeholder from `fill`.
///
/// `Err` carries the first placeholder that nothing filled, as the layout wrote
/// it, so a message about a hole names the characters a person will search for.
/// A layout that
/// rendered an empty string in its place would name a file nobody asked for,
/// which is why [`place`] refuses rather than writing one and why the audit
/// reading reports a document as unmeasured rather than as a name that
/// disagrees.
///
/// The padding is applied here rather than by the caller, so the width a layout
/// declares is read in one place. See [`pad`].
pub fn render_layout(
    layout: &str,
    fill: impl Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut name = String::new();
    for segment in segments(layout) {
        match segment {
            Segment::Literal(text) => name.push_str(text),
            Segment::Placeholder { key, specifier } => match fill(key) {
                Some(value) => name.push_str(&pad(&value, specifier)),
                None => {
                    return Err(match specifier {
                        Some(specifier) => format!("{key}:{specifier}"),
                        None => key.to_string(),
                    })
                }
            },
        }
    }
    Ok(name)
}

/// A title, as the token a file name and an identifier carry.
///
/// Lower case, ASCII alphanumerics kept, every run of anything else collapsed
/// to one hyphen. The alphabet is this crate's, because spec 2 states nothing
/// about a slug beyond that it is a free token, and a file name has to survive
/// a filesystem that the specification does not describe.
pub fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut pending = false;
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            if pending && !slug.is_empty() {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_lowercase());
        } else {
            pending = true;
        }
    }
    slug
}
