// SPDX-License-Identifier: Apache-2.0
//! [`Route`] and [`Explanation`], in the shape a program reads.
//!
//! [#321](https://github.com/headwater-ai/headwater/issues/321) asks that
//! `route` and `explain` "emit JSON that `python3 -m json.tool` parses". These
//! two documents are that JSON. They are the first machine surface either read
//! has ever had: the MCP server serves both as [`Route::render`] and
//! [`Explanation::render`] inside a text block, and a caller who wanted the
//! parts had to take them out of prose.
//!
//! # Three rules that hold for every document this module writes
//!
//! **The version is the document's own shape and never the engine's.** It is
//! the rule `headwater_adapter::json::VERSION` states for the finding shape,
//! and for the same reason: a consumer outside this repository holds no clone
//! of the engine, so the bytes have to name what they are. Two engines that
//! write one shape should not make a reader re-read it.
//!
//! **An absent value is an absent member.** [`Json`] carries no `null`
//! ([Q2](../../../../docs/spec/09-decisions.md#q2--schema-format)), so an
//! `Option::None` writes nothing at all rather than a member with an empty
//! string in it. A member whose absence would be ambiguous is written on every
//! run instead, and each one below says so where that is the case.
//!
//! **Every function here destructures its source exhaustively.** A field added
//! to [`Route`], [`Pointer`], [`Explanation`], [`Permitted`] or [`Neighbour`]
//! and not to the document below does not compile. That is the guard this
//! module has instead of a census: the finding shape is audited by
//! `headwater_adapter::census` because a run holds a set of findings whose
//! membership is the claim, and a read holds one value whose *fields* are the
//! claim. A field is a thing the compiler can count and a set member is not.

use crate::explain::Permitted;
use crate::route::{Matched, Route, Silence};
use crate::{Explanation, Neighbour, Pointer};
use headwater_graph::declarations::Governs;
use headwater_yaml::json::Json;

/// The version of the two documents this module writes.
///
/// One constant for both, because they ship together and a reader who pins one
/// is pinning this module. `1.0` is the first.
pub const VERSION: &str = "1.0";

/// One route as JSON.
pub fn route(route: &Route) -> String {
    of_route(route).render_pretty()
}

/// One explanation as JSON.
pub fn explain(explanation: &Explanation) -> String {
    of_explanation(explanation).render_pretty()
}

/// The route document.
///
/// # `pointers` is written on every run, and that is what a caller decides on
///
/// A route that offers nothing writes `"pointers": []`, and a route that offers
/// three writes three. That member is the answer to the only question a caller
/// of this verb asks — did the corpus have anything for this task — and
/// `.claude/hooks/intent.sh` is the caller that asks it. Before this document
/// existed that hook decided by running `grep -q ' — '` over the rendered
/// report, because an em dash inside a summary was the only machine-visible
/// difference between a pointer line and the four ways `route` reports silence.
/// An author who wrote a summary with no em dash in it, or a prose line that
/// happened to hold one, moved that decision.
///
/// # `text` is the rendered report, and it is here because one run has two
/// readers
///
/// The hook above decides from `pointers` and then puts something in an agent's
/// context, and what belongs there is the report a person reads. A hook that
/// composed those lines out of the members below would be a second copy of
/// [`Pointer::render`], which is the drift this crate is written to avoid — the
/// MCP server and the terminal already share that one rendering. So the
/// document carries it, byte for byte, and `engine/crates/cli/tests/json.rs`
/// holds the two runs against each other.
///
/// The other three documents this change adds carry no such member. This one
/// has a reader for it and none of them does.
fn of_route(route: &Route) -> Json {
    let Route {
        task,
        terms,
        separating,
        distinctive,
        anchors,
        matched,
        pointers,
        withheld,
        silence,
    } = route;
    let mut members: Vec<(&'static str, Json)> = vec![
        ("version", Json::string(VERSION)),
        ("task", Json::string(task.clone())),
        ("terms", strings(terms)),
        ("separating", strings(separating)),
        ("distinctive", strings(distinctive)),
        ("anchors", strings(anchors)),
        (
            "matched",
            Json::Array(matched.iter().map(of_matched).collect()),
        ),
        (
            "pointers",
            Json::Array(pointers.iter().map(of_pointer).collect()),
        ),
        ("withheld", number(*withheld)),
    ];
    if let Some(silence) = silence {
        members.push(("silence", of_silence(silence)));
    }
    // Plain, unconditionally. A JSON document is a machine format, and
    // `main.rs` reads the terminal only for `Format::Text`. An escape sequence
    // inside a string member would reach a program that never asked for one.
    members.push((
        "text",
        Json::string(route.render(headwater_check::paint::ColorMode::Plain)),
    ));
    Json::object(members)
}

fn of_matched(matched: &Matched) -> Json {
    let Matched { purpose, score } = matched;
    Json::object([
        ("purpose", Json::string(purpose.clone())),
        ("score", number(*score as usize)),
    ])
}

/// Why a route offered nothing, as a token and as the sentence the report
/// prints.
///
/// Both, because they answer different questions. `reason` is what a caller
/// branches on and it is stable across every rewording of the prose beside it.
/// `says` is [`Silence::name`] rather than a second wording of it.
fn of_silence(silence: &Silence) -> Json {
    Json::object([
        ("reason", Json::string(silence.token())),
        ("says", Json::string(silence.name())),
    ])
}

/// One pointer.
///
/// `unwarranted` is written on every pointer, including `false`. It is a
/// two-valued fact rather than a value that may be missing, and spec 5 requires
/// a pointer to an `asserted` document to state that warrant, so a consumer
/// that met no member could not tell a warranted document from a producer with
/// nothing to say about warrants.
fn of_pointer(pointer: &Pointer) -> Json {
    let Pointer {
        path,
        id,
        kind,
        name,
        purpose,
        summary,
        unwarranted,
    } = pointer;
    let mut members: Vec<(&'static str, Json)> = vec![
        ("path", Json::string(path.clone())),
        ("kind", Json::string(kind.clone())),
    ];
    if let Some(id) = id {
        members.push(("id", Json::string(id.clone())));
    }
    if let Some(name) = name {
        members.push(("name", Json::string(name.clone())));
    }
    if let Some(purpose) = purpose {
        members.push(("purpose", Json::string(purpose.clone())));
    }
    if let Some(summary) = summary {
        members.push(("summary", Json::string(summary.clone())));
    }
    members.push(("unwarranted", Json::Bool(*unwarranted)));
    Json::object(members)
}

/// The explanation document, in the order [`Explanation::render`] prints it,
/// which is spec 2's own order.
///
/// `kind` is absent for a document the census did not type, and that is the
/// case this verb most exists for: `derivation` then carries the step that
/// stopped, and `facets`, `sections`, `permitted` and `related` are empty
/// because nothing is required of a document that is not one.
fn of_explanation(explanation: &Explanation) -> Json {
    let Explanation {
        path,
        id,
        kind,
        derivation,
        purpose,
        summary,
        warrant,
        facets,
        sections,
        permitted,
        related,
    } = explanation;
    let mut members: Vec<(&'static str, Json)> = vec![
        ("version", Json::string(VERSION)),
        ("path", Json::string(path.clone())),
    ];
    if let Some(id) = id {
        members.push(("id", Json::string(id.clone())));
    }
    if let Some(kind) = kind {
        members.push(("kind", Json::string(kind.clone())));
    }
    members.push(("derivation", strings(derivation)));
    if let Some((name, intent)) = purpose {
        let mut purpose: Vec<(&'static str, Json)> = vec![("name", Json::string(name.clone()))];
        if let Some(intent) = intent {
            purpose.push(("intent", Json::string(intent.clone())));
        }
        members.push(("purpose", Json::object(purpose)));
    }
    if let Some(summary) = summary {
        members.push(("summary", Json::string(summary.clone())));
    }
    if let Some(warrant) = warrant {
        members.push(("warrant", Json::string(warrant.clone())));
    }
    members.extend([
        ("facets", strings(facets)),
        ("sections", strings(sections)),
        (
            "permitted",
            Json::Array(permitted.iter().map(of_permitted).collect()),
        ),
        (
            "related",
            Json::Array(related.iter().map(of_neighbour).collect()),
        ),
    ]);
    Json::object(members)
}

fn of_permitted(permitted: &Permitted) -> Json {
    let Permitted {
        relation,
        to,
        inverse,
    } = permitted;
    let mut members: Vec<(&'static str, Json)> = vec![
        ("relation", Json::string(relation.clone())),
        ("to", strings(to)),
    ];
    if let Some(inverse) = inverse {
        members.push(("inverse", Json::string(inverse.clone())));
    }
    Json::object(members)
}

/// One edge, from the end being explained.
///
/// `inbound` is written always, for the reason `unwarranted` is: it is a
/// two-valued fact that decides how a reader takes the relation name beside it.
/// `cue_is_declared` is written wherever a cue is, and nowhere else, because it
/// is a fact about that cue rather than about the edge.
///
/// `governs` is a token rather than the bracketed phrase [`Neighbour::render`]
/// appends, because the phrase is a sentence and this is a value with three
/// states. `neither` is written rather than omitted, so a consumer never has to
/// decide whether an absent member means "no reading order" or "this producer
/// does not report one".
fn of_neighbour(neighbour: &Neighbour) -> Json {
    let Neighbour {
        relation,
        inbound,
        pointer,
        target,
        cue,
        cue_is_declared,
        governs,
    } = neighbour;
    let mut members: Vec<(&'static str, Json)> = vec![
        ("relation", Json::string(relation.clone())),
        ("inbound", Json::Bool(*inbound)),
        ("target", Json::string(target.clone())),
    ];
    if let Some(pointer) = pointer {
        members.push(("pointer", of_pointer(pointer)));
    }
    if let Some(cue) = cue {
        members.push(("cue", Json::string(cue.clone())));
        members.push(("cue_is_declared", Json::Bool(*cue_is_declared)));
    }
    members.push((
        "governs",
        Json::string(match governs {
            Governs::Source => "source",
            Governs::Target => "target",
            Governs::Neither => "neither",
        }),
    ));
    Json::object(members)
}

fn strings(values: &[String]) -> Json {
    Json::Array(
        values
            .iter()
            .map(|value| Json::string(value.clone()))
            .collect(),
    )
}

fn number(value: usize) -> Json {
    Json::Raw(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn silent(silence: Silence) -> Route {
        Route {
            task: "xyzzy".to_string(),
            terms: vec!["xyzzy".to_string()],
            separating: Vec::new(),
            distinctive: Vec::new(),
            anchors: Vec::new(),
            matched: Vec::new(),
            pointers: Vec::new(),
            withheld: 0,
            silence: Some(silence),
        }
    }

    /// Every silence carries a token, and no two silences share one.
    ///
    /// The tokens are what a caller branches on, so a duplicate would collapse
    /// two facts about a corpus that spec 5 keeps apart on purpose.
    #[test]
    fn each_silence_is_named_by_a_token_of_its_own() {
        let every = [
            Silence::NoPurposes,
            Silence::NoTerms,
            Silence::NoPurposeMatched,
            Silence::NoDocumentReached,
        ];
        let mut seen: Vec<&str> = Vec::new();
        for silence in &every {
            let token = silence.token();
            assert!(!token.is_empty(), "{silence:?} carries no token");
            assert!(
                !seen.contains(&token),
                "{silence:?} shares the token `{token}` with another silence"
            );
            seen.push(token);
        }
    }

    /// A route with nothing in it still writes `pointers`, and writes it empty.
    ///
    /// This is the case `.claude/hooks/intent.sh` decides on, and the one an
    /// emitter that omitted empty members would break silently.
    #[test]
    fn a_silent_route_writes_an_empty_pointer_set_rather_than_no_member() {
        let document = of_route(&silent(Silence::NoPurposeMatched)).render_pretty();
        assert!(
            document.contains("\"pointers\": []"),
            "a silent route names an empty pointer set: {document}"
        );
        assert!(
            document.contains("\"reason\": \"no_purpose_matched\""),
            "and it says which silence it is: {document}"
        );
    }

    /// The `text` member is the report, byte for byte.
    #[test]
    fn the_text_member_is_the_rendered_report() {
        let route = silent(Silence::NoTerms);
        let document = of_route(&route);
        let Json::Object(members) = &document else {
            panic!("the route document is an object");
        };
        let text = members
            .iter()
            .find(|(key, _)| key == "text")
            .map(|(_, value)| value)
            .expect("the route document carries `text`");
        assert_eq!(
            text,
            &Json::string(route.render(headwater_check::paint::ColorMode::Plain))
        );
    }

    /// No escape byte reaches this document, whatever an author wrote.
    ///
    /// The writer escapes every control character below `0x20`, so a summary
    /// carrying an escape sequence arrives as the six characters `\u001b` and
    /// never reaches a consumer. #321's color clause asks for that
    /// independently of any terminal reading, and a summary is the one value
    /// here that is prose an author wrote.
    #[test]
    fn an_escape_sequence_in_a_summary_is_escaped_rather_than_carried() {
        let mut route = silent(Silence::NoTerms);
        route.silence = None;
        route.pointers = vec![Pointer {
            path: "docs/a.md".to_string(),
            id: None,
            kind: "decision".to_string(),
            name: None,
            purpose: None,
            summary: Some("a \u{1b}[31mred\u{1b}[0m summary".to_string()),
            unwarranted: false,
        }];
        let document = of_route(&route).render_pretty();
        assert!(
            !document.contains('\u{1b}'),
            "no escape byte reaches the document"
        );
        assert!(
            document.contains("\\u001b[31mred"),
            "and the sequence is carried as an escape: {document}"
        );
    }
}
