// SPDX-License-Identifier: Apache-2.0
//! The templates a package ships, held against the taxonomy that package
//! resolves to.
//!
//! A template is the file an adopter copies to start a document, and until
//! [#378](https://github.com/headwater-ai/headwater/issues/378) nothing in this
//! engine opened one. `required_kind` in [`crate::package`] stated the seam in
//! its own words — *a key that is not here keeps the existence check and nothing
//! more* — and `templates` was the example it named. The cost of that seam was
//! measured on [PR #376](https://github.com/headwater-ai/headwater/pull/376):
//! two of one entry's three templates wrote `spec_layer: functional` where the
//! bundle beside them declares `functional_spec`, and a document copied from
//! either one resolves to no kind, so no rule reads it, no check reports it, and
//! `headwater check --strict` exits 0. Five commits, a green CI run, the commit
//! gate and four harness hooks all passed it. A person reading three files by
//! hand caught it.
//!
//! # Where the reader runs, and why it is `publish`
//!
//! [`crate::package::publish_at`] calls [`holds`] after the two name
//! declarations are held to each other and before `--out` is observed, so a
//! refusal here fires before an output directory exists. `taxonomy validate`
//! cannot carry this: it resolves the *consumer's* selection, and a bundle no
//! consumer selected is a bundle it never reads. Publish is the only verb that
//! sees every bundle a package ships, and it is the last moment anybody can be
//! stopped before an adopter copies the file.
//!
//! # What it reads, and what it refuses to read
//!
//! The kind a template teaches is its file stem. Every template in this
//! repository's library names one — `decision.md` teaches `decision`,
//! `functional_spec.md` teaches `functional_spec` — and front matter alone
//! cannot say, because a homogeneous shelf carries no discriminator at all.
//!
//! Five checks, and no sixth:
//!
//! 1. The stem names a kind the package declares, and that kind is not abstract.
//! 2. Every top-level key that names a facet with an enumerated `values` list
//!    carries a value in that list.
//! 3. A facet some shelf uses as a discriminator carries the stem's own kind
//!    name.
//! 4. No facet the stem's kind forbids is present.
//! 5. No top-level key names a relation the package declares, the `inverse`
//!    half of one included.
//!
//! That count is prose and nothing reads it. `BRANCHES` is the list a test
//! holds, and it is longer than five because three of its branches are about
//! reading the file rather than about what the file says.
//!
//! **A placeholder is skipped rather than typed.** A placeholder is a scalar
//! whose trimmed text opens `{{` and closes `}}`. Without that rule the reader
//! would refuse `status_since: "{{today}}"` on every template in the library,
//! because `status_since` declares `type: date`. The corollary is that this
//! reader does no type checking at all: a template is placeholder-bearing by
//! construction and typing one is a category error.
//!
//! **Required-facet presence is deliberately not here.** A template is a
//! skeleton an author fills, so an absent key is not a defect and a wrong value
//! is. Five templates of this library carry no `id:` at all and every one of
//! them is correct.
//!
//! **Front matter that no loader accepts is its own refusal.** Skipping it would
//! give any publisher a documented bypass of every check above — ship a template
//! that does not parse and the reader reports a pass. A validator with a
//! documented bypass is worse than no validator.
//!
//! **A Markdown file that declares no front matter at all is not a template —
//! unless its own stem states a kind this package declares.** A file whose
//! stem names no kind (spec 7's own example manifest ships `templates/note.md`
//! as prose beside the templates, under this very key) is skipped: every check
//! above reads front matter, so skipping it takes no check away. But a stem
//! that names a kind is a template's own claim to teach it, and this reader
//! cannot tell "deliberately no front matter" apart from "a block exists and
//! the parser failed to see it" — a leading blank line is exactly that case,
//! and it is publisher-controllable. So a kind-named stem with no readable
//! block is refused, on the same ground as the paragraph before it: the
//! difference between a block that opens and does not parse and a block this
//! reader cannot find at all is not one this reader can see, and neither may
//! be a bypass.

use crate::error::ResolveError;
use headwater_yaml::{Mapping, Value};
use std::path::{Path, PathBuf};

/// Every branch of [`Reason`], named once.
///
/// The macro writes two things out of this one list: [`Reason::variant`], which
/// gives a refusal a name that a test can assert against rather than a message
/// shape, and [`BRANCHES`], which is what the coverage case in
/// `tests/templates.rs` holds the fixture suite to. PR #461 found what two
/// hand-kept lists cost — three branches were in the match and absent from the
/// constant, so a test claiming total coverage quietly excluded them — and one
/// list cannot drift from itself. The match is exhaustive rather than a `_ =>`,
/// so a branch added to the enum does not compile until it is named here, and it
/// then fails the coverage case until a fixture reaches it.
macro_rules! branches {
    ($($name:ident),+ $(,)?) => {
        impl Reason {
            /// The name of this branch, for a fixture that asserts coverage.
            pub fn variant(&self) -> &'static str {
                match self {
                    $(Reason::$name { .. } => stringify!($name),)+
                }
            }
        }

        /// Every refusal this reader can produce, by name.
        pub const BRANCHES: &[&str] = &[$(stringify!($name)),+];
    };
}

/// One template this reader refuses, and why.
#[derive(Clone, Debug)]
pub struct Refused {
    /// The template's own path, as a reader of the repository would write it.
    /// A person edits this file, and the manifest that ships it is not where
    /// the defect is.
    pub path: String,
    pub reason: Reason,
}

/// Why one template is refused.
#[derive(Clone, Debug)]
pub enum Reason {
    /// The file could not be read as text at all.
    Unreadable { why: String },
    /// The front matter is not YAML a loader accepts. `{{today}}` unquoted is
    /// the live case: YAML reads it as a flow mapping in the position of a
    /// value, and the whole block is then unreadable.
    Unparseable { why: String },
    /// The stem names no kind the package declares.
    KindUnknown { kind: String },
    /// The stem names an abstract kind, and spec 2 says no document is one.
    KindAbstract { kind: String },
    /// A facet with an enumerated value set carries a value outside it.
    FacetValueNotPermitted {
        facet: String,
        value: String,
        permitted: Vec<String>,
        /// Whether some shelf discriminates on this facet, which decides what
        /// a document carrying the value loses.
        discriminates: bool,
    },
    /// A discriminator carries a value that is in the facet's set and is not
    /// the kind this template teaches.
    DiscriminatorDisagrees {
        facet: String,
        value: String,
        kind: String,
    },
    /// The kind forbids the facet, and the template declares it anyway.
    ForbiddenFacetPresent { facet: String, kind: String },
    /// The stem names a declared kind, and the file carries no front-matter
    /// block a parser can find. Unlike a genuine prose file (skipped, see
    /// `read_one`), a stem that names a kind is a template's own claim to
    /// teach it, and that claim must be checkable — including when the block
    /// exists but the parser cannot see it, which is indistinguishable from
    /// its own perspective and must not be indistinguishable from a refusal.
    NoFrontMatterForKnownKind { kind: String },
    /// A relation the taxonomy declares is written as a top-level key.
    /// [HW-DR-0004](../../../../docs/decisions/0004-relation-storage.md) puts a
    /// relation under a `relations:` block and nowhere else, so a top-level key
    /// of that name is an edge no reader of the graph can see.
    RelationAtTopLevel { relation: String },
}

branches![
    Unreadable,
    Unparseable,
    KindUnknown,
    KindAbstract,
    FacetValueNotPermitted,
    DiscriminatorDisagrees,
    ForbiddenFacetPresent,
    NoFrontMatterForKnownKind,
    RelationAtTopLevel,
];

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::Unreadable { why } => write!(
                f,
                "this template cannot be read: {why}. A publisher ships it and an adopter copies \
                 it, so a file nothing can open is a file nothing can hold to the taxonomy \
                 beside it"
            ),
            Reason::Unparseable { why } => write!(
                f,
                "the front matter of this template does not parse: {why}. A placeholder written \
                 bare is read as a flow mapping, and a quoted placeholder parses: write \
                 `status_since: \"{{{{today}}}}\"`. Nothing substitutes into a template, so the \
                 quotes are what an author deletes along with the placeholder"
            ),
            Reason::KindUnknown { kind } => write!(
                f,
                "the file name states the kind a template teaches, and `{kind}` is not a kind \
                 this package declares. A person copies this file to start a document, and a \
                 document of no kind resolves to no kind, so no rule reads it and no check \
                 reports it"
            ),
            Reason::KindAbstract { kind } => write!(
                f,
                "the file name states the kind a template teaches, and `{kind}` is abstract. No \
                 document is ever an abstract kind, so a person who copies this file starts a \
                 document that resolves to no kind, and no rule reads it"
            ),
            Reason::FacetValueNotPermitted {
                facet,
                value,
                permitted,
                discriminates,
            } => write!(
                f,
                "`{facet}: {value}` is not a value `facets.{facet}` declares ({}). {}",
                permitted
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<String>>()
                    .join(", "),
                match discriminates {
                    true =>
                        "A person copies this file to start a document, and a document carrying \
                         this value resolves to no kind, so no rule reads it and no check reports \
                         it",
                    false =>
                        "A person copies this file to start a document, and every document they \
                         start carries the value, so the first check an adopter runs reports each \
                         one of them",
                }
            ),
            Reason::DiscriminatorDisagrees { facet, value, kind } => write!(
                f,
                "`{facet}: {value}` disagrees with the kind this template teaches, which its file \
                 name states as `{kind}`. A shelf reads `{facet}` to decide which kind a document \
                 is, so a person who copies this file starts a `{value}` under a `{kind}` \
                 heading, and one of the two is wrong"
            ),
            Reason::ForbiddenFacetPresent { facet, kind } => write!(
                f,
                "`{facet}` is a facet `kinds.{kind}` forbids, and this template teaches `{kind}`. \
                 A person copies this file to start a document, and the document carries a facet \
                 its own kind refuses"
            ),
            Reason::NoFrontMatterForKnownKind { kind } => write!(
                f,
                "the file name states the kind a template teaches, `{kind}`, and this file has no \
                 front-matter block a parser can find — no `---` on its own first line. A template \
                 with no checkable block is indistinguishable from one this reader never saw, so a \
                 file named for a real kind is held to it even here: add the block, or rename the \
                 file so it no longer claims a kind"
            ),
            Reason::RelationAtTopLevel { relation } => write!(
                f,
                "`{relation}` is a relation this package declares, and this template writes it as \
                 a top-level key. A relation is declared under a `relations:` block and nowhere \
                 else, so a person who copies this file starts a document whose `{relation}` edge \
                 no graph reads and no reciprocal check reports. Move it under `relations:`, and \
                 write the target as an identifier"
            ),
        }
    }
}

/// Hold every template a package ships to the taxonomy it resolves to.
///
/// The taxonomy is the maximal selection — the base with every bundle the
/// package ships — because that is the most any artifact can hold and a bundle
/// is add-only, so a value some selection admits is a value this artifact
/// admits. [`crate::package::shipped`] is what builds it and its doc comment
/// carries the argument.
///
/// **Every bad template is reported, not the first one.** `reachable` and
/// `agrees` collect for the reason `reachable`'s doc comment states: a second
/// run should not have to discover the second defect.
pub(crate) fn holds(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    taxonomy: &Mapping,
) -> Result<(), Vec<ResolveError>> {
    let refused = refusals(root, directory, manifest, taxonomy);
    match refused.is_empty() {
        true => Ok(()),
        false => Err(refused
            .iter()
            .map(|one| crate::package::refusal_at(&one.path, &one.reason.to_string()))
            .collect()),
    }
}

/// The same walk, as structured reasons rather than as errors.
///
/// The fixture suite reads this so that a case asserts against a branch name
/// rather than against a sentence, which is what keeps [`BRANCHES`] holdable.
pub fn refusals(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    taxonomy: &Mapping,
) -> Vec<Refused> {
    let mut out = Vec::new();
    for path in files(directory, manifest) {
        read_one(
            &crate::package::display(root, &path),
            &path,
            taxonomy,
            &mut out,
        );
    }
    out
}

/// Every template file a package ships, in a settled order.
///
/// Two shapes reach this, and both are walked. `<contents.bundles>/<name>/
/// templates/*.md` is where this repository's library keeps them, one set per
/// bundle, and it is the only shape any manifest here declares.
/// `<contents.templates>/*.md` is the package-level shape spec 7's example
/// manifest block writes; no manifest in this repository declares it, and
/// covering it costs one directory read.
///
/// A bundle directory with no `bundle.yml` is skipped, on the rule
/// [`crate::package::shipped`] already applies: it is a bundle no consumer can
/// select, so its templates are held against a taxonomy that does not include
/// it. `bundles/README.md`, a regular file this repository's own bundle root
/// carries, is skipped by the same rule.
fn files(directory: &Path, manifest: &Mapping) -> Vec<PathBuf> {
    let contents = crate::package::contents_of(manifest);
    let mut out = Vec::new();

    if let Some(declared) = scalar(&contents, crate::package::TEMPLATES) {
        out.extend(markdown_under(&directory.join(declared)));
    }

    if let Some(declared) = scalar(&contents, crate::package::BUNDLES) {
        let at = directory.join(declared);
        let Ok(entries) = std::fs::read_dir(&at) else {
            return out;
        };
        let mut bundles: Vec<std::ffi::OsString> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| at.join(entry.file_name()).join("bundle.yml").is_file())
            .map(|entry| entry.file_name())
            .collect();
        bundles.sort();
        for name in bundles {
            out.extend(markdown_under(
                &at.join(name).join(crate::package::TEMPLATES),
            ));
        }
    }

    out
}

/// Every `*.md` directly under one directory, by name.
fn markdown_under(at: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(at) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|kind| kind == "md"))
        .collect();
    out.sort();
    out
}

/// One template, and every refusal it earns.
fn read_one(named: &str, at: &Path, taxonomy: &Mapping, out: &mut Vec<Refused>) {
    let mut refuse = |reason| {
        out.push(Refused {
            path: named.to_string(),
            reason,
        });
    };

    let text = match std::fs::read_to_string(at) {
        Ok(text) => text,
        Err(error) => {
            return refuse(Reason::Unreadable {
                why: error.to_string(),
            })
        }
    };

    let Some(stem) = at.file_stem().and_then(|stem| stem.to_str()) else {
        return refuse(Reason::Unreadable {
            why: "its name is not text".to_string(),
        });
    };

    let kinds = taxonomy.get("kinds").and_then(|node| node.value.as_map());
    let named_kind = kinds
        .and_then(|kinds| kinds.get(stem))
        .and_then(|node| node.value.as_map());

    let document = match headwater_doc::parse(&text) {
        Ok(document) => document,
        // A Markdown file with no `---` block at all is not a document
        // skeleton, so it is not a template and this reader says nothing about
        // it — UNLESS its own stem states a kind this package declares. Spec
        // 7's own example manifest ships `templates/note.md` as "prose for a
        // publisher's own authors", and `note` names no kind, so it is skipped
        // on that ground alone. A file named `technical_spec.md` is a
        // template's own claim to teach that kind, and this reader cannot
        // tell "deliberately no front matter" apart from "a block exists and
        // `headwater-doc` failed to see it" (a leading blank line is exactly
        // this — the block is real one line down and unreachable from here).
        // Skipping on stem alone would make that indistinguishable case a
        // silent pass, which is the bypass a block that opens and does not
        // parse is already refused for.
        Err(errors)
            if errors
                .iter()
                .all(|error| error.reason == headwater_doc::Reason::NoFrontMatter) =>
        {
            return match named_kind {
                None => (),
                Some(_) => refuse(Reason::NoFrontMatterForKnownKind {
                    kind: stem.to_string(),
                }),
            };
        }
        Err(errors) => {
            return refuse(Reason::Unparseable {
                why: errors
                    .iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<String>>()
                    .join("; "),
            })
        }
    };

    let Some(declaration) = named_kind else {
        return refuse(Reason::KindUnknown {
            kind: stem.to_string(),
        });
    };
    if headwater_yaml::core_schema::flag(declaration, "abstract").unwrap_or(false) {
        return refuse(Reason::KindAbstract {
            kind: stem.to_string(),
        });
    }

    let forbidden = forbidden(kinds, stem);
    let discriminators = discriminators(taxonomy);
    let relations = relations(taxonomy);

    for entry in &document.facets {
        let facet = entry.key.value.as_str();

        // The key is in the wrong place, so nothing about its value excuses it
        // and this arm runs first. A relation name at the top level is not a
        // facet at all, so every arm below would read it as one.
        if relations.iter().any(|name| name == facet) {
            refuse(Reason::RelationAtTopLevel {
                relation: facet.to_string(),
            });
            continue;
        }

        // Presence is the defect here, so a placeholder does not excuse it and
        // this arm runs before the skip below.
        if forbidden.iter().any(|name| name == facet) {
            refuse(Reason::ForbiddenFacetPresent {
                facet: facet.to_string(),
                kind: stem.to_string(),
            });
            continue;
        }

        let Some(scalar) = entry.value.value.as_scalar() else {
            continue;
        };
        if is_placeholder(&scalar.text) {
            continue;
        }

        if let Some(permitted) = permitted(taxonomy, facet) {
            if !permitted.iter().any(|value| value == &scalar.text) {
                refuse(Reason::FacetValueNotPermitted {
                    facet: facet.to_string(),
                    value: scalar.text.clone(),
                    permitted,
                    discriminates: discriminators.iter().any(|name| name == facet),
                });
                continue;
            }
        }

        if discriminators.iter().any(|name| name == facet) && scalar.text != stem {
            refuse(Reason::DiscriminatorDisagrees {
                facet: facet.to_string(),
                value: scalar.text.clone(),
                kind: stem.to_string(),
            });
        }
    }
}

/// A scalar an author has not filled in yet.
///
/// The rule is stated in
/// [spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing)
/// and it is what keeps this reader off `status_since: "{{today}}"`, whose facet
/// declares `type: date`.
fn is_placeholder(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("{{") && trimmed.ends_with("}}") && trimmed.len() >= 4
}

/// The enumerated values one facet declares, and nothing where it declares no
/// list.
///
/// It reads a resolved taxonomy and never a source, for the reason
/// [`crate::migration::declares`] gives: `values: $vocabularies.lifecycle_state`
/// is a reference, and a reader of the source would answer that `status`
/// declares no value at all.
fn permitted(taxonomy: &Mapping, facet: &str) -> Option<Vec<String>> {
    let items = taxonomy
        .get("facets")
        .and_then(|node| node.value.as_map())
        .and_then(|facets| facets.get(facet))
        .and_then(|node| node.value.as_map())
        .and_then(|declaration| declaration.get("values"))
        .and_then(|node| node.value.as_seq())?;
    Some(
        items
            .iter()
            .filter_map(|item| crate::migration::named(&item.value))
            .map(str::to_string)
            .collect(),
    )
}

/// Every relation name a document of this taxonomy can write, the inverse
/// halves included.
///
/// It reads the resolved taxonomy rather than a bundle source for the reason
/// [`permitted`] gives: a bundle adds relations to the base, and a reader of one
/// source answers for one bundle.
///
/// **The `inverse` values are half the population and the whole of the defect.**
/// A reciprocal half is not a key of `relations:` — `design-spec` declares
/// `applied_in` and `cites_evidence`, and `applies` and `cited_by` reach a
/// document only as the `inverse` those two name. Two of the four keys the
/// library shipped at the top level are of that half, so a walk over the keys
/// alone reports two of the four and reads as a working check.
///
/// **It is the second enumeration of this population, and a test is what holds
/// it to the first.** `headwater_graph::declarations::Declarations::named` is
/// the reader a real document meets, and it admits the same two halves. This
/// crate cannot call it, because `headwater-graph` depends on this one and the
/// reverse is a dependency inversion, so the two are held against each other
/// rather than merged: `headwater-graph`'s `tests/vocabulary.rs` resolves this
/// repository and fails on any name one collects and the other does not. It is
/// `pub` for that test and for no caller.
pub fn relations(taxonomy: &Mapping) -> Vec<String> {
    let Some(declared) = taxonomy
        .get("relations")
        .and_then(|node| node.value.as_map())
    else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for entry in declared {
        let name = entry.key.value.as_str().to_string();
        if !out.contains(&name) {
            out.push(name);
        }
        let Some(inverse) = entry
            .value
            .value
            .as_map()
            .and_then(|relation| relation.get("inverse"))
            .and_then(|node| node.value.as_scalar())
        else {
            continue;
        };
        if !out.contains(&inverse.text) {
            out.push(inverse.text.clone());
        }
    }
    out
}

/// Every facet that some shelf reads to decide a document's kind.
fn discriminators(taxonomy: &Mapping) -> Vec<String> {
    let Some(shelves) = taxonomy.get("shelves").and_then(|node| node.value.as_map()) else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for entry in shelves {
        let Some(name) = entry
            .value
            .value
            .as_map()
            .and_then(|shelf| shelf.get("discriminator"))
            .and_then(|node| node.value.as_scalar())
        else {
            continue;
        };
        if !out.contains(&name.text) {
            out.push(name.text.clone());
        }
    }
    out
}

/// Every facet the kind forbids, its ancestors included.
///
/// The walk climbs `is_a` and is bounded by the number of declared kinds, so a
/// taxonomy whose chain runs in a circle stops rather than spins. `check`'s
/// `Shape::ancestry` reads the same chain for the same reason; that crate
/// depends on this one, so the walk is here rather than borrowed from it.
fn forbidden(kinds: Option<&Mapping>, of: &str) -> Vec<String> {
    let Some(kinds) = kinds else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut next = Some(of.to_string());
    while let Some(name) = next {
        if seen.contains(&name) {
            break;
        }
        seen.push(name.clone());
        let Some(declaration) = kinds.get(&name).and_then(|node| node.value.as_map()) else {
            break;
        };
        if let Some(list) = declaration
            .get("facets")
            .and_then(|node| node.value.as_map())
            .and_then(|facets| facets.get("forbid"))
            .and_then(|node| node.value.as_seq())
        {
            for item in list {
                if let Value::Scalar(scalar) = &item.value {
                    if !out.contains(&scalar.text) {
                        out.push(scalar.text.clone());
                    }
                }
            }
        }
        next = declaration
            .get("is_a")
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone());
    }
    out
}

fn scalar(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_placeholder_is_a_placeholder_and_a_word_is_not() {
        assert!(is_placeholder("{{today}}"));
        assert!(is_placeholder("  {{human | agent | mixed}}  "));
        assert!(!is_placeholder("functional"));
        assert!(!is_placeholder("{}"));
        assert!(!is_placeholder("{{today"));
    }

    #[test]
    fn every_branch_of_the_enum_is_named_once() {
        let mut sorted = BRANCHES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            BRANCHES.len(),
            "a branch is named twice in `branches!`"
        );
    }
}
