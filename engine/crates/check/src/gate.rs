// SPDX-License-Identifier: Apache-2.0
//! The gate: a published read set, held against a later tree.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
//! rules what a gate reads, and this module is that ruling as code. The ruling
//! is one test and it decides three things.
//!
//! **The test is a comparison over listed inputs.** A gate opens the artifact
//! that [`crate::ReadSet`] wrote, and for each `input` line it hashes the file
//! at that path in the tree in front of it. A hash that moved voids the
//! verdict. Nothing else is read. Spec 12 asked for a diff of two trees at one
//! time, and
//! [HW-OBL-0028](../../../../docs/obligations/0028-a-run-cannot-report-the-corpus-tree-because-nothing-computes.md)
//! holds why that test cannot ship: a diff of two trees needs a tree, and no
//! run computes one. This test needs neither a tree nor a run.
//!
//! **The clock voids a verdict.** A verdict is about one state of the corpus
//! and one day. The rules that read the injected clock are on `windowed` lines,
//! and a gate asked about a later day than the run's voids the verdict rather
//! than carry a windowed expectation across it.
//!
//! **Membership of the census is an input, and no list of members carries it.**
//! This is the decision that limits the whole test, so this module states it
//! twice: once in [`Verdict::render`], on every run, and once here.
//!
//! A read set is a list of `(path, digest)` pairs, and a pair is a member. The
//! list therefore records what the run read and never records that those were
//! all there was. Two consequences follow, and they are different sizes.
//!
//! The small one is that a verdict about a listed document is exactly as good
//! as it looks: the bytes are there to compare, so the gate decides it. The
//! large one is that a document the tree gained is on no list, so it moves no
//! hash. Such a document generates instances that this run never held, and the
//! gate cannot see one of them. So the gate reports the reach of its own
//! answer, and it never reports that a corpus is green.
//!
//! A **barrier** is where that limit consumes the answer entirely. A
//! corpus-grained instance decides its verdict from the extent of the census
//! rather than from the contents of any member: `identifier.claimed_twice`
//! fires on the *presence of a second claimant*, so its verdict rests on the
//! absence of a document. A comparison over listed inputs reports that such a
//! verdict survives a merge that added the second claimant, and it is wrong
//! every time. So a barrier never carries, whatever the listed hashes did.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
//! fixes which way a doubtful case falls: "a false invalidation costs one run.
//! A false survival ships an invalid corpus with a green report." Every refusal
//! here is that asymmetry read once.
//!
//! # A taxonomy-grained instance is no barrier, and the lock is why
//!
//! [`crate::Grain::Taxonomy`] reads the lock and no document. The artifact
//! carries the lock digest, and this module compares it, so a taxonomy-grained
//! verdict is decided by an ordinary comparison. The barrier is the corpus
//! grain alone, because the extent of the census is the one input the artifact
//! cannot state.

use crate::context::Date;
use crate::instance::Input;
use headwater_yaml::json::Json;

/// A read set as a gate reads it back.
///
/// [`crate::ReadSet`] is what a run produces, and its rule names are the
/// `'static` names of the registry. A parsed artifact holds whatever the file
/// says, so the two types are not one type. The round trip is a test rather
/// than a hope: [`Recorded::parse`] is fed [`crate::ReadSet::render`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    pub lock: String,
    pub clock: Date,
    pub barriers: Vec<String>,
    /// The rules whose verdict rests on the change a run was scoped to. See
    /// [`crate::readset::ReadSet::scoped`]: a change is not a corpus path, so
    /// no line here names one and no comparison over listed hashes reaches it.
    pub scoped: Vec<String>,
    pub windowed: Vec<String>,
    pub versions: Vec<(String, u32)>,
    pub inputs: Vec<Input>,
}

/// A line this module cannot read, named by its number.
///
/// A gate that guessed at a malformed artifact would decide a verdict from a
/// file it did not understand, which is the failure the whole module exists to
/// refuse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Malformed {
    pub line: usize,
    pub reason: String,
}

impl Malformed {
    pub fn render(&self) -> String {
        format!("line {}: {}", self.line, self.reason)
    }
}

impl Recorded {
    /// Read the artifact back.
    ///
    /// Every keyword is required to be one this module knows. An artifact
    /// written by a later engine with a keyword this one has never seen is
    /// refused rather than half-read, because the keyword a gate skipped is
    /// exactly where a later engine would put the next reason not to decide.
    pub fn parse(text: &str) -> Result<Self, Malformed> {
        let mut lock: Option<String> = None;
        let mut clock: Option<Date> = None;
        let mut barriers = Vec::new();
        let mut scoped = Vec::new();
        let mut windowed = Vec::new();
        let mut versions = Vec::new();
        let mut inputs = Vec::new();

        for (offset, line) in text.lines().enumerate() {
            let at = offset + 1;
            let refuse = |reason: &str| Malformed {
                line: at,
                reason: reason.to_string(),
            };
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split(' ').collect();
            match fields.as_slice() {
                ["lock", digest] => lock = Some((*digest).to_string()),
                ["clock", day] => match Date::parse(day) {
                    Some(day) => clock = Some(day),
                    None => return Err(refuse("the clock is not a date of the form YYYY-MM-DD")),
                },
                ["barrier", rule] => barriers.push((*rule).to_string()),
                ["change-scoped", rule] => scoped.push((*rule).to_string()),
                ["windowed", rule] => windowed.push((*rule).to_string()),
                ["version", rule, edition] => match edition.parse::<u32>() {
                    Ok(edition) => versions.push(((*rule).to_string(), edition)),
                    Err(_) => return Err(refuse("the edition of a rule is not a whole number")),
                },
                // `-` is the one entry that carries no hash, and the writer
                // spells it so that a reader cannot mistake it for one.
                ["input", path, "-"] => inputs.push(Input::new(*path, None)),
                ["input", path, digest] => inputs.push(Input::new(*path, Some(digest))),
                _ => return Err(refuse("no keyword of a read set opens this line")),
            }
        }

        match (lock, clock) {
            (Some(lock), Some(clock)) => Ok(Recorded {
                lock,
                clock,
                barriers,
                scoped,
                windowed,
                versions,
                inputs,
            }),
            (None, _) => Err(Malformed {
                line: 0,
                reason: "the artifact states no lock, so it describes no taxonomy".to_string(),
            }),
            (_, None) => Err(Malformed {
                line: 0,
                reason: "the artifact states no clock, so it describes no day".to_string(),
            }),
        }
    }
}

/// One reason a verdict does not carry. Closed, and matched exhaustively.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    /// The taxonomy moved, so every result rests on a lock that is not there.
    LockMoved { recorded: String, found: String },
    /// A corpus-grained rule ran. See the module comment.
    Barrier { rule: String },
    /// A rule read the version a document stood at before a change, and this
    /// artifact names no change.
    ChangeScoped { rule: String },
    /// A rule read the clock, and the day the gate was asked about is not the
    /// day the run evaluated.
    DayMoved {
        rule: String,
        recorded: Date,
        asked: Date,
    },
    /// A listed input holds other bytes now.
    Moved {
        path: String,
        recorded: String,
        found: String,
    },
    /// A listed input is not in the tree at all.
    Gone { path: String },
    /// A listed input carried no hash when it was read, so there is nothing to
    /// compare it against.
    Unhashed { path: String },
}

impl Reason {
    /// The name a caller branches on.
    ///
    /// Deliberately not the first words of [`Reason::render`]. That is a
    /// sentence written for a person and it is rewritten whenever the wording
    /// improves; this is a token, and a consumer of `headwater gate --json`
    /// that switched on it keeps working across such a rewrite.
    pub fn token(&self) -> &'static str {
        match self {
            Reason::LockMoved { .. } => "lock_moved",
            Reason::Barrier { .. } => "barrier",
            Reason::ChangeScoped { .. } => "change_scoped",
            Reason::DayMoved { .. } => "day_moved",
            Reason::Moved { .. } => "moved",
            Reason::Gone { .. } => "gone",
            Reason::Unhashed { .. } => "unhashed",
        }
    }

    /// One reason as JSON: the token, the sentence, and the values that
    /// sentence was composed from.
    ///
    /// The values are members rather than only prose, because the whole point
    /// of the machine form is that a caller does not have to take a digest back
    /// out of a sentence. `says` is [`Reason::render`] rather than a second
    /// wording of it.
    fn json(&self) -> Json {
        let mut members: Vec<(&'static str, Json)> = vec![
            ("reason", Json::string(self.token())),
            ("says", Json::string(self.render())),
        ];
        let text = |value: &String| Json::string(value.clone());
        match self {
            Reason::LockMoved { recorded, found } => {
                members.extend([("recorded", text(recorded)), ("found", text(found))]);
            }
            Reason::Barrier { rule } | Reason::ChangeScoped { rule } => {
                members.push(("rule", text(rule)));
            }
            Reason::DayMoved {
                rule,
                recorded,
                asked,
            } => members.extend([
                ("rule", text(rule)),
                ("recorded", Json::string(recorded.render())),
                ("asked", Json::string(asked.render())),
            ]),
            Reason::Moved {
                path,
                recorded,
                found,
            } => members.extend([
                ("path", text(path)),
                ("recorded", text(recorded)),
                ("found", text(found)),
            ]),
            Reason::Gone { path } | Reason::Unhashed { path } => members.push(("path", text(path))),
        }
        Json::object(members)
    }

    pub fn render(&self) -> String {
        match self {
            Reason::LockMoved { recorded, found } => {
                format!("the taxonomy lock reads {found} and this verdict rests on {recorded}")
            }
            Reason::Barrier { rule } => format!(
                "{rule} is a barrier: its verdict is a predicate over the extent of the census, \
                 and a list of members states no extent"
            ),
            Reason::ChangeScoped { rule } => format!(
                "{rule} read the version each document stood at before a change, and a read set \
                 lists corpus paths rather than changes"
            ),
            Reason::DayMoved {
                rule,
                recorded,
                asked,
            } => format!(
                "{rule} read the clock on {}, and this gate is asked about {}",
                recorded.render(),
                asked.render()
            ),
            Reason::Moved {
                path,
                recorded,
                found,
            } => format!("{path} reads {found} and this verdict rests on {recorded}"),
            Reason::Gone { path } => format!("{path} is not in this tree"),
            Reason::Unhashed { path } => {
                format!("{path} carried no hash when it was read, so nothing compares")
            }
        }
    }
}

/// What a gate decided, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// Empty exactly when the verdict carries.
    pub reasons: Vec<Reason>,
    /// How many inputs the artifact listed. It is the reach of the answer, and
    /// [`Verdict::render`] prints it whichever way the answer went.
    pub listed: usize,
}

impl Verdict {
    pub fn carries(&self) -> bool {
        self.reasons.is_empty()
    }

    /// The report.
    ///
    /// The last line is printed on a carrying verdict as well as on a voided
    /// one, and it is the sentence that keeps this artifact honest. A gate that
    /// printed "the verdict carries" alone would be read as "the corpus is
    /// green", and a read set cannot say that about a corpus that gained a
    /// document.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let listed = self.listed;
        match self.carries() {
            true => {
                let _ = writeln!(
                    out,
                    "the verdicts this run reached about the {listed} inputs it listed carry to \
                     this tree"
                );
            }
            false => {
                let _ = writeln!(
                    out,
                    "the verdicts this run reached do not carry to this tree"
                );
                for reason in &self.reasons {
                    let _ = writeln!(out, "  {}", reason.render());
                }
            }
        }
        let _ = writeln!(out, "{LIMIT}");
        out
    }

    /// The verdict as JSON, in the shape `headwater gate --json` writes.
    ///
    /// # Why `limit` is a member whose value never varies
    ///
    /// [`Verdict::render`] prints [`LIMIT`] on a carrying verdict as well as on
    /// a voided one, and the doc comment above says why: a gate that printed
    /// "the verdict carries" alone would be read as "the corpus is green". A
    /// machine reader makes that mistake more easily than a person does, not
    /// less, because `"carries": true` is exactly the shape a caller wants to
    /// branch on. So the sentence is here, from the same constant the report
    /// prints, and a consumer that renders this document to a person renders
    /// the caveat with it.
    ///
    /// The destructuring below is exhaustive on purpose: a field added to
    /// [`Verdict`] and not to this document does not compile.
    pub fn render_json(&self) -> String {
        let Verdict { reasons, listed } = self;
        Json::object([
            ("version", Json::string(VERSION)),
            ("carries", Json::Bool(self.carries())),
            ("listed", Json::Raw(listed.to_string())),
            (
                "reasons",
                Json::Array(reasons.iter().map(Reason::json).collect()),
            ),
            ("limit", Json::string(LIMIT)),
        ])
        .render_pretty()
    }
}

/// The version of the document [`Verdict::json`] writes.
///
/// The document's own shape and never the engine's, which is the rule the
/// finding shape states for the same reason: a consumer outside this repository
/// holds no clone of the engine, so the bytes have to name what they are.
pub const VERSION: &str = "1.0";

/// The sentence that keeps this artifact honest, printed on every verdict.
///
/// One constant with two readers — [`Verdict::render`] and [`Verdict::json`] —
/// rather than two literals that agree until somebody edits one of them.
pub const LIMIT: &str = "this states nothing about a document this tree gained. A read set lists \
                         what a run read, and never that those were all there was";

/// Hold a published read set against the tree in front of it.
///
/// `digest_of` answers for one path in that tree, and `None` is a file that is
/// not there. The tree arrives as a function rather than as a path so that this
/// decision is testable without one, which is the same reason the clock is
/// injected into a run.
///
/// The order of the reasons is fixed: the lock, then the barriers, then the
/// clock, then the inputs in the order the artifact listed them. Two runs of
/// one gate over one tree print one report.
pub fn decide(
    recorded: &Recorded,
    lock: &str,
    asked: Date,
    digest_of: impl Fn(&str) -> Option<String>,
) -> Verdict {
    let mut reasons = Vec::new();

    if recorded.lock != lock {
        reasons.push(Reason::LockMoved {
            recorded: recorded.lock.clone(),
            found: lock.to_string(),
        });
    }

    for rule in &recorded.barriers {
        reasons.push(Reason::Barrier { rule: rule.clone() });
    }

    // Unconditional, as a barrier is, and for a reason of the same shape. A day
    // moved or it did not, and a gate is told which. A change is named nowhere
    // in this artifact, so there is no comparison to make and the verdict of
    // such a rule never carries.
    for rule in &recorded.scoped {
        reasons.push(Reason::ChangeScoped { rule: rule.clone() });
    }

    if asked != recorded.clock {
        for rule in &recorded.windowed {
            reasons.push(Reason::DayMoved {
                rule: rule.clone(),
                recorded: recorded.clock,
                asked,
            });
        }
    }

    for input in &recorded.inputs {
        match (&input.digest, digest_of(&input.path)) {
            (None, _) => reasons.push(Reason::Unhashed {
                path: input.path.clone(),
            }),
            (Some(_), None) => reasons.push(Reason::Gone {
                path: input.path.clone(),
            }),
            (Some(recorded), Some(found)) if *recorded != found => reasons.push(Reason::Moved {
                path: input.path.clone(),
                recorded: recorded.clone(),
                found,
            }),
            (Some(_), Some(_)) => {}
        }
    }

    Verdict {
        reasons,
        listed: recorded.inputs.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::{Instance, Outcome};
    use crate::readset::{ReadSet, Rule};
    use crate::scope::Grain;

    fn day(text: &str) -> Date {
        Date::parse(text).expect("a date")
    }

    fn recorded(text: &str) -> Recorded {
        Recorded::parse(text).expect("an artifact this module wrote")
    }

    /// One document, one hash, no barrier and no windowed rule.
    fn plain() -> Recorded {
        recorded("lock sha256:lock\nclock 2026-08-13\nversion a.rule 1\ninput docs/a.md sha256:a\n")
    }

    fn tree<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |path: &str| {
            pairs
                .iter()
                .find(|(known, _)| *known == path)
                .map(|(_, digest)| (*digest).to_string())
        }
    }

    /// The writer and the reader agree, and this is the test that says so.
    #[test]
    fn what_a_run_renders_is_what_a_gate_parses() {
        let set = ReadSet::of(
            "sha256:lock",
            day("2026-08-13"),
            &[
                Rule {
                    name: "wide.rule",
                    version: 2,
                    needs_clock: false,
                    needs_prior: false,
                },
                Rule {
                    name: "dated.rule",
                    version: 1,
                    needs_clock: true,
                    needs_prior: false,
                },
            ],
            &[
                Instance::of(
                    "wide.rule",
                    Grain::Corpus,
                    vec![Input::new("docs/a.md", Some("sha256:a"))],
                    Outcome::Passed,
                ),
                Instance::of(
                    "dated.rule",
                    Grain::Document,
                    vec![Input::new("docs/b.md", None)],
                    Outcome::Passed,
                ),
            ],
        );
        let read = Recorded::parse(&set.render()).expect("the artifact this run wrote");
        assert_eq!(read.lock, "sha256:lock");
        assert_eq!(read.clock, day("2026-08-13"));
        assert_eq!(read.barriers, vec!["wide.rule".to_string()]);
        assert_eq!(read.windowed, vec!["dated.rule".to_string()]);
        assert_eq!(
            read.versions,
            vec![("wide.rule".to_string(), 2), ("dated.rule".to_string(), 1)]
        );
        assert_eq!(
            read.inputs,
            vec![
                Input::new("docs/a.md", Some("sha256:a")),
                Input::new("docs/b.md", None),
            ]
        );
    }

    /// Every listed hash stands, so the verdict carries — and the report still
    /// states what it does not cover.
    #[test]
    fn an_unchanged_tree_carries_the_verdict() {
        let verdict = decide(
            &plain(),
            "sha256:lock",
            day("2026-08-13"),
            tree(&[("docs/a.md", "sha256:a")]),
        );
        assert!(verdict.carries());
        assert!(verdict.render().contains("carry to this tree"));
        assert!(verdict
            .render()
            .contains("nothing about a document this tree gained"));
    }

    /// A listed input that holds other bytes voids the verdict, and the report
    /// names the file.
    #[test]
    fn a_moved_hash_voids_the_verdict() {
        let verdict = decide(
            &plain(),
            "sha256:lock",
            day("2026-08-13"),
            tree(&[("docs/a.md", "sha256:other")]),
        );
        assert!(!verdict.carries());
        assert_eq!(
            verdict.reasons,
            vec![Reason::Moved {
                path: "docs/a.md".to_string(),
                recorded: "sha256:a".to_string(),
                found: "sha256:other".to_string(),
            }]
        );
    }

    /// A listed input that is not in the tree voids the verdict too, and it is
    /// a different reason from a hash that moved.
    #[test]
    fn a_deleted_input_voids_the_verdict() {
        let verdict = decide(&plain(), "sha256:lock", day("2026-08-13"), tree(&[]));
        assert_eq!(
            verdict.reasons,
            vec![Reason::Gone {
                path: "docs/a.md".to_string()
            }]
        );
    }

    /// A lock that moved voids every result at once, because the lock digest is
    /// a component of every cache key.
    #[test]
    fn a_moved_lock_voids_the_verdict() {
        let verdict = decide(
            &plain(),
            "sha256:other",
            day("2026-08-13"),
            tree(&[("docs/a.md", "sha256:a")]),
        );
        assert!(!verdict.carries());
        assert!(matches!(verdict.reasons[0], Reason::LockMoved { .. }));
    }

    /// This is the construction the issue names. The artifact lists one
    /// document, that document is untouched, and the corpus-grained verdict
    /// still does not carry — because a merge that added a second claimant
    /// moves no listed hash.
    #[test]
    fn a_barrier_never_carries_even_when_every_listed_hash_stands() {
        let held = recorded(
            "lock sha256:lock\nclock 2026-08-13\nbarrier identifier.claimed_twice\n\
             version identifier.claimed_twice 1\ninput docs/a.md sha256:a\n",
        );
        let verdict = decide(
            &held,
            "sha256:lock",
            day("2026-08-13"),
            tree(&[("docs/a.md", "sha256:a")]),
        );
        assert!(!verdict.carries());
        assert_eq!(
            verdict.reasons,
            vec![Reason::Barrier {
                rule: "identifier.claimed_twice".to_string()
            }]
        );
        assert!(verdict
            .render()
            .contains("a list of members states no extent"));
    }

    /// The clock is the second thing that voids a verdict with no tree change
    /// at all, and it reaches the rules that read it.
    #[test]
    fn a_later_day_voids_a_windowed_rule() {
        let held = recorded(
            "lock sha256:lock\nclock 2026-08-13\nwindowed relation.participation.overdue\n\
             version relation.participation.overdue 1\ninput docs/a.md sha256:a\n",
        );
        let standing = tree(&[("docs/a.md", "sha256:a")]);
        assert!(decide(&held, "sha256:lock", day("2026-08-13"), &standing).carries());
        let later = decide(&held, "sha256:lock", day("2026-09-13"), &standing);
        assert_eq!(
            later.reasons,
            vec![Reason::DayMoved {
                rule: "relation.participation.overdue".to_string(),
                recorded: day("2026-08-13"),
                asked: day("2026-09-13"),
            }]
        );
    }

    /// A run with no windowed rule is about a tree alone, so the day moving
    /// decides nothing.
    #[test]
    fn a_later_day_carries_a_run_that_read_no_clock() {
        let verdict = decide(
            &plain(),
            "sha256:lock",
            day("2027-01-01"),
            tree(&[("docs/a.md", "sha256:a")]),
        );
        assert!(verdict.carries());
    }

    /// An input the walk never hashed stops the gate deciding, which is what
    /// the read set publishes it for.
    #[test]
    fn an_input_with_no_hash_stops_the_gate() {
        let held = recorded("lock sha256:lock\nclock 2026-08-13\ninput docs/a.md -\n");
        let verdict = decide(
            &held,
            "sha256:lock",
            day("2026-08-13"),
            tree(&[("docs/a.md", "sha256:a")]),
        );
        assert_eq!(
            verdict.reasons,
            vec![Reason::Unhashed {
                path: "docs/a.md".to_string()
            }]
        );
    }

    /// A keyword this module does not know is refused rather than skipped.
    #[test]
    fn an_unknown_keyword_is_refused() {
        let refusal = Recorded::parse("lock sha256:l\nclock 2026-08-13\nrumour docs/a.md\n")
            .expect_err("a line no keyword opens");
        assert_eq!(refusal.line, 3);
        assert!(refusal.render().contains("no keyword of a read set"));
    }

    /// An artifact with no lock describes no taxonomy, so no verdict can rest
    /// on it.
    #[test]
    fn an_artifact_with_no_lock_is_refused() {
        let refusal = Recorded::parse("clock 2026-08-13\n").expect_err("no lock");
        assert!(refusal.render().contains("states no lock"));
    }

    /// A change-scoped verdict never carries, whatever the listed hashes did.
    ///
    /// The barrier's rule, one input further out. A barrier rests on the extent
    /// of the census and a list of members states no extent. This rests on the
    /// change a run was scoped to, and a list of corpus paths names no change.
    /// The tree below is the tree the run read, hash for hash, which is what
    /// makes the refusal a property of the input rather than of the bytes.
    #[test]
    fn a_change_scoped_verdict_never_carries_even_when_every_listed_hash_stands() {
        let held = recorded(
            "lock sha256:lock\nclock 2026-08-13\nchange-scoped warrant.promoted\n\
             version warrant.promoted 1\ninput docs/a.md sha256:a\n",
        );
        assert_eq!(held.scoped, vec!["warrant.promoted".to_string()]);

        let verdict = decide(
            &held,
            "sha256:lock",
            day("2026-08-13"),
            tree(&[("docs/a.md", "sha256:a")]),
        );
        assert!(!verdict.carries(), "{:?}", verdict.reasons);
        assert!(matches!(
            verdict.reasons.first(),
            Some(Reason::ChangeScoped { .. })
        ));
        assert!(verdict
            .render()
            .contains("lists corpus paths rather than changes"));
    }
}
