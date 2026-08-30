// SPDX-License-Identifier: Apache-2.0
//! The adoption payload: the debt a corpus declared when a taxonomy first
//! reached it.
//!
//! [Q12](../../../../docs/spec/09-decisions.md#q12--migration-path-for-an-existing-corpus)
//! rules that adoption is a migration from no taxonomy. Before the first
//! `headwater init` a corpus is governed by nothing, so every document in it is
//! trivially valid, and the findings that the first taxonomy raises are that
//! taxonomy's migration payload. `headwater infer` computes it from the diff
//! between nothing and one, and the block sits in the lock, which is committed
//! and reviewed ([`crate::adoption::read`] is what reads it back).
//!
//! # The grain is a pair, and the reason is arithmetic
//!
//! A task holds `(document, rule)` pairs. [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)
//! says why the coarser grain fails: "A label by document alone would blanket
//! every finding on a named document for the whole migration. Defects
//! introduced yesterday would then read as expected breakage." So a pair is
//! matched on rule **and** path, and a path that carries a `*` is refused. A
//! glob is a blanket with a pair's syntax, and the whole of what this mechanism
//! buys is that yesterday's regression stays loud while declared debt stays
//! patient.
//!
//! **A pair is a cell, not a finding, and that is the one thing it buys an
//! adopter for free.** A pair holds every finding of its rule on its document,
//! including a finding raised after the payload was written. So the sentence
//! spec 7 uses for the pair grain — "the pair grain keeps yesterday's
//! regression loud while the declared debt stays patient" — holds across
//! documents and across rules, and does not hold inside one cell. A second
//! violation of a declared rule in a declared document is absorbed. The finer
//! grain that would not absorb it is `(document, rule, line)`, and a line
//! number moves whenever a paragraph above it does, so a payload at that grain
//! would expire on the next edit rather than on its date. The cell is the
//! coarsest grain that cannot blanket a rule or a document, and the absorption
//! is what it costs.
//!
//! **No threshold ever converts an accounting into a silence.** Q12 names the
//! shape it is refusing: a tool that excludes offending files one at a time and
//! disables the rule once the list passes a limit. First contact is the one
//! moment at which every rule exceeds any such limit, so a threshold of that
//! shape switches off most of the rule set and reports the result green. There
//! is therefore no count anywhere in this module that can turn a pair into a
//! rule-wide exemption, and the format has no place to write one.
//!
//! # A held finding is reclassified rather than hidden
//!
//! This is the difference from [`crate::suppression`], and it is the whole of
//! why the two are separate modules. Spec 7: `migration-pending` findings are
//! "counted, visible in coverage, never blocking, and never suppressed
//! individually". A suppressed finding is one a reader does not see. A pending
//! finding is one a reader sees, with the task, the owner and the expiry beside
//! it, and it does not fail a `--strict` run.
//!
//! # Precedence, and how the partition holds
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! fixes waiver, then migration-pending, then suppression, "so the three
//! inventories partition the escaped findings, and no finding is counted three
//! times". This runs before suppression, so a finding a task holds never
//! reaches a directive and cannot appear in both inventories. The partition is
//! a consequence of the call order rather than a rule that something checks
//! afterwards. Waiver is still absent, and [`crate::register`] prints it as
//! absent for the reason it always did.
//!
//! # What a task deliberately does not carry
//!
//! **A to-version.** Spec 7 gives a migration state a from-version and a
//! to-version. The from-version is absent at first contact, which is Q12's one
//! field. The to-version is the taxonomy this payload was written against, and
//! the payload sits inside that taxonomy's lock, which declares its package and
//! version four lines above. A second copy could disagree with the first, and
//! nothing could say which of the two the run honored.
//!
//! **A severity.** A pending finding keeps the severity its rule gave it. Spec
//! 12 puts severity on the check and spec 4 on the obligation, and a payload is
//! neither. An adopter who wants a finding quieter is asking for a different
//! taxonomy, not for a different accounting of the same one.
//!
//! # A key this engine does not read
//!
//! [`read`] refuses a pair that names a rule this engine does not carry, on the
//! argument [`crate::suppression::read`] makes about `allow=`: a directive
//! against a rule that does not exist is a directive its author believes is
//! working. The same holds of a key, and the two levels want different answers.
//!
//! A key beside `tasks` holds nothing, so the run names it and reads the tasks
//! beside it ([`Unread`]). A key inside a task, or inside one of its pairs, may
//! mean the task is not the debt this engine read out of it, so the task is
//! refused and every finding it named is reported ([`Refused`]).
//!
//! `from` and `to` are named rather than lumped in, because they are the two
//! fields spec 7 gives a migration state and the two that
//! [#78](https://github.com/headwater-ai/headwater/issues/78) exists to add. No
//! verb writes either one today. An adopter who writes one by hand used to get
//! a green run and a false belief, which is the state this naming ends.

use crate::context::Date;
use crate::finding::Finding;
use headwater_yaml::{Mapping, Value};

/// One `(document, rule)` pair a task accounts for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair {
    pub path: String,
    pub rule: String,
}

/// Whether a task is still holding anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// In force. The pairs it names are reported as pending.
    Open,
    /// Past its expiry. The findings it named are reported as findings, which
    /// is the conversation the expiry exists to force.
    Expired,
}

/// One declared unit of debt, with the name and the date that spec 4 ranks a
/// migration state above a suppression for.
#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub statement: String,
    pub owner: String,
    pub until: Date,
    pub pairs: Vec<Pair>,
    pub state: State,
    /// Findings this task held on this run.
    pub held: usize,
    /// Pairs it declared that raised no finding. This is the task shrinking,
    /// and it is reported so that progress is visible before the expiry.
    pub closed: Vec<Pair>,
}

/// A task this engine could not read, and why.
///
/// It holds nothing, so every finding it named is reported. The precedent is
/// [`crate::suppression::Refused`]: a declaration the engine cannot read must
/// not quietly behave like one it can.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub task: String,
    pub why: String,
}

/// A key of the block that this engine does not read, and why that is worth a
/// line.
///
/// It holds nothing and it refuses nothing, so the tasks beside it are read as
/// they always were. What it costs today is a belief: an adopter writes a key,
/// no run mentions it, and the silence reads as agreement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unread {
    pub key: String,
    pub why: String,
}

/// What reading the `adoption` block of a lock found.
#[derive(Clone, Debug, Default)]
pub struct Declared {
    pub tasks: Vec<Task>,
    pub refused: Vec<Refused>,
    pub unread: Vec<Unread>,
}

/// What a run has to say about the payload.
#[derive(Clone, Debug, Default)]
pub struct Ledger {
    pub tasks: Vec<Task>,
    pub refused: Vec<Refused>,
    /// Keys of the block that nothing read.
    pub unread: Vec<Unread>,
    /// The findings the payload held, in report order.
    pub pending: Vec<Finding>,
}

/// Read the `adoption` block of a lock.
///
/// `rules` is the rule set of this engine, so a pair that names a rule nothing
/// runs is refused by name rather than silently matching nothing. That is the
/// same argument [`crate::suppression::read`] makes about `allow=`: a directive
/// against a rule that does not exist is a directive its author believes is
/// working.
pub fn read(block: &Mapping, rules: &[&'static str]) -> Declared {
    let mut declared = Declared::default();

    // The block's own keys, before its tasks. `tasks` is the whole of what this
    // engine reads here, so every sibling of it is a key an author wrote and a
    // run honored nothing of. Naming one costs the reading nothing: the block
    // still holds the tasks it declares, and they are still read.
    for entry in block {
        if entry.key.value != "tasks" {
            declared.unread.push(Unread {
                key: entry.key.value.clone(),
                why: unread(&entry.key.value),
            });
        }
    }

    let Some(items) = block.get("tasks").and_then(|node| node.value.as_seq()) else {
        declared.refused.push(Refused {
            task: "adoption".to_string(),
            why: "it declares no `tasks` sequence, so it accounts for nothing".to_string(),
        });
        return declared;
    };

    for (index, item) in items.iter().enumerate() {
        // A task with no readable identity is still named, by position, so a
        // reader can find the entry this refusal is about.
        let name = item
            .value
            .as_map()
            .and_then(|map| scalar(map, "id"))
            .map(str::to_string)
            .unwrap_or_else(|| format!("task {}", index + 1));
        match task(item.value.as_map(), rules) {
            Ok(read) => declared.tasks.push(read),
            Err(why) => declared.refused.push(Refused { task: name, why }),
        }
    }
    declared
}

/// Why a key nothing reads is worth saying out loud.
///
/// The two named cases are the ones an adopter has a reason to write. `from`
/// and `to` are spec 7's two fields of a migration state, and `severity` is the
/// one this module's own header says the format deliberately has no place for.
/// A message that said only "unknown key" would leave all three sounding like a
/// typo.
fn unread(key: &str) -> String {
    match key {
        "from" | "to" => "it names a migration state. Spec 7 gives one a from-version and a \
             to-version, and no verb of this engine writes either into this block"
            .to_string(),
        "severity" => "a pending finding keeps the severity its rule gave it, and this format \
             has no place to write one"
            .to_string(),
        _ => "nothing in this engine reads it, so what it declares is not what this run honored"
            .to_string(),
    }
}

fn task(map: Option<&Mapping>, rules: &[&'static str]) -> Result<Task, String> {
    let map = map.ok_or("it is not a mapping")?;
    // Refused rather than noted, which is the difference from the block above.
    // A key here sits inside the unit that holds findings, so a task carrying
    // one may be an accounting this engine did not read. It holds nothing until
    // somebody says what the key means.
    for entry in map {
        let key = entry.key.value.as_str();
        if !["id", "statement", "owner", "until", "pairs"].contains(&key) {
            return Err(format!("it declares `{key}`, and {}", unread(key)));
        }
    }
    let id = scalar(map, "id")
        .ok_or("it names no `id`, and a task a report cannot name is one nobody can close")?;
    let statement = scalar(map, "statement")
        .ok_or("it states no `statement`, so nothing in it says what the debt is")?;
    // Spec 4 ranks a migration state above a suppression because it carries an
    // owner and a task list where a suppression carries one author's judgment.
    // A task with no owner is a suppression with more syntax.
    let owner = scalar(map, "owner").ok_or(
        "it names no `owner`, which is the field spec 4 ranks this above a suppression for",
    )?;
    let until = scalar(map, "until")
        .ok_or("it states no `until`, and debt with no expiry is a permanent exemption")?;
    let until = Date::parse(until)
        .ok_or_else(|| format!("`until: {until}` is not a date written `YYYY-MM-DD`"))?;

    let items = map
        .get("pairs")
        .and_then(|node| node.value.as_seq())
        .ok_or("it declares no `pairs` sequence")?;
    if items.is_empty() {
        return Err("it declares no pair, so it accounts for nothing".to_string());
    }

    let mut pairs = Vec::with_capacity(items.len());
    for item in items {
        let entry = item.value.as_map().ok_or("a pair is not a mapping")?;
        for field in entry {
            let key = field.key.value.as_str();
            if !["path", "rule"].contains(&key) {
                return Err(format!("a pair declares `{key}`, and {}", unread(key)));
            }
        }
        let path = entry
            .get("path")
            .and_then(scalar_of)
            .ok_or("a pair names no `path`")?;
        let rule = entry
            .get("rule")
            .and_then(scalar_of)
            .ok_or("a pair names no `rule`")?;
        // Spec 7: a label by document alone blankets every finding on that
        // document. A glob is that label with a pair's syntax.
        if path.contains('*') {
            return Err(format!(
                "the pair `{path}` is a pattern, and a payload holds one document at a time. \
                 A pattern would blanket a document this taxonomy has not seen yet"
            ));
        }
        if !rules.contains(&rule) {
            return Err(format!("no rule of this engine is named `{rule}`"));
        }
        pairs.push(Pair {
            path: path.to_string(),
            rule: rule.to_string(),
        });
    }

    Ok(Task {
        id: id.to_string(),
        statement: statement.to_string(),
        owner: owner.to_string(),
        until,
        pairs,
        state: State::Open,
        held: 0,
        closed: Vec::new(),
    })
}

fn scalar<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key).and_then(scalar_of)
}

fn scalar_of(node: &headwater_yaml::Spanned<Value>) -> Option<&str> {
    node.value.as_scalar().map(|scalar| scalar.text.as_str())
}

/// Hold every finding a task accounts for, and report what remains.
///
/// This runs before [`crate::suppression::apply`], which is the precedence spec
/// 4 fixes. A finding held here never reaches a directive, so the two
/// inventories partition by construction.
pub fn apply(findings: Vec<Finding>, declared: Declared, now: Date) -> (Vec<Finding>, Ledger) {
    let Declared {
        mut tasks,
        refused,
        unread,
    } = declared;
    for task in &mut tasks {
        // Expiry is decided once, against the injected clock, before any
        // finding is compared. `until` is the last day the task holds.
        if task.until < now {
            task.state = State::Expired;
        }
    }

    let mut kept = Vec::with_capacity(findings.len());
    let mut pending = Vec::new();
    // Which declared pairs were met this run. A pair not met is a pair that
    // stopped failing, which is the number that makes a payload visibly shrink.
    let mut met: Vec<Vec<bool>> = tasks
        .iter()
        .map(|task| vec![false; task.pairs.len()])
        .collect();

    for finding in findings {
        let hit = tasks.iter_mut().enumerate().find_map(|(index, task)| {
            if task.state == State::Expired {
                return None;
            }
            let at = task
                .pairs
                .iter()
                .position(|pair| pair.rule == finding.rule && pair.path == finding.path)?;
            Some((index, at, task))
        });
        match hit {
            Some((index, at, task)) => {
                task.held += 1;
                met[index][at] = true;
                pending.push(finding);
            }
            None => kept.push(finding),
        }
    }

    for (index, task) in tasks.iter_mut().enumerate() {
        if task.state == State::Expired {
            continue;
        }
        task.closed = task
            .pairs
            .iter()
            .enumerate()
            .filter(|(at, _)| !met[index][*at])
            .map(|(_, pair)| pair.clone())
            .collect();
    }

    (
        kept,
        Ledger {
            tasks,
            refused,
            unread,
            pending,
        },
    )
}

impl Ledger {
    /// Whether this run has anything to say about a payload at all.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty() && self.refused.is_empty() && self.unread.is_empty()
    }

    /// Pairs still failing. This is the number spec 7 puts beside coverage:
    /// "a payload that does not move is then visible from the second run rather
    /// than from the expiry".
    ///
    /// Pairs rather than findings, because a pair is the unit an adopter closes
    /// and the two numbers move apart. One document that raises four findings of
    /// one rule is one pair, and fixing three of the four moves neither count.
    /// [`Ledger::held`] is the other number, and the report prints both.
    pub fn open(&self) -> usize {
        self.tasks
            .iter()
            .filter(|task| task.state == State::Open)
            .map(|task| task.pairs.len() - task.closed.len())
            .sum()
    }

    /// Findings the payload held. The number a reader compares against the
    /// findings list to see what a green run is standing on.
    pub fn held(&self) -> usize {
        self.tasks.iter().map(|task| task.held).sum()
    }

    /// Declared pairs that raised no finding, which is the payload shrinking.
    pub fn closed(&self) -> usize {
        self.tasks
            .iter()
            .filter(|task| task.state == State::Open)
            .map(|task| task.closed.len())
            .sum()
    }

    /// Pending findings per rule, largest first and then by name. This is what
    /// the register reads to attribute an escape to an obligation.
    pub fn by_rule(&self) -> Vec<(&str, usize)> {
        let mut counts: Vec<(&str, usize)> = Vec::new();
        for finding in &self.pending {
            match counts.iter_mut().find(|(rule, _)| *rule == finding.rule) {
                Some((_, count)) => *count += 1,
                None => counts.push((finding.rule, 1)),
            }
        }
        counts.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
        counts
    }

    /// The ledger as text, for the report.
    ///
    /// The first line is the count that spec 7 requires on every run. The task
    /// lines carry the owner and the expiry, because a number with no name
    /// beside it is a number nobody works.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        if self.is_empty() {
            return out;
        }
        out.push_str("adoption\n");
        let _ = writeln!(
            out,
            "  {} pairs open, {} closed, holding {} {}, in {} {}",
            self.open(),
            self.closed(),
            self.held(),
            verb(self.held(), "finding", "findings"),
            self.tasks.len(),
            verb(self.tasks.len(), "task", "tasks")
        );
        for task in &self.tasks {
            match task.state {
                State::Open => {
                    let _ = writeln!(
                        out,
                        "  {} {} open, {} closed, holding {} {}, owner {}, until {}",
                        task.id,
                        task.pairs.len() - task.closed.len(),
                        task.closed.len(),
                        task.held,
                        verb(task.held, "finding", "findings"),
                        task.owner,
                        task.until
                    );
                }
                State::Expired => {
                    let _ = writeln!(
                        out,
                        "  {} lapsed on {}, and the {} pairs it named are reported, owner {}",
                        task.id,
                        task.until,
                        task.pairs.len(),
                        task.owner
                    );
                }
            }
            out.push_str(&crate::filled(&task.statement, 4));
        }
        for refused in &self.refused {
            out.push_str(&crate::filled(
                &format!("{} holds nothing: {}", refused.task, refused.why),
                2,
            ));
        }
        for unread in &self.unread {
            out.push_str(&crate::filled(
                &format!(
                    "the block declares `{}`, which nothing here reads: {}",
                    unread.key, unread.why
                ),
                2,
            ));
        }
        out
    }
}

fn verb(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    match count {
        1 => singular,
        _ => plural,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Severity;

    fn payload(source: &str) -> Mapping {
        headwater_yaml::load(source)
            .expect("the payload loads")
            .value
            .as_map()
            .expect("it is a mapping")
            .clone()
    }

    fn finding(path: &str, rule: &'static str) -> Finding {
        Finding {
            rule,
            severity: Severity::Warn,
            obligation: None,
            path: path.to_string(),
            line: 1,
            column: 1,
            message: "a message".to_string(),
            remediation: "a remediation".to_string(),
            patch: None,
        }
    }

    const RULES: [&str; 2] = ["facet.required.missing", "voice.forbidden_construction"];

    fn one_task(pairs: &str) -> Mapping {
        payload(&format!(
            "\
tasks:
  - id: AD-1
    statement: the corpus states no summary
    owner: the docs guild
    until: 2027-01-01
    pairs:
{pairs}
"
        ))
    }

    fn day(text: &str) -> Date {
        Date::parse(text).expect("a date")
    }

    /// The pair grain, which is the whole mechanism.
    ///
    /// Spec 7 refuses the document grain by name: "a label by document alone
    /// would blanket every finding on a named document for the whole
    /// migration. Defects introduced yesterday would then read as expected
    /// breakage." So a task that names one rule on one document holds that and
    /// holds neither neighbor.
    #[test]
    fn a_task_holds_one_pair_and_not_the_document_or_the_rule_around_it() {
        let block = one_task("      - {path: docs/a.md, rule: facet.required.missing}");
        let declared = read(&block, &RULES);
        assert!(declared.refused.is_empty(), "{:?}", declared.refused);

        let findings = vec![
            finding("docs/a.md", "facet.required.missing"),
            // Same document, another rule. Yesterday's regression.
            finding("docs/a.md", "voice.forbidden_construction"),
            // Same rule, another document.
            finding("docs/b.md", "facet.required.missing"),
        ];
        let (kept, ledger) = apply(findings, declared, day("2026-08-13"));

        assert_eq!(ledger.pending.len(), 1);
        assert_eq!(ledger.pending[0].path, "docs/a.md");
        assert_eq!(ledger.pending[0].rule, "facet.required.missing");
        assert_eq!(kept.len(), 2, "the two neighbors stayed loud");
        assert_eq!(ledger.open(), 1);
    }

    /// A declared pair that stops failing is reported as closed.
    ///
    /// Spec 7 wants a payload that does not move to be visible "from the second
    /// run rather than from the expiry", and a count of what is still open only
    /// says that with the number that closed beside it.
    #[test]
    fn a_pair_that_stopped_failing_is_reported_closed() {
        let block = one_task(
            "      - {path: docs/a.md, rule: facet.required.missing}\n\
             \x20     - {path: docs/b.md, rule: facet.required.missing}",
        );
        let (kept, ledger) = apply(
            vec![finding("docs/a.md", "facet.required.missing")],
            read(&block, &RULES),
            day("2026-08-13"),
        );
        assert!(kept.is_empty());
        assert_eq!(ledger.open(), 1);
        assert_eq!(ledger.closed(), 1);
        assert_eq!(ledger.tasks[0].closed[0].path, "docs/b.md");
    }

    /// An expired task holds nothing, and `until` is the last day it holds.
    #[test]
    fn an_expired_task_holds_nothing_and_the_findings_come_back() {
        let block = one_task("      - {path: docs/a.md, rule: facet.required.missing}");
        let (kept, ledger) = apply(
            vec![finding("docs/a.md", "facet.required.missing")],
            read(&block, &RULES),
            day("2027-01-02"),
        );
        assert_eq!(kept.len(), 1, "the finding is reported");
        assert!(ledger.pending.is_empty());
        assert_eq!(ledger.tasks[0].state, State::Expired);
        assert_eq!(ledger.open(), 0);

        // The last day it holds.
        let (kept, _) = apply(
            vec![finding("docs/a.md", "facet.required.missing")],
            read(&block, &RULES),
            day("2027-01-01"),
        );
        assert!(kept.is_empty(), "`until` is inclusive");
    }

    /// A path with a `*` is a blanket wearing a pair's syntax.
    #[test]
    fn a_pattern_is_refused_because_it_would_blanket_a_document_nothing_has_seen() {
        let block = one_task("      - {path: docs/*.md, rule: facet.required.missing}");
        let declared = read(&block, &RULES);
        assert!(declared.tasks.is_empty());
        assert_eq!(declared.refused.len(), 1);
        assert!(
            declared.refused[0].why.contains("one document at a time"),
            "{:?}",
            declared.refused
        );
    }

    /// The field spec 4 ranks a migration state above a suppression for.
    #[test]
    fn a_task_with_no_owner_is_refused() {
        let block = payload(
            "\
tasks:
  - id: AD-1
    statement: the corpus states no summary
    until: 2027-01-01
    pairs:
      - {path: docs/a.md, rule: facet.required.missing}
",
        );
        let declared = read(&block, &RULES);
        assert!(declared.tasks.is_empty());
        assert!(
            declared.refused[0].why.contains("`owner`"),
            "{:?}",
            declared.refused
        );

        // And a refused task holds nothing, so the finding is reported.
        let (kept, ledger) = apply(
            vec![finding("docs/a.md", "facet.required.missing")],
            declared,
            day("2026-08-13"),
        );
        assert_eq!(kept.len(), 1);
        assert_eq!(ledger.refused.len(), 1);
    }

    /// Debt with no expiry is a permanent exemption.
    #[test]
    fn a_task_with_no_expiry_or_an_unreadable_one_is_refused() {
        for line in ["", "    until: soon\n"] {
            let block = payload(&format!(
                "\
tasks:
  - id: AD-1
    statement: a statement
    owner: an owner
{line}    pairs:
      - {{path: docs/a.md, rule: facet.required.missing}}
"
            ));
            let declared = read(&block, &RULES);
            assert!(declared.tasks.is_empty(), "accepted `{line}`");
            assert_eq!(declared.refused.len(), 1);
        }
    }

    /// A pair against a rule nothing runs is a pair its author believes works.
    #[test]
    fn a_pair_naming_a_rule_this_engine_does_not_carry_is_refused() {
        let block = one_task("      - {path: docs/a.md, rule: voice.no_such_rule}");
        let declared = read(&block, &RULES);
        assert!(declared.tasks.is_empty());
        assert!(
            declared.refused[0].why.contains("voice.no_such_rule"),
            "{:?}",
            declared.refused
        );
    }

    /// The same argument, one level up, where it used to cost nothing.
    ///
    /// A key beside `tasks` is named and the tasks beside it are still read.
    /// `from` and `severity` are the two an adopter has a reason to write, and
    /// both used to ride through with nothing said anywhere.
    #[test]
    fn a_block_level_key_this_engine_does_not_read_is_named_and_holds_nothing() {
        let block = payload(
            "\
from: 99.0.0
severity: quiet
tasks:
  - id: AD-1
    statement: a statement
    owner: an owner
    until: 2027-01-01
    pairs:
      - {path: docs/a.md, rule: facet.required.missing}
",
        );
        let declared = read(&block, &RULES);
        assert!(declared.refused.is_empty(), "{:?}", declared.refused);
        assert_eq!(declared.tasks.len(), 1, "the task beside it is still read");
        assert_eq!(declared.unread.len(), 2);
        assert_eq!(declared.unread[0].key, "from");
        assert!(
            declared.unread[0].why.contains("migration state"),
            "{:?}",
            declared.unread
        );
        assert_eq!(declared.unread[1].key, "severity");
        assert!(
            declared.unread[1].why.contains("no place to write one"),
            "{:?}",
            declared.unread
        );

        // It is a note and not a refusal, so the pair is still held.
        let (kept, ledger) = apply(
            vec![finding("docs/a.md", "facet.required.missing")],
            declared,
            day("2026-08-13"),
        );
        assert!(kept.is_empty());
        assert_eq!(ledger.unread.len(), 2);
        assert!(ledger.render().contains("`from`"), "{}", ledger.render());
    }

    /// Inside a task the same key is a refusal, because the task may not be the
    /// debt this engine read out of it.
    #[test]
    fn a_task_level_key_this_engine_does_not_read_refuses_the_task() {
        let block = payload(
            "\
tasks:
  - id: AD-1
    statement: a statement
    owner: an owner
    until: 2027-01-01
    to: 4.0.0
    pairs:
      - {path: docs/a.md, rule: facet.required.missing}
",
        );
        let declared = read(&block, &RULES);
        assert!(declared.unread.is_empty(), "{:?}", declared.unread);
        assert!(declared.tasks.is_empty());
        assert_eq!(declared.refused.len(), 1);
        assert_eq!(declared.refused[0].task, "AD-1");
        assert!(
            declared.refused[0].why.contains("`to`"),
            "{:?}",
            declared.refused
        );

        // A refused task holds nothing, so the finding it named is reported.
        let (kept, _) = apply(
            vec![finding("docs/a.md", "facet.required.missing")],
            declared,
            day("2026-08-13"),
        );
        assert_eq!(kept.len(), 1);
    }

    /// And inside a pair, which is the innermost place an author writes one.
    #[test]
    fn a_pair_key_this_engine_does_not_read_refuses_the_task() {
        let block = one_task("      - {path: docs/a.md, rule: facet.required.missing, note: soon}");
        let declared = read(&block, &RULES);
        assert!(declared.tasks.is_empty());
        assert!(
            declared.refused[0].why.contains("a pair declares `note`"),
            "{:?}",
            declared.refused
        );
    }

    /// A block with no tasks accounts for nothing and says so.
    #[test]
    fn a_block_with_no_tasks_is_refused_rather_than_read_as_no_debt() {
        let declared = read(&payload("to: acme/x\n"), &RULES);
        assert!(declared.tasks.is_empty());
        assert_eq!(declared.refused.len(), 1);
        // And the key it does declare is named rather than passed over.
        assert_eq!(declared.unread.len(), 1);
        assert_eq!(declared.unread[0].key, "to");
    }

    /// A task that declares no pair accounts for nothing.
    #[test]
    fn a_task_with_an_empty_pair_list_is_refused() {
        let block = payload(
            "\
tasks:
  - id: AD-1
    statement: a statement
    owner: an owner
    until: 2027-01-01
    pairs: []
",
        );
        let declared = read(&block, &RULES);
        assert!(declared.tasks.is_empty());
        assert!(
            declared.refused[0].why.contains("accounts for nothing"),
            "{:?}",
            declared.refused
        );
    }

    /// The first task that names a pair holds it, and only one does.
    ///
    /// Two tasks that name one pair is an authoring mistake, and the number
    /// that matters is that the pair is counted once. A payload whose parts
    /// sum to more than the findings it holds is a payload nobody can audit.
    #[test]
    fn a_pair_named_by_two_tasks_is_held_once() {
        let block = payload(
            "\
tasks:
  - id: AD-1
    statement: first
    owner: an owner
    until: 2027-01-01
    pairs:
      - {path: docs/a.md, rule: facet.required.missing}
  - id: AD-2
    statement: second
    owner: another owner
    until: 2027-01-01
    pairs:
      - {path: docs/a.md, rule: facet.required.missing}
",
        );
        let (kept, ledger) = apply(
            vec![finding("docs/a.md", "facet.required.missing")],
            read(&block, &RULES),
            day("2026-08-13"),
        );
        assert!(kept.is_empty());
        assert_eq!(ledger.pending.len(), 1);
        assert_eq!(ledger.open(), 1);
        assert_eq!(ledger.tasks[0].held, 1);
        assert_eq!(ledger.tasks[1].held, 0);
        // The second task reports its pair as closed, which is true of this
        // run: nothing it named was reported under it.
        assert_eq!(ledger.tasks[1].closed.len(), 1);
    }
}
