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
    /// `purposes`, the reader intents the corpus serves. Spec 2 declares them
    /// once at the taxonomy level and has kinds reference them, and
    /// [spec 5](../../../../docs/spec/05-ai-integration.md#intent-time-routing)
    /// routes a task description over them before it reads any prose.
    pub purposes: Vec<Purpose>,
    /// `regimes.voice`, which a Document check reads through the kind that
    /// binds it.
    pub voice: Vec<VoiceRegime>,
    /// `regimes.language`, on the same terms.
    pub language: Vec<LanguageRegime>,
}

/// A voice regime: the constructions its prose does not use.
///
/// The member is a list of category names and never patterns.
/// [Spec 13](../../../../docs/spec/13-open-obligations.md) records the meta-schema
/// gap that no value set states the names, so the engine knows a closed set of
/// them and an instance that meets a name outside it skips with a reason. A
/// regime that forbids nothing is the narrative regime, and it generates no
/// instance at all.
#[derive(Clone, Debug)]
pub struct VoiceRegime {
    pub name: String,
    pub forbid: Vec<String>,
    pub span: Span,
}

/// One term a corpus retired.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired):
/// "Each entry carries the term, a required reason, and an optional
/// replacement", and the replacement is what decides whether the fix is a
/// substitution or a rewrite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetiredTerm {
    pub term: String,
    pub reason: String,
    pub replacement: Option<String>,
}

/// A language regime: the tag, and the controlled language the prose is held to.
///
/// `controlled` and `profile` are two strings and the meta-schema marks both as
/// a gap, for a reason it states: spec 2 names `none`, `ste-house` and
/// `ste-strict`, and this repository's own overlay writes `ASD-STE100` with a
/// separate `profile`. So the engine matches what it knows and skips the rest,
/// which is the same posture as an unreadable participation window.
#[derive(Clone, Debug)]
pub struct LanguageRegime {
    pub name: String,
    /// BCP 47, as declared. `en-US` is the only spelling variant this engine
    /// has a rule for, and a tag it does not know decides nothing.
    pub tag: String,
    /// The controlled language, and nothing for a regime that declares none.
    pub controlled: Option<String>,
    pub profile: Option<String>,
    /// How the source is written, and nothing for a regime that fixes no form.
    /// A closed set in the meta-schema, so a value outside it never arrives.
    pub source_form: Option<String>,
    /// The terms this corpus retired, in the order the regime lists them.
    pub retired_terms: Vec<RetiredTerm>,
    pub span: Span,
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

/// A reader intent the corpus serves, as `purposes` declares it.
///
/// Both members are prose an author wrote for a reader, and routing reads them
/// as the terms a task description is matched against. `answers` carries the
/// questions the purpose answers, which is the closest thing a taxonomy holds
/// to a task description, so it is the stronger of the two signals.
#[derive(Clone, Debug)]
pub struct Purpose {
    pub name: String,
    /// What a document serving this purpose is for, in one sentence.
    pub intent: Option<String>,
    /// The questions this purpose answers, as the declaration writes them.
    pub answers: Vec<String>,
    pub span: Span,
}

/// A kind, read down to what a generated check needs.
#[derive(Clone, Debug)]
pub struct Kind {
    pub name: String,
    pub is_a: Option<String>,
    /// The reader intent this kind serves, as this kind declares it. Spec 2
    /// permits a concrete kind to inherit one from an abstract parent, and
    /// [`Shape::purpose_of`] is the inherited answer.
    pub purpose: Option<String>,
    /// `facets.require`, as this kind declares it and without its ancestors.
    /// [`Shape::required_facets`] is the inherited set.
    pub require: Vec<String>,
    pub forbid: Vec<String>,
    /// The name of the voice regime this kind binds, inherited through
    /// [`Shape::voice_of`].
    pub voice: Option<String>,
    /// The name of the language regime, on the same terms.
    pub language: Option<String>,
    /// `sections.require`, as this kind declares it. [`Shape::required_sections`]
    /// is the inherited set.
    pub sections: Vec<String>,
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

        if let Some(purposes) = root.get("purposes") {
            match &purposes.value {
                Value::Map(map) => {
                    for entry in map {
                        let body = entry.value.value.as_map();
                        shape.purposes.push(Purpose {
                            name: entry.key.value.clone(),
                            intent: body.and_then(|map| scalar(map, "intent")),
                            answers: body.map(|map| sequence(map, "answers")).unwrap_or_default(),
                            span: entry.key.span,
                        });
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!(
                        "`purposes` is {}, and it names reader intents",
                        other.kind_name()
                    ),
                    span: purposes.span,
                }),
            }
        }

        if let Some(regimes) = root.get("regimes").and_then(|node| node.value.as_map()) {
            if let Some(voice) = regimes.get("voice").and_then(|node| node.value.as_map()) {
                for entry in voice {
                    let Some(map) = entry.value.value.as_map() else {
                        continue;
                    };
                    shape.voice.push(VoiceRegime {
                        name: entry.key.value.clone(),
                        forbid: sequence(map, "forbid"),
                        span: entry.key.span,
                    });
                }
            }
            if let Some(language) = regimes.get("language").and_then(|node| node.value.as_map()) {
                for entry in language {
                    let Some(map) = entry.value.value.as_map() else {
                        continue;
                    };
                    shape.language.push(LanguageRegime {
                        name: entry.key.value.clone(),
                        tag: scalar(map, "tag").unwrap_or_default(),
                        controlled: scalar(map, "controlled"),
                        profile: scalar(map, "profile"),
                        source_form: scalar(map, "source_form"),
                        retired_terms: retired_terms(map),
                        span: entry.key.span,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(shape)
        } else {
            Err(errors)
        }
    }

    /// The voice regime a kind is held to, through the chain that binds it.
    ///
    /// A kind that binds none, and whose ancestors bind none, answers to no
    /// voice rule. That is the narrative case of
    /// [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#voice), and
    /// it is an absence rather than a regime that permits everything.
    pub fn voice_of(&self, kind: &str) -> Option<&VoiceRegime> {
        let name = self
            .ancestry(kind)
            .iter()
            .find_map(|step| step.voice.clone())?;
        self.voice.iter().find(|regime| regime.name == name)
    }

    /// The language regime a kind is held to, on the same terms.
    pub fn language_of(&self, kind: &str) -> Option<&LanguageRegime> {
        let name = self
            .ancestry(kind)
            .iter()
            .find_map(|step| step.language.clone())?;
        self.language.iter().find(|regime| regime.name == name)
    }

    /// Every section a document of this kind owes, in one order.
    ///
    /// Inherited the way [`Shape::required_facets`] is inherited, and from the
    /// root of the chain down, so the order a report prints does not move when
    /// a kind gains a parent. A section has no `forbid`, because the
    /// meta-schema declares none: a contract states `require` and `optional`,
    /// and what is neither is a heading the contract says nothing about.
    pub fn required_sections(&self, kind: &str) -> Vec<String> {
        let mut required: Vec<String> = Vec::new();
        for step in self.ancestry(kind).iter().rev() {
            for name in &step.sections {
                if !required.iter().any(|known| known == name) {
                    required.push(name.clone());
                }
            }
        }
        required
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

    /// The purpose a kind serves, through the chain that declares it.
    ///
    /// Spec 2: "Every concrete kind declares the reader intent that it serves,
    /// or inherits it from an abstract parent." So this walks the same chain
    /// [`Shape::voice_of`] walks, and a kind under a parent that declares one
    /// serves it. A name no `purposes` block declares reads as no purpose
    /// rather than as an invented one: `taxonomy validate` owns referential
    /// integrity, and a reader that invented a node would hide the defect.
    pub fn purpose_of(&self, kind: &str) -> Option<&Purpose> {
        let name = self
            .ancestry(kind)
            .into_iter()
            .find_map(|step| step.purpose.clone())?;
        self.purposes.iter().find(|purpose| purpose.name == name)
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
        purpose: scalar(map, "purpose"),
        require: facets
            .map(|map| sequence(map, "require"))
            .unwrap_or_default(),
        forbid: facets
            .map(|map| sequence(map, "forbid"))
            .unwrap_or_default(),
        voice: scalar(map, "voice"),
        language: scalar(map, "language"),
        sections: map
            .get("sections")
            .and_then(|node| node.value.as_map())
            .map(|sections| sequence(sections, "require"))
            .unwrap_or_default(),
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

/// The retired terms of one language regime.
///
/// An entry without a `term` or without a `reason` is dropped rather than
/// guessed at: the meta-schema requires both, so a source that reaches here
/// missing one has already been refused, and inventing a reason would put words
/// in a finding that no taxonomy wrote.
fn retired_terms(map: &Mapping) -> Vec<RetiredTerm> {
    let Some(items) = map
        .get("retired_terms")
        .and_then(|node| node.value.as_seq())
    else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let entry = item.value.as_map()?;
            Some(RetiredTerm {
                term: scalar(entry, "term")?,
                reason: scalar(entry, "reason")?,
                replacement: scalar(entry, "replacement"),
            })
        })
        .collect()
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
        assert_eq!(
            shape.facet("status").expect("declared").values,
            ["draft", "current"]
        );
        assert_eq!(
            shape.facet("doc_type").expect("declared").values,
            ["design_spec", "evaluation"]
        );
        assert!(shape
            .facet("status_since")
            .expect("declared")
            .values
            .is_empty());
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
        assert!(!shape
            .required_facets("evaluation")
            .contains(&"doc_type".to_string()));
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
        assert_eq!(
            expectation.when,
            [("status".to_string(), "current".to_string())]
        );
        assert_eq!(expectation.within_days, Some(30));
        assert_eq!(expectation.since_role, "state_entered");
        assert_eq!(expectation.severity, Some(Severity::Warn));
        assert_eq!(
            shape
                .facet_in_role("state_entered")
                .map(|facet| facet.name.as_str()),
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

    /// A purpose is declared once and referenced by kinds, and a concrete kind
    /// may inherit the one its abstract parent declares.
    #[test]
    fn a_kind_serves_the_purpose_it_declares_or_the_one_it_inherits() {
        let shape = shape(
            "purposes:\n  \
             rationale:\n    intent: explain why a choice was made\n    \
             answers: [\"why is it this way\", \"what was rejected\"]\n\
             kinds:\n  \
             governed_document: {abstract: true, purpose: rationale}\n  \
             decision: {is_a: governed_document}\n  \
             note: {purpose: nowhere}\n  \
             bare: {}\n",
        );
        let purpose = shape.purposes.first().expect("declared");
        assert_eq!(purpose.name, "rationale");
        assert_eq!(
            purpose.intent.as_deref(),
            Some("explain why a choice was made")
        );
        assert_eq!(purpose.answers.len(), 2);

        assert_eq!(
            shape.purpose_of("decision").map(|p| p.name.as_str()),
            Some("rationale"),
            "the parent declares it"
        );
        // A purpose no `purposes` block declares is no purpose here. Referential
        // integrity is `taxonomy validate`'s, and a reader that invented the
        // node would hide the defect from the report that names it.
        assert!(shape.purpose_of("note").is_none());
        assert!(shape.purpose_of("bare").is_none());
    }
}
