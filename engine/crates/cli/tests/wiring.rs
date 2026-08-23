// SPDX-License-Identifier: Apache-2.0
//! What the binary decides, at the grain where nothing else can see it.
//!
//! # The defect this target exists for
//!
//! Every crate below this one is tested against inputs its own tests build. The
//! wiring is where a caller decides *which* library value to read, and a test
//! that constructs the value cannot catch a caller that reads the one beside
//! it. Three defects of exactly that shape have been found in `main.rs`, all
//! three by hand and none by the suite:
//!
//! 1. `Loaded::runs()` took `Plan::over(..).selected` and dropped the refusal
//!    beside it, so a corpus whose plan stopped partway still published a rate
//!    over a denominator no document declares (#172).
//! 2. `probe grade` tested `plan.selected.is_empty()` where the rule is
//!    `Refusal::stops_a_grade`, so every late refusal graded against the probes
//!    a planner read before it gave up (#173).
//! 3. `check --change` reads a manifest, binds it, and scopes a context to it.
//!    A flag that reached nothing would produce the report of a run with no
//!    flag, which is the correct output for a full-corpus run (#177).
//!
//! The fix for each one is held at the library grain and two of them are held
//! by a type: `Plan::gradable` is the one route to a gradable selection, and
//! `Change` is the only value `Context::over` takes. Neither says anything
//! about a caller that stops asking. The one measurement that made #174 an
//! issue rather than an observation is that reverting the `probe grade` guard
//! left the whole suite green, and each test below was watched failing against
//! the pre-fix form of the decision it names.
//!
//! # Why this drives the binary rather than a function
//!
//! A test of an extracted wiring function proves the function and not the
//! wiring, which is the defect restated. So each case here runs the built
//! binary over a repository root and reads what a caller reads. Cargo builds
//! the binary for this target and names it in `CARGO_BIN_EXE_headwater`, under
//! both `cargo test` and `cargo test --release`.
//!
//! # The root each case runs over
//!
//! A fixture corpus, and the repository's own lock and corpus descriptor. The
//! lock is copied rather than committed here for the reason no rule of this
//! repository is written down twice: `taxonomy resolve --check` holds that file
//! to its sources on every pull request, and a second copy under `fixtures/`
//! would be a resolved taxonomy that nothing checks and that goes stale in
//! silence. So a taxonomy that stops declaring what these documents are fails
//! these tests loudly, which is the report a stale copy would not make.
//!
//! Each root holds one to three documents, so a case costs one process and a
//! walk of three files. The whole target is well under a second.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository this test tree sits in.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// A repository root assembled over one fixture corpus.
struct Root {
    at: PathBuf,
}

impl Root {
    /// The fixture corpus under `fixtures/<case>`, plus the lock and the corpus
    /// descriptor this repository resolves for itself.
    ///
    /// `label` names the test rather than the case, and it is a parameter for a
    /// reason that cost this file one debugging pass: cargo runs the cases of
    /// one target as threads of one process, so two of them over one fixture
    /// corpus share a process identifier, and a directory named after the case
    /// is a directory one case removes while the other is reading it.
    fn over(case: &str, label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-wiring-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        copy(&fixtures().join(case), &at);
        let headwater = at.join(".headwater");
        std::fs::create_dir_all(&headwater).expect("the declaration directory is there");
        for name in ["taxonomy.lock", "taxonomy.yml"] {
            std::fs::copy(
                repository().join(".headwater").join(name),
                headwater.join(name),
            )
            .expect("the declaration copies");
        }
        Root { at }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.at.join(relative)
    }

    /// One invocation, with the root named rather than inherited from the
    /// working directory, because `cargo test` runs every target from one place
    /// and a case that reached this repository would check the wrong corpus.
    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

/// What one invocation reported. The two streams are held apart, because
/// `check` writes a run statistic to standard error and the artifact to
/// standard output, and a case that read them merged would assert over both.
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Ran {
    fn says(&self, text: &str) -> bool {
        self.out.contains(text)
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// `check --change` reaches the verdict, and not only the parser.
///
/// The failure this holds is silent by construction. A run that carries no
/// change reports every instance of `warrant.promoted` as skipped, which is the
/// right output for a full-corpus run, so a flag that was read and dropped
/// produces exactly the report of a run that passed no flag. Nothing in an exit
/// status or a finding count tells the two apart, and the assertion below is
/// therefore about what the scoped report says that the unscoped one does not.
#[test]
fn a_change_the_flag_named_reaches_the_verdict_and_not_only_the_parser() {
    let root = Root::over("change", "change-reaches-the-verdict");
    let manifest = root.path("manifest.txt");
    let prior = fixtures().join("change-prior/0001-the-warrant-a-person-set.md");
    std::fs::write(
        &manifest,
        format!(
            "headwater change 1\nprior\tdocs/decisions/0001-the-warrant-a-person-set.md\t{}\n",
            prior.display()
        ),
    )
    .expect("the manifest writes");

    // The state under test: one document at the `accepted` warrant, whose
    // prior version stands at `asserted`. Loose on purpose, so that the
    // decisive failure below is the one about the flag.
    let unscoped = root.run(&["check", "--no-cache", "--now", "2026-08-01"]);
    assert_eq!(unscoped.code, Some(0), "{}{}", unscoped.out, unscoped.err);
    assert!(
        unscoped.says("change-scoped-only"),
        "a run carrying no change reports the promotion instance as skipped:\n{}",
        unscoped.out
    );

    let scoped = root.run(&[
        "check",
        "--no-cache",
        "--now",
        "2026-08-01",
        "--change",
        &manifest.display().to_string(),
    ]);
    assert_eq!(scoped.code, Some(0), "{}{}", scoped.out, scoped.err);
    // The decision: the manifest the flag named reached `Context::scoped_to`.
    // Both lines below are false of a run that read the manifest and dropped
    // it, and the second is the reading the change produces.
    assert!(
        scoped.says("scoped to a change: 1 documents named, 0 added, 1 with a prior version"),
        "the report states the change this run was scoped to:\n{}",
        scoped.out
    );
    assert!(
        scoped.says("1 promoted from `asserted` to `accepted`"),
        "the run counts the promotion the change carried:\n{}",
        scoped.out
    );
    assert!(
        scoped.out != unscoped.out,
        "a run scoped to a change reports something a full-corpus run does not"
    );
}

/// `probe grade` reads `Plan::gradable` and never the selection beside it.
///
/// The fixture corpus holds two probes. The first is well formed and the second
/// declares a category outside the closed set, so `Plan::over` returns from
/// inside the loop that composes the selection and leaves exactly one probe
/// behind it. That state is the whole instrument: a plan that refused with an
/// *empty* selection is refused by `plan.selected.is_empty()` as well, and a
/// case built over one would pass against the defect it was written for.
///
/// The transcript this grades cannot confirm against this corpus and no fixture
/// could: the plan refuses, so it composes no selection digest, and a
/// transcript naming one that matched would describe a plan this tree cannot
/// compose. That is why the second assertion is about the intake rather than
/// about a verdict. Under the pre-fix guard the verb walks past the refusal and
/// hands the transcript to `Record::read`, which reports that it reached no
/// grader; under the rule it never gets there.
#[test]
fn a_plan_that_stopped_partway_grades_nothing_through_the_verb() {
    let root = Root::over("probes", "grade-reads-gradable");

    // The state under test, and the reason the case is decisive rather than
    // accidental: the plan refuses and the selection it leaves holds one probe
    // of the two. Loose on purpose.
    let planned = root.run(&["probe", "plan"]);
    assert_eq!(planned.code, Some(0), "{}{}", planned.out, planned.err);
    assert!(
        planned.says("over 1 of the 3 classified documents"),
        "the plan stops partway and leaves a selection of one:\n{}",
        planned.out
    );
    assert!(
        planned.says("This run does not start"),
        "the plan refuses this corpus:\n{}",
        planned.out
    );

    // The decision. Under `plan.selected.is_empty()` the selection holds one
    // probe, the guard does not fire, and the verb walks on into the intake
    // with the part of a selection the planner managed.
    let graded = root.run(&[
        "probe",
        "grade",
        &root
            .path("docs/probe-runs/first-regression.md")
            .display()
            .to_string(),
    ]);
    assert_eq!(graded.code, Some(0), "{}{}", graded.out, graded.err);
    assert!(
        graded.says("Nothing was graded. `headwater probe plan` refuses this corpus: the probe at docs/probes/0002-the-category-is-outside-the-closed-set.md"),
        "the verb reports the refusal the plan composed, which only `gradable` hands it:\n{}",
        graded.out
    );
    assert!(
        !graded.says("This transcript reached no grader"),
        "the refusal stopped this verb before it read the transcript at all:\n{}",
        graded.out
    );
}

/// The `Loaded::runs()` call site hands the generator the plan and not the
/// selection.
///
/// `Runs::graded_against` is the rule and it is held by two tests of its own
/// crate. What no test held is the decision, inside the binary, to call it: a
/// caller that composed a plan and never handed it over leaves `Runs` at its
/// default, and the projection then reports that nothing composed a selection
/// and nothing said why. That message and the one below are the two states this
/// case tells apart, and only one of them names the corpus.
#[test]
fn the_generator_is_handed_the_refusal_beside_the_selection() {
    let root = Root::over("probes", "generate-carries-the-refusal");
    let generated = root.run(&["generate"]);
    assert_eq!(
        generated.code,
        Some(0),
        "{}{}",
        generated.out,
        generated.err
    );

    // Loose on purpose, and looser than it reads: a run that composed nothing
    // reports the declaration's pattern here and a run that carries a refusal
    // reports the file it names, so this holds in both states and the decisive
    // failure below is the one about the call site.
    assert!(
        generated.says("probe_result docs/probe-results/"),
        "the probe_result declaration reaches this corpus:\n{}",
        generated.out
    );
    // The decision: the reason is the plan's refusal, which the call site
    // carries only by handing the whole plan over.
    assert!(
        generated.says("it does not compose them over this corpus: the probe at docs/probes/0002-the-category-is-outside-the-closed-set.md"),
        "the projection names the refusal the plan composed:\n{}",
        generated.out
    );
    assert!(
        !generated.says("nothing composed a probe selection and nothing said why"),
        "a refusal reached the generator, so the caller that composed nothing is not this one:\n{}",
        generated.out
    );
}

/// One invocation with no root named, from a directory that is not a corpus.
///
/// [`Root::run`] cannot express this case. It appends `--root` to every
/// invocation unconditionally, and it appends it to a directory it has just
/// filled with a lock and a corpus descriptor, so nothing that helper runs can
/// say what the binary does for a caller who has no repository at all. The bare
/// `Command` below is the difference, and it is the whole point of the case
/// under it.
///
/// The directory is keyed on the process identifier **and** a label, for the
/// reason [`Root::over`] records: cargo runs the cases of one target as threads
/// of one process, so a key that is the pid alone is a directory a second case
/// removes while the first is reading it.
fn outside_a_corpus(label: &str, arguments: &[&str]) -> Ran {
    let at = std::env::temp_dir().join(format!(
        "headwater-cli-wiring-{}-{label}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("the directory is there");
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .current_dir(&at)
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// `--version` and `-V` print the constant a `requires_engine` range is read
/// against, from outside a corpus, on standard output alone.
///
/// Before this case the binary answered both spellings with exit 1, nothing on
/// standard output and 25,194 bytes of usage text on standard error, because a
/// word opening with `-` that no arm names reaches `fail`. So the three
/// assertions below each failed, and the first line of the failure named the
/// flag as one this binary does not know.
///
/// # What this case does not prove, and where the guarantee actually lives
///
/// Every crate of this workspace declares `version.workspace = true`, so
/// [`headwater_resolve::release::ENGINE`] and this crate's own
/// `env!("CARGO_PKG_VERSION")` are the same string. The equality below
/// therefore passes against either read, and it cannot tell them apart. **The
/// guarantee is that the implementation names one constant, not that this test
/// holds it there**, and a reviewer has to read the line in `main.rs` to see
/// it. The engine already paid for the other shape: it advertised the
/// placeholder `0.0.0` as `serverInfo.version` over MCP and the value sat wrong
/// through five milestones, because a value exactly one surface reports is a
/// value nobody audits.
#[test]
fn the_version_flag_prints_the_engine_constant_outside_a_corpus() {
    for (label, flag) in [("version-long", "--version"), ("version-short", "-V")] {
        let ran = outside_a_corpus(label, &[flag]);
        assert_eq!(
            ran.code,
            Some(0),
            "`{flag}` is a question rather than a mistake:\n{}{}",
            ran.out,
            ran.err
        );
        assert_eq!(
            ran.err, "",
            "`{flag}` writes nothing to standard error, so a caller may read the answer with the streams apart"
        );
        assert_eq!(
            ran.out.lines().count(),
            1,
            "`{flag}` prints one line and nothing else:\n{}",
            ran.out
        );
        assert_eq!(
            ran.out.trim_end(),
            headwater_resolve::release::ENGINE,
            "`{flag}` prints the version a `requires_engine` range is read against"
        );
    }
}

/// A command line this binary cannot parse is refused in a few lines, and the
/// grammar is one command away rather than under the sentence.
///
/// Two invocations, because the two arms reach `fail` from different places: an
/// unknown flag is refused inside the argument loop, before any verb is
/// decided, and an unknown first word is refused after that loop against
/// [`headwater_verbs::parse`]. A case over one of them says nothing about the
/// other.
///
/// # What each assertion holds, and why none of them is a byte count
///
/// The marker is `--root <path>`, a line of the usage body that no refusal
/// message contains. Its **absence** is what says the grammar did not print.
/// The obvious alternative — pinning the length of standard error — passes for
/// the wrong reason the moment anybody rewords a message, and fails for the
/// wrong reason the moment anybody edits `USAGE`, which is a fixture nobody
/// reads. #306 asked for the marker for that reason.
///
/// Absence alone is satisfied by a binary that prints nothing at all, so three
/// assertions stand beside it: the word the caller got wrong is echoed back,
/// `headwater --help` is named as where the grammar is, and the whole refusal
/// fits in five lines. Together they say the refusal is short **and** still
/// tells the caller what happened.
///
/// The status is asserted as exactly 1 rather than as non-zero. This binary
/// promises one failing status and no other — `docs/interfaces/headwater-check.md`
/// states eleven reasons for exit 1 under the sentence "There is no third
/// status" — so a 2 here would be a defect that a `!= 0` assertion would pass.
///
/// Standard output is asserted empty, because a refusal that puts one byte
/// there corrupts every caller that reads a report from this binary by pipe.
///
/// # This case was watched failing
///
/// Against the parent commit `6e29f5d`, `headwater check --nonsense` wrote 0
/// bytes to standard output and **25,473 bytes over 360 lines** to standard
/// error, carrying the marker 27 times, and `headwater versoin` wrote 25,655
/// bytes over 360 lines. The exit status and the empty standard output already
/// held; the marker, the line bound and the pointer did not.
#[test]
fn a_refused_command_line_names_the_grammar_rather_than_printing_it() {
    for (label, arguments, offender) in [
        (
            "unknown-flag",
            ["check", "--nonsense"].as_slice(),
            "--nonsense",
        ),
        ("unknown-verb", ["versoin"].as_slice(), "versoin"),
    ] {
        let ran = outside_a_corpus(label, arguments);
        assert_eq!(
            ran.code,
            Some(1),
            "`headwater {}` is refused with the one failing status this binary has:\n{}",
            arguments.join(" "),
            ran.err
        );
        assert_eq!(
            ran.out, "",
            "a refusal writes nothing to standard output, so a caller reading a report by pipe reads a report or nothing"
        );
        assert!(
            !ran.err.contains("--root <path>"),
            "`headwater {}` printed the usage body: `--root <path>` is a line of it and it is on standard error:\n{}",
            arguments.join(" "),
            ran.err
        );
        assert!(
            ran.err.contains(offender),
            "`headwater {}` says `{offender}` back to the caller, so the refusal names what was wrong:\n{}",
            arguments.join(" "),
            ran.err
        );
        assert!(
            ran.err.contains("headwater --help"),
            "`headwater {}` names where the grammar is:\n{}",
            arguments.join(" "),
            ran.err
        );
        assert!(
            ran.err.lines().count() <= 5,
            "`headwater {}` refuses in five lines or fewer, and it wrote {}:\n{}",
            arguments.join(" "),
            ran.err.lines().count(),
            ran.err
        );
    }
}

/// `--help` answers on standard output alone, and it is not a failure.
///
/// The case above deletes the usage body from every refusal, which leaves
/// exactly one caller of `USAGE` in the binary. Nothing else now holds that
/// caller, so a change that deleted the printing altogether, or that moved it
/// to standard error beside the refusals, would take this whole surface away
/// with the suite green.
///
/// The three assertions are the ones
/// [`the_version_flag_prints_the_engine_constant_outside_a_corpus`] makes about
/// `--version`, for the same reason: a question is answered on standard output
/// with exit 0, and standard error stays empty so a caller may keep the two
/// apart. What is asserted about the body is that it is long and carries the
/// marker the refusals must not — the inverse of the case above, and the
/// statement that the grammar went somewhere rather than nowhere.
#[test]
fn the_help_flag_answers_on_standard_output_outside_a_corpus() {
    for (label, flag) in [("help-long", "--help"), ("help-short", "-h")] {
        let ran = outside_a_corpus(label, &[flag]);
        assert_eq!(
            ran.code,
            Some(0),
            "`{flag}` is a question rather than a mistake:\n{}{}",
            ran.out,
            ran.err
        );
        assert_eq!(
            ran.err, "",
            "`{flag}` writes nothing to standard error, so a caller may read the answer with the streams apart"
        );
        assert!(
            ran.out.contains("--root <path>"),
            "`{flag}` is where the grammar is, and `--root <path>` is a line of it:\n{}",
            ran.out
        );
        assert!(
            ran.out.lines().count() > 100,
            "`{flag}` prints the whole grammar, and it printed {} lines",
            ran.out.lines().count()
        );
    }
}
