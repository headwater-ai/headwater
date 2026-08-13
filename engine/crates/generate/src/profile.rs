// SPDX-License-Identifier: Apache-2.0
//! The export profile: the declaration that says what leaves a corpus.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-profile-carries-a-filter)
//! makes an export profile "an entry under `projections`" that names an
//! audience, an emitter target, an output path, a filter over facet values and
//! a tombstone grain. The meta-schema already declares every one of those
//! members. What did not exist until now is a reader for them, so
//! [`crate::Projections::read`] kept a kind and an output path and dropped the
//! rest, and the corpus descriptor recorded that a taxonomy could declare no
//! profile at all.
//!
//! # A profile is a group of entries, and the name is what groups them
//!
//! Spec 6 asks that "every projection inside a profile regenerates from the
//! filtered graph", so a profile holds more than one projection. The
//! meta-schema writes `profile` on each entry, which makes the name the thing
//! that groups them. Two entries that name one profile are two artifacts for
//! one audience.
//!
//! That leaves one question the specification does not reach: the filter and the
//! tombstone grain sit on each entry, so two entries of one profile could
//! declare two different filters. **They may not, and the engine refuses a
//! disagreement.** A profile is an audience, a filter is what that audience may
//! see, and two answers for one audience is the same defect confluence refuses
//! in an overlay ([spec 7](../../../../docs/spec/07-distribution-and-federation.md)).
//! One of the two would win, the winner would be whichever the reader reached
//! last, and the artifact that lost would carry more than the profile permits.
//!
//! # An entry that names no profile is in `default`
//!
//! Spec 6: "A corpus with one audience declares one profile with no filter,
//! which is the first release." So an entry that names no profile is not
//! profile-less, it is the one profile. The engine names it `default` rather
//! than leave it anonymous, because `--profile` selects by name and a name that
//! cannot be typed cannot be selected. An adopter who then declares a profile
//! called `default` merges into it, and the confluence rule above holds them to
//! one filter.

use headwater_census::shelves::DeclarationError;
use headwater_yaml::value::{Mapping, Value};
use headwater_yaml::Span;

/// The profile name an entry that declares none belongs to.
pub const DEFAULT: &str = "default";

/// An emitter target: the vocabulary an export speaks.
///
/// Seven, from [Q13](../../../../docs/spec/09-decisions.md#q13--linkml-and-shacl-as-substrate)'s
/// staging order. Two are built. The other five parse, because a taxonomy that
/// names one has declared something legal and the engine owes it a reason
/// rather than "not a format". Q13 puts each of the five behind a named
/// external consumer, and none exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emitter {
    /// The native property graph, with no loss.
    Json,
    /// JSON Schema for front matter.
    JsonSchema,
    Shacl,
    Rdf,
    Skos,
    Okf,
    LinkMl,
}

impl Emitter {
    /// The name as `--format` writes it and as a taxonomy declares it.
    pub fn name(self) -> &'static str {
        match self {
            Emitter::Json => "json",
            Emitter::JsonSchema => "jsonschema",
            Emitter::Shacl => "shacl",
            Emitter::Rdf => "rdf",
            Emitter::Skos => "skos",
            Emitter::Okf => "okf",
            Emitter::LinkMl => "linkml",
        }
    }

    /// Every target, in the order spec 6 writes them on the verb.
    pub const ALL: [Emitter; 7] = [
        Emitter::Json,
        Emitter::JsonSchema,
        Emitter::Shacl,
        Emitter::Rdf,
        Emitter::Skos,
        Emitter::Okf,
        Emitter::LinkMl,
    ];

    pub fn parse(text: &str) -> Option<Emitter> {
        Emitter::ALL.into_iter().find(|one| one.name() == text)
    }

    /// Whether this engine emits it.
    ///
    /// Spec 6: "Only `json` and `jsonschema` ship in the first release, and each
    /// later format waits for a consumer who asks for it."
    pub fn is_built(self) -> bool {
        matches!(self, Emitter::Json | Emitter::JsonSchema)
    }
}

/// The tombstone grain: what a reader of a filtered view learns about what is
/// missing.
///
/// Spec 6 gives the two values and makes `counted` the default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grain {
    /// A placeholder stands where each withheld node or edge would have been,
    /// carrying the identifier of the rule that withheld it.
    Counted,
    /// The view states that it is filtered. Nothing else.
    Sealed,
}

impl Grain {
    pub fn name(self) -> &'static str {
        match self {
            Grain::Counted => "counted",
            Grain::Sealed => "sealed",
        }
    }

    fn parse(text: &str) -> Option<Grain> {
        match text {
            "counted" => Some(Grain::Counted),
            "sealed" => Some(Grain::Sealed),
            _ => None,
        }
    }
}

/// One clause of a filter: a facet, and the values it names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub facet: String,
    pub values: Vec<String>,
}

/// A filter over facet values.
///
/// # Two directions, and only one of them is default-deny
///
/// Spec 6 asks for a filter that is **default-deny**: "a node class, an edge
/// class, or an attribute that no profile names does not travel", because "a
/// filter stated as a list of exclusions grows a hole every time the schema
/// grows". Spec 2's own example writes an exclusion. Both are here, and they are
/// not two spellings of one rule.
///
/// - An `include` clause is default-deny over one facet. A document travels only
///   when it states that facet and states one of the named values. A document
///   that states no value for the facet does **not** travel, which is the part
///   that makes it deny by default: a facet value added to the taxonomy next
///   month reaches no profile until somebody names it.
/// - An `exclude` clause names values that do not travel, and it grows the hole
///   spec 6 warns about. It is admitted because the specification's own example
///   uses one, and because a corpus whose confidentiality facet is required has
///   no hole to grow.
///
/// A document travels when it satisfies every `include` clause and no `exclude`
/// clause. An empty filter withholds nothing, which is the first release's one
/// profile.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub include: Vec<Clause>,
    pub exclude: Vec<Clause>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    /// Whether a document with these facet values travels, and the rule that
    /// withheld it when it does not.
    ///
    /// The rule identifier is built from the declaration and never from the
    /// document. Spec 6: "Free prose in a tombstone is a channel, and a reason
    /// that quotes the document is a leak wearing a label. The rule identifier
    /// is what a reader needs to ask for access, and it is all that they get."
    /// So the identifier names the profile, the direction and the facet, and it
    /// never names the value that matched.
    pub fn admits(&self, profile: &str, facets: &Mapping) -> Admission {
        for clause in &self.include {
            let held = values_at(facets, &clause.facet);
            if !held.iter().any(|value| clause.values.contains(value)) {
                return Admission::Withheld(format!("{profile}.include.{}", clause.facet));
            }
        }
        for clause in &self.exclude {
            let held = values_at(facets, &clause.facet);
            if held.iter().any(|value| clause.values.contains(value)) {
                return Admission::Withheld(format!("{profile}.exclude.{}", clause.facet));
            }
        }
        Admission::Carried
    }
}

/// What a filter decided about one document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Admission {
    Carried,
    /// Withheld, and the identifier of the rule that withheld it.
    Withheld(String),
}

/// The values a document states for a facet.
///
/// A scalar is one value and a sequence is several, because a facet may be
/// declared either way and a filter over `audience: [engineer, operator]` has to
/// see both. A facet the document does not state has no values at all, which is
/// what makes an `include` clause deny by default.
fn values_at(facets: &Mapping, name: &str) -> Vec<String> {
    let Some(node) = facets.get(name) else {
        return Vec::new();
    };
    match &node.value {
        Value::Seq(items) => items
            .iter()
            .filter_map(|item| item.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .collect(),
        other => other
            .as_scalar()
            .map(|scalar| vec![scalar.text.clone()])
            .unwrap_or_default(),
    }
}

/// An export profile: one audience, one filter, one tombstone grain, and every
/// projection that serves it.
#[derive(Clone, Debug)]
pub struct Profile {
    /// The audience it serves, which is the name `--profile` selects.
    pub name: String,
    pub filter: Filter,
    pub tombstone: Grain,
    /// The index in [`crate::Projections::declared`] of each entry that names
    /// this profile, in declaration order.
    pub entries: Vec<usize>,
}

/// What one `projections` entry says about the profile it belongs to.
///
/// Held separately from [`Profile`] because a profile is assembled from every
/// entry that names it, and an entry states only its own half.
#[derive(Clone, Debug)]
pub struct Membership {
    pub name: String,
    /// The emitter target, and `None` for an entry that names none. A
    /// `graph_export` that names none takes [`Emitter::Json`], the native
    /// export, because that is the one an adopter gets without asking for a
    /// vocabulary.
    pub emitter: Option<Emitter>,
    pub filter: Filter,
    /// The grain as the entry wrote it, and `None` where it wrote none. Kept
    /// apart from the default so that two entries of one profile, one silent
    /// and one explicit, do not read as a disagreement.
    pub tombstone: Option<Grain>,
    pub span: Span,
}

impl Membership {
    /// Read the profile members of one `projections` entry.
    pub(crate) fn read(
        body: &Mapping,
        index: usize,
        span: Span,
        errors: &mut Vec<DeclarationError>,
    ) -> Membership {
        let name = body
            .get("profile")
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .unwrap_or_else(|| DEFAULT.to_string());

        let emitter = match body.get("format").and_then(|node| node.value.as_scalar()) {
            Some(scalar) => match Emitter::parse(&scalar.text) {
                Some(emitter) => Some(emitter),
                None => {
                    errors.push(DeclarationError {
                        message: format!(
                            "`projections.{index}.format` is `{}`, and the emitter targets are {}",
                            scalar.text,
                            list(&Emitter::ALL.map(|one| one.name()))
                        ),
                        span,
                    });
                    None
                }
            },
            None => None,
        };

        let tombstone = match body
            .get("tombstone")
            .and_then(|node| node.value.as_scalar())
        {
            Some(scalar) => match Grain::parse(&scalar.text) {
                Some(grain) => Some(grain),
                None => {
                    errors.push(DeclarationError {
                        message: format!(
                            "`projections.{index}.tombstone` is `{}`, and the grains are \
                             `counted` and `sealed`",
                            scalar.text
                        ),
                        span,
                    });
                    None
                }
            },
            None => None,
        };

        let mut filter = Filter::default();
        if let Some(node) = body.get("filter") {
            match node.value.as_map() {
                Some(map) => {
                    filter.include = clauses(map, "include");
                    filter.exclude = clauses(map, "exclude");
                }
                None => errors.push(DeclarationError {
                    message: format!(
                        "`projections.{index}.filter` is {}, and a filter is a block of \
                         `include` and `exclude`",
                        node.value.kind_name()
                    ),
                    span,
                }),
            }
        }

        Membership {
            name,
            emitter,
            filter,
            tombstone,
            span,
        }
    }
}

fn clauses(filter: &Mapping, direction: &str) -> Vec<Clause> {
    let Some(map) = filter.get(direction).and_then(|node| node.value.as_map()) else {
        return Vec::new();
    };
    map.iter()
        .map(|entry| Clause {
            facet: entry.key.value.clone(),
            values: match &entry.value.value {
                Value::Seq(items) => items
                    .iter()
                    .filter_map(|item| item.value.as_scalar())
                    .map(|scalar| scalar.text.clone())
                    .collect(),
                other => other
                    .as_scalar()
                    .map(|scalar| vec![scalar.text.clone()])
                    .unwrap_or_default(),
            },
        })
        .collect()
}

fn list(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Group every entry by the profile it names, and refuse a disagreement.
///
/// The confluence rule of the module header, enforced. Two entries that name one
/// profile and declare two filters are refused, and so are two that declare two
/// tombstone grains. An entry that declares neither inherits both, which is what
/// lets an adopter write the filter once on the export and add a shelf index to
/// the same audience without repeating it.
pub(crate) fn group(memberships: &[Membership]) -> Result<Vec<Profile>, Vec<DeclarationError>> {
    let mut profiles: Vec<Profile> = Vec::new();
    let mut errors = Vec::new();
    for (index, membership) in memberships.iter().enumerate() {
        let at = match profiles.iter().position(|one| one.name == membership.name) {
            Some(at) => at,
            None => {
                profiles.push(Profile {
                    name: membership.name.clone(),
                    filter: Filter::default(),
                    tombstone: Grain::Counted,
                    entries: Vec::new(),
                });
                profiles.len() - 1
            }
        };
        let profile = &mut profiles[at];
        profile.entries.push(index);

        if !membership.filter.is_empty() {
            match profile.filter.is_empty() {
                true => profile.filter = membership.filter.clone(),
                false if profile.filter != membership.filter => errors.push(DeclarationError {
                    message: format!(
                        "`projections.{index}` is in the profile `{}` and declares a second \
                         filter for it. A profile is one audience, so it has one filter, and \
                         two would leave the artifact that lost carrying more than the profile \
                         permits",
                        membership.name
                    ),
                    span: membership.span,
                }),
                false => {}
            }
        }

        if let Some(grain) = membership.tombstone {
            match profile.entries.len() == 1 {
                true => profile.tombstone = grain,
                false if declared_grain(memberships, profile) != Some(grain) => {
                    errors.push(DeclarationError {
                        message: format!(
                            "`projections.{index}` is in the profile `{}` and declares the \
                             tombstone grain `{}`, which is not the grain another entry of that \
                             profile declares",
                            membership.name,
                            grain.name()
                        ),
                        span: membership.span,
                    })
                }
                false => profile.tombstone = grain,
            }
        }
    }
    match errors.is_empty() {
        true => Ok(profiles),
        false => Err(errors),
    }
}

/// The grain the earlier entries of a profile already declared, if any.
fn declared_grain(memberships: &[Membership], profile: &Profile) -> Option<Grain> {
    profile
        .entries
        .iter()
        .filter_map(|at| memberships.get(*at))
        .find_map(|membership| membership.tombstone)
}
