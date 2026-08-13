// SPDX-License-Identifier: Apache-2.0
//! Suppression, and the inventory that makes it observable.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#suppression) states the
//! whole mechanism in one sentence, and every clause of it is a constraint here.
//! "Suppression is permitted, bounded, and observable. It is scoped to a file or
//! block, it carries an expiry, and it states a reason from a closed set." The
//! reasons are `false_positive`, which says the finding is wrong, and
//! `accepted_deviation`, which says it is right and tolerated for now.
//!
//! # Why the two reasons are not one field of prose
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#voice) gives the
//! reason: "a reason field that only holds prose collects no statistic at all".
//! The promotion machinery of spec 4 runs on a false-positive rate, and a rate
//! needs a numerator that says *wrong* rather than *tolerated*. So the reason is
//! a value from a closed set, and the free prose sits beside it rather than
//! instead of it.
//!
//! # This is the runner's, and a check never sees it
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#suppression-is-the-runners)
//! puts the filter here: "checks know nothing about suppressions. The runner
//! filters findings, records exactly what it filtered, and feeds the suppression
//! inventory into the coverage report. A check that handled its own suppressions
//! could hide them."
//!
//! Two consequences follow from where in a run that filtering happens, and both
//! are correctness rather than tidiness.
//!
//! **A cache holds what a check decided and never what a reader saw.** The
//! filter runs over the findings of instances that already have outcomes, so a
//! suppressed finding cannot become a cached `Passed`. An instance with four
//! findings of which one is suppressed still caches four. Suppress at the check
//! and the inventory loses the entry it exists to count, invisibly and for as
//! long as the cache entry lives.
//!
//! **Coverage does not move.** A suppression is not a skip. The instance ran,
//! the document is checked, and the only thing that changed is which findings
//! reach a report. A mechanism that made a document look unchecked would put a
//! coverage finding where an author had already stated a judgment.
//!
//! # The form, and why every field is required
//!
//! A directive is an HTML comment in the body of the document it acts on:
//!
//! ```text
//! <!-- headwater allow=<rule> scope=file|block until=<YYYY-MM-DD>
//!      reason=false_positive|accepted_deviation note=<free prose> -->
//! ```
//!
//! The keys may come in any order and `note` runs to the end. `note` is the one
//! that is optional, because it is the only one nothing counts.
//!
//! A directive that names a rule this engine does not carry, or a reason outside
//! the closed set, or a date that does not read, suppresses nothing and is
//! reported as refused. That is the failure this design most needs to catch: an
//! author who mistypes a rule name and believes a finding is handled is worse
//! off than one who wrote no directive at all, because the belief is what stops
//! them looking again.
//!
//! **Scope.** `scope=file` covers the whole document. `scope=block` covers the
//! block the comment sits in when it is written inside one, and the block that
//! follows it when the comment stands alone. That is the same reach the trailing
//! comments of `tools/ste-lint.py` have, so a corpus can move from one to the
//! other without rewriting where its hatches sit.
//!
//! **Expiry.** An expired directive suppresses nothing. The findings come back
//! and the inventory says which directive lapsed. Spec 4 made expiry mandatory
//! for a stated reason: the local mechanism, "the one that an individual author
//! reaches for at a red check", was the leakier of the two against a waiver, and
//! that is backwards.
//!
//! # What the inventory reports, and what it does not
//!
//! Four states, and each one is a different piece of news. `applied` is a
//! judgment in force. `expired` is a judgment that lapsed. `unused` is a
//! directive that matched no finding, which is either a fixed defect nobody
//! swept up or a directive that never worked. `refused` is a directive this
//! engine could not read.
//!
//! Spec 3 asks for one report beyond the counts: "a shelf that collects
//! exemptions is a shelf whose kind assignment is wrong, and the engine reports
//! that concentration". So the inventory groups by shelf as well as by rule.
//!
//! Spec 4 also says that "a rule with fifty suppressions is not a rule — it is a
//! finding about the taxonomy". No number is written here. The count per rule is
//! the instrument, and the threshold is a judgment that no section states.
//!
//! # Two things that are declared and do not exist
//!
//! Spec 4 fixes a precedence — **waiver, then migration-pending, then
//! suppression** — so that three inventories partition the escaped findings and
//! none is counted twice. Neither of the other two mechanisms exists in this
//! engine, so the precedence has nothing to order and no code states it. When a
//! waiver lands, it lands above this.
//!
//! A withholding finding is not suppressible, because "a suppression is one
//! author's local judgment, and the error that it releases is a disclosure that
//! nobody recalls". No rule here produces one. An export profile is where that
//! class of finding arrives, and the exemption belongs beside it.

use crate::context::Date;
use crate::finding::Finding;
use headwater_census::census::{Census, Outcome as Classification};
use headwater_census::resolve::Step;
use headwater_doc::body::{Block, BlockKind};

/// Why a finding is suppressed. The closed set spec 4 declares, and nothing
/// else reaches this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// The finding is wrong. This is the numerator of the false-positive rate
    /// that spec 4's promotion machinery runs on.
    FalsePositive,
    /// The finding is right, and tolerated for now.
    AcceptedDeviation,
}

impl Reason {
    pub fn name(self) -> &'static str {
        match self {
            Reason::FalsePositive => "false_positive",
            Reason::AcceptedDeviation => "accepted_deviation",
        }
    }

    fn read(text: &str) -> Option<Self> {
        match text {
            "false_positive" => Some(Reason::FalsePositive),
            "accepted_deviation" => Some(Reason::AcceptedDeviation),
            _ => None,
        }
    }
}

/// How much of a document one directive covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extent {
    /// The whole document.
    File,
    /// One block, as a line range that a finding's line falls inside.
    Block { from: usize, to: usize },
}

impl Extent {
    fn covers(self, line: usize) -> bool {
        match self {
            Extent::File => true,
            // A finding with no line is a finding about the document rather
            // than about a place in it, and a block-scoped directive is a
            // statement about a place. So it does not reach one.
            Extent::Block { from, to } => line >= from && line <= to && line > 0,
        }
    }
}

/// What became of one directive over one run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// In force, and it hid at least one finding.
    Applied,
    /// In force, and no finding matched it.
    Unused,
    /// Past its expiry. The findings it names are reported.
    Expired,
}

/// One directive this run read.
#[derive(Clone, Debug)]
pub struct Suppression {
    /// The document that carries the directive, which is also the only
    /// document it can act on.
    pub path: String,
    /// The shelf that document sits on, for the concentration report. `None`
    /// where the census recorded no shelf, which no typed row does.
    pub shelf: Option<String>,
    pub rule: String,
    pub extent: Extent,
    /// The line the directive itself is written on.
    pub line: usize,
    pub until: Date,
    pub reason: Reason,
    /// Free prose, and the one field nothing counts.
    pub note: String,
    /// The findings this directive hid, in the order the runner met them.
    ///
    /// The findings themselves rather than a count of them. Spec 12 asks the
    /// runner to record "exactly what it filtered", and a count is not that:
    /// a reader of the report gets a number either way, and a
    /// [CI adapter](../../adapter/src/lib.rs) that has to show a suppressed
    /// finding as suppressed rather than as absent needs the finding. Empty for
    /// a directive that is unused or expired.
    pub hid: Vec<Finding>,
    pub state: State,
}

/// A directive this engine could not read, and why.
#[derive(Clone, Debug)]
pub struct Refused {
    pub path: String,
    pub line: usize,
    pub why: String,
}

/// Every directive of one run, and what became of each.
#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub suppressions: Vec<Suppression>,
    pub refused: Vec<Refused>,
}

/// The prefix that makes a comment a directive rather than a comment.
const MARKER: &str = "headwater";

/// Every directive the corpus declares, in census order.
///
/// It reads the documents the census already parsed, for the reason every
/// phase after the walk does: a second read of one corpus can disagree with the
/// first, and here the disagreement would be a directive that one pass honored
/// and the other did not.
pub fn declared(census: &Census, rules: &[&'static str]) -> (Vec<Suppression>, Vec<Refused>) {
    let mut found = Vec::new();
    let mut refused = Vec::new();
    for row in &census.rows {
        let Some(document) = &row.document else {
            continue;
        };
        let shelf = match &row.outcome {
            Classification::Typed { derivation, .. } => {
                derivation.steps.iter().find_map(|step| match step {
                    Step::ShelfMatched { shelf, .. } => Some(shelf.clone()),
                    _ => None,
                })
            }
            _ => None,
        };
        for (at, block) in document.body.blocks.iter().enumerate() {
            for text in comments(block) {
                let Some(directive) = directive(text) else {
                    continue;
                };
                let line = block.span.start.line;
                match read(
                    &directive,
                    &row.path,
                    shelf.clone(),
                    line,
                    extent(document.body.blocks.as_slice(), at, block),
                    rules,
                ) {
                    Ok(suppression) => found.push(suppression),
                    Err(why) => refused.push(Refused {
                        path: row.path.clone(),
                        line,
                        why,
                    }),
                }
            }
        }
    }
    (found, refused)
}

/// Apply every directive to the findings of one run.
///
/// The findings that survive come back in the order they arrived, and the
/// inventory holds what each directive did. Nothing here reads a check and
/// nothing here writes a cache: see the module comment for why both matter.
pub fn apply(
    findings: Vec<Finding>,
    mut suppressions: Vec<Suppression>,
    refused: Vec<Refused>,
    now: Date,
) -> (Vec<Finding>, Inventory) {
    for suppression in &mut suppressions {
        // Expiry is decided once, against the injected clock, before any
        // finding is compared. A directive that lapsed is not a directive.
        if suppression.until < now {
            suppression.state = State::Expired;
        }
    }

    let mut kept = Vec::with_capacity(findings.len());
    for finding in findings {
        let hit = suppressions.iter_mut().find(|suppression| {
            suppression.state != State::Expired
                && suppression.rule == finding.rule
                && suppression.path == finding.path
                && suppression.extent.covers(finding.line)
        });
        match hit {
            Some(suppression) => {
                suppression.hid.push(finding);
                suppression.state = State::Applied;
            }
            None => kept.push(finding),
        }
    }

    (
        kept,
        Inventory {
            suppressions,
            refused,
        },
    )
}

impl Inventory {
    fn of(&self, state: State) -> impl Iterator<Item = &Suppression> {
        self.suppressions
            .iter()
            .filter(move |suppression| suppression.state == state)
    }

    /// Findings this run did not report because an author suppressed them.
    pub fn hidden(&self) -> usize {
        self.suppressions
            .iter()
            .map(|suppression| suppression.hid.len())
            .sum()
    }

    /// Every finding this run filtered, with the directive that filtered it.
    ///
    /// In the order the directives were read, which is census order, and within
    /// one directive in the order the runner met the findings. That is the one
    /// order [`crate::finding::sorted`] already put them in, so a consumer that
    /// interleaves these with [`crate::Run::findings`] sorts the union rather
    /// than trusting either sequence.
    pub fn hidden_findings(&self) -> Vec<(&Finding, &Suppression)> {
        self.suppressions
            .iter()
            .flat_map(|suppression| {
                suppression
                    .hid
                    .iter()
                    .map(move |finding| (finding, suppression))
            })
            .collect()
    }

    /// Counts by whatever key `by` reads, largest first and then by name, so
    /// that the rule or the shelf that collects exemptions is the first line a
    /// reader meets.
    fn grouped<'a>(
        &'a self,
        by: impl Fn(&'a Suppression) -> Option<&'a str>,
    ) -> Vec<(&'a str, usize)> {
        let mut groups: Vec<(&str, usize)> = Vec::new();
        for suppression in self.of(State::Applied) {
            let Some(key) = by(suppression) else {
                continue;
            };
            match groups.iter_mut().find(|(known, _)| *known == key) {
                Some((_, count)) => *count += suppression.hid.len(),
                None => groups.push((key, suppression.hid.len())),
            }
        }
        groups.sort_by(|(left, one), (right, two)| two.cmp(one).then(left.cmp(right)));
        groups
    }

    /// Suppressed findings per rule.
    pub fn by_rule(&self) -> Vec<(&str, usize)> {
        self.grouped(|suppression| Some(suppression.rule.as_str()))
    }

    /// Suppressed findings per shelf, which is the concentration spec 3 asks
    /// for: a shelf that collects exemptions is a shelf whose kind assignment
    /// is wrong.
    pub fn by_shelf(&self) -> Vec<(&str, usize)> {
        self.grouped(|suppression| suppression.shelf.as_deref())
    }

    /// Suppressed findings per reason, in the order spec 4 lists them, and
    /// only for a reason that has one. This is the count that the promotion
    /// machinery reads.
    pub fn by_reason(&self) -> Vec<(Reason, usize)> {
        [Reason::FalsePositive, Reason::AcceptedDeviation]
            .into_iter()
            .map(|reason| {
                (
                    reason,
                    self.of(State::Applied)
                        .filter(|suppression| suppression.reason == reason)
                        .map(|suppression| suppression.hid.len())
                        .sum(),
                )
            })
            .filter(|(_, count)| *count > 0)
            .collect()
    }

    /// Whether this run has anything to say about suppression at all.
    pub fn is_empty(&self) -> bool {
        self.suppressions.is_empty() && self.refused.is_empty()
    }

    /// The inventory as text, for the coverage report.
    ///
    /// Counts only. A directive's note is prose that an author wrote for
    /// another author, and it belongs where it is written.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        if self.is_empty() {
            return out;
        }
        out.push_str("suppressions\n");
        let _ = writeln!(
            out,
            "  {} findings hidden by {} directives",
            self.hidden(),
            self.of(State::Applied).count()
        );
        for (reason, count) in self.by_reason() {
            let _ = writeln!(out, "  {count:5} {}", reason.name());
        }
        for (rule, count) in self.by_rule() {
            let _ = writeln!(out, "  {count:5} of {rule}");
        }
        // The concentration. One shelf is no concentration, and the line is
        // still printed: a reader who has to ask whether the report ran cannot
        // read a silence as an answer.
        for (shelf, count) in self.by_shelf() {
            let _ = writeln!(out, "  {count:5} on the shelf {shelf}");
        }
        for state in [State::Expired, State::Unused] {
            let count = self.of(state).count();
            if count > 0 {
                let _ = match state {
                    State::Expired => writeln!(
                        out,
                        "  {count:5} expired, and the findings they named are reported"
                    ),
                    State::Unused => writeln!(out, "  {count:5} matched no finding"),
                    State::Applied => Ok(()),
                };
            }
        }
        for refused in &self.refused {
            let _ = writeln!(
                out,
                "  {}:{} suppresses nothing: {}",
                refused.path, refused.line, refused.why
            );
        }
        out
    }
}

/// The comments one block carries, whether it is a comment of its own or one
/// written inside a paragraph.
///
/// Raw HTML reaches the parse either as its own block or as a run of
/// [`headwater_doc::body::Ownership::Code`] inside the block it interrupts, and
/// a directive is legitimate in both places. See
/// [`headwater_doc::body`] for why the parser keeps it at all.
fn comments(block: &Block) -> impl Iterator<Item = &str> {
    block
        .runs
        .iter()
        .map(|run| run.text.trim())
        // Filtered before anything is owned. This runs over every run of text
        // in the corpus, and a `String` per run is the whole cost of a scan
        // that ends up reading two comments.
        .filter(|text| text.starts_with("<!--") && text.ends_with("-->"))
}

/// The inside of a comment, when it is one of ours.
fn directive(comment: &str) -> Option<String> {
    let inside = comment
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim()
        .to_string();
    let rest = inside.strip_prefix(MARKER)?;
    // `headwater` and `headwaters` are different words, and a comment that
    // begins with the second is not a directive that lost a field.
    match rest.starts_with([' ', ':']) {
        true => Some(rest.trim_start_matches([' ', ':']).to_string()),
        false => None,
    }
}

/// What a block-scoped directive covers.
///
/// A comment inside a block covers that block. A comment that is a block of its
/// own covers the block after it, which is the only reading that makes a
/// standalone hatch above a paragraph mean anything.
fn extent(blocks: &[Block], at: usize, block: &Block) -> Extent {
    let alone = matches!(block.kind, BlockKind::Html);
    let covered = match alone {
        true => blocks.get(at + 1).unwrap_or(block),
        false => block,
    };
    Extent::Block {
        from: covered.span.start.line,
        to: covered.span.end.line,
    }
}

/// One directive, read. `Err` carries the sentence the inventory prints.
fn read(
    directive: &str,
    path: &str,
    shelf: Option<String>,
    line: usize,
    block: Extent,
    rules: &[&'static str],
) -> Result<Suppression, String> {
    let mut allow: Option<String> = None;
    let mut scope: Option<String> = None;
    let mut until: Option<String> = None;
    let mut reason: Option<String> = None;
    let mut note = String::new();

    let mut rest = directive;
    loop {
        let current = rest.trim_start();
        if current.is_empty() {
            break;
        }
        // One token, then the key inside it. Splitting the token off first is
        // what makes `allow=x file until=y` report the bare word rather than a
        // field name with a space in it.
        let (token, tail) = match current.split_once(char::is_whitespace) {
            Some((token, tail)) => (token, tail),
            None => (current, ""),
        };
        let Some((key, value)) = token.split_once('=') else {
            return Err(format!("`{token}` is not `key=value`"));
        };
        // The note runs to the end, because it is prose and prose holds
        // spaces. Every other value is one word.
        if key == "note" {
            note = current[key.len() + 1..].trim().to_string();
            break;
        }
        let slot = match key {
            "allow" => &mut allow,
            "scope" => &mut scope,
            "until" => &mut until,
            "reason" => &mut reason,
            other => return Err(format!("`{other}` is not a field of a suppression")),
        };
        if slot.is_some() {
            return Err(format!("`{key}` is written twice"));
        }
        *slot = Some(value.to_string());
        rest = tail;
    }

    let allow = allow.ok_or("it names no rule, so `allow=` is missing")?;
    if !rules.contains(&allow.as_str()) {
        return Err(format!("no rule of this engine is named `{allow}`"));
    }
    let extent = match scope.as_deref() {
        Some("file") => Extent::File,
        Some("block") => block,
        Some(other) => return Err(format!("`scope={other}` is neither `file` nor `block`")),
        None => return Err("it states no scope, so `scope=` is missing".to_string()),
    };
    let until = until.ok_or("it states no expiry, so `until=` is missing")?;
    let until = Date::parse(&until)
        .ok_or_else(|| format!("`until={until}` is not a date written `YYYY-MM-DD`"))?;
    let reason = reason.ok_or("it states no reason, so `reason=` is missing")?;
    let reason = Reason::read(&reason).ok_or_else(|| {
        format!("`reason={reason}` is neither `false_positive` nor `accepted_deviation`")
    })?;

    Ok(Suppression {
        path: path.to_string(),
        shelf,
        rule: allow,
        extent,
        line,
        until,
        reason,
        note,
        hid: Vec::new(),
        state: State::Unused,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Severity;

    /// The rules a directive may name, which is the engine's own list rather
    /// than a copy of two of it. A test that carried its own names would pass
    /// after a rule was renamed and the refusal message went stale.
    const RULES: [&str; crate::RULES.len()] = crate::RULES;

    fn day(text: &str) -> Date {
        Date::parse(text).expect("a date")
    }

    fn one(text: &str) -> Result<Suppression, String> {
        read(
            &directive(text).expect("a directive"),
            "a.md",
            Some("specification".to_string()),
            3,
            Extent::Block { from: 5, to: 7 },
            &RULES,
        )
    }

    fn finding(rule: &'static str, line: usize) -> Finding {
        Finding {
            rule,
            severity: Severity::Warn,
            obligation: None,
            path: "a.md".to_string(),
            line,
            column: 1,
            message: "a message".to_string(),
            remediation: "do the thing".to_string(),
            patch: None,
        }
    }

    #[test]
    fn a_directive_reads_its_four_fields_in_any_order() {
        let read = one(
            "<!-- headwater reason=false_positive allow=language.controlled.not_met \
             until=2027-01-01 scope=file note=it is a list of citations -->",
        )
        .expect("it reads");
        assert_eq!(read.rule, "language.controlled.not_met");
        assert_eq!(read.extent, Extent::File);
        assert_eq!(read.until, day("2027-01-01"));
        assert_eq!(read.reason, Reason::FalsePositive);
        assert_eq!(read.note, "it is a list of citations");
    }

    /// A comment that is not ours is not a directive, and a word that merely
    /// starts with the marker is not either.
    #[test]
    fn only_our_comments_are_directives() {
        assert!(directive("<!-- a note to a reader -->").is_none());
        assert!(directive("<!-- headwaters allow=x -->").is_none());
        assert!(directive("<!-- ste-lint: allow sentence-length -->").is_none());
        assert!(directive("<!-- headwater: allow=x -->").is_some());
    }

    /// Every way a directive can fail to read, and each one suppresses nothing.
    ///
    /// The typo case is the one this list exists for. An author who mistypes a
    /// rule name and believes the finding is handled is worse off than one who
    /// wrote nothing, so the engine says so rather than ignoring the line.
    #[test]
    fn a_directive_this_engine_cannot_read_suppresses_nothing() {
        for (text, expected) in [
            (
                "<!-- headwater allow=language.controled.not_met scope=file until=2027-01-01 \
                 reason=false_positive -->",
                "no rule of this engine is named `language.controled.not_met`",
            ),
            (
                "<!-- headwater scope=file until=2027-01-01 reason=false_positive -->",
                "it names no rule, so `allow=` is missing",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction until=2027-01-01 \
                 reason=false_positive -->",
                "it states no scope, so `scope=` is missing",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction scope=paragraph \
                 until=2027-01-01 reason=false_positive -->",
                "`scope=paragraph` is neither `file` nor `block`",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction scope=file \
                 reason=false_positive -->",
                "it states no expiry, so `until=` is missing",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction scope=file until=soon \
                 reason=false_positive -->",
                "`until=soon` is not a date written `YYYY-MM-DD`",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction scope=file until=2027-01-01 -->",
                "it states no reason, so `reason=` is missing",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction scope=file until=2027-01-01 \
                 reason=it-is-fine -->",
                "`reason=it-is-fine` is neither `false_positive` nor `accepted_deviation`",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction scope=file until=2027-01-01 \
                 reason=false_positive expires=never -->",
                "`expires` is not a field of a suppression",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction scope=file scope=block \
                 until=2027-01-01 reason=false_positive -->",
                "`scope` is written twice",
            ),
            (
                "<!-- headwater allow=voice.forbidden_construction file until=2027-01-01 -->",
                "`file` is not `key=value`",
            ),
        ] {
            assert_eq!(one(text).expect_err("it is refused"), expected);
        }
    }

    /// A file-scoped directive reaches every finding of its rule in its file,
    /// and no finding of another rule or another file.
    #[test]
    fn a_file_scoped_directive_covers_the_document_and_nothing_else() {
        let mut elsewhere = finding("language.controlled.not_met", 40);
        elsewhere.path = "b.md".to_string();
        let (kept, inventory) = apply(
            vec![
                finding("language.controlled.not_met", 6),
                finding("language.controlled.not_met", 90),
                finding("voice.forbidden_construction", 6),
                elsewhere,
            ],
            vec![one(
                "<!-- headwater allow=language.controlled.not_met scope=file until=2027-01-01 \
                 reason=accepted_deviation -->",
            )
            .expect("it reads")],
            Vec::new(),
            day("2026-08-12"),
        );
        assert_eq!(kept.len(), 2);
        assert_eq!(inventory.hidden(), 2);
        assert_eq!(inventory.by_reason(), vec![(Reason::AcceptedDeviation, 2)]);
        assert_eq!(inventory.by_shelf(), vec![("specification", 2)]);
    }

    /// A block-scoped directive reaches the lines of its block and stops.
    #[test]
    fn a_block_scoped_directive_covers_its_block_alone() {
        let (kept, inventory) = apply(
            vec![
                finding("language.controlled.not_met", 5),
                finding("language.controlled.not_met", 7),
                finding("language.controlled.not_met", 8),
            ],
            vec![one(
                "<!-- headwater allow=language.controlled.not_met scope=block until=2027-01-01 \
                 reason=false_positive -->",
            )
            .expect("it reads")],
            Vec::new(),
            day("2026-08-12"),
        );
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].line, 8);
        assert_eq!(inventory.hidden(), 2);
    }

    /// An expired directive suppresses nothing, and the inventory says which
    /// one lapsed rather than dropping it.
    #[test]
    fn an_expired_directive_returns_its_findings() {
        let (kept, inventory) = apply(
            vec![finding("language.controlled.not_met", 6)],
            vec![one(
                "<!-- headwater allow=language.controlled.not_met scope=file until=2026-08-11 \
                 reason=false_positive -->",
            )
            .expect("it reads")],
            Vec::new(),
            day("2026-08-12"),
        );
        assert_eq!(kept.len(), 1);
        assert_eq!(inventory.hidden(), 0);
        assert_eq!(inventory.of(State::Expired).count(), 1);
        assert!(inventory.render().contains("1 expired"));
    }

    /// A directive expires the day after its date, so `until` is the last day
    /// it holds rather than the first day it does not.
    #[test]
    fn a_directive_holds_on_the_day_it_names() {
        let text = "<!-- headwater allow=language.controlled.not_met scope=file until=2026-08-12 \
                    reason=false_positive -->";
        let (kept, _) = apply(
            vec![finding("language.controlled.not_met", 6)],
            vec![one(text).expect("it reads")],
            Vec::new(),
            day("2026-08-12"),
        );
        assert!(kept.is_empty());
    }

    /// A directive that matched nothing is reported rather than forgotten.
    #[test]
    fn a_directive_that_hid_nothing_is_reported_as_unused() {
        let (_, inventory) = apply(
            Vec::new(),
            vec![one(
                "<!-- headwater allow=voice.forbidden_construction scope=file until=2027-01-01 \
                 reason=false_positive -->",
            )
            .expect("it reads")],
            Vec::new(),
            day("2026-08-12"),
        );
        assert_eq!(inventory.of(State::Unused).count(), 1);
        assert!(inventory.render().contains("1 matched no finding"));
    }
}
