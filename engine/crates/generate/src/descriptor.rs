// SPDX-License-Identifier: Apache-2.0
//! The corpus descriptor: what a machine reads when it arrives with a location
//! and nothing else.
//!
//! [Q20](../../../../docs/spec/09-decisions.md) fixes it at
//! `.headwater/corpus.json` and makes it engine-defined, for the reason that a
//! reader who must consult the taxonomy to find it already knows what it says.
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold)
//! names what it carries: the root path, the taxonomy identity and version, the
//! lock hash, the entry points, and each declared export profile.
//!
//! # Three of those five are read, one is derived, and one cannot be printed
//!
//! The root, the identity and the lock hash are read from the consumer
//! declaration and the lock, and they arrive here as strings in [`Identity`] so
//! that this crate keeps no dependency on the resolver. That is the posture
//! [`headwater_census::walk::Corpus::declared`] already takes for the same
//! reason.
//!
//! **An entry point is derived, because nothing declares one.** No declaration
//! in the language holds an entry point, and the specification names the field
//! without stating its form. Two rules already settled supply it between them.
//! [Spec 1](../../../../docs/spec/01-conceptual-model.md#projections) says a
//! machine reads the descriptor to learn "where to start", and
//! [spec 2](../../../../docs/spec/02-taxonomy-model.md#reading-precedence-is-derived)
//! derives reading precedence from the relation family and the nucleus, naming
//! "reading order in generated indexes" as one of its three consumers. So a
//! shelf's entry point is the document its derived reading order puts first,
//! computed by the same [`headwater_query::Surface::by_precedence`] that
//! `route` and the shelf index call. A descriptor, an index and a route
//! therefore cannot disagree about which document a reader opens first.
//!
//! **An entry point carries no summary, and the grain is the reason.** A
//! descriptor is committed and `generate --check` compares its bytes, so every
//! member of it is a thing that a contributor must regenerate after changing.
//! A summary is prose in front matter. Carrying one here would make an ordinary
//! wording edit fail the gate over a corpus whose structure nobody touched,
//! and a gate that fires on prose teaches everyone to bypass it. A path and an
//! identifier move when the corpus moves, which is what a descriptor is for.
//!
//! **An export profile cannot be printed at all, and an empty list would be a
//! false statement.** [Spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-profile-carries-a-filter)
//! makes a profile an entry under `projections` that names an audience, a
//! target, a filter over facet values and a tombstone grain. The `projections`
//! block reads a kind and an output path, and the meta-schema declares no other
//! member, so no taxonomy can declare a profile yet. Printing `[]` would tell a
//! reader that this corpus exports nothing. What is true is that nothing can
//! declare an export. So the field is absent and the `unstated` block below
//! carries the difference, which is the accounting
//! [spec 4](../../../../docs/spec/04-assurance-model.md) requires everywhere a
//! run produces less than it was asked for.
//!
//! # Two things the five did not mention, and one of them is a correction
//!
//! **A root with no exclusions is an overstatement of the corpus.** This
//! repository declares `docs` as its root and excludes `docs/taxonomies/**`,
//! with a reason. A descriptor that printed the root alone would tell a cold
//! reader that every file under `docs` is governed content, which is false
//! here and false in any repository that declares an exclusion. The exclusion
//! and its reason are already required to be stated, and no artifact a cold
//! reader can reach states them, so the descriptor carries both.
//!
//! **`corpora` is a list of one.** Spec 7 says a repository holds one or more
//! corpora and the engine reads one consumer declaration, so this engine
//! produces exactly one entry. The list is the shape spec 7 fixes, and a second
//! root arrives into it rather than into a changed shape.
//!
//! # The marker is a member, not a comment
//!
//! JSON carries no comment, which is why [`crate::marker`] answers `None` for
//! it. The marker is what stops this engine from destroying an authored file,
//! so a format with no comment needs the same permission by another route. The
//! two candidates were a top-level member and the rule that a path the engine
//! fixes needs no marker at all. The second is cheaper and it is wrong: it
//! would have this engine overwrite whatever it found at `.headwater/`, and an
//! adopter who hand-wrote a descriptor before this verb shipped is exactly the
//! reader Q20 is addressed to.
//!
//! The member also does a second job, and that is what settles the choice.
//! Spec 7 rules that **absence must not read as presence**: a reader that
//! fetches the descriptor and receives a host's default page must conclude
//! *absent* rather than *malformed*. That requires a self-identifying member
//! whatever the marker rule decides. One member answers both, and two would be
//! two copies of one fact
//! ([principle 2](../../../../docs/spec/00-vision-and-scope.md#design-principles)).

use crate::{marker_text, Identity, Kind, Output, Plan};
use headwater_query::json::Json;
use headwater_query::Surface;

/// The path Q20 fixes, relative to the repository root.
pub const PATH: &str = ".headwater/corpus.json";

/// The descriptor's own version, as `Major.Minor`.
///
/// Spec 7 attaches a client behavior to it, which is what keeps it from being
/// a string that nobody reads. A major above what the reader understands is a
/// hard failure with a message. A minor mismatch is a warning and the reader
/// continues. So a member added later moves the minor, and a member removed or
/// re-meant moves the major.
pub const VERSION: &str = "1.0";

pub(crate) fn emit(surface: &Surface<'_>, identity: &Identity, plan: &mut Plan) {
    let taxonomy = Json::object([
        ("package", Json::string(identity.package.as_str())),
        ("version", Json::string(identity.version.as_str())),
        ("lock", Json::string(identity.lock.as_str())),
    ]);

    let excluded = Json::Array(
        identity
            .exclusions
            .iter()
            .map(|(path, reason)| {
                Json::object([
                    ("path", Json::string(path)),
                    ("reason", Json::string(reason)),
                ])
            })
            .collect(),
    );

    let corpus = Json::object([
        ("root", Json::string(identity.corpus_root.as_str())),
        ("excluded", excluded),
        ("taxonomy", taxonomy),
        ("entry_points", Json::Array(entry_points(surface))),
    ]);

    let value = Json::object([
        (
            crate::MARKER,
            Json::string(marker_text(Kind::CorpusDescriptor)),
        ),
        ("descriptor_version", Json::string(VERSION)),
        ("corpora", Json::Array(vec![corpus])),
        ("unstated", unstated()),
    ]);

    plan.outputs.push(Output {
        path: PATH.to_string(),
        kind: Kind::CorpusDescriptor,
        bytes: value.render_pretty(),
    });
}

/// One entry point per shelf that holds a document, in the taxonomy's order.
///
/// A shelf with no document contributes no row. An entry point into nothing is
/// the same assertion the shelf index refuses to write: a file that says a
/// shelf is there when the tree has no document on it.
fn entry_points(surface: &Surface<'_>) -> Vec<Json> {
    let mut out = Vec::new();
    for shelf in &surface.taxonomy().shelves {
        let on_shelf: Vec<_> = surface
            .documents()
            .into_iter()
            .filter(|document| {
                crate::shelf_of(document.path, surface).is_some_and(|s| s.name == shelf.name)
            })
            .collect();
        if on_shelf.is_empty() {
            continue;
        }
        let mut ordered = crate::pointers(surface, &on_shelf);
        surface.by_precedence(&mut ordered);
        let first = &ordered[0];
        let mut members = vec![
            ("shelf".to_string(), Json::string(shelf.name.as_str())),
            ("path".to_string(), Json::string(first.path.as_str())),
        ];
        // A document with no identifier is a document a reader reaches by path,
        // and an `id` of the empty string would be an identifier that resolves
        // to nothing. The member is absent instead.
        if let Some(id) = &first.id {
            members.push(("id".to_string(), Json::string(id.as_str())));
        }
        out.push(Json::Object(members));
    }
    out
}

/// The fields spec 7 names that this engine cannot compute, and why.
///
/// Reported inside the artifact rather than omitted from it. A reader that
/// found no `exports` member and no accounting would have to guess between a
/// corpus that exports nothing and an engine that cannot say, and those have
/// different remedies. This is the same refusal-with-a-reason that
/// [`crate::Unwritten`] carries in the report, moved inside the file, because
/// the file is what a cold reader gets and the report is not.
fn unstated() -> Json {
    Json::Array(vec![Json::object([
        ("field", Json::string("exports")),
        (
            "reason",
            Json::string(
                "no taxonomy can declare an export profile yet. An export profile names an \
                 audience, an emitter target, a filter and a tombstone grain, and a `projections` \
                 entry carries a kind and an output path. An empty list here would say that this \
                 corpus exports nothing, and what is true is that nothing can declare an export.",
            ),
        ),
    ])])
}
