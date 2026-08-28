// SPDX-License-Identifier: Apache-2.0
//! The capture-cost store as JSON: what `headwater capture --json` writes.
//!
//! # Why the emitter is here and not in the CLI crate
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md) makes the CLI a
//! shell that parses arguments and prints what a crate returned, and every
//! other JSON emitter of this engine sits in the crate that owns the data it
//! writes. This crate owns [`crate::reading`] — the store, the readings, the
//! assisted totals and the reach — so the document that reports them is
//! written here beside the types, and `main.rs` keeps only the join between
//! the census and the identifier index, which is a boundary decision like the
//! rest of that file.
//!
//! # The text report is a separate reader and stays where it is
//!
//! The report a person reads is laid out for a person: it says "1 reading"
//! rather than "1 readings", it leads with the population, and it prints
//! nothing at all where a number would be empty rather than zero. None of that
//! is this document's business. A consumer of these bytes gets every member on
//! every run, and decides for itself what an empty collection means.

use crate::reading::{
    by_kind, by_surface, locks, total, Classified, Reach, Reading, Unreadable, STORE,
};
use headwater_check::Date;
use headwater_yaml::json::Json;

/// The version of this document's own shape.
///
/// A consumer outside the repository reads these bytes and holds no clone, so
/// the shape has to name itself. It is not the engine's version: two engines
/// that write one shape should not make a reader re-read it.
///
/// `1.0` is the first shape that names itself. The members that moved before
/// this constant existed — the store gaining `readings`, the report gaining
/// `locks` — are moves no consumer could have been told about, because there
/// was nothing here to tell it with. That is the defect
/// [#343](https://github.com/headwater-ai/headwater/issues/343) closed, and a
/// member added from here on moves the minor by the rule the other emitters of
/// this engine state for themselves.
pub const VERSION: &str = "1.0";

/// The store, read back, as one JSON document.
pub fn render(
    readings: &[Reading],
    unreadable: &[Unreadable],
    classified: &Classified,
    reach: &Reach,
) -> String {
    document(readings, unreadable, classified, reach).render_pretty()
}

fn document(
    readings: &[Reading],
    unreadable: &[Unreadable],
    classified: &Classified,
    reach: &Reach,
) -> Json {
    let pair = |value: (usize, usize)| {
        Json::Array(vec![
            Json::Raw(value.0.to_string()),
            Json::Raw(value.1.to_string()),
        ])
    };
    let count = |value: usize| Json::Raw(value.to_string());
    let day = |value: Option<Date>| match value {
        Some(date) => Json::string(date.render()),
        None => Json::Array(vec![]),
    };

    let total = total(readings);
    let first = readings.iter().map(|reading| reading.date).min();
    let last = readings.iter().map(|reading| reading.date).max();

    Json::object([
        ("version", Json::string(VERSION)),
        ("store", Json::string(STORE)),
        ("readings", count(readings.len())),
        (
            "unreadable_lines",
            Json::Array(
                unreadable
                    .iter()
                    .map(|line| Json::Raw(line.line.to_string()))
                    .collect(),
            ),
        ),
        (
            "locks",
            Json::Array(locks(readings).iter().map(Json::string).collect()),
        ),
        ("first_reading", day(first)),
        ("last_reading", day(last)),
        ("fields", pair(total.fields)),
        ("sections", pair(total.sections)),
        ("identifier", pair(total.identifier)),
        ("edge_halves", pair(total.edge_halves)),
        ("supplied", count(total.supplied())),
        ("denominator", count(total.total())),
        (
            "by_kind",
            Json::Array(
                by_kind(readings)
                    .into_iter()
                    .map(|(kind, taken, assisted)| {
                        Json::object([
                            ("kind", Json::string(kind)),
                            ("readings", count(taken)),
                            ("supplied", count(assisted.supplied())),
                            ("denominator", count(assisted.total())),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "by_surface",
            Json::Array(
                by_surface(readings)
                    .into_iter()
                    .map(|(surface, taken, assisted)| {
                        Json::object([
                            (
                                "surface",
                                match surface {
                                    Some(surface) => Json::string(surface.name()),
                                    // Absent rather than a name, on the rule
                                    // the store itself follows: a reading that
                                    // states no surface is not one of the two
                                    // arms.
                                    None => Json::Array(vec![]),
                                },
                            ),
                            ("readings", count(taken)),
                            ("supplied", count(assisted.supplied())),
                            ("denominator", count(assisted.total())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("reached", count(reach.reached.len())),
        ("classified", count(classified.paths.len())),
        (
            "resolving_to_nothing",
            Json::Array(
                reach
                    .lost
                    .iter()
                    .map(|at| Json::string(&readings[*at].document))
                    .collect(),
            ),
        ),
    ])
}
