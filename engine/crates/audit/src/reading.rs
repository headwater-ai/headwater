// SPDX-License-Identifier: Apache-2.0
//! The adoption store: one reading of the payload per run of `taxonomy audit
//! --record`.
//!
//! [Q12](../../../../docs/spec/09-decisions.md#q12--migration-path-for-an-existing-corpus)
//! rules that adoption is a migration from no taxonomy, and
//! [HW-OBL-0008](../../../../docs/obligations/0008-an-adoption-payload-has-a-first-reading-and-no-elapsed-time.md)
//! names the instrument in its own words: "the remaining pair count over time,
//! against the fraction of payloads that reach zero before the expiry". A verb
//! that prints one number states a value and never a trend, and that record ends
//! with the sentence this store answers: "What no engine can supply is time".
//!
//! # Why a store, and it is not the capture-cost argument
//!
//! [`headwater_scaffold::reading`] is the shape this file copies and it is the
//! wrong argument for it. Capture cost is written at run time because no later
//! reader can recover it: `created_by` sits on the relation type, so a committed
//! corpus cannot say which edge a scaffolder wrote. An adoption reading **is**
//! recoverable. The lock and the corpus are both committed, and `headwater check
//! --now <date>` is deterministic over them, so any reading of any past day
//! could be retaken from the tree of that day.
//!
//! The reason this store exists is [spec
//! 12](../../../../docs/spec/12-check-layer.md)'s instead: **a change reaches
//! this engine as a named set of inputs, and never as a second tree**. No crate
//! here walks git history and none opens a socket. So the tree of a past day is
//! an input no run of this engine has, and the only way it can hold a series is
//! to be handed one reading at a time, one per invocation, by a caller who
//! decided to take one.
//!
//! # What a reading holds, and why each member is in it
//!
//! One line per **invocation**, and never one per task. A run over a corpus that
//! declares no payload has to be recordable, `tasks: []` is a real state, and a
//! per-task line cannot hold it. A store that skipped that state would make
//! "nobody has recorded anything" and "the payload is gone" one file.
//!
//! - `lock` is the digest of the taxonomy the reading was taken under. The
//!   payload is a set of `(document, rule)` pairs, and a rule the taxonomy
//!   stopped running closes a pair with no change in what anybody wrote. So two
//!   readings taken under two digests are two measurements, and the report says
//!   which is which rather than averaging them.
//! - `date` is the injected clock, so a reading is reproducible under `--now`,
//!   and it is the value the elapsed time between two readings is made of.
//! - `refused` counts the tasks the block declared that this engine could not
//!   read. A refused task is a task nobody is measuring, and a series that
//!   dropped it would report a payload shrinking when it went dark.
//! - `tasks` is one entry per task, in the order the lock declares them.
//!
//! A task entry holds its own `until`, so a later reader can say whether the
//! task reached zero before its expiry without re-opening the lock of that day.
//! It holds the `state` the check layer decided rather than a second comparison
//! of `until` against `date`, because a second comparison here is a second
//! answer to a question that layer already answered. **A task that lapsed reads
//! as its whole pair set open**, which is not a defect of this reading:
//! [`headwater_check::adoption::apply`] holds nothing for an expired task, so
//! every pair it named is reported and none of them is closed.
//!
//! # What a reading deliberately does not hold
//!
//! **No pair list and no document path.** The pairs are in the committed lock
//! and the findings are in the check report. A store that copied them would be a
//! second copy of the payload that a lock edit can falsify, and the count is the
//! thing that decays.
//!
//! **No owner.** The lock names the owner and the check report prints it beside
//! every task, so a reader joins on the task identifier. A per-owner series over
//! time is a performance measure, and [spec
//! 3](../../../../docs/spec/03-authoring-and-lifecycle.md#what-the-capture-cost-store-records-and-what-it-refuses-to)'s
//! argument against a per-author capture-cost figure applies here unchanged.
//!
//! **No key of the block that nothing read.** Such a key holds nothing and
//! refuses nothing, so it moves no count here. The check report is where one is
//! named.
//!
//! **No wall-clock time and no duration.** Neither is reproducible.
//!
//! **No engine version and no host.** Neither is a fact about the payload.
//!
//! # Where it lands
//!
//! [`STORE`], under `.headwater/`, which is outside the corpus root. No census
//! row covers it, no language regime binds it, and no rule reads it — the same
//! boundary `.headwater/capture-cost.jsonl` sits on. It is committed plain text,
//! so every reader recomputes each figure from the lines.

use headwater_check::adoption::{Ledger, State};
use headwater_check::Date;
use headwater_yaml::json::Json;
use std::path::Path;

/// Where the store lives, relative to the repository root.
///
/// One line per reading. A line is appended and no line is ever rewritten, so
/// two branches that each took a reading both keep it under the `merge=union`
/// attribute this repository declares for the path. A reading depends on no
/// other reading, and [`append`] refuses a duplicate `(lock, date)`, which is
/// the one state a union resolution could produce that the capture-cost store
/// cannot.
pub const STORE: &str = ".headwater/adoption.jsonl";

/// One task of the payload, as the store holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskReading {
    pub id: String,
    /// The task's own expiry. Held here so that "reached zero before the
    /// expiry" is answerable from the store alone.
    pub until: Date,
    /// `open` or `expired`, as [`headwater_check::adoption::apply`] decided it
    /// on that run.
    pub state: State,
    /// Declared pairs still raising a finding.
    pub open: usize,
    /// Declared pairs that raised none, which is the payload shrinking.
    pub closed: usize,
    /// Findings the task held out of the report.
    pub held: usize,
}

/// One run of `taxonomy audit --record`, as the store holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub lock: String,
    pub date: Date,
    /// Tasks the block declared that this engine could not read.
    pub refused: usize,
    /// One entry per task, in the order the lock declares them.
    pub tasks: Vec<TaskReading>,
}

/// The name the store writes for a state, and the name a report groups by.
///
/// Here rather than on [`State`] because the check crate is the one behind the
/// commit gate, and a store is not its business. The set is closed at two, so a
/// third value waits on a third state.
pub fn state_name(state: State) -> &'static str {
    match state {
        State::Open => "open",
        State::Expired => "expired",
    }
}

/// The state a name states, or nothing where this engine does not know it.
pub fn state_of(text: &str) -> Option<State> {
    match text {
        "open" => Some(State::Open),
        "expired" => Some(State::Expired),
        _ => None,
    }
}

impl Reading {
    /// The reading a run produced, derived from the ledger rather than tracked
    /// beside it, the way [`headwater_scaffold::reading::Reading::of`] derives
    /// from a plan.
    ///
    /// The per-task counts come off the task rather than off
    /// [`Ledger::open`] and [`Ledger::closed`], which sum over open tasks
    /// alone. A series built from the two aggregates would report a lapsed
    /// task's pairs as neither open nor closed, and a payload that lapsed
    /// would read as a payload that finished.
    pub fn of(ledger: &Ledger, lock: &str, date: Date) -> Reading {
        Reading {
            lock: lock.to_string(),
            date,
            refused: ledger.refused.len(),
            tasks: ledger
                .tasks
                .iter()
                .map(|task| TaskReading {
                    id: task.id.clone(),
                    until: task.until,
                    state: task.state,
                    open: task.pairs.len() - task.closed.len(),
                    closed: task.closed.len(),
                    held: task.held,
                })
                .collect(),
        }
    }

    /// One line of the store, with no trailing newline.
    ///
    /// The member order is fixed and the writer never sorts, so a reading
    /// written by a later engine diffs against an earlier one line for line.
    pub fn render(&self) -> String {
        let number = |value: usize| Json::Raw(value.to_string());
        let mut members = vec![
            ("lock", Json::string(&self.lock)),
            ("date", Json::string(self.date.render())),
        ];
        // Absent rather than zero, on the rule `surface` and `id` already follow
        // in the capture-cost store: a member with an empty value is a term that
        // names nothing.
        if self.refused > 0 {
            members.push(("refused", number(self.refused)));
        }
        members.push((
            "tasks",
            Json::Array(
                self.tasks
                    .iter()
                    .map(|task| {
                        Json::object([
                            ("id", Json::string(&task.id)),
                            ("until", Json::string(task.until.render())),
                            ("state", Json::string(state_name(task.state))),
                            ("open", number(task.open)),
                            ("closed", number(task.closed)),
                            ("held", number(task.held)),
                        ])
                    })
                    .collect(),
            ),
        ));
        Json::object(members).render()
    }

    /// One line of the store, read back.
    ///
    /// JSON is a subset of the YAML 1.2 core schema, so the loader that reads
    /// every taxonomy source reads this too and the store needs no second
    /// parser. A member this cannot read is an error naming the member, because
    /// a reading that silently lost a term would move a figure and report
    /// nothing.
    pub fn parse(line: &str) -> Result<Reading, String> {
        let root = headwater_yaml::load(line)
            .map_err(|errors| format!("it is not a JSON object ({} parse errors)", errors.len()))?;
        let map = root
            .value
            .as_map()
            .ok_or_else(|| "it is not a JSON object".to_string())?;
        let text = |key: &str| -> Result<String, String> {
            map.get(key)
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .ok_or_else(|| format!("it names no `{key}`"))
        };
        let date = |key: &str, written: &str| -> Result<Date, String> {
            Date::parse(written).ok_or_else(|| format!("`{key}` is `{written}` rather than a date"))
        };
        let refused = match map.get("refused").and_then(|node| node.value.as_scalar()) {
            None => 0,
            Some(scalar) => scalar
                .text
                .parse::<usize>()
                .map_err(|_| format!("`refused` is `{}` rather than a count", scalar.text))?,
        };
        let entries = map
            .get("tasks")
            .and_then(|node| node.value.as_seq())
            .ok_or_else(|| "it names no `tasks`".to_string())?;
        let mut tasks = Vec::with_capacity(entries.len());
        for entry in entries {
            let task = entry
                .value
                .as_map()
                .ok_or_else(|| "a member of `tasks` is not an object".to_string())?;
            let member = |key: &str| -> Result<String, String> {
                task.get(key)
                    .and_then(|node| node.value.as_scalar())
                    .map(|scalar| scalar.text.clone())
                    .ok_or_else(|| format!("a task names no `{key}`"))
            };
            let count = |key: &str| -> Result<usize, String> {
                let written = member(key)?;
                written
                    .parse::<usize>()
                    .map_err(|_| format!("`{key}` is `{written}` rather than a count"))
            };
            let until = member("until")?;
            let state = member("state")?;
            tasks.push(TaskReading {
                id: member("id")?,
                until: date("until", &until)?,
                // A state a later engine writes and this one does not know is
                // an error, on the rule the capture-cost surface follows. A
                // reading counted under neither state would be a task that this
                // report silently stopped watching.
                state: state_of(&state).ok_or_else(|| {
                    format!("`state` is `{state}` rather than `open` or `expired`")
                })?,
                open: count("open")?,
                closed: count("closed")?,
                held: count("held")?,
            });
        }
        let written = text("date")?;
        Ok(Reading {
            lock: text("lock")?,
            date: date("date", &written)?,
            refused,
            tasks,
        })
    }

    /// Pairs of this reading still raising a finding, over every task.
    pub fn open(&self) -> usize {
        self.tasks.iter().map(|task| task.open).sum()
    }

    /// Pairs of this reading that raised none.
    pub fn closed(&self) -> usize {
        self.tasks.iter().map(|task| task.closed).sum()
    }

    /// Findings this reading's tasks held out of the report.
    pub fn held(&self) -> usize {
        self.tasks.iter().map(|task| task.held).sum()
    }
}

/// A line the store holds that no reading could be read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable {
    /// One-based, so it names the line a person opens the file to.
    pub line: usize,
    pub why: String,
}

/// Whether an append put a line on the store, or found the reading already
/// there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appended {
    /// The store gained the line.
    Written,
    /// A reading at this `(lock, date)` already stood, and nothing was written.
    Held,
}

/// Put one reading on the end of the store, creating the file if it is absent.
///
/// **A reading the store already holds at this lock and this date is not
/// appended twice.** `engine/crates/cli/src/lib.rs` promises in the `--now` help
/// text that "two audits of one tree at one date write the same bytes", and a
/// verb that appended on every run would falsify that on its second run. The key
/// is the pair rather than the date alone, because two taxonomies over one tree
/// on one day are two measurements.
pub fn append(root: &Path, reading: &Reading) -> Result<Appended, String> {
    use std::io::Write;
    let (held, _) = load(root)?;
    if held
        .iter()
        .any(|stood| stood.lock == reading.lock && stood.date == reading.date)
    {
        return Ok(Appended::Held);
    }
    let path = root.join(STORE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{}", reading.render()).map_err(|error| error.to_string())?;
    Ok(Appended::Written)
}

/// Every reading the store holds, and every line it holds that is not one.
///
/// An absent store is an empty store rather than an error. A corpus over which
/// nobody has taken a reading is the ordinary first state, and it is a different
/// state from a corpus that declares no payload.
pub fn load(root: &Path) -> Result<(Vec<Reading>, Vec<Unreadable>), String> {
    let path = root.join(STORE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], vec![])),
        Err(error) => return Err(error.to_string()),
    };
    let mut readings = Vec::new();
    let mut unreadable = Vec::new();
    for (at, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match Reading::parse(line) {
            Ok(reading) => readings.push(reading),
            Err(why) => unreadable.push(Unreadable { line: at + 1, why }),
        }
    }
    Ok((readings, unreadable))
}

/// Every distinct lock digest the readings span, in first-seen order.
///
/// More than one means the readings were taken under more than one taxonomy,
/// and the report says so rather than trending across them.
pub fn locks(readings: &[Reading]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for reading in readings {
        if !out.contains(&reading.lock) {
            out.push(reading.lock.clone());
        }
    }
    out
}

/// The earliest and the latest date the readings carry, in file order.
///
/// `None` over an empty store. The two dates are what the elapsed time between
/// the first reading and the last is made of, and it is the one quantity
/// [HW-OBL-0008](../../../../docs/obligations/0008-an-adoption-payload-has-a-first-reading-and-no-elapsed-time.md)
/// says no engine can supply.
pub fn span(readings: &[Reading]) -> Option<(Date, Date)> {
    let mut dates = readings.iter().map(|reading| reading.date);
    let first = dates.next()?;
    Some(dates.fold((first, first), |(low, high), date| {
        (low.min(date), high.max(date))
    }))
}

/// Every entry the readings hold for one task, in the order they were taken.
pub fn appearances<'a>(readings: &'a [Reading], id: &str) -> Vec<&'a TaskReading> {
    readings
        .iter()
        .flat_map(|reading| reading.tasks.iter())
        .filter(|task| task.id == id)
        .collect()
}

/// The fraction HW-OBL-0008 names: `(tasks that stood at zero on or before
/// their expiry, distinct tasks the readings have seen)`.
///
/// The denominator is every task identifier the store holds, and never the
/// tasks a corpus declares today. A task that closed and was deleted from the
/// lock is a payload that reached zero, and a denominator over the lock in front
/// of the run would drop exactly the successes it exists to count.
///
/// A task counts in the numerator when some reading of it states `open` at 0
/// and that reading was taken on or before the `until` it carried. `until` is
/// the last day a task holds, which is the comparison
/// [`headwater_check::adoption::apply`] makes.
pub fn zeroed(readings: &[Reading]) -> (usize, usize) {
    let mut seen: Vec<&str> = Vec::new();
    let mut reached: Vec<&str> = Vec::new();
    for reading in readings {
        for task in &reading.tasks {
            if !seen.contains(&task.id.as_str()) {
                seen.push(&task.id);
            }
            // A task entry carries no date of its own, because a reading is one
            // invocation and every task in it was read at one moment. So the
            // date of the reading is the date of the entry.
            if task.open == 0 && task.until >= reading.date && !reached.contains(&task.id.as_str())
            {
                reached.push(&task.id);
            }
        }
    }
    (reached.len(), seen.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_reading() -> Reading {
        Reading {
            lock: "sha256:abc".to_string(),
            date: Date::parse("2026-08-28").expect("a date"),
            refused: 0,
            tasks: vec![TaskReading {
                id: "AD-1".to_string(),
                until: Date::parse("2027-06-30").expect("a date"),
                state: State::Open,
                open: 0,
                closed: 1,
                held: 0,
            }],
        }
    }

    #[test]
    fn a_reading_renders_as_one_line_in_a_fixed_member_order() {
        assert_eq!(
            a_reading().render(),
            "{\"lock\":\"sha256:abc\",\"date\":\"2026-08-28\",\"tasks\":[{\"id\":\"AD-1\",\
             \"until\":\"2027-06-30\",\"state\":\"open\",\"open\":0,\"closed\":1,\"held\":0}]}"
        );
    }

    #[test]
    fn what_the_writer_emits_is_what_the_reader_reads() {
        let reading = a_reading();
        assert_eq!(Reading::parse(&reading.render()), Ok(reading));
    }

    /// A corpus that declares no payload is a reading of none, and the member
    /// stays. An absent `tasks` is a line this reader refuses.
    #[test]
    fn a_corpus_with_no_payload_is_a_reading_with_no_tasks() {
        let mut reading = a_reading();
        reading.tasks = vec![];
        assert!(reading.render().contains("\"tasks\":[]"));
        assert_eq!(Reading::parse(&reading.render()), Ok(reading));
    }

    /// Absent rather than zero, and a non-zero count round trips.
    #[test]
    fn a_refused_count_is_absent_at_zero_and_written_above_it() {
        assert!(!a_reading().render().contains("refused"));
        let mut reading = a_reading();
        reading.refused = 2;
        assert!(reading.render().contains("\"refused\":2"));
        assert_eq!(Reading::parse(&reading.render()), Ok(reading));
    }

    #[test]
    fn a_member_that_cannot_be_read_is_an_error_naming_the_member() {
        let line = a_reading().render().replace("\"lock\"", "\"lcok\"");
        assert_eq!(Reading::parse(&line), Err("it names no `lock`".to_string()));
        let line = a_reading().render().replace("\"tasks\"", "\"taks\"");
        assert_eq!(
            Reading::parse(&line),
            Err("it names no `tasks`".to_string())
        );
        let line = a_reading().render().replace("\"held\"", "\"hled\"");
        assert_eq!(
            Reading::parse(&line),
            Err("a task names no `held`".to_string())
        );
        let line = a_reading()
            .render()
            .replace("2026-08-28", "the twenty eighth");
        assert_eq!(
            Reading::parse(&line),
            Err("`date` is `the twenty eighth` rather than a date".to_string())
        );
    }

    /// A state a later engine writes and this one does not know is an error,
    /// and never a task counted under neither state.
    #[test]
    fn a_state_this_engine_does_not_know_is_an_error_naming_the_member() {
        let line = a_reading().render().replace("\"open\",", "\"renewed\",");
        assert_eq!(
            Reading::parse(&line),
            Err("`state` is `renewed` rather than `open` or `expired`".to_string())
        );
    }

    #[test]
    fn an_absent_store_is_an_empty_store() {
        let scratch = std::env::temp_dir().join("headwater-adoption-reading-absent");
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("a scratch directory");
        assert_eq!(load(&scratch), Ok((vec![], vec![])));
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn a_line_the_store_cannot_read_is_named_by_its_line_number_and_never_counted() {
        let scratch = std::env::temp_dir().join("headwater-adoption-reading-unreadable");
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("a scratch directory");
        std::fs::create_dir_all(scratch.join(".headwater")).expect("the directory is there");
        std::fs::write(
            scratch.join(STORE),
            format!("{}\nnot a reading at all\n", a_reading().render()),
        )
        .expect("it writes");
        let (readings, unreadable) = load(&scratch).expect("it loads");
        assert_eq!(readings, vec![a_reading()]);
        assert_eq!(unreadable.len(), 1);
        assert_eq!(unreadable[0].line, 2);
        let _ = std::fs::remove_dir_all(&scratch);
    }

    /// The byte-identity promise of the `--now` help text, in the writer.
    #[test]
    fn a_reading_the_store_already_holds_is_not_appended_twice() {
        let scratch = std::env::temp_dir().join("headwater-adoption-reading-append");
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("a scratch directory");
        assert_eq!(append(&scratch, &a_reading()), Ok(Appended::Written));
        assert_eq!(append(&scratch, &a_reading()), Ok(Appended::Held));
        let (readings, _) = load(&scratch).expect("it loads");
        assert_eq!(readings, vec![a_reading()]);

        // One tree, one date, a second taxonomy. Two measurements, so two
        // lines, and the key is the pair rather than the date alone.
        let mut second = a_reading();
        second.lock = "sha256:def".to_string();
        assert_eq!(append(&scratch, &second), Ok(Appended::Written));
        let (readings, _) = load(&scratch).expect("it loads");
        assert_eq!(readings.len(), 2);
        assert_eq!(
            locks(&readings),
            vec!["sha256:abc".to_string(), "sha256:def".to_string()]
        );
        let _ = std::fs::remove_dir_all(&scratch);
    }

    /// The fraction the obligation names, over the two directions that separate
    /// it: a payload that reached zero in time, and one that never did.
    #[test]
    fn a_payload_counts_as_reaching_zero_only_where_a_reading_says_so_in_time() {
        let mut first = a_reading();
        first.tasks[0].open = 1;
        first.tasks[0].closed = 0;
        first.tasks[0].held = 1;
        assert_eq!(zeroed(&[first.clone()]), (0, 1));

        let mut later = a_reading();
        later.date = Date::parse("2026-12-01").expect("a date");
        assert_eq!(zeroed(&[first.clone(), later.clone()]), (1, 1));

        // The same task at zero, read after its expiry. A payload that reached
        // zero late did not reach zero before the expiry, which is the whole of
        // what the fraction measures.
        let mut late = later.clone();
        late.date = Date::parse("2027-07-01").expect("a date");
        assert_eq!(zeroed(&[first, late]), (0, 1));

        // Two tasks, one of each.
        let mut both = later;
        both.tasks.push(TaskReading {
            id: "AD-2".to_string(),
            until: Date::parse("2027-06-30").expect("a date"),
            state: State::Open,
            open: 3,
            closed: 0,
            held: 3,
        });
        assert_eq!(zeroed(&[both]), (1, 2));
    }

    /// `until` is the last day a task holds, so a reading taken on the expiry
    /// itself is in time.
    ///
    /// The case above reads well before the expiry and well after it, and a
    /// comparison written `>` rather than `>=` passes both. The boundary day is
    /// the one value that separates them, and it is not a free choice here:
    /// [`headwater_check::adoption::apply`] expires a task on `until < now`, so
    /// a task read on its `until` is still open, and a fraction that called
    /// that day late would disagree with the run that produced the reading.
    /// `an_expired_task_holds_nothing_and_the_findings_come_back` in the check
    /// crate is the mirror of this case.
    #[test]
    fn a_reading_taken_on_the_expiry_itself_is_in_time() {
        let mut on_the_day = a_reading();
        on_the_day.date = Date::parse("2027-06-30").expect("a date");
        assert_eq!(on_the_day.tasks[0].until, on_the_day.date);
        assert_eq!(zeroed(&[on_the_day.clone()]), (1, 1));

        let mut the_day_after = on_the_day;
        the_day_after.date = Date::parse("2027-07-01").expect("a date");
        assert_eq!(zeroed(&[the_day_after]), (0, 1));
    }

    #[test]
    fn the_span_is_the_earliest_and_the_latest_date_whatever_order_they_arrived_in() {
        assert_eq!(span(&[]), None);
        let mut later = a_reading();
        later.date = Date::parse("2026-12-01").expect("a date");
        let mut earlier = a_reading();
        earlier.date = Date::parse("2026-01-01").expect("a date");
        assert_eq!(
            span(&[later, earlier]),
            Some((
                Date::parse("2026-01-01").expect("a date"),
                Date::parse("2026-12-01").expect("a date")
            ))
        );
    }
}
