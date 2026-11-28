// SPDX-License-Identifier: Apache-2.0
//! The injected values, and the one of them that exists.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)
//! names two inputs that are about time, and rules that both are injected and
//! never fetched: "`ctx.now` is a bound value, never a syscall". A windowed
//! participation expectation reads a declared origin date from the document and
//! compares it against that value, with no history walk.
//!
//! Both are here now. The clock is a date. The prior version "is available
//! **only** in change-scoped evaluation", so it is the set of documents one
//! change carries, and [`crate::change`] is that set. A run with no change is
//! the ordinary full-corpus run, and every instance of a check that declares
//! the input is reported as skipped rather than passed.
//!
//! The two are injected the same way and for the same reason: a check that
//! could fetch either one would decide its verdict from something the report
//! does not state, and no reader could reproduce the run from what it printed.
//!
//! # A check never reaches a [`Context`]
//!
//! The runner reads one, and it hands the value to a view only where the check
//! declared `NEEDS_CLOCK`. So the same declaration that admits the clock to a
//! check is the one that puts it in the cache key, and neither can be done
//! without the other. [`crate::cache`] is the other half of that sentence.
//!
//! # The grain is a day, and that is a decision rather than a convenience
//!
//! A participation expectation is declared in days (`within: 30d`), so a
//! verdict changes at a day boundary and at no finer one. A clock of finer
//! grain would put a value in every key that is different on every run, which
//! turns a cache into a store that never hits. A day is the coarsest value that
//! cannot change a verdict, which is the same rule the key applies to every
//! other component.

use crate::change::Change;

/// A calendar date, held as a whole number of days from 1970-01-01.
///
/// No time zone and no time of day. Both would be precision this engine cannot
/// use: the dates it compares against come out of front matter as `YYYY-MM-DD`,
/// written by a person who meant a day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    days: i64,
}

/// The values a run injects, which no check can fetch for itself.
///
/// Not `Copy`, because the change carries the documents it names. It is held by
/// reference everywhere a run reads it, which is what it was before the second
/// injected value arrived.
#[derive(Clone, Debug)]
pub struct Context {
    now: Date,
    change: Option<Change>,
    orphaned: std::collections::BTreeSet<String>,
    baseline: bool,
    /// The directory the paths of a `source-tree` anchor are read from, which
    /// is the corpus root on disk, and `None` where the caller named none.
    /// See [`Context::reading`].
    base: Option<std::path::PathBuf>,
}

impl Context {
    /// A run at a stated date, over a whole corpus. Every test that records a
    /// report uses this one, because a recorded report is a function of its
    /// inputs and the clock is one of them.
    pub const fn at(now: Date) -> Self {
        Context {
            now,
            change: None,
            orphaned: std::collections::BTreeSet::new(),
            baseline: false,
            base: None,
        }
    }

    /// A run at a stated date, scoped to one change.
    ///
    /// The only constructor that produces a change-scoped run, so the answer to
    /// "is this run change-scoped" is the presence of the value rather than a
    /// second flag beside it.
    pub fn over(now: Date, change: Change) -> Self {
        Context {
            now,
            change: Some(change),
            orphaned: std::collections::BTreeSet::new(),
            baseline: false,
            base: None,
        }
    }

    /// The change this run read, and nothing for a run that names no change.
    /// The change does not narrow the documents the run checks.
    ///
    /// A check never reaches this. [`crate::scope`] reads it, hands one
    /// document's prior version to a view that declared the input, and puts the
    /// same value in the cache key.
    pub fn change(&self) -> Option<&Change> {
        self.change.as_ref()
    }

    /// A run at today's date, read from the system clock.
    ///
    /// The one syscall, and it is in the injector rather than in a check. The
    /// CLI is the only caller. `None` when the host clock is before the epoch,
    /// which is a machine this engine declines to guess for.
    pub fn from_system_clock() -> Option<Self> {
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?;
        Some(Context {
            now: Date::from_days(i64::try_from(elapsed.as_secs() / 86_400).ok()?),
            change: None,
            orphaned: std::collections::BTreeSet::new(),
            baseline: false,
            base: None,
        })
    }

    /// The same run, with a change it reads. The clock it already holds is
    /// unchanged, because a run is about one day whatever set of documents it
    /// is about.
    pub fn scoped_to(self, change: Change) -> Self {
        Context {
            change: Some(change),
            ..self
        }
    }

    /// The same run, told to offer a baseline stamp (#2016).
    ///
    /// `headwater check --baseline` sets it. `relation.target.suspect` then
    /// offers `verified_revision` and `verified_as: baseline` on each edge
    /// that recorded no digest, with no change that states a reading. The
    /// rule reads it as a statement beside the change, so it is a line of the
    /// cache key like each of those.
    pub fn with_baseline(self, baseline: bool) -> Self {
        Context { baseline, ..self }
    }

    /// Whether this run offers a baseline stamp. See [`Context::with_baseline`].
    pub fn baseline(&self) -> bool {
        self.baseline
    }

    /// The same run, told where the current bytes of a governed file are read
    /// from (#2100).
    ///
    /// `headwater check` sets it to the corpus root. A change-scoped run then
    /// classifies each touched file of a suspect `source-tree` edge, which
    /// needs the current bytes beside the prior ones the manifest named. The
    /// classification is read after every rule ran, as the gate is, so no
    /// check reaches this value and no cache key holds it. A run that names
    /// no base classifies nothing, and its gate is the one it had before.
    pub fn reading(self, base: std::path::PathBuf) -> Self {
        Context {
            base: Some(base),
            ..self
        }
    }

    /// Where the current bytes of a governed file are read from. See
    /// [`Context::reading`].
    pub fn base(&self) -> Option<&std::path::Path> {
        self.base.as_deref()
    }

    /// The same run, told which marked files the projection plan claims no
    /// output for.
    ///
    /// The set is `headwater generate`'s answer, and it is the same set that
    /// verb lists as orphaned: a file that carries the generated marker and
    /// that no output of the plan writes. This crate cannot compute it,
    /// because the generate crate depends on this one and the emitters compute
    /// the output paths. So the caller that builds the plan injects it, and
    /// the CLI is that caller.
    ///
    /// Empty unless a caller sets it, which is every test that does not name
    /// it and every run whose taxonomy gives no rule a use for it. A rule
    /// reads it only through [`crate::scope::CorpusCheck::NEEDS_ORPHANED`],
    /// which also puts it in that rule's cache key.
    pub fn with_orphaned(self, orphaned: std::collections::BTreeSet<String>) -> Self {
        Context { orphaned, ..self }
    }

    /// The marked files no output of the projection plan claims, by path.
    pub fn orphaned(&self) -> &std::collections::BTreeSet<String> {
        &self.orphaned
    }

    pub fn now(&self) -> Date {
        self.now
    }
}

impl Date {
    /// A date as it is written in front matter: `YYYY-MM-DD`, and nothing else.
    ///
    /// A value this cannot read returns `None` rather than a guess. The caller
    /// then reports that it could not evaluate, which is the visible skip
    /// [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
    /// asks for, and never a comparison against a date nobody wrote.
    ///
    /// The calendar is [`headwater_meta::shape::calendar_date`], the one that
    /// `facet.value.not_of_type` and the meta-schema read, so a value this
    /// refuses is the value that rule reports.
    pub fn parse(text: &str) -> Option<Self> {
        let (year, month, day) = headwater_meta::shape::calendar_date(text.trim())?;
        Some(Date {
            days: days_from_civil(year, month, day),
        })
    }

    pub const fn from_days(days: i64) -> Self {
        Date { days }
    }

    /// This date, moved by whole days.
    ///
    /// `headwater infer` uses it for the default expiry of a task it proposes.
    /// The arithmetic is on the day count rather than on the calendar, so it
    /// cannot produce a date that does not exist.
    pub const fn plus_days(&self, days: i64) -> Self {
        Date {
            days: self.days + days,
        }
    }

    /// Whole days from `origin` to this date. Negative when this date is
    /// earlier, which a caller reads as a window that has not opened.
    pub fn days_since(&self, origin: Date) -> i64 {
        self.days - origin.days
    }

    /// The date as it is written, which is the form that goes into a cache key
    /// and into a message.
    pub fn render(&self) -> String {
        let (year, month, day) = civil_from_days(self.days);
        format!("{year:04}-{month:02}-{day:02}")
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}

/// Days from 1970-01-01, by Howard Hinnant's `days_from_civil`.
///
/// The proleptic Gregorian calendar, which is what YAML 1.2 and ISO 8601 both
/// mean by a date. Written out rather than taken from a dependency, because the
/// whole of what this engine needs from a calendar is this function and its
/// inverse.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let month = i64::from(month);
    let day = i64::from(day);
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The inverse of [`days_from_civil`], by the same author and the same paper.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (
        year + i64::from(month <= 2),
        month as u32,
        u32::try_from(day).unwrap_or(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_date_round_trips_through_the_day_count() {
        for text in [
            "1970-01-01",
            "2000-02-29",
            "2026-08-12",
            "1900-03-01",
            "2100-12-31",
        ] {
            assert_eq!(Date::parse(text).expect(text).render(), text);
        }
    }

    #[test]
    fn the_epoch_is_day_zero_and_the_arithmetic_is_whole_days() {
        let epoch = Date::parse("1970-01-01").expect("the epoch");
        assert_eq!(Date::parse("1970-01-02").expect("d").days_since(epoch), 1);
        assert_eq!(
            Date::parse("2026-08-12")
                .expect("d")
                .days_since(Date::parse("2026-07-13").expect("d")),
            30
        );
        // A window that has not opened yet reads as a negative elapsed count
        // rather than as an error, and the caller compares rather than guesses.
        assert_eq!(epoch.days_since(Date::parse("1970-01-31").expect("d")), -30);
    }

    /// A value this cannot read is `None`, and never a date nobody wrote.
    #[test]
    fn a_value_that_is_not_a_date_reads_as_nothing() {
        for text in [
            "",
            "2026-8-12",
            "2026/08/12",
            "2026-13-01",
            "2026-02-30",
            "2025-02-29",
            "2026-08-12T09:00:00Z",
            "yesterday",
        ] {
            assert_eq!(Date::parse(text), None, "{text} read as a date");
        }
    }

    /// The date this layer computes with and the date `facet.value.not_of_type`
    /// and the meta-schema admit are one decision (HW-OBL-0171). Before it, the
    /// meta-schema read digit positions and admitted `2026-02-30`, which this
    /// reader refused, so a value passed one gate and was skipped by the next.
    #[test]
    fn the_type_test_and_the_date_reader_agree() {
        for text in [
            "2026-02-30",
            "0000-00-00",
            "99999-99-99",
            "4294967296-01-01",
            "2024-02-29",
            "2025-02-29",
            "2026-12-31",
            "2026-8-12",
        ] {
            let scalar = headwater_yaml::Scalar {
                text: text.to_string(),
                style: headwater_yaml::Style::Plain,
            };
            assert_eq!(
                headwater_meta::ScalarType::Date.admits(&scalar),
                Date::parse(text).is_some(),
                "the two readers disagree on {text}"
            );
        }
        let admitted = ["2024-02-29", "2026-12-31"];
        for text in admitted {
            assert!(Date::parse(text).is_some(), "{text} is a date");
        }
    }
}
