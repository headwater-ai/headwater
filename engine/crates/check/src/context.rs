// SPDX-License-Identifier: Apache-2.0
//! The injected values, and the one of them that exists.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)
//! names two inputs that are about time, and rules that both are injected and
//! never fetched: "`ctx.now` is a bound value, never a syscall". A windowed
//! participation expectation reads a declared origin date from the document and
//! compares it against that value, with no history walk.
//!
//! The clock is here. The prior version is not, because it "is available
//! **only** in change-scoped evaluation", which is
//! [#58](https://github.com/headwater-ai/headwater/issues/58), and a field that
//! nothing enforces is the comment [`crate::scope`] exists to delete.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    now: Date,
}

impl Context {
    /// A run at a stated date. Every test that records a report uses this one,
    /// because a recorded report is a function of its inputs and the clock is
    /// one of them.
    pub const fn at(now: Date) -> Self {
        Context { now }
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
        })
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
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year: i64 = text[0..4].parse().ok()?;
        let month: u32 = text[5..7].parse().ok()?;
        let day: u32 = text[8..10].parse().ok()?;
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return None;
        }
        Some(Date {
            days: days_from_civil(year, month, day),
        })
    }

    pub const fn from_days(days: i64) -> Self {
        Date { days }
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

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => match (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
            true => 29,
            false => 28,
        },
    }
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
}
