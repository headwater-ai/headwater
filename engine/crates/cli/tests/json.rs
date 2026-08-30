// SPDX-License-Identifier: Apache-2.0
//! `--json`: the two spellings of one target, and every document it writes.
//!
//! # The clause
//!
//! [#321](https://github.com/headwater-ai/headwater/issues/321): "`--json` is
//! accepted wherever `--format json` already is, and `route`, `explain`, `gate`
//! and `conformance` emit JSON that `python3 -m json.tool` parses."
//!
//! Two halves and they fail differently. The first is an aliasing claim, and
//! the way it goes wrong is that one name reaches a slightly different code
//! path — so it is held by `cmp` over both streams and the exit status, and
//! never by reading two outputs that look alike. The second is a claim that
//! bytes are JSON, and the way *that* goes wrong is that the producer's own
//! reader is forgiving in the same places the producer is loose. So the reading
//! below is done by a parser this repository did not write.
//!
//! # Why the external parser, and what happens where it is absent
//!
//! `python3 -m json.tool` is the clause's own bar and it is deliberately not
//! this system: `headwater_yaml` is both halves of a protocol here, and a
//! round-trip through it would prove that one crate agrees with itself.
//! [`oracle`] runs it and `HEADWATER_JSON_ORACLE` turns "it did not run" into a
//! failure. That is the shape `engine/crates/adapter/tests/fixtures.rs`
//! already uses for the SARIF validator, and `engine/crates/hash/tests/oracle.rs`
//! for the SHA-256 one. The `Test` step of `.github/workflows/ci.yml` sets four
//! such variables, which is those three and `HEADWATER_STOCK_VALIDATOR`. The
//! in-tree parse runs either way, so a machine with no `python3` still holds the
//! shape and says which half it did not run.
//!
//! # The root
//!
//! This repository, because three of the four new documents are about a corpus
//! and the interesting values only exist over a real one: `conformance` has
//! rules with gaps and a waiver, `gate` has two barriers, and `route` has
//! pointers with summaries in them. Every invocation is well under a second.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository this test tree sits in.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn scratch() -> PathBuf {
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join("json");
    std::fs::create_dir_all(&at).expect("the directory is there");
    at
}

/// The two streams held apart, because an artifact is on one and an account of
/// a refusal is on the other.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: Vec<u8>,
    err: Vec<u8>,
}

impl Ran {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.out).into_owned()
    }
}

fn ran(arguments: &[&str]) -> Ran {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: output.stdout,
        err: output.stderr,
    }
}

/// A task this corpus answers, and one it does not.
const ANSWERED: &str = "what does a check know about the front matter of a document";
const UNANSWERED: &str = "xyzzy plugh frobnicate quuxbar";

/// A read set of this tree, written once per run of this target.
fn read_set() -> PathBuf {
    let path = scratch().join("run.readset");
    if !path.exists() {
        let written = ran(&["check", "--read-set", path.to_str().expect("a path")]);
        assert_eq!(
            written.code,
            Some(0),
            "the read set is written: {written:?}"
        );
    }
    path
}

/// A file at the shape `sweep plan` asks an agent to write back, carrying this
/// taxonomy's own digest.
///
/// The digest is read out of the plan rather than written here, so a taxonomy
/// change moves it by itself. A literal would refuse on the first resolve after
/// any package edit.
fn sweep_return() -> PathBuf {
    let path = scratch().join("return.yml");
    if !path.exists() {
        let plan = ran(&["sweep", "plan"]).text();
        let taxonomy = plan
            .lines()
            .find(|line| line.trim_start().starts_with("taxonomy: sha256:"))
            .expect("the plan states the taxonomy it was written against")
            .trim()
            .to_string();
        std::fs::write(&path, format!("{taxonomy}\nslice: .\nfindings: []\n"))
            .expect("the return file writes");
    }
    path
}

/// The four command lines that accept both spellings of one target.
///
/// **`check` carries `--no-cache`, and without it the comparison below is a
/// coin toss.** `docs/interfaces/headwater-check.md` says why: the report goes
/// to standard output and the cache accounting goes to standard error, "because
/// it is a fact about the disk of one machine rather than about the corpus". So
/// the first run of a pair evaluates and writes the cache and the second serves
/// from it, and their standard-error lines differ by construction — on a cold
/// tree, and not on a warm one. `--no-cache` makes both runs report the same
/// accounting because neither reads or writes a cache, and spec 12 fixes that
/// the flag moves no byte of standard output, so nothing about the artifact
/// under comparison is weakened.
///
/// The other three write no such line: `capture` and `sweep report` put nothing
/// on standard error, and `export` puts a loss-set account there that is a fact
/// about the corpus.
///
/// **Every command line here succeeds, and that is the scope of the property.**
/// The equality below is about the artifact a run writes. It is not about a
/// refusal: HW-DR-0043 rules that a message a person reads names the spelling
/// they typed, so `export --json --check` and `export --format json --check`
/// deliberately write *different* standard error.
/// [`a_refusal_names_the_spelling_the_caller_typed`] holds that half.
fn both_spellings() -> Vec<(&'static str, Vec<String>)> {
    let returned = sweep_return().to_str().expect("a path").to_string();
    vec![
        ("check", vec!["check".to_string(), "--no-cache".to_string()]),
        ("capture", vec!["capture".to_string()]),
        // This corpus declares two profiles now: `default` (every projection
        // that names none, which is the other five) and `site` (the one
        // `graph_export` entry, #414 piece A). `--format` writes one artifact
        // to a pipe, so it refuses to guess between them, and `--profile`
        // disambiguates the same way a caller with a real second audience
        // would have to.
        (
            "export",
            vec![
                "export".to_string(),
                "--profile".to_string(),
                "site".to_string(),
            ],
        ),
        (
            "sweep report",
            vec!["sweep".to_string(), "report".to_string(), returned],
        ),
    ]
}

/// Every JSON document this binary writes, named.
fn documents() -> Vec<(&'static str, Ran)> {
    let returned = sweep_return();
    let recorded = read_set();
    vec![
        ("check --json", ran(&["check", "--json"])),
        ("capture --json", ran(&["capture", "--json"])),
        (
            "export --json",
            ran(&["export", "--json", "--profile", "site"]),
        ),
        (
            "sweep report --json",
            ran(&[
                "sweep",
                "report",
                returned.to_str().expect("a path"),
                "--json",
            ]),
        ),
        ("route --json, offered", ran(&["route", "--json", ANSWERED])),
        (
            "route --json, silent",
            ran(&["route", "--json", UNANSWERED]),
        ),
        (
            "explain --json",
            ran(&["explain", "--json", "docs/spec/12-check-layer.md"]),
        ),
        (
            "gate --json",
            ran(&[
                "gate",
                "--json",
                "--read-set",
                recorded.to_str().expect("a path"),
            ]),
        ),
        ("conformance --json", ran(&["conformance", "--json"])),
        (
            "conformance --json --level",
            ran(&["conformance", "--json", "--level", "L1"]),
        ),
    ]
}

/// The two spellings write the same bytes, on both streams, with one status.
///
/// `cmp` and not an eyeball. The way an alias goes wrong is that one name
/// reaches a code path that is *almost* the other, and two artifacts that both
/// look like JSON reports of the same run is exactly what that produces.
#[test]
fn the_two_spellings_of_one_target_write_the_same_bytes() {
    for (name, base) in both_spellings() {
        let mut with_flag: Vec<&str> = base.iter().map(String::as_str).collect();
        with_flag.push("--json");
        let mut with_format: Vec<&str> = base.iter().map(String::as_str).collect();
        with_format.extend(["--format", "json"]);

        let flagged = ran(&with_flag);
        let formatted = ran(&with_format);
        assert_eq!(
            flagged.code, formatted.code,
            "`{name} --json` and `{name} --format json` exit alike"
        );
        assert_eq!(
            flagged.out, formatted.out,
            "`{name} --json` writes the bytes `{name} --format json` writes"
        );
        assert_eq!(
            flagged.err, formatted.err,
            "`{name} --json` accounts for itself as `{name} --format json` does"
        );
        assert!(
            !flagged.out.is_empty(),
            "`{name} --json` writes something, so the comparison above is not two empty files"
        );
    }
}

/// A command line that names one target twice is refused.
///
/// Neither resolved nor silently preferred. A precedence rule is how a caller
/// states a value and the engine substitutes its own, which is the defect
/// [#337](https://github.com/headwater-ai/headwater/issues/337) and
/// [#338](https://github.com/headwater-ai/headwater/issues/338) are open about,
/// and this verb surface has just gained a second name for one value.
///
/// Exit exactly 1 and never `clap`'s own 2: `docs/interfaces/headwater-check.md`
/// states eleven reasons for exit 1 under *"There is no third status"*.
#[test]
fn a_command_line_that_names_one_target_twice_is_refused() {
    for (name, base) in both_spellings() {
        let mut arguments: Vec<&str> = base.iter().map(String::as_str).collect();
        arguments.extend(["--json", "--format", "json"]);
        let refused = ran(&arguments);
        assert_eq!(
            refused.code,
            Some(1),
            "`{name} --json --format json` is refused with exit 1: {refused:?}"
        );
        let says = String::from_utf8_lossy(&refused.err);
        assert!(
            says.contains("--json") && says.contains("--format"),
            "the refusal names both spellings: {says}"
        );
        assert!(
            refused.out.is_empty(),
            "and it writes no half-artifact: {}",
            refused.text()
        );
    }
}

/// Every command line that refuses under a JSON target, in both spellings.
///
/// Ten refusals and the four the second spelling reaches. Each one is decided
/// before anything is written: a target this corpus does not carry, a flag
/// with no value, a rung the ladder does not name, a pair of flags the parser
/// holds in conflict, or a choice the engine will not make for a caller.
///
/// `check` is deliberately absent past the parse conflict, because five of its
/// eleven exit-1 reasons are decided *after* the report is on standard output.
/// [`a_run_that_completed_and_then_failed_still_wrote_its_document`] holds that
/// half, and the pair of them is the boundary rather than either alone.
fn refusals() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "explain --json, no such document",
            vec!["explain", "--json", "docs/spec/no-such-part.md"],
        ),
        ("explain --json, no target", vec!["explain", "--json"]),
        (
            "gate --json, no such read set",
            vec!["gate", "--json", "--read-set", "no-such.readset"],
        ),
        ("gate --json, no read set", vec!["gate", "--json"]),
        (
            "conformance --json, no such rung",
            vec!["conformance", "--json", "--level", "L99"],
        ),
        ("route --json, no task", vec!["route", "--json"]),
        (
            "sweep report --json, no such file",
            vec!["sweep", "report", "no-such.yml", "--json"],
        ),
        (
            "sweep report --format json, no such file",
            vec!["sweep", "report", "no-such.yml", "--format", "json"],
        ),
        ("export --json --check", vec!["export", "--json", "--check"]),
        (
            "export --format json --check",
            vec!["export", "--format", "json", "--check"],
        ),
        // This corpus declares `default` and `site`, so a target named with no
        // profile is the two-or-more-profiles refusal and needs no scratch
        // corpus. `both_spellings()` passes `--profile site` for that reason.
        ("export --json, two profiles", vec!["export", "--json"]),
        (
            "export --format json, two profiles",
            vec!["export", "--format", "json"],
        ),
        (
            "check --json --format json",
            vec!["check", "--json", "--format", "json"],
        ),
        (
            "capture --json --format json",
            vec!["capture", "--json", "--format", "json"],
        ),
    ]
}

/// A refusal is an English sentence on standard error and never a document.
///
/// [HW-DR-0043](../../../../docs/decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md)
/// rules that `--json` names the shape of an artifact and moves neither the
/// stream a refusal is written on nor the grammar it is written in. A consumer
/// that reads standard output on exit 1 therefore reads nothing, and the
/// account is on the other stream for it to print or to log.
///
/// The way this goes wrong is a half-artifact: an emitter that opened a
/// document, wrote an opening brace and a member or two, and then met the
/// condition it refuses on. That leaves bytes on standard output that no parser
/// completes, and it is what the emptiness assertion below is for.
#[test]
fn a_refusal_writes_no_document_and_accounts_for_itself_on_the_other_stream() {
    for (name, arguments) in refusals() {
        let refused = ran(&arguments);
        assert_eq!(
            refused.code,
            Some(1),
            "`{name}` is refused with exit 1: {refused:?}"
        );
        assert!(
            refused.out.is_empty(),
            "`{name}` writes nothing to standard output, and it wrote: {}",
            refused.text()
        );
        assert!(
            !refused.err.is_empty(),
            "`{name}` says why on standard error: {refused:?}"
        );
    }
}

/// The other side of the boundary, so the case above is not read as a rule.
///
/// "Standard output is empty when the status is 1" is **false** for `check`.
/// `docs/interfaces/headwater-check.md` states it under *Exit status*: the
/// report is written before the last five of the eleven reasons are decided, so
/// a run that exits 1 for one of those five still put a whole report there.
/// Without this case, a future change that suppressed the report on any
/// non-zero exit would pass the case above and break the contract.
///
/// An unwritable read-set path is one of those five. The run evaluates the
/// corpus, writes the document, fails to record the read set, and says so on
/// standard error.
#[test]
fn a_run_that_completed_and_then_failed_still_wrote_its_document() {
    let unwritable = scratch().join("no-such-directory").join("run.readset");
    let unwritable = unwritable.to_str().expect("a path");
    for (name, arguments) in [
        (
            "check --json",
            vec!["check", "--json", "--read-set", unwritable],
        ),
        (
            "check --format json",
            vec!["check", "--format", "json", "--read-set", unwritable],
        ),
    ] {
        let failed = ran(&arguments);
        assert_eq!(
            failed.code,
            Some(1),
            "`{name}` with an unwritable read set exits 1: {failed:?}"
        );
        assert!(
            !failed.err.is_empty(),
            "`{name}` says which path it could not write: {failed:?}"
        );
        let artifact = failed.text();
        let parsed = headwater_yaml::load(&artifact)
            .unwrap_or_else(|_| panic!("`{name}` put a whole JSON report on standard output"));
        assert!(
            member(&parsed.value, "version").is_some(),
            "`{name}` wrote the whole document and not a prefix of one: {artifact}"
        );
    }
}

/// A refusal names the spelling the caller typed, and never the other one.
///
/// `chosen()` folds `--json` onto `--format json` so that the two write the
/// same artifact byte for byte, which is what
/// [`the_two_spellings_of_one_target_write_the_same_bytes`] holds. A message a
/// person reads is not an artifact, and quoting a flag that is not on the
/// command line in front of them is a wrong instruction rather than a
/// cosmetic one. `export` is the whole surface: it is the only verb whose
/// refusals are reached with a target already named.
///
/// Both directions in one case, because the way this goes wrong is that one
/// spelling is fixed and the other is left saying the first one's name.
#[test]
fn a_refusal_names_the_spelling_the_caller_typed() {
    for (name, arguments, typed, untyped) in [
        (
            "export --json --check",
            vec!["export", "--json", "--check"],
            "--json",
            "--format",
        ),
        (
            "export --format json --check",
            vec!["export", "--format", "json", "--check"],
            "--format",
            "--json",
        ),
        (
            "export --json, two profiles",
            vec!["export", "--json"],
            "--json",
            "--format",
        ),
        (
            "export --format json, two profiles",
            vec!["export", "--format", "json"],
            "--format",
            "--json",
        ),
    ] {
        let refused = ran(&arguments);
        assert_eq!(
            refused.code,
            Some(1),
            "`{name}` is refused with exit 1: {refused:?}"
        );
        let says = String::from_utf8_lossy(&refused.err);
        assert!(
            says.contains(typed),
            "`{name}` names `{typed}`, the spelling it was given: {says}"
        );
        assert!(
            !says.contains(untyped),
            "`{name}` does not name `{untyped}`, which nobody typed: {says}"
        );
    }
}

/// Every document parses, and the parser is not this system.
#[test]
fn every_document_this_binary_writes_is_read_by_a_parser_that_is_not_this_one() {
    let mut outside = 0;
    for (name, run) in documents() {
        assert!(
            !run.out.is_empty(),
            "`{name}` writes a document at all: {run:?}"
        );
        let artifact = run.text();
        headwater_yaml::load(&artifact)
            .unwrap_or_else(|errors| panic!("`{name}` does not parse in tree: {errors:?}"));
        if let Some(refusal) = oracle(&artifact) {
            panic!("`{name}` is not JSON by the clause's own reading: {refusal}");
        }
        if std::env::var_os("HEADWATER_JSON_ORACLE").is_some() {
            outside += 1;
        }
    }
    if std::env::var_os("HEADWATER_JSON_ORACLE").is_some() {
        assert_eq!(outside, 10, "every document reached the outside parser");
    }
}

/// `python3 -m json.tool` over one artifact: `None` where it read the bytes,
/// and the refusal where it did not.
///
/// Where `python3` is absent this returns `None` too, and the note says so.
/// `HEADWATER_JSON_ORACLE` turns that into a failure, so this reading cannot go
/// quiet by losing a dependency. That is the shape the SARIF validator and the
/// SHA-256 oracle already have here, and the `Test` step of CI sets all three of
/// those variables along with `HEADWATER_STOCK_VALIDATOR`.
fn oracle(artifact: &str) -> Option<String> {
    let required = std::env::var_os("HEADWATER_JSON_ORACLE").is_some();
    let written = scratch().join("artifact.json");
    std::fs::write(&written, artifact).expect("the artifact writes");
    let ran = Command::new("python3")
        .args(["-m", "json.tool", written.to_str().expect("a path")])
        .output();
    let reason = match ran {
        Ok(output) if output.status.success() => return None,
        // A stand-in that answers non-zero with nothing on its stderr is the
        // shape a shadowed interpreter takes, so the status is the reason where
        // there is no other.
        Ok(output) => match String::from_utf8_lossy(&output.stderr).trim() {
            "" => format!("it exited {}", output.status),
            said => said.to_string(),
        },
        Err(error) => error.to_string(),
    };
    // A parser that ran and refused is the finding. A parser that could not run
    // is a fact about this host, and only the first is returned as a refusal.
    if reason.contains("Expecting") || reason.contains("Invalid") || reason.contains("Extra data") {
        return Some(reason);
    }
    assert!(
        !required,
        "HEADWATER_JSON_ORACLE is set and `python3 -m json.tool` did not run, so nothing outside \
         this repository read these bytes: {reason}"
    );
    eprintln!(
        "note: `python3 -m json.tool` did not run ({reason}), so only the in-tree reader ran"
    );
    None
}

/// The JSON form selects the artifact and never the exit status.
///
/// This is what the help text of `--json` says on the four verbs that declare
/// no `--format`, and it is the half of that sentence a reader would not think
/// to check. A flag that quietly turned a voided gate into a passing one would
/// be the worst defect this change could ship.
#[test]
fn the_json_form_moves_no_exit_status() {
    let recorded = read_set();
    let recorded = recorded.to_str().expect("a path");
    let pairs: Vec<(&str, Vec<&str>)> = vec![
        ("route, offered", vec!["route", ANSWERED]),
        ("route, silent", vec!["route", UNANSWERED]),
        (
            "explain, a document",
            vec!["explain", "docs/spec/12-check-layer.md"],
        ),
        (
            "explain, nothing of that name",
            vec!["explain", "docs/spec/no-such-part.md"],
        ),
        ("gate, a read set", vec!["gate", "--read-set", recorded]),
        (
            "gate, no such file",
            vec!["gate", "--read-set", "no-such.readset"],
        ),
        ("conformance", vec!["conformance"]),
        ("conformance, a rung", vec!["conformance", "--level", "L1"]),
        (
            "conformance, no such rung",
            vec!["conformance", "--level", "L99"],
        ),
    ];
    for (name, base) in pairs {
        let plain = ran(&base);
        let mut with_flag = base.clone();
        with_flag.push("--json");
        let flagged = ran(&with_flag);
        assert_eq!(
            plain.code, flagged.code,
            "`{name}` exits alike with and without `--json`: {plain:?} against {flagged:?}"
        );
    }
}

/// No escape byte reaches a document, whatever an author wrote in a summary.
///
/// #321's colour clause asks for this to be asserted independently of any
/// terminal reading, and `engine/crates/cli/tests/width.rs` holds it in three
/// places for the surfaces that existed then. These four documents are new
/// writers, and a new writer is how that guarantee is lost.
#[test]
fn no_escape_byte_reaches_a_document_this_binary_writes() {
    for (name, run) in documents() {
        assert!(
            !run.out.contains(&0x1b),
            "`{name}` writes no escape byte on standard output"
        );
        assert!(
            !run.err.contains(&0x1b),
            "`{name}` writes no escape byte on standard error"
        );
    }
}

/// Every document this binary writes names its own shape, and not the engine's.
///
/// A consumer outside this repository holds no clone, so a document that named
/// nothing could only be pinned by the version of the tool that wrote it — and
/// two engines that write one shape should not make a reader re-read it.
///
/// **This runs over all ten entries of `documents()`, which is what
/// [#343](https://github.com/headwater-ai/headwater/issues/343) closed.** The
/// case shipped with #321 naming four of those ten. It left out the four
/// emitters that predate that change — `check`, `sweep report`, `export` and
/// `capture` — because two of them did not meet it: `export` named its shape
/// under a different key, `export_version`, and `capture` named no shape at
/// all. It also left out the second entry of `route` and of `conformance`,
/// which met it all along. Both defects are closed: `export` and `capture` now
/// write `version` like the other eight entries do.
///
/// `export` writes `export_version` beside `version`, out of the one constant,
/// because dropping the earlier key is a member removed and a major bump under
/// the rule that constant's own doc comment states.
///
/// Ten entries, eight command lines: `route` and `conformance` each write two
/// documents here.
#[test]
fn every_document_this_binary_writes_names_its_own_shape() {
    for (name, run) in documents() {
        let value = headwater_yaml::load(&run.text())
            .unwrap_or_else(|errors| panic!("`{name}` does not parse: {errors:?}"))
            .value;
        let version = member(&value, "version")
            .unwrap_or_else(|| panic!("`{name}` names its own shape in a `version` member"));
        assert_ne!(
            version,
            headwater_resolve::release::ENGINE,
            "`{name}` names its shape and not the engine that wrote it"
        );
    }
}

/// A route with nothing to offer writes an empty pointer set, not no member.
///
/// This is the member `.claude/hooks/intent.sh` decides on, and the case it
/// decides wrong if the emitter ever starts omitting empty collections.
#[test]
fn a_silent_route_writes_an_empty_pointer_set_and_says_why() {
    let silent = ran(&["route", "--json", UNANSWERED]);
    assert_eq!(silent.code, Some(0), "a silence is a result: {silent:?}");
    let artifact = silent.text();
    assert!(
        artifact.contains("\"pointers\": []"),
        "the pointer set is there and it is empty:\n{artifact}"
    );
    assert!(
        artifact.contains("\"reason\":"),
        "and the silence says which of the four it is:\n{artifact}"
    );

    let offered = ran(&["route", "--json", ANSWERED]);
    assert_eq!(offered.code, Some(0), "{offered:?}");
    assert!(
        !offered.text().contains("\"pointers\": []"),
        "a task this corpus answers offers pointers, so the case above is not vacuous"
    );
}

/// The `text` a route carries is the report the same run would have printed.
///
/// `.claude/hooks/intent.sh` hands that member to an agent, so a member that
/// drifted from the rendering would put an agent and a terminal in front of
/// two different accounts of one corpus. Byte for byte, out of two runs.
#[test]
fn the_text_a_route_carries_is_the_report_the_same_run_would_print() {
    for task in [ANSWERED, UNANSWERED] {
        let printed = ran(&["route", task]);
        let document = ran(&["route", "--json", task]);
        let value = headwater_yaml::load(&document.text())
            .expect("the route document parses")
            .value;
        let carried = member(&value, "text").expect("the route document carries `text`");
        assert_eq!(
            carried,
            printed.text(),
            "the `text` member is the report, byte for byte"
        );
    }
}

/// One scalar member of a mapping, as a string.
fn member(value: &headwater_yaml::Value, key: &str) -> Option<String> {
    value
        .as_map()
        .and_then(|map| map.get(key))
        .and_then(|spanned| spanned.value.as_scalar())
        .map(headwater_yaml::core_schema::as_str)
        .map(str::to_string)
}
