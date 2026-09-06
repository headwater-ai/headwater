// SPDX-License-Identifier: Apache-2.0
//! The two declarations kind resolution reads, and nothing else.
//!
//! Kind resolution needs shelves, and it needs to know which kinds are abstract,
//! because [spec 2](../../../../docs/spec/02-taxonomy-model.md#abstract-kinds)
//! says that no document resolves to one. This module reads exactly those two
//! declarations out of a resolved taxonomy and refuses what it cannot use.
//!
//! # What this is not
//!
//! It is not the meta-schema. The meta-schema is
//! [M2](https://github.com/headwater-ai/headwater/milestone/2), it owns shape,
//! and it validates a taxonomy source as a whole. Nothing here validates: a key
//! this module does not read passes through untouched, and a `shelves` block
//! that satisfies resolution can still be a taxonomy that `taxonomy validate`
//! rejects for a reason resolution never looks at. The engine README says the
//! same thing about the loader, for the same reason — a component that guesses
//! at shape is a second schema that nobody declared.
//!
//! It is also not the overlay resolver, which is
//! [#50](https://github.com/headwater-ai/headwater/issues/50). The input here is
//! an already-resolved taxonomy: one mapping of shelves, one mapping of kinds.
//! Where those two mappings come from is the caller's business.

use headwater_meta::pattern::Pattern;
use headwater_yaml::{core_schema, Mapping, Span, Value};

/// The resolved shelves, in declaration order.
#[derive(Clone, Debug, Default)]
pub struct Taxonomy {
    pub shelves: Vec<Shelf>,
    pub kinds: Vec<Kind>,
}

#[derive(Clone, Debug)]
pub struct Shelf {
    pub name: String,
    /// The name a reader sees where an emitter prints this shelf, where the
    /// declaration carries one. `None` is the shelf that declares none, and
    /// the fall-through to [`Shelf::name`] belongs to the emitter rather than
    /// here: a reading that filled this in would leave no caller able to tell
    /// a declared display name from a key that happens to read well.
    pub title: Option<String>,
    pub pattern: Pattern,
    pub body: ShelfBody,
    /// The span of the shelf's name, which is what a finding about the
    /// *declaration* points at.
    pub span: Span,
}

/// Steps 2 and 3 of resolution, as a closed set.
///
/// [Q1](../../../../docs/spec/09-decisions.md#q1--implementation-language) gives
/// the reason for the shape: the classification outcome in the census is a
/// closed set, so a change to it becomes a compiler-generated list of the sites
/// to fix rather than a search.
#[derive(Clone, Debug)]
pub enum ShelfBody {
    /// The directory states the kind, so no facet may restate it.
    Homogeneous { kind: String },
    /// The directory admits several kinds, and a facet says which.
    Heterogeneous {
        discriminator: String,
        kinds: Vec<String>,
    },
}

#[derive(Clone, Debug)]
pub struct Kind {
    pub name: String,
    /// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#abstract-kinds): an
    /// abstract kind is a kind that no document ever is.
    pub is_abstract: bool,
}

/// A declaration this module could not read, at the line that declared it.
#[derive(Clone, Debug)]
pub struct DeclarationError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for DeclarationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.span.start, self.message)
    }
}

impl std::error::Error for DeclarationError {}

impl Taxonomy {
    /// Read `shelves` and `kinds` from the root of a resolved taxonomy.
    ///
    /// Both are optional at this layer. A taxonomy with no shelves classifies
    /// nothing, which is a census in which every file is unshelved, and that is
    /// a true report rather than an error.
    pub fn read(root: &Mapping) -> Result<Self, Vec<DeclarationError>> {
        let mut errors = Vec::new();
        let mut taxonomy = Taxonomy::default();

        if let Some(shelves) = root.get("shelves") {
            match &shelves.value {
                Value::Map(map) => {
                    for entry in map {
                        match read_shelf(&entry.key.value, &entry.value.value, entry.key.span) {
                            Ok(shelf) => taxonomy.shelves.push(shelf),
                            Err(error) => errors.push(error),
                        }
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!("`shelves` is {}, and it names shelves", other.kind_name()),
                    span: shelves.span,
                }),
            }
        }

        if let Some(kinds) = root.get("kinds") {
            match &kinds.value {
                Value::Map(map) => {
                    for entry in map {
                        taxonomy.kinds.push(Kind {
                            name: entry.key.value.clone(),
                            is_abstract: is_abstract(&entry.value.value),
                        });
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!("`kinds` is {}, and it names kinds", other.kind_name()),
                    span: kinds.span,
                }),
            }
        }

        if errors.is_empty() {
            Ok(taxonomy)
        } else {
            Err(errors)
        }
    }

    pub fn kind(&self, name: &str) -> Option<&Kind> {
        self.kinds.iter().find(|kind| kind.name == name)
    }
}

fn is_abstract(value: &Value) -> bool {
    value
        .as_map()
        .and_then(|map| core_schema::flag(map, "abstract"))
        .unwrap_or(false)
}

fn read_shelf(name: &str, value: &Value, span: Span) -> Result<Shelf, DeclarationError> {
    let map = value.as_map().ok_or_else(|| DeclarationError {
        message: format!(
            "shelf `{name}` is {}, and a shelf is a mapping",
            value.kind_name()
        ),
        span,
    })?;

    let path = scalar(map, "path").ok_or_else(|| DeclarationError {
        message: format!("shelf `{name}` declares no `path`, so no file can reach it"),
        span,
    })?;

    // `homogeneous` is declared in every shelf this project has written, and the
    // body is what decides the meaning: a shelf that says it is homogeneous and
    // names no kind has said nothing. Reading the flag and then requiring the
    // matching body reports the disagreement instead of resolving it silently.
    let declared = core_schema::flag(map, "homogeneous");
    let homogeneous = declared.unwrap_or_else(|| map.get("kind").is_some());

    let body = if homogeneous {
        let kind = scalar(map, "kind").ok_or_else(|| DeclarationError {
            message: format!(
                "shelf `{name}` is homogeneous and names no `kind`, so placement states nothing"
            ),
            span,
        })?;
        ShelfBody::Homogeneous { kind }
    } else {
        let discriminator = scalar(map, "discriminator").ok_or_else(|| DeclarationError {
            message: format!(
                "shelf `{name}` is heterogeneous and names no `discriminator`, \
                 so nothing says which kind a document is"
            ),
            span,
        })?;
        let kinds = sequence(map, "kinds").ok_or_else(|| DeclarationError {
            message: format!("shelf `{name}` is heterogeneous and names no `kinds`"),
            span,
        })?;
        ShelfBody::Heterogeneous {
            discriminator,
            kinds,
        }
    };

    Ok(Shelf {
        name: name.to_string(),
        title: scalar(map, "title"),
        pattern: Pattern::new(&path),
        body,
        span,
    })
}

fn scalar(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|value| value.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

fn sequence(map: &Mapping, key: &str) -> Option<Vec<String>> {
    let items = map.get(key)?.value.as_seq()?;
    Some(
        items
            .iter()
            .filter_map(|item| item.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(source: &str) -> Result<Taxonomy, Vec<DeclarationError>> {
        let root = headwater_yaml::load(source).expect("the fixture loads");
        Taxonomy::read(root.value.as_map().expect("a mapping"))
    }

    const SOURCE: &str = "\
kinds:
  governed_document: {abstract: true}
  design_spec: {is_a: governed_document}
  evaluation: {is_a: governed_document}
shelves:
  spec_series:
    path: docs/spec/**
    homogeneous: false
    discriminator: doc_type
    kinds: [design_spec, decision_register]
  evaluations:
    path: docs/evaluations/**
    homogeneous: true
    kind: evaluation
";

    #[test]
    fn both_shelf_bodies_read() {
        let taxonomy = read(SOURCE).expect("reads");
        assert_eq!(taxonomy.shelves.len(), 2);
        match &taxonomy.shelves[0].body {
            ShelfBody::Heterogeneous {
                discriminator,
                kinds,
            } => {
                assert_eq!(discriminator, "doc_type");
                assert_eq!(kinds, &["design_spec", "decision_register"]);
            }
            other => panic!("{other:?}"),
        }
        match &taxonomy.shelves[1].body {
            ShelfBody::Homogeneous { kind } => assert_eq!(kind, "evaluation"),
            other => panic!("{other:?}"),
        }
    }

    /// The declared display name reaches the reading, and the shelf that
    /// declares none reads as `None` rather than as its own key.
    ///
    /// Both halves in one case on purpose. A reading that filled the key in
    /// would satisfy the first assertion and fail the second, and it is the
    /// second that keeps the fall-through in one place —
    /// `headwater_generate::shelf_label` — where three emitters read it.
    #[test]
    fn a_declared_display_name_is_read_and_an_undeclared_one_is_not_invented() {
        let taxonomy = read(&SOURCE.replace(
            "  spec_series:\n",
            "  spec_series:\n    title: The specification series\n",
        ))
        .expect("reads");
        assert_eq!(
            taxonomy.shelves[0].title.as_deref(),
            Some("The specification series")
        );
        assert_eq!(taxonomy.shelves[1].title, None);
    }

    #[test]
    fn an_abstract_kind_is_read_as_one() {
        let taxonomy = read(SOURCE).expect("reads");
        assert!(
            taxonomy
                .kind("governed_document")
                .expect("declared")
                .is_abstract
        );
        assert!(!taxonomy.kind("design_spec").expect("declared").is_abstract);
        assert!(taxonomy.kind("playbook").is_none());
    }

    #[test]
    fn a_shelf_that_says_nothing_about_its_kinds_is_refused_at_its_own_line() {
        let errors = read("shelves:\n  broken:\n    path: docs/**\n    homogeneous: true\n")
            .expect_err("refused");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].span.start.line, 2);
        assert!(
            errors[0].to_string().contains("names no `kind`"),
            "{}",
            errors[0]
        );
    }

    #[test]
    fn declaration_order_is_kept_because_a_report_is_read_by_a_person() {
        let taxonomy = read(SOURCE).expect("reads");
        let names: Vec<&str> = taxonomy.shelves.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["spec_series", "evaluations"]);
    }
}
