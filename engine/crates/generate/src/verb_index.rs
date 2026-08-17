// SPDX-License-Identifier: Apache-2.0
//! The verb index: every verb the binary dispatches, and the contract that
//! describes it where one exists.
//!
//! # The valuable cell is the empty one
//!
//! [HW-EVAL-specifying-the-engine](../../../../docs/evaluations/specifying-the-engine.md)
//! names a verb index as a projection that falls out of the graph, and
//! [#257](https://github.com/headwater-ai/headwater/issues/257) states the
//! design point: an index of the contracts that exist is a list of work already
//! done, and nobody needs one. This index carries a row for every verb, and a
//! verb that no document describes carries a mark in the last column. That cell
//! is the finding, and `generate --check` puts it in a diff a reviewer reads.
//!
//! # Where the rows come from, and why that is not a directory read
//!
//! `headwater_verbs::VERBS` is the dispatch table of the binary, and this
//! emitter takes it as an argument. So the rows are the command surface of the
//! engine that wrote the file, and the file goes stale the day a verb is added.
//!
//! [HW-OBL-0128](../../../../docs/obligations/0128-nothing-holds-a-crate-to-having-a-contract-under-a-root-that-excludes-it.md)
//! asks whether a projection may derive a row from a path outside the corpus
//! root, and this one does not answer that question. It reads no directory and
//! opens no file outside the corpus. A compile-time constant of the engine is
//! the same class of input as the set of check rules that the register already
//! projects, and [Q29](../../../../docs/decisions/0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md)
//! is about content of the corpus rather than about data of the engine.
//!
//! # The join is the whole command line
//!
//! A contract is matched to a verb by the value of the facet in the `name` role:
//! `headwater check` names the verb `check`. The bare word is not the join,
//! because `check` is not what the document is called and a man page is titled
//! by the command line it describes.
//!
//! # Two declines, and both are whole
//!
//! Spec 6 rules that a projection which cannot name every row produces no file.
//! A document on the shelf whose name matches no verb declines the file, and so
//! does a document with no name at all. Either one would otherwise write an
//! index that silently dropped a description somebody wrote.
//!
//! An empty shelf is not a decline, and that is the difference from a shelf
//! index. An index of no documents asserts that a shelf is there; an index of
//! seventeen verbs that nobody has described is the strongest form of this
//! artifact rather than the weakest.

use crate::{shelf_index, shelf_of, Declaration, Kind, Output, Plan, Unwritten};
use headwater_query::Surface;
use headwater_verbs::Verb;

/// The mark a verb with no contract carries.
///
/// Bold, and a phrase rather than a dash, because the row has to read as a
/// finding in a diff. A blank cell reads as a column somebody forgot to fill.
const UNDESCRIBED: &str = "**no contract**";

pub(crate) fn emit(
    surface: &Surface<'_>,
    declaration: &Declaration,
    verbs: &[Verb],
    plan: &mut Plan,
) {
    let path = declaration.output.clone();
    let decline = |plan: &mut Plan, reason: String| {
        plan.unwritten.push(Unwritten {
            at: path.clone(),
            kind: Kind::VerbIndex,
            reason,
        });
    };

    // One shelf, named. An index of every shelf would join a document of any
    // kind to a verb by its title, and a `for` that names none says exactly
    // that. Both are refused here rather than resolved by a default.
    let [shelf_name] = declaration.shelves.as_slice() else {
        decline(
            plan,
            format!(
                "names {} shelves, and a verb index reads exactly one: the shelf that holds the \
                 documents which describe a verb",
                declaration.shelves.len()
            ),
        );
        return;
    };
    if !surface
        .taxonomy()
        .shelves
        .iter()
        .any(|shelf| &shelf.name == shelf_name)
    {
        decline(
            plan,
            format!("names the shelf `{shelf_name}`, which this taxonomy does not declare"),
        );
        return;
    }

    // What each document on the shelf is called, which is the join.
    let mut described: Vec<(String, String)> = Vec::new();
    for document in surface.documents() {
        if !shelf_of(document.path, surface).is_some_and(|shelf| &shelf.name == shelf_name) {
            continue;
        }
        let Some(name) = surface.name(&document) else {
            decline(
                plan,
                match surface.name_facet() {
                    None => format!(
                        "`{}` is on the shelf `{shelf_name}` and this taxonomy declares no facet \
                         in the `name` role, so no document on it can be joined to a verb",
                        document.path
                    ),
                    Some(facet) => format!(
                        "`{}` is on the shelf `{shelf_name}` and states no `{facet}`, so nothing \
                         says which verb it describes",
                        document.path
                    ),
                },
            );
            return;
        };
        described.push((name, document.path.to_string()));
    }
    described.sort();

    // Two documents that claim one verb, and neither is dropped in silence.
    if let Some(pair) = described.windows(2).find(|pair| pair[0].0 == pair[1].0) {
        decline(
            plan,
            format!(
                "`{}` and `{}` are both called `{}`, so two documents claim one verb",
                pair[0].1, pair[1].1, pair[0].0
            ),
        );
        return;
    }

    // A description of a verb this binary does not dispatch. The index would
    // otherwise drop it, and a dropped description is the one thing worse than
    // a missing one: somebody wrote it and no reader can find it.
    let names: Vec<String> = verbs.iter().map(Verb::described_as).collect();
    if let Some((name, at)) = described
        .iter()
        .find(|(name, _)| !names.contains(name))
        .cloned()
    {
        decline(
            plan,
            format!(
                "`{at}` is called `{name}` and this binary dispatches no such verb, so the index \
                 would drop it"
            ),
        );
        return;
    }

    plan.outputs.push(Output {
        bytes: render(&path, verbs, &described),
        path,
        kind: Kind::VerbIndex,
    });
}

/// One row for every verb, in the order the dispatch table carries them.
fn render(output: &str, verbs: &[Verb], described: &[(String, String)]) -> String {
    let base = shelf_index::parent_of(output);
    let contract_of = |verb: &Verb| -> Option<&(String, String)> {
        let name = verb.described_as();
        described.iter().find(|(called, _)| called == &name)
    };
    let with = verbs
        .iter()
        .filter(|verb| contract_of(verb).is_some())
        .count();
    let without = verbs.len() - with;

    let mut out = String::new();
    let mark = headwater_mark::marker(Kind::VerbIndex.name(), output)
        .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
    out.push_str(&mark);
    out.push_str("\n\n# The command surface\n\n");
    out.push_str(&format!(
        "`{}` dispatches {} {}. {} of them {} a contract on this shelf, and {} {} none.\n\n",
        headwater_verbs::BINARY,
        verbs.len(),
        match verbs.len() {
            1 => "verb",
            _ => "verbs",
        },
        with,
        match with {
            1 => "has",
            _ => "have",
        },
        without,
        match without {
            1 => "has",
            _ => "have",
        },
    ));
    out.push_str(
        "A verb with no contract carries a mark in the last column, and that cell is what this \
         file is for. An index of the contracts that exist would say nothing about the verbs \
         nobody has described yet.\n\n",
    );
    out.push_str("| Verb | What a caller types | Contract |\n|---|---|---|\n");
    for verb in verbs {
        let forms: Vec<String> = verb
            .forms()
            .into_iter()
            .map(|form| format!("`{form}`"))
            .collect();
        let contract = match contract_of(verb) {
            Some((name, at)) => format!("[{name}]({})", shelf_index::relative(&base, at)),
            None => UNDESCRIBED.to_string(),
        };
        out.push_str(&format!(
            "| `{}` | {} | {contract} |\n",
            verb.name,
            forms.join(", ")
        ));
    }
    out
}
