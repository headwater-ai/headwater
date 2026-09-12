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
/// Help text with every run of whitespace collapsed to one space.
///
/// Clap lays a help string out at the terminal width, so a sentence a reader
/// sees as one sentence is several lines in the bytes. A case that asserts
/// about the words of a help string reads this form, and a case that asserts
/// about a short phrase clap never breaks may read the bytes.
fn flattened(help: &str) -> String {
    help.split_whitespace().collect::<Vec<_>>().join(" ")
}

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

/// The gap the doc comment above names, closed: this reads the source text of
/// the `--version` arm and holds it to naming the shared constant rather than
/// a local `env!("CARGO_PKG_VERSION")` read.
///
/// [`the_version_flag_prints_the_engine_constant_outside_a_corpus`] cannot tell
/// the two apart, because every crate agrees on the number today. This case
/// reads `main.rs` itself, so a regression to a local `env!` read fails here
/// even while every crate's manifest version still matches by coincidence
/// (#308: `grade::VERSION`, `mcp.rs`'s `serverInfo.version` and
/// `headwater_resolve_version()` each read their own crate's `env!` and
/// disagreed the moment one manifest moved on its own — reproduced by hand:
/// give `resolve`, `probe` and `query` the distinct versions `0.1.7`, `0.1.8`,
/// `0.1.9`, rebuild, and `--version` still said `0.1.7` while
/// `serverInfo.version` said `0.1.9` and `probe plan`'s `harness:` said
/// `0.1.8`, three numbers from one binary, none of them wrong on its own
/// terms).
#[test]
fn the_version_flag_reads_the_named_constant_and_not_a_local_env_read() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (_, after) = text.split_once("if cli.version {").unwrap_or_else(|| {
        panic!("`if cli.version` is no longer the shape of the version arm in main.rs")
    });
    // The block's own statements are indented eight spaces and its closing
    // brace sits at four, so the split below cannot land inside the `{}` of
    // the `println!` format string the way a plain `split_once('}')` would.
    let (arm, _) = after.split_once("\n    }").unwrap_or_else(|| {
        panic!("no closing brace found for the `if cli.version` arm in main.rs")
    });
    assert!(
        arm.contains("headwater_resolve::release::ENGINE"),
        "the version arm no longer names the shared constant:\n{arm}"
    );
    assert!(
        !arm.contains("env!(\"CARGO_PKG_VERSION\")"),
        "the version arm reads env!(\"CARGO_PKG_VERSION\") directly, which is the crate main.rs \
         happens to sit in and not the number a `requires_engine` range is read against:\n{arm}"
    );
}

/// Every `.rs` file under `engine/`, recursively, skipping `target`.
fn source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some("target") {
                continue;
            }
            source_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// The permanent form of the grep issue #308 ran by hand: exactly one site
/// under `engine/` may define the engine version from `CARGO_PKG_VERSION`, and
/// it is `release.rs`'s own constant. Every other reader takes
/// [`headwater_resolve::release::ENGINE`] rather than reading the number a
/// second time from its own crate's manifest.
///
/// A line that only mentions the read in prose (a comment or a doc comment)
/// is not a second site, so a line whose trimmed text starts with `//` is
/// skipped — which is why this needle is an escaped string rather than a raw
/// one: a raw string literal would put the plain, unescaped text
/// `env!("CARGO_PKG_VERSION")` into this very file, and the walk below would
/// then count its own search pattern as a second site.
#[test]
fn exactly_one_site_defines_the_engine_version_constant() {
    let needle = "env!(\"CARGO_PKG_VERSION\")";
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    source_files(&root, &mut files);
    let mut sites: Vec<(PathBuf, usize)> = Vec::new();
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if line.contains(needle) {
                sites.push((path.clone(), index + 1));
            }
        }
    }
    assert_eq!(
        sites.len(),
        1,
        "a second env!(\"CARGO_PKG_VERSION\") read appeared outside the one site that defines \
         the constant: {sites:?}"
    );
    let defined_in = root.join("crates/resolve/src/release.rs");
    assert_eq!(
        sites[0].0.canonicalize().unwrap(),
        defined_in.canonicalize().unwrap(),
        "the one site reading CARGO_PKG_VERSION is not release.rs's own definition: {sites:?}"
    );
}

/// A command line this binary cannot parse is refused in a few lines, and the
/// grammar is one command away rather than under the sentence.
///
/// Two invocations, because the two reach `fail` from different places: an
/// unknown flag is refused by the parse of the verb that did not declare it,
/// and an unknown first word is refused by the external-subcommand form that
/// `headwater_cli::Verb` declares. A case over one of them says nothing about
/// the other.
///
/// # What each assertion holds, and why none of them is a byte count
///
/// The marker is `--root <path>`, a line of the help body that no refusal
/// message contains. Its **absence** is what says the grammar did not print.
/// The obvious alternative — pinning the length of standard error — passes for
/// the wrong reason the moment anybody rewords a message, and fails for the
/// wrong reason the moment anybody edits the help, which is a fixture nobody
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

/// A refusal that is a fact about the corpus does not name the grammar.
///
/// `headwater init` over a fixture root refuses because `Root::over` already
/// wrote `.headwater/taxonomy.yml` there — nothing about `headwater init`
/// itself is wrong, it is the form `--help` shows. #331 moves this site from
/// `fail` to `refuse`, which drops the grammar pointer this refusal never
/// earned. This fails on `main`, where every such refusal still names it.
#[test]
fn a_refusal_about_the_corpus_does_not_name_the_grammar() {
    let root = Root::over("change", "corpus-fact-no-grammar-pointer");
    let ran = root.run(&["init"]);
    assert_eq!(
        ran.code,
        Some(1),
        "`headwater init` over an already-bound root fails:\n{}",
        ran.err
    );
    assert!(
        ran.err.contains(headwater_resolve::package::CONSUMER),
        "the refusal names the file that is already there:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("headwater --help"),
        "a corpus fact reached through a correct command line names the grammar, and should not:\n{}",
        ran.err
    );
}

/// `--help` answers on standard output alone, and it is not a failure.
///
/// Every refusal of this binary points at `headwater --help` and prints none of
/// the grammar itself, so this is the one surface the grammar has. Nothing else
/// holds it: a change that stopped printing it, or that moved it to standard
/// error beside the refusals, would take the whole surface away with the rest of
/// the suite green.
///
/// The first two assertions are the ones
/// [`the_version_flag_prints_the_engine_constant_outside_a_corpus`] makes about
/// `--version`, for the same reason: a question is answered on standard output
/// with exit 0, and standard error stays empty so a caller may keep the two
/// apart.
///
/// # What is asserted about the body, and what used to be
///
/// That it names `--root <path>`, which is the marker the refusals must not
/// carry, and that it names every verb the dispatch table carries. Until the
/// parser migration the second of those was `lines().count() > 100`, which was a
/// proxy for "the grammar is here" against a 357-line literal. A count is a
/// fixture nobody reads: it passes for the wrong reason as soon as the layout
/// moves, and [#321](https://github.com/headwater-ai/headwater/issues/321) moves
/// it deliberately. The verb list is the thing the count stood in for, and it is
/// held directly.
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
        for verb in headwater_verbs::VERBS {
            assert!(
                ran.out.contains(verb.name),
                "`{flag}` is where a caller finds a verb, and it does not name `{}`:\n{}",
                verb.name,
                ran.out
            );
        }
    }
}

/// A flag that belongs to another verb is refused rather than accepted and
/// ignored.
///
/// This is the reversal
/// [HW-DR-0033](../../../../docs/decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md)
/// records, at the surface a caller meets. `--level` is read by `conformance`
/// and by nothing else. Under the flat namespace `headwater check --level L0`
/// exited **0** and wrote the whole report, which
/// `docs/interfaces/headwater-check.md` stated as a promise, and a caller who
/// believed the flag had done something read a report that ignored it.
///
/// # The corpus is the point of this case rather than a setting for it
///
/// The first invocation is not scaffolding. Run from a directory that is not a
/// corpus, `headwater check` exits 1 with nothing on standard output *whatever*
/// the parser does, so the asserted outcome would be the ambient one and
/// deleting the refusal would break nothing. Over this fixture root `check`
/// exits 0 and writes a report, so exit 1 with an empty standard output is
/// reachable through the refusal and through nothing else.
///
/// The last invocation is the other direction: the flag still reaches the verb
/// that declares it, so what was withdrawn is the namespace and not the flag.
#[test]
fn a_flag_that_belongs_to_another_verb_is_refused_rather_than_ignored() {
    let root = Root::over("change", "level-belongs-to-conformance");

    let ran = root.run(&["check", "--no-cache", "--now", "2026-08-01"]);
    assert_eq!(
        ran.code,
        Some(0),
        "over this root the verb succeeds, which is what makes the refusal below the only route to a 1:\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        !ran.out.is_empty(),
        "over this root the verb writes a report, which is what makes an empty standard output below decisive"
    );

    let refused = root.run(&[
        "check",
        "--level",
        "L0",
        "--no-cache",
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(
        refused.code,
        Some(1),
        "`--level` is not a flag `check` reads, and this binary has one failing status:\n{}{}",
        refused.out,
        refused.err
    );
    assert_eq!(
        refused.out, "",
        "a refused command line writes no report, so a caller reading by pipe reads a report or nothing"
    );
    assert!(
        refused.err.contains("--level"),
        "the refusal names the flag the caller wrote:\n{}",
        refused.err
    );

    let read = root.run(&["conformance", "--level", "L0", "--now", "2026-08-01"]);
    assert!(
        !read.err.contains("--level"),
        "`--level` reaches the verb that declares it, whatever that verb then reports:\n{}",
        read.err
    );
}

// ---------------------------------------------------------------------------
// Which of the three refusal helpers each site calls. #455 settled the eight
// sites #331 could not, and the test it settled them by is one question: is
// there a spelling of this request that gets past this refusal? The three cases
// below are the three answers.
// ---------------------------------------------------------------------------

/// A verb this binary parses and this engine has never implemented does not
/// send the caller to the grammar.
///
/// `query` is a real member of `headwater_verbs::VERBS`, on purpose (#146: a
/// wait a caller cannot discover is a wait nobody reads), so `headwater --help`
/// lists it and repeats the sentence the refusal just made. There is no other
/// spelling of the request, so #455 moves this site to `refuse`.
///
/// This case was watched failing against `a316a23`, where the run wrote two
/// lines and the second was ``headwater: run `headwater --help` for the
/// grammar``.
#[test]
fn a_verb_this_engine_never_implemented_does_not_name_the_grammar() {
    let ran = outside_a_corpus("query-states-a-wait", &["query", "anything"]);
    assert_eq!(
        ran.code,
        Some(1),
        "`headwater query` is refused with the one failing status this binary has:\n{}",
        ran.err
    );
    assert_eq!(
        ran.out, "",
        "a refusal writes nothing to standard output, so a caller reading by pipe reads a report or nothing"
    );
    assert!(
        ran.err.contains("no document states what an expression is"),
        "the refusal states the wait rather than a fault:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("headwater --help"),
        "the grammar lists `query` and says the same thing, so this points the caller at a repeat of the sentence above it:\n{}",
        ran.err
    );
}

/// The one site of the eight #455 keeps at `fail`, and the measurement that
/// keeps it there.
///
/// `taxonomy vendor` with no pin names two remedies: write `taxonomy.digest`
/// into the consumer declaration, or pass `--expect`. The second is a flag, and
/// the second half below is the proof it is a complete route — the run with
/// `--expect` reaches past this site, to the artifact that is not a published
/// package. A caller who does not know `--expect` exists is the caller the
/// grammar pointer is for.
///
/// This case is green throughout rather than red then green. It is the recorded
/// evidence for a ruling that would otherwise be an assertion in a comment that
/// nothing holds.
#[test]
fn a_refusal_a_flag_repairs_still_names_the_grammar() {
    let root = Root::over("change", "vendor-pin-names-the-grammar");
    let at = root.path(".headwater/taxonomy.yml");
    let text = std::fs::read_to_string(&at).expect("the declaration reads");
    let without: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("digest:"))
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(&at, without).expect("the declaration writes");
    let fetched = root.path("fetched");
    std::fs::create_dir_all(&fetched).expect("the directory is there");
    let fetched = fetched.to_str().expect("the path is utf-8").to_string();

    let ran = root.run(&["taxonomy", "vendor", &fetched]);
    assert_eq!(
        ran.code,
        Some(1),
        "an unpinned artifact is refused:\n{}",
        ran.err
    );
    assert!(
        ran.err.contains("nothing pins this artifact"),
        "the refusal is the pin site and not something earlier:\n{}",
        ran.err
    );
    assert!(
        ran.err.contains("headwater --help"),
        "a refusal a flag repairs names where that flag is written down:\n{}",
        ran.err
    );

    let ran = root.run(&[
        "taxonomy",
        "vendor",
        &fetched,
        "--expect",
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    ]);
    assert!(
        !ran.err.contains("nothing pins this artifact"),
        "`--expect` is a complete route past the pin, which is why the site stays at `fail`:\n{}",
        ran.err
    );
}

/// The two refusals nothing can execute, held by reading the source.
///
/// Both fire only if this engine emitted YAML it cannot read back. No command
/// line reaches either, so no case can drive them, and the population they
/// belong to is the whole point of `defect`. What is held here is the mapping
/// from the message to the helper that carries it, which is the same shape as
/// [`the_version_flag_reads_the_named_constant_and_not_a_local_env_read`].
///
/// This case was watched failing against `a316a23`, where both needles resolved
/// to `fail(`.
#[test]
fn every_refusal_about_a_value_this_run_built_goes_out_through_defect() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let text = std::fs::read_to_string(&path).expect("main.rs reads");
    for needle in [
        "the payload this run built is not a mapping",
        "the payload this run built does not load",
    ] {
        let (before, _) = text
            .split_once(needle)
            .unwrap_or_else(|| panic!("`{needle}` is no longer a message in main.rs"));
        let (_, helper) = ["fail(", "refuse(", "defect("]
            .iter()
            .filter_map(|opener| before.rfind(opener).map(|at| (at, *opener)))
            .max()
            .expect("a refusal helper opens the call");
        assert_eq!(
            helper, "defect(",
            "`{needle}` goes out through `{helper}`. A value this run's own code built is a \
             defect of this engine rather than a fact about the caller or the corpus"
        );
    }
}

/// What `defect` prints, which nothing held until a review found it.
///
/// The case above asserts which helper opens a call and says nothing about what
/// that helper writes. A review rewrote `defect`'s body to print a
/// `github.com` address and to drop the engine constant, and the whole suite
/// stayed green. Both properties are stated in the doc comment and in the
/// interface contract, so both are assertions this repository makes to a
/// caller, and neither was held.
///
/// This reads the source rather than driving the binary, and the reason is the
/// point of the helper rather than a gap in this case. Both call sites are
/// unreachable: every scalar the payload carries goes through `quoted`, so no
/// corpus and no command line produces a payload that fails to load. A case
/// that drove the site would need a route that no longer exists.
///
/// The URL assertion is not decoration. This repository has no published home
/// (Q31 is open), so an address in caller-facing output would be an address
/// that answers nothing.
#[test]
fn the_defect_helper_names_the_engine_version_and_no_address() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let text = std::fs::read_to_string(&path).expect("main.rs reads");
    let (_, after) = text
        .split_once("fn defect(message: &str) -> ExitCode {")
        .expect("`fn defect` is no longer the shape of the helper in main.rs");
    let (body, _) = after
        .split_once("\n}")
        .expect("no closing brace found for `fn defect` in main.rs");
    assert!(
        body.contains("headwater_resolve::release::ENGINE"),
        "a report of a defect needs the version it was found in, and this names no version:\n{body}"
    );
    for address in ["http://", "https://", "github.com"] {
        assert!(
            !body.contains(address),
            "`defect` writes `{address}` to a caller. This repository has no published home, so \
             the address would answer nothing:\n{body}"
        );
    }
    assert!(
        body.contains("ExitCode::FAILURE"),
        "this binary has one failing status and `defect` returns it:\n{body}"
    );
}

/// A refusal the consumer declaration caused does not name the grammar.
///
/// `headwater import` reads `.headwater/taxonomy.yml` through
/// `headwater_import::declared`, which is the file `refuse`'s own doc comment
/// names as its population. An `imports` entry that names no `at` is a fact
/// about that file alone, and no spelling of `headwater import` gets past it.
///
/// This is the ninth site. #455 named eight and this was not among them, but
/// #331's first Done-when bullet is a predicate over every call site of `fail`,
/// so a site left here is that bullet unmet. Found by a review of #460 rather
/// than by the reading that produced the eight.
///
/// This case was watched failing before the site moved, where the run wrote the
/// message and then ``headwater: run `headwater --help` for the grammar``.
#[test]
fn a_refusal_the_consumer_declaration_caused_does_not_name_the_grammar() {
    let root = Root::over("change", "import-declaration-no-grammar-pointer");
    let at = root.path(".headwater/taxonomy.yml");
    let mut text = std::fs::read_to_string(&at).expect("the declaration reads");
    text.push_str("\nimports:\n  upstream:\n    channel: stable\n");
    std::fs::write(&at, text).expect("the declaration writes");

    let ran = root.run(&["import"]);
    assert_eq!(
        ran.code,
        Some(1),
        "an import declaration this engine cannot read is refused:\n{}",
        ran.err
    );
    assert!(
        ran.err.contains("`imports.upstream` names no `at`"),
        "the refusal names the entry that is incomplete:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("headwater --help"),
        "the consumer declaration is a fixed location this engine reads on every run, and no \
         command line reaches past what it says:\n{}",
        ran.err
    );
}

/// Every scalar `headwater infer` writes survives a load, whatever it holds.
///
/// The two `defect` sites in `infer` were reachable when they were written, and
/// a review reached both. A newline in `--owner` wrote a raw line break inside
/// a double-quoted scalar, and a `}` in a document's filename broke the
/// unquoted flow mapping the pairs were emitted as. In both cases the run
/// printed that neither the corpus nor the command line caused it, which was
/// false.
///
/// A third route was worse than either: a comma in a filename did not fail at
/// all. The flow mapping read `docs/spec/a,b.md` as the value `docs/spec/a`,
/// the run exited 0, and the lock declared debt against a document that does
/// not exist.
///
/// So this drives the three routes rather than the helper. `--write` is what
/// makes the case decisive: without it the payload is printed and never loaded,
/// and the load is the step that used to fail.
#[test]
fn a_payload_this_verb_writes_loads_whatever_a_path_or_an_owner_holds() {
    let root = Root::over("change", "infer-quotes-every-scalar");
    // The one case here that reaches the resolver, because `--write` resolves
    // before it writes and the lock it produces is this case's evidence.
    // `Root::over` copies the two declarations every other case needs, and a
    // resolution needs the sources behind them as well.
    copy(&repository().join("packages"), &root.path("packages"));
    std::fs::copy(
        repository().join(".headwater/overlay.yml"),
        root.path(".headwater/overlay.yml"),
    )
    .expect("the overlay copies");
    for (label, name) in [
        ("a closing brace", "91-brace}here.md"),
        ("a comma", "91-comma,here.md"),
        ("a quotation mark", "91-quote\"here.md"),
    ] {
        let from = root.path("docs/decisions/0001-the-warrant-a-person-set.md");
        let to = root.path(&format!("docs/decisions/{name}"));
        std::fs::copy(&from, &to).unwrap_or_else(|why| panic!("the {label} case copies: {why}"));
    }

    let ran = root.run(&[
        "infer",
        "--write",
        "--owner",
        "alice\nbob\r\tcarol",
        "--now",
        "2026-08-01",
    ]);
    assert!(
        !ran.err.contains("does not load"),
        "the payload loads back, so no scalar this run wrote broke it:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("this is a defect in engine"),
        "no input reaches the defect helper, which is what makes that helper's population empty:\n{}",
        ran.err
    );
    assert_eq!(
        ran.code,
        Some(0),
        "the run completes:\n{}{}",
        ran.out,
        ran.err
    );

    let lock = std::fs::read_to_string(root.path(".headwater/taxonomy.lock"))
        .expect("the lock reads back");
    assert!(
        lock.contains("91-comma,here.md"),
        "a comma in a path used to truncate the value silently, and the lock declared debt \
         against a document that does not exist:\n{lock}"
    );
    assert!(
        lock.contains("91-brace}here.md"),
        "a closing brace in a path used to break the payload:\n{lock}"
    );

    // The fourth route, and the one repairing the other three exposed. The lock
    // writer escaped `\n` and `\t` and let a `\r` through raw, so `--write`
    // exited 0 and wrote a file the next command could not read. Nothing short
    // of reading it back says whether that is fixed.
    let read = root.run(&["check", "--no-cache", "--now", "2026-08-01"]);
    assert!(
        !read.err.contains("the lock cannot be read"),
        "the lock this run wrote loads again. A carriage return in an owner used to write a lock \
         no later command could read, with this run still exiting 0:\n{}",
        read.err
    );
    assert_eq!(
        read.code,
        Some(0),
        "and the corpus still checks over it:\n{}{}",
        read.out,
        read.err
    );
}

/// A discriminator stated on the command line reaches the terminal as a
/// refusal, and no document is written.
///
/// The library grain records this in the scaffolder's transcript. What only the
/// binary can say is the other three halves of it: the exit status a caller
/// scripts against, the one line on standard error, and the absence of a file.
/// Before [#338](https://github.com/headwater-ai/headwater/issues/338) this
/// command exited 0, printed nothing about the value it was handed, and left a
/// `design_spec` on the shelf.
///
/// The unnarrowed run beside it is the control. Without it a case could pass
/// because the fixture root refuses `headwater new` for some reason of its own,
/// and the flag would never be what the exit status measured.
#[test]
fn a_discriminator_stated_on_the_command_line_refuses_and_writes_nothing() {
    let root = Root::over("change", "facet-names-the-discriminator");

    let refused = root.run(&[
        "new",
        "design_spec",
        "--title",
        "A part the caller renamed",
        "--facet",
        "doc_type=review_record",
    ]);
    assert_eq!(
        refused.code,
        Some(1),
        "a value for the discriminator refuses:\n{}{}",
        refused.out,
        refused.err
    );
    assert!(
        refused.err.contains("review_record") && refused.err.contains("design_spec"),
        "the refusal names the value stated and the kind that decides it:\n{}",
        refused.err
    );
    assert!(
        !refused.out.contains("wrote"),
        "nothing is reported as written:\n{}",
        refused.out
    );
    let shelf = root.path("docs/spec");
    assert!(
        !shelf.exists(),
        "and nothing is on the shelf: {}",
        shelf.display()
    );

    // The control. The same command with no `--facet` writes the document, so
    // the exit status above is the flag and not the root.
    let wrote = root.run(&["new", "design_spec", "--title", "A part the caller renamed"]);
    assert_eq!(
        wrote.code,
        Some(0),
        "the same run with no stated facet writes:\n{}{}",
        wrote.out,
        wrote.err
    );
    let written = root.path("docs/spec/01-a-part-the-caller-renamed.md");
    let text = std::fs::read_to_string(&written).expect("the document reads");
    assert!(
        text.contains("doc_type: design_spec"),
        "and the discriminator it writes is the kind:\n{text}"
    );
}

/// The `--change` help names the header a manifest must open with, and a
/// manifest written from that help reads.
///
/// # The defect this holds
///
/// The help carried into #335 from the pre-clap parser described a manifest as
/// nothing but its `added` and `prior` lines. It said "Each line names one
/// document the change carries", and it never mentioned the
/// `headwater change 1` first line that
/// [`headwater_check::change::FORMAT`] requires. A caller who wrote the file
/// the help described was refused, and the sentence sat wrong across a parser
/// rewrite with every gate green, because no rule of this engine reads a
/// sentence about this engine.
///
/// # Why both halves are here
///
/// The string half alone would pass against a help that named the header and a
/// reader that stopped requiring it. The behavior half alone is
/// `a_change_the_flag_named_reaches_the_verdict_and_not_only_the_parser`
/// above, which writes the header and so never sees the refusal. The pair is
/// the claim: *the manifest the help describes is the manifest the verb
/// accepts*, and it is the shape #339 asks for over the 64 restored strings.
#[test]
fn the_change_help_names_the_header_a_manifest_must_open_with() {
    let help = outside_a_corpus("change-help", &["check", "--help"]);
    assert_eq!(
        help.code,
        Some(0),
        "`check --help` is a question rather than a mistake:\n{}{}",
        help.out,
        help.err
    );
    // The decision. This line fails against the string as #335 restored it.
    assert!(
        flattened(&help.out).contains("headwater change 1"),
        "the `--change` help names the header a manifest opens with:\n{}",
        help.out
    );

    // And the behavior the sentence now describes, both ways round.
    let root = Root::over("change", "change-help-header");
    let prior = fixtures().join("change-prior/0001-the-warrant-a-person-set.md");
    let body = format!(
        "prior\tdocs/decisions/0001-the-warrant-a-person-set.md\t{}\n",
        prior.display()
    );

    let headless = root.path("headless.txt");
    std::fs::write(&headless, &body).expect("the manifest writes");
    let refused = root.run(&[
        "check",
        "--no-cache",
        "--now",
        "2026-08-01",
        "--change",
        &headless.display().to_string(),
    ]);
    assert_eq!(
        refused.code,
        Some(1),
        "a manifest with no header is refused rather than read:\n{}{}",
        refused.out,
        refused.err
    );
    assert!(
        refused.err.contains("headwater change 1"),
        "and the refusal names the line that is missing:\n{}",
        refused.err
    );

    let headed = root.path("headed.txt");
    std::fs::write(
        &headed,
        format!("{}\n{body}", headwater_check::change::FORMAT),
    )
    .expect("the manifest writes");
    let read = root.run(&[
        "check",
        "--no-cache",
        "--now",
        "2026-08-01",
        "--change",
        &headed.display().to_string(),
    ]);
    assert_eq!(
        read.code,
        Some(0),
        "the same manifest under that header reads:\n{}{}",
        read.out,
        read.err
    );
    assert!(
        read.says("scoped to a change"),
        "and the run is scoped to it:\n{}",
        read.out
    );
}

/// `route` never claims silence, because it is never silent.
///
/// # The defect this holds
///
/// The restored description said "It is silent when nothing matches." The verb
/// prints at least four lines and exits 0 on a task that matches no purpose,
/// and it does so deliberately:
/// `headwater_query::route` makes silence a *property of the pointer set*
/// rather than of the output, so that a caller can tell "no purpose answers
/// this" from "the corpus declares none". A caller reading the old help
/// learned the opposite and would have waited for output that a working run
/// already wrote.
///
/// The two halves are the same pair as the case above: the string must not
/// promise silence, and the verb must not be silent.
#[test]
fn route_promises_no_silence_and_is_never_silent() {
    let help = outside_a_corpus("route-help", &["route", "--help"]);
    assert_eq!(
        help.code,
        Some(0),
        "`route --help` is a question rather than a mistake:\n{}{}",
        help.out,
        help.err
    );
    // The decision. This line fails against the string as #335 restored it.
    assert!(
        !flattened(&help.out).contains("silent when nothing matches"),
        "the description does not promise a silence this verb never keeps:\n{}",
        help.out
    );

    let root = Root::over("change", "route-is-never-silent");
    let ran = root.run(&["route", "a task no purpose of this corpus answers"]);
    assert_eq!(
        ran.code,
        Some(0),
        "a route that matches nothing is a result rather than a mistake:\n{}{}",
        ran.out,
        ran.err
    );
    assert_eq!(
        ran.err, "",
        "and it writes nothing to standard error:\n{}",
        ran.err
    );
    assert!(
        ran.out.lines().count() >= 3,
        "a route that matches nothing still writes its report:\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("no purpose") || ran.out.contains("declares no purpose"),
        "and it says which of the two reasons applies:\n{}",
        ran.out
    );
}

/// `check --format` states which target declares its loss inside the artifact,
/// and only SARIF does.
///
/// # The defect this holds
///
/// The restored help ended "Each names what it could not carry", which is
/// false of three of the four targets. `text` and `json` declare an empty loss
/// set, so there is nothing for them to name. `markdown` declares four losses
/// in `headwater_adapter::markdown::LOSS` and deliberately writes none of them
/// into the artifact, because a job summary is prose and "an artifact that
/// declared its own loss would be declaring it to a person who cannot act on
/// it". So a consumer who read the help and looked in the Markdown for the
/// loss set found none, and the sentence was a claim about the source read as
/// a claim about the output.
#[test]
fn only_the_sarif_artifact_declares_its_own_loss_set() {
    let help = outside_a_corpus("format-help", &["check", "--help"]);
    assert_eq!(help.code, Some(0), "{}{}", help.out, help.err);
    // The decision. This line fails against the string as #335 restored it.
    // The comparison is over the flattened help, because clap wraps a help
    // string across lines and a sentence read for its words is not there to
    // find in the laid-out form.
    assert!(
        !flattened(&help.out).contains("Each names what it could not carry"),
        "the help does not claim a loss set every target writes:\n{}",
        help.out
    );

    let root = Root::over("change", "loss-set-per-target");
    let mut carries = Vec::new();
    for target in ["text", "json", "sarif", "markdown"] {
        let ran = root.run(&[
            "check",
            "--no-cache",
            "--now",
            "2026-08-01",
            "--format",
            target,
        ]);
        assert_eq!(
            ran.code,
            Some(0),
            "`--format {target}` writes an artifact:\n{}{}",
            ran.out,
            ran.err
        );
        carries.push((target, ran.out.contains("loss_set")));
    }
    assert_eq!(
        carries,
        vec![
            ("text", false),
            ("json", false),
            ("sarif", true),
            ("markdown", false)
        ],
        "only the SARIF artifact carries its own loss set"
    );
}

/// `capture` pools readings across taxonomies, and its description says so.
///
/// # The defect this holds
///
/// The restored description ended "it never averages readings taken under two
/// taxonomies". It does. `main.rs` reads the distinct locks in the store, and
/// where there is more than one it prints the pooled fraction anyway and warns
/// that the number is not a trend. The description named the remedy that was
/// considered and rejected, so a reader learned the verb refuses a comparison
/// it in fact makes.
///
/// The store below carries two readings under two different locks, which is
/// the smallest input that separates the two arms.
#[test]
fn capture_pools_across_taxonomies_and_names_every_one() {
    let help = outside_a_corpus("capture-help", &["capture", "--help"]);
    assert_eq!(help.code, Some(0), "{}{}", help.out, help.err);
    // The decision. This line fails against the string as #335 restored it.
    assert!(
        !flattened(&help.out).contains("never averages readings taken under two taxonomies"),
        "the description does not claim a refusal this verb never makes:\n{}",
        help.out
    );

    let root = Root::over("change", "capture-pools-across-locks");
    let store = root.path(".headwater/capture-cost.jsonl");
    std::fs::write(
        &store,
        "{\"lock\":\"sha256:aaaa\",\"date\":\"2026-08-01\",\"kind\":\"decision\",\
         \"document\":\"docs/decisions/0001-the-warrant-a-person-set.md\",\"id\":\"DR-ONE\",\
         \"fields\":[4,5],\"sections\":[3,3],\"identifier\":[1,1],\"edge_halves\":[0,0]}\n\
         {\"lock\":\"sha256:bbbb\",\"date\":\"2026-08-02\",\"kind\":\"decision\",\
         \"document\":\"docs/decisions/0001-the-warrant-a-person-set.md\",\"id\":\"DR-TWO\",\
         \"fields\":[2,5],\"sections\":[3,3],\"identifier\":[1,1],\"edge_halves\":[0,0]}\n",
    )
    .expect("the store writes");

    let ran = root.run(&["capture"]);
    assert_eq!(
        ran.code,
        Some(0),
        "`capture` reads the store back:\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        ran.says("2 readings"),
        "it read both readings:\n{}",
        ran.out
    );
    // The behavior the corrected sentence describes: one fraction over both,
    // 4 + 2 supplied of 5 + 5 fields plus the sections and the identifiers.
    assert!(
        ran.says("14 of 18"),
        "it pools the two readings into one fraction:\n{}",
        ran.out
    );
    assert!(
        ran.says("2 taxonomies produced these readings"),
        "and it names the count of taxonomies it pooled across:\n{}",
        ran.out
    );
    for lock in ["sha256:aaaa", "sha256:bbbb"] {
        assert!(ran.says(lock), "and it names {lock}:\n{}", ran.out);
    }
}

/// **The decisive case for the `check-rule` resolver.** A rule identifier this
/// engine ships binds, and one it does not is refused by name.
///
/// It runs the binary rather than a function, and that is the whole point of
/// putting it in this target. `Resolvers::over(&corpus)` is called at 37 sites
/// under `engine/crates/*/tests/` and at one site in `main.rs`. A case built
/// through any of the 37 sees no check-rule resolver, so it reports that
/// `check_rule` names a resolver this run does not have, which is exactly what
/// the tree said before this resolver existed. Only a run of the binary
/// assembles the set the way a user does.
///
/// Both arms are asserted, and the second is what stops the first from passing
/// vacuously: a resolver that refused every string would also refuse the typo,
/// and a resolver that bound every string would bind it too.
#[test]
fn a_check_rule_this_engine_ships_is_an_edge_endpoint_and_a_typo_is_not() {
    let root = Root::over("check-rule-anchor", "check-rule-binds-and-refuses");
    let ran = root.run(&["check", "--no-cache", "--now", "2026-09-06"]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);

    // The binding arm. The report names the anchor kind, the identifier and
    // the resolver that owns it, so a reader can see which component answered.
    assert!(
        ran.says("check_rule `section.required.missing` via check-rule"),
        "a rule this engine ships is a bound target:\n{}",
        ran.out
    );

    // The refusal arm. The identifier is in the message, because the author is
    // looking at the line that spells it.
    assert!(
        ran.says("this engine implements no rule `no.such.rule`"),
        "a rule this engine does not ship is refused by name:\n{}",
        ran.out
    );
    // And it is refused rather than normalized into something that binds.
    assert!(
        !ran.says("check_rule `no.such.rule` via check-rule"),
        "nothing guessed a binding for it:\n{}",
        ran.out
    );

    // **The measured limit of this change, pinned so that a later reader meets
    // it here rather than in a corpus.** A bound anchor edge is a target the
    // graph reports and it is not a neighbour: `Adjacency::of` in
    // `headwater_check::scope` skips every target that is not a document, so
    // `relation.participation.overdue` cannot see this edge and reports the
    // requirement as reaching nothing. The adjudication of #411 predicted the
    // opposite, and this is the measurement that corrects it. #855 holds the
    // work, and inverting this assertion is what closes it.
    let overdue = root.run(&["check", "--no-cache", "--now", "2026-12-31"]);
    assert_eq!(overdue.code, Some(0), "{}{}", overdue.out, overdue.err);
    assert!(
        overdue.says("`requirement-verified`: 116 days"),
        "a check-rule verifier does not yet settle the expectation:\n{}",
        overdue.out
    );
}
