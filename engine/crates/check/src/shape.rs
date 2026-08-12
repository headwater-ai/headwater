// SPDX-License-Identifier: Apache-2.0
//! The declarations the Shape and Graph checks are generated from: `facets`
//! and `kinds`.
//!
//! The same posture as [`crate::register`], [`headwater_census::shelves`] and
//! [`headwater_graph::declarations`], for the same reason. Nothing here
//! validates a taxonomy. The meta-schema owns shape, `taxonomy validate` owns
//! referential integrity, and this module reads what a generated check needs
//! and refuses only what it cannot use.
//!
//! # Why this is not an addition to `headwater_census::shelves`
//!
//! That module opens by saying what it is: "the two declarations kind
//! resolution reads, and nothing else". It reads the `abstract` flag on a kind
//! and no other field, on purpose, because the meta-schema owns shape.
//! [#56](https://github.com/headwater-ai/headwater/issues/56) left the choice
//! open between widening that reader and adding one. Adding one keeps each
//! reader's list of fields equal to what its own phase needs, which is the
//! property that makes any of them readable.
//!
//! # The `is_a` chain, and the three checks that need it
//!
//! A kind inherits from its parent. `governed_document` requires four facets
//! and `design_spec` requires two more, so a design spec owes six. A relation
//! that declares `to: [governed_document]` admits a `review_record`, because a
//! review record is one. Neither fact is in a document, and both are one walk
//! up [`Kind::is_a`].
//!
//! The walk is bounded by the number of declared kinds. A taxonomy whose `is_a`
//! edges form a cycle is a taxonomy `taxonomy validate` refuses, and a check
//! that looped on one would hang a run instead of reporting it.

use crate::finding::Severity;
use headwater_census::shelves::DeclarationError;
use headwater_yaml::{Mapping, Span, Value};

/// The facet and kind declarations of a resolved taxonomy.
#[derive(Clone, Debug, Default)]
pub struct Shape {
    /// In declaration order, because a report is read by a person.
    pub facets: Vec<Facet>,
    pub kinds: Vec<Kind>,
}

/// A facet, read down to what a generated check needs.
#[derive(Clone, Debug)]
pub struct Facet {
    pub name: String,
    /// What the facet is for. The invariant core names roles rather than facet
    /// names, and a participation expectation says `since: state_entered`
    /// rather than `since: status_since`.
    pub role: Option<String>,
    /// Whether every document declares it, whatever its kind.
    pub required: bool,
    /// The values the facet admits, and empty for a facet that declares no set.
    /// A vocabulary reference is already resolved in the lock, so both spellings
    /// arrive here as a list.
    pub values: Vec<String>,
    pub span: Span,
}

/// A kind, read down to what a generated check needs.
#[derive(Clone, Debug)]
pub struct Kind {
    pub name: String,
    pub is_a: Option<String>,
    /// `facets.require`, as this kind declares it and without its ancestors.
    /// [`Shape::required_facets`] is the inherited set.
    pub require: Vec<String>,
    pub forbid: Vec<String>,
    /// `relations.expect`, which is where a windowed participation expectation
    /// is declared ([spec 2](../../../../docs/spec/02-taxonomy-model.md#participation-expectations)).
    pub expectations: Vec<Expectation>,
    pub span: Span,
}

/// One windowed participation expectation, as the taxonomy declares it.
///
/// Three fields are optional because a value this engine cannot read is kept
/// rather than dropped. A dropped expectation is an instance that never
/// existed, and a coverage report cannot say why. A kept one is an instance
/// that skips with a reason, which is what
/// [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
/// asks for.
#[derive(Clone, Debug)]
pub struct Expectation {
    pub id: String,
    /// The relation name as the declaration writes it, which may be an inverse.
    pub relation: String,
    /// The kind the far end must be, or any kind when the declaration names
    /// none.
    pub to_kind: Option<String>,
    /// The facet values a document must carry for this expectation to apply.
    pub when: Vec<(String, String)>,
    /// The window in whole days, and nothing for a window this engine cannot
    /// read.
    pub within_days: Option<i64>,
    /// The facet *role* the window is measured from.
    pub since_role: String,
    /// The severity of the finding, and nothing for a word outside the set.
    pub severity: Option<Severity>,
    pub rationale: Option<String>,
}

impl Shape {
    /// Read `facets` and `kinds` from the root of a resolved taxonomy.
    ///
    /// Both are optional at this layer. A taxonomy that declares neither
    /// generates no Shape check, which is a true report of a taxonomy that
    /// declares nothing for one to be generated from.
    pub fn read(root: &Mapping) -> Result<Self, Vec<DeclarationError>> {
        let mut errors = Vec::new();
        let mut shape = Shape::default();

        if let Some(facets) = root.get("facets") {
            match &facets.value {
                Value::Map(map) => {
                    for entry in map {
                        match read_facet(&entry.key.value, &entry.value.value, entry.key.span) {
                            Ok(facet) => shape.facets.push(facet),
                            Err(error) => errors.push(error),
                        }
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!("`facets` is {}, and it names facets", other.kind_name()),
                    span: facets.span,
                }),
            }
        }

        if let Some(kinds) = root.get("kinds") {
            match &kinds.value {
                Value::Map(map) => {
                    for entry in map {
                        match read_kind(&entry.key.value, &entry.value.value, entry.key.span) {
                            Ok(kind) => shape.kinds.push(kind),
                            Err(error) => errors.push(error),
                        }
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!("`kinds` is {}, and it names kinds", other.kind_name()),
                    span: kinds.span,
                }),
            }
        }

        if errors.is_empty() {
            Ok(shape)
        } else {
            Err(errors)
        }
    }

    pub fn facet(&self, name: &str) -> Option<&Facet> {
        self.facets.iter().find(|facet| facet.name == name)
    }

    pub fn kind(&self, name: &str) -> Option<&Kind> {
        self.kinds.iter().find(|kind| kind.name == name)
    }

    /// The facet that carries a role, and nothing when no facet does.
    ///
    /// The first one wins. Two facets in one role is a taxonomy defect that
    /// `taxonomy validate` owns, and guessing between them here would put a
    /// second opinion about it in the check layer.
    pub fn facet_in_role(&self, role: &str) -> Option<&Facet> {
        self.facets
            .iter()
            .find(|facet| facet.role.as_deref() == Some(role))
    }

    /// A kind and every kind it descends from, nearest first.
    ///
    /// Bounded by the number of declared kinds, so a cycle stops rather than
    /// hangs. See the module comment.
    pub fn ancestry(&self, name: &str) -> Vec<&Kind> {
        let mut chain = Vec::new();
        let mut next = Some(name.to_string());
        while let Some(current) = next {
            let Some(kind) = self.kind(&current) else {
                break;
            };
            if chain.len() >= self.kinds.len() {
                break;
            }
            chain.push(kind);
            next = kind.is_a.clone();
        }
        chain
    }

    /// Whether a kind is an `ancestor`, itself included.
    ///
    /// This is what makes `to: [governed_document]` admit a `review_record`. A
    /// relation endpoint that compared the two names directly would report
    /// every inherited endpoint in the corpus as a violation.
    pub fn descends_from(&self, kind: &str, ancestor: &str) -> bool {
        self.ancestry(kind).iter().any(|step| step.name == ancestor)
    }

    /// Every facet a document of this kind owes, in one order.
    ///
    /// Three declarations decide it, and this is the only place they are read
    /// together. A facet that declares `required: true` is owed by every
    /// document. A kind's `facets.require` adds to that, and so does each of
    /// its ancestors'. A kind's `facets.forbid` takes away, because a kind that
    /// forbids a facet and inherits a requirement for it is a contradiction
    /// that `taxonomy validate` reports — and telling an author to add a facet
    /// their own kind forbids would be a second, wrong report of it.
    pub fn required_facets(&self, kind: &str) -> Vec<String> {
        let ancestry = self.ancestry(kind);
        let forbidden: Vec<&str> = ancestry
            .iter()
            .flat_map(|step| step.forbid.iter().map(String::as_str))
            .collect();

        let mut required: Vec<String> = Vec::new();
        let owe = |name: &str, required: &mut Vec<String>| {
            if !forbidden.contains(&name) && !required.iter().any(|known| known == name) {
                required.push(name.to_string());
            }
        };

        // Facet declaration order first, then the kinds from the root of the
        // chain down. So the order a report prints is the order a taxonomy
        // reads, and it does not move when a kind gains a parent.
        for facet in self.facets.iter().filter(|facet| facet.required) {
            owe(&facet.name, &mut required);
        }
        for step in ancestry.iter().rev() {
            for name in &step.require {
                owe(name, &mut required);
            }
        }
        required
    }
}

fn read_facet(name: &str, value: &Value, span: Span) -> Result<Facet, DeclarationError> {
    let map = value.as_map().ok_or_else(|| DeclarationError {
        message: format!(
            "facet `{name}` is {}, and a facet is a mapping",
            value.kind_name()
        ),
        span,
    })?;
    Ok(Facet {
        name: name.to_string(),
        role: scalar(map, "role"),
        required: map
            .get("required")
            .and_then(|node| node.value.as_scalar())
            .and_then(headwater_yaml::core_schema::as_bool)
            .unwrap_or(false),
        values: read_values(map),
        span,
    })
}

/// The value set of a facet, in either of the two forms a resolved taxonomy
/// writes.
///
/// A vocabulary entry is a mapping with a `value`, because the value carries a
/// lifecycle role beside it. A plain enumeration is a list of scalars. Both are
/// a list of admitted strings to a check, and the difference is the taxonomy's
/// business rather than a rule's.
fn read_values(map: &Mapping) -> Vec<String> {
    let Some(items) = map.get("values").and_then(|node| node.value.as_seq()) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| match &item.value {
            Value::Map(entry) => scalar(entry, "value"),
            other => other.as_scalar().map(|scalar| scalar.text.clone()),
        })
        .collect()
}

fn read_kind(name: &str, value: &Value, span: Span) -> Result<Kind, DeclarationError> {
    let map = value.as_map().ok_or_else(|| DeclarationError {
        message: format!(
            "kind `{name}` is {}, and a kind is a mapping",
            value.kind_name()
        ),
        span,
    })?;
    let facets = map.get("facets").and_then(|node| node.value.as_map());
    Ok(Kind {
        name: name.to_string(),
        is_a: scalar(map, "is_a"),
        require: facets.map(|map| sequence(map, "require")).unwrap_or_default(),
        forbid: facets.map(|map| sequence(map, "forbid")).unwrap_or_default(),
        expectations: read_expectations(map),
        span,
    })
}

fn read_expectations(kind: &Mapping) -> Vec<Expectation> {
    let Some(items) = kind
        .get("relations")
        .and_then(|node| node.value.as_map())
        .and_then(|relations| relations.get("expect"))
        .and_then(|node| node.value.as_seq())
    else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let map = item.value.as_map()?;
            // A relation and an origin are what make this an expectation at
            // all. A declaration missing either states nothing this engine can
            // check, and `taxonomy validate` is where that is reported.
            let relation = scalar(map, "relation")?;
            let since_role = scalar(map, "since")?;
            Some(Expectation {
                id: scalar(map, "id").unwrap_or_else(|| relation.clone()),
                relation,
                to_kind: scalar(map, "to_kind"),
                when: map
                    .get("when")
                    .and_then(|node| node.value.as_map())
                    .map(|when| {
                        when.iter()
                            .filter_map(|entry| {
                                let value = entry.value.value.as_scalar()?;
                                Some((entry.key.value.clone(), value.text.clone()))
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                within_days: scalar(map, "within").as_deref().and_then(days),
                since_role,
                severity: match scalar(map, "severity") {
                    None => Some(Severity::Warn),
                    Some(word) => match word.as_str() {
                        "error" => Some(Severity::Error),
                        "warn" => Some(Severity::Warn),
                        "info" => Some(Severity::Info),
                        // A word outside the set. Kept as an unreadable
                        // severity rather than guessed at, because guessing
                        // would report at a loudness nobody declared.
                        _ => None,
                    },
                },
                rationale: scalar(map, "rationale"),
            })
        })
        .collect()
}

/// A window as whole days. `30d` and `30` are the two spellings this reads.
///
/// Anything else is a window this engine cannot read, and the instance that
/// would have used it skips with a reason rather than assuming a length.
fn days(text: &str) -> Option<i64> {
    let text = text.trim();
    let digits = text.strip_suffix('d').unwrap_or(text);
    digits.parse().ok().filter(|days| *days >= 0)
}

fn scalar(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

fn sequence(map: &Mapping, key: &str) -> Vec<String> {
    map.get(key)
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(source: &str) -> Shape {
        let root = headwater_yaml::load(source).expect("the source loads");
        Shape::read(root.value.as_map().expect("a mapping")).expect("the shape reads")
    }

    const SOURCE: &str = "\
facets:
  status:
    role: state
    required: true
    values:
      - {value: draft, role: initial}
      - {value: current, role: live}
  status_since:
    role: state_entered
    required: true
  doc_type:
    required: false
    values: [design_spec, evaluation]
kinds:
  governed_document:
    abstract: true
    facets: {require: [status, status_since]}
  design_spec:
    is_a: governed_document
    facets: {require: [doc_type]}
  evaluation:
    is_a: governed_document
    facets: {forbid: [doc_type]}
    relations:
      expect:
        - id: evidence-cited
          relation: cited_by
          to_kind: decision_register
          when: {status: current}
          within: 30d
          since: state_entered
          severity: warn
          rationale: an evaluation that no register cites closed nothing
";

    #[test]
    fn a_value_set_reads_from_a_vocabulary_and_from_a_plain_list() {
        let shape = shape(SOURCE);
        assert_eq!(shape.facet("status").expect("declared").values, ["draft", "current"]);
        assert_eq!(
            shape.facet("doc_type").expect("declared").values,
            ["design_spec", "evaluation"]
        );
        assert!(shape.facet("status_since").expect("declared").values.is_empty());
    }

    /// The chain is what a required-facet check reads, and it is the whole of
    /// the difference between a kind and its parent.
    #[test]
    fn a_kind_owes_what_its_ancestors_require() {
        let shape = shape(SOURCE);
        assert_eq!(
            shape.required_facets("design_spec"),
            ["status", "status_since", "doc_type"]
        );
        assert_eq!(
            shape.required_facets("governed_document"),
            ["status", "status_since"]
        );
    }

    /// A kind that forbids a facet does not owe it, whatever an ancestor or a
    /// global `required: true` says. The contradiction is the taxonomy's to
    /// report, and a check that reported it too would send an author to add a
    /// key their own kind refuses.
    #[test]
    fn a_forbidden_facet_is_never_owed() {
        let shape = shape(&SOURCE.replace("      - {value: draft, role: initial}\n", ""));
        assert!(!shape.required_facets("evaluation").contains(&"doc_type".to_string()));
    }

    #[test]
    fn a_kind_descends_from_its_ancestors_and_from_itself() {
        let shape = shape(SOURCE);
        assert!(shape.descends_from("design_spec", "governed_document"));
        assert!(shape.descends_from("design_spec", "design_spec"));
        assert!(!shape.descends_from("governed_document", "design_spec"));
        assert!(!shape.descends_from("design_spec", "evaluation"));
    }

    /// A cycle stops rather than hanging the run that met it.
    #[test]
    fn a_cycle_in_the_chain_terminates() {
        let shape = shape("kinds:\n  a: {is_a: b}\n  b: {is_a: a}\n");
        assert_eq!(shape.ancestry("a").len(), 2);
        assert!(!shape.descends_from("a", "c"));
    }

    #[test]
    fn an_expectation_reads_its_window_its_origin_and_its_severity() {
        let shape = shape(SOURCE);
        let expectation = &shape.kind("evaluation").expect("declared").expectations[0];
        assert_eq!(expectation.id, "evidence-cited");
        assert_eq!(expectation.relation, "cited_by");
        assert_eq!(expectation.to_kind.as_deref(), Some("decision_register"));
        assert_eq!(expectation.when, [("status".to_string(), "current".to_string())]);
        assert_eq!(expectation.within_days, Some(30));
        assert_eq!(expectation.since_role, "state_entered");
        assert_eq!(expectation.severity, Some(Severity::Warn));
        assert_eq!(
            shape.facet_in_role("state_entered").map(|facet| facet.name.as_str()),
            Some("status_since")
        );
    }

    /// A window this engine cannot read is kept as unreadable, so the instance
    /// that meets it skips with a reason instead of vanishing.
    #[test]
    fn an_unreadable_window_or_severity_is_kept_as_unreadable() {
        let shape = shape(
            "kinds:\n  k:\n    relations:\n      expect:\n        \
             - {relation: r, since: state_entered, within: soon, severity: loud}\n",
        );
        let expectation = &shape.kind("k").expect("declared").expectations[0];
        assert_eq!(expectation.within_days, None);
        assert_eq!(expectation.severity, None);
        // And with no `id`, the relation names it, so a report never prints an
        // expectation with no name at all.
        assert_eq!(expectation.id, "r");
    }
}
