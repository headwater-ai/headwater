// SPDX-License-Identifier: Apache-2.0
//! `headwater` — three verbs of it.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#cli) lists ten.
//! This binary carries `check`, `taxonomy validate` and `taxonomy resolve`.
//!
//! # Why there is a binary at all
//!
//! [#46](https://github.com/headwater-ai/headwater/issues/46) held the question
//! open: either M1 ends with a `headwater` binary, or it ends with a test that
//! CI runs. The epic decided it. M1 is done when the loop "runs in this
//! repository's CI over `docs/`, advisory", and a test harness that runs under
//! `cargo test` cannot be advisory — a failing test fails the job, which is the
//! opposite posture. Exit status is the thing CI reads, so the thing that owns
//! the posture has to be the thing that exits.
//!
//! # Advisory is the default for `check`, and never for the taxonomy
//!
//! Spec 6: "The CLI is advisory by default (exit 0 with findings on stdout).
//! Use `--strict` for gates." That is a rule about findings over a *corpus*. A
//! taxonomy that does not validate is not a finding, and spec 2 rules that the
//! engine never applies one and that there is no partial-load mode. So
//! `taxonomy validate` exits non-zero on a refusal with no flag to soften it,
//! and `check` stays advisory over the documents it reads.
//!
//! # Where the taxonomy comes from, and why it is not the sources
//!
//! From the lock. Spec 6: "Everything downstream reads the lock, never the
//! sources. Thus a check result depends on a hash that a reviewer can see in a
//! diff." `check` therefore reads `.headwater/taxonomy.lock` and refuses to run
//! without one. `taxonomy resolve` is what writes it, and it writes one only
//! when every rule of `taxonomy validate` passes — so the artifact carries the
//! no-partial-load rule rather than a call that a caller may forget.
//!
//! This is what [#51](https://github.com/headwater-ai/headwater/issues/51)
//! changed. Until it landed, `check` resolved the sources on every run, so a
//! verdict rested on files that nothing had hashed and a reviewer read three
//! sources to see what one run used.
//!
//! # The cache is beside the lock, and it changes nothing a reader sees
//!
//! `check` keeps a cache at `.headwater/cache/checks`, keyed on the lock
//! digest among the components
//! [spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
//! names. It is not committed and it holds no corpus content: every entry is
//! recomputable from the tree it was written over.
//!
//! `--no-cache` is the differential that spec 12 asks for rather than a way
//! out of a bad cache. The two runs write the same bytes to standard output,
//! and the standing test in `crates/check/tests/cache.rs` is that comparison
//! under `cargo test`. What the cached run writes to standard error is the hit
//! count, which belongs there because it is a fact about a disk.

use headwater_adapter::{Format, Subject};
use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Shape};
use headwater_cli::{JsonWord, ProbeWord, SweepWord, TaxonomyWord, Verb};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::mcp::Written;
use headwater_query::{Budget, Surface};
use headwater_resolve::render_errors;
// The graph reads have a `Surface` of their own, and this one is the entry
// point a run of the authoring verb was made at. Two different subjects, so the
// name that is more precise here is the one this file uses.
use headwater_scaffold::reading::Surface as EntryPoint;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    // `clap` exits **2** on a parse error, and this binary has one failing
    // status and it is 1: `docs/interfaces/headwater-check.md` lists eleven
    // reasons for it under "There is no third status", and a 2 anywhere makes
    // that sentence false. `Command` exposes no setting for the error exit
    // code, so the only route is `try_parse` and never letting `clap` call
    // `exit` itself. That is one site rather than a rule each error path keeps.
    //
    // `use_stderr()` is what carries the other half. It is false for exactly
    // the two errors that are answers rather than mistakes — `--help`, and
    // `--version` for a `clap` that owns it — and `Error::print` already routes
    // those to standard output. Returning `FAILURE` unconditionally here would
    // turn `--help` into a failure, and printing to the wrong stream would
    // break the zero-bytes-on-standard-error guarantee that
    // `tests/wiring.rs` holds for both `--help` and `--version`.
    // The masthead is printed here, ahead of `clap`'s own help writer, rather
    // than inside the template `first_screen` builds: `paint::wants_root_help`
    // and `paint::banner`'s doc comments say why a template cannot carry it.
    // `-h`/`--help` at the root is answered inside `parsed()` below, before
    // this function ever sees a `Cli`, so the masthead has to print before
    // that call rather than after it.
    if headwater_cli::paint::wants_root_help() {
        print!(
            "{}",
            headwater_cli::paint::banner(
                headwater_resolve::release::ENGINE,
                headwater_cli::paint::stdout_color()
            )
        );
    }

    let cli = match headwater_cli::parsed() {
        Ok(cli) => cli,
        Err(error) => match error.use_stderr() {
            true => return fail(&headwater_cli::headline(&error)),
            false => {
                let _ = error.print();
                return ExitCode::SUCCESS;
            }
        },
    };

    // The value is `headwater_resolve::release::ENGINE` and not a fourth
    // `env!("CARGO_PKG_VERSION")`. That constant is what a `requires_engine`
    // range is compared against, so the number a caller reads here is the
    // number that decides whether a package loads, rather than a second string
    // that agrees with it today because every crate declares
    // `version.workspace = true`. The engine has already paid for the other
    // shape: it advertised the placeholder `0.0.0` as `serverInfo.version` over
    // MCP and the value sat wrong through five milestones, because a value
    // exactly one surface reports is a value nobody audits.
    //
    // It is read before `let root` below, so a caller with no corpus and no
    // `--root` still gets an answer, and it is declared global so that it works
    // after a verb as well as before one.
    if cli.version {
        println!("{}", headwater_resolve::release::ENGINE);
        return ExitCode::SUCCESS;
    }

    let root = match cli.root {
        Some(path) => path,
        None => match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => return refuse(&format!("no working directory: {error}")),
        },
    };

    let Some(verb) = cli.verb else {
        return fail("no verb. Try `headwater check`");
    };

    dispatch(&root, verb)
}

/// One command line to one verb.
///
/// # The dispatch table is `headwater_verbs::VERBS`, and this is no longer
/// where it decides
///
/// It was, until [HW-DR-0033](../../../../docs/decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md).
/// `main` resolved the first word against the table before it entered a
/// `match verb.as_slice()` of string patterns, so an arm the table did not
/// carry never ran, and `tests/verbs.rs` scraped the arms out of this file as
/// source text to close the other direction. The patterns are gone: the arms
/// below are variants of `headwater_cli::Verb`, which `clap` builds a command
/// tree from, and that tree is what `tests/verbs.rs` now holds against the
/// table in both directions. A name in either one and not the other fails, and
/// the failure names it.
///
/// # What is still decided here, and why it is not `clap`'s
///
/// Every refusal below states what a caller may type next, out of
/// `headwater_verbs`. `clap` would answer an unknown first word with
/// `unrecognized subcommand` and at most one near miss, and this binary
/// enumerates the whole set — which is what [#257](https://github.com/headwater-ai/headwater/issues/257)
/// asked for, and why each level declares an external-subcommand form rather
/// than letting the parse refuse. A required operand is optional to the parser
/// for the same reason: `headwater gate` names what a read set is and how to
/// produce one, which a missing-argument message cannot.
fn dispatch(root: &Path, verb: Verb) -> ExitCode {
    match verb {
        Verb::Check {
            strict,
            fix,
            no_cache,
            now,
            change,
            read_set,
            register,
            format,
            json,
        } => check(
            root,
            Asked {
                strict,
                cached: !no_cache,
                fixing: fix,
                now,
                read_set,
                register_out: register,
                format: chosen(json, format),
                change,
            },
        ),
        Verb::Gate {
            read_set,
            now,
            json,
        } => gate(root, read_set, now, json),
        Verb::Route { task, budget, json } => match task.is_empty() {
            true => fail("`route` takes a task description. Try `headwater route \"add rate limiting to the ingest API\"`"),
            false => route(root, &task.join(" "), budget, json),
        },
        Verb::Explain { target, json } => match target {
            None => fail("`explain` takes a path or an identifier"),
            Some(target) => explain(root, &target, json),
        },
        Verb::Mcp { now, write } => mcp(root, now, write),
        Verb::New {
            kind,
            title,
            relates,
            facet,
            now,
        } => match kind {
            None => fail(
                "`new` takes a kind. Try `headwater new decision --title \"Adopt an overlay\"`",
            ),
            Some(kind) => new(root, &kind, title, &relates, &facet, now),
        },
        Verb::Capture { format, json } => capture(root, chosen(json, format)),
        Verb::Sweep { word } => match word {
            None => fail(&format!(
                "`sweep` takes a second word: {}",
                headwater_verbs::words_of("sweep")
            )),
            Some(SweepWord::Plan { under }) => sweep_plan(root, under),
            Some(SweepWord::Report { path, format, json }) => match path {
                None => fail(
                    "`sweep report` takes the path of the file an agent wrote back. \
                     `headwater sweep plan` prints the shape of it",
                ),
                Some(path) => sweep_report(root, Path::new(&path), chosen(json, format)),
            },
            Some(SweepWord::Other(words)) => no_such_second_word("sweep", &words),
        },
        Verb::Probe { word } => match word {
            None => fail(&format!(
                "`probe` takes a second word: {}",
                headwater_verbs::words_of("probe")
            )),
            Some(ProbeWord::Plan {
                tier,
                arm,
                category,
                seed,
            }) => probe_plan(
                root,
                tier.as_deref(),
                arm.as_deref(),
                category.as_deref(),
                seed,
            ),
            Some(ProbeWord::Record { path }) => match path {
                None => fail(
                    "`probe record` takes the path of a transcript a recorder wrote. \
                     `headwater probe plan` prints the shape of it",
                ),
                Some(path) => probe_record(root, Path::new(&path)),
            },
            Some(ProbeWord::Grade { path }) => match path {
                None => fail(
                    "`probe grade` takes the path of a transcript a recorder wrote. It grades that \
                     transcript against the probes this corpus declares",
                ),
                Some(path) => probe_grade(root, Path::new(&path)),
            },
            Some(ProbeWord::Stale) => probe_stale(root),
            Some(ProbeWord::Other(words)) => no_such_second_word("probe", &words),
        },
        Verb::Generate { check } => generate(root, check),
        Verb::Import {
            name,
            expect,
            write,
        } => import(root, name.as_deref(), expect.as_deref(), write),
        Verb::Export {
            profile,
            format,
            at,
            check,
            json,
        } => export(root, profile, chosen(json, format), typed(json), at, check),
        Verb::Init { corpus, package } => init(root, corpus, package),
        Verb::Infer {
            owner,
            until,
            write,
            now,
        } => infer(root, owner, until, write, now),
        Verb::Conformance { level, now, json } => {
            conformance(root, level.as_deref(), now, json)
        }
        // Spec 6 lists this verb and no document of the specification states
        // what an expression is. The engine names the gap rather than invent a
        // form, which is the posture the resolver takes over a `$package`
        // reference for the same reason.
        //
        // `refuse` rather than `fail` (#455): no spelling of "run a query"
        // gets past this, and `query` is a declared member of `VERBS`, so the
        // grammar `fail` would point at lists it and repeats the sentence
        // above.
        Verb::Query { .. } => refuse(
            "`query <expression>` is listed in spec 6 and no document states what an expression \
             is, so this engine implements none. See `docs/spec/13-open-obligations.md`. \
             `headwater route` and `headwater explain` are the reads that exist",
        ),
        Verb::Json { word } => match word {
            None => fail(&format!(
                "`json` takes a second word: {}",
                headwater_verbs::words_of("json")
            )),
            Some(JsonWord::Field { path }) => match path.is_empty() {
                true => fail(
                    "`json field` takes the path of keys to a member. Try \
                     `headwater json field tool_input file_path`",
                ),
                false => json_field(&path),
            },
            Some(JsonWord::Count { path }) => json_count(&path),
            Some(JsonWord::Quote) => json_quote(),
            Some(JsonWord::Other(words)) => no_such_second_word("json", &words),
        },
        Verb::Help { verb } => print_help_for(&verb),
        Verb::Completions { shell } => completions(shell),
        Verb::Taxonomy { word } => match word {
            None => fail(&format!(
                "`taxonomy` takes a second word: {}",
                headwater_verbs::words_of("taxonomy")
            )),
            Some(TaxonomyWord::Validate) => validate(root),
            Some(TaxonomyWord::Resolve { check }) => resolve(root, check),
            Some(TaxonomyWord::Audit { now, record }) => audit(root, now, record),
            Some(TaxonomyWord::Publish {
                package,
                from,
                assembly,
                out,
                json,
            }) => {
                publish(
                    root,
                    package.as_deref(),
                    from.as_deref(),
                    assembly.as_deref(),
                    out.as_deref(),
                    json,
                )
            }
            Some(TaxonomyWord::Vendor { path, expect }) => match path {
                None => fail(
                    "`taxonomy vendor` takes the path of a package somebody already fetched. \
                     This engine opens no socket, so it checks a directory it is handed",
                ),
                Some(path) => vendor(root, Path::new(&path), expect.as_deref()),
            },
            Some(TaxonomyWord::Diff { path, to, now }) => match path {
                None => fail(
                    "`taxonomy diff` takes the path of a published artifact somebody already \
                     fetched. This engine opens no socket, so it compares against a directory it \
                     is handed, and `--to <version>` states which version that directory is \
                     expected to be",
                ),
                Some(path) => diff(root, Path::new(&path), to.as_deref(), now),
            },
            Some(TaxonomyWord::Migrate {
                path,
                to,
                apply,
                now,
            }) => match path {
                None => fail(
                    "`taxonomy migrate` takes the path of a published artifact somebody already \
                     fetched. This engine opens no socket, so it applies a payload it is handed, \
                     and `--to <version>` states which version that directory is expected to be. \
                     Without `--apply` it reports what it would write and writes nothing",
                ),
                Some(path) => migrate(root, Path::new(&path), to.as_deref(), now, apply),
            },
            Some(TaxonomyWord::Other(words)) => fail(&format!(
                "`taxonomy {}` is not a verb this binary carries yet. It carries {}",
                words.first().map(String::as_str).unwrap_or_default(),
                headwater_verbs::words_of("taxonomy")
            )),
        },
        // The message keeps its wording. It names the first word of the command
        // line, so `headwater chekc` answers with every word this binary does
        // carry rather than with the one `clap` thought was closest.
        Verb::Other(words) => fail(&format!(
            "`{}` is not a verb this binary carries yet. It carries {}",
            words.first().map(String::as_str).unwrap_or_default(),
            headwater_verbs::listed()
        )),
    }
}

/// `--json`, resolved against `--format`.
///
/// # Why this is a spelling and not a second switch
///
/// [#321](https://github.com/headwater-ai/headwater/issues/321) asks that
/// "`--json` is accepted wherever `--format json` already is". So the two names
/// reach one value here, and every verb below receives the `--format` it always
/// received. **No artifact downstream of this function can tell which name a
/// caller typed**, which is the property that makes the two artifacts
/// byte-identical rather than merely similar, and
/// `engine/crates/cli/tests/json.rs` holds it under
/// `the_two_spellings_of_one_target_write_the_same_bytes`.
///
/// **A refusal is not an artifact, and it does name the spelling.** A sentence
/// that quotes `--format` at somebody who wrote `--json` sends them to a flag
/// that is not on their command line, which is a wrong instruction rather than
/// an untidy one. [`typed`] carries the name a caller wrote past this
/// substitution rather than through it, so the fold above is untouched. See
/// `docs/decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md`.
///
/// **`json` is true only where `--format` was absent**, because the parser
/// declares the two in conflict. So the arm below is a substitution and never a
/// precedence rule over a value a caller stated. A precedence rule is how a
/// flag comes to do nothing silently, which is what
/// [#337](https://github.com/headwater-ai/headwater/issues/337) and
/// [#338](https://github.com/headwater-ai/headwater/issues/338) are open about,
/// and `engine/crates/cli/tests/json.rs` holds the refusal on every verb that
/// declares both.
fn chosen(json: bool, format: Option<String>) -> Option<String> {
    match json {
        true => Some("json".to_string()),
        false => format,
    }
}

/// The spelling of the JSON target a caller wrote, for a message they read.
///
/// # Why this sits beside [`chosen`] and never inside it
///
/// [`chosen`] folds the two names onto one value so that the artifact is the
/// same bytes under either name. That fold is the whole point of it and it
/// stays. A refusal is not an artifact, and the two properties are separate:
/// one is about bytes on standard output, the other is about a sentence on
/// standard error.
///
/// The two travel as different types — `Option<String>` for the target and
/// `&'static str` for the name — so a site that reached for one and got the
/// other would not compile. That is what keeps a later edit from conflating
/// them again.
///
/// `wide_refusal` in `engine/crates/cli/src/lib.rs` already makes this
/// discrimination for `--wide`, and it reads the raw `ArgMatches` because its
/// refusal fires before dispatch. Here [`dispatch`] already holds `json` and
/// `format` apart at every arm, so nothing needs a second read of the matches.
///
/// **`export` is the whole surface.** It is the only verb whose refusals are
/// reached with a target already named, and both of them are in it. The four
/// other `--format` literals in this file fire either on a target that is
/// neither `text` nor `json`, which `--json` cannot produce, or in the branch
/// that runs when no target was named at all.
const fn typed(json: bool) -> &'static str {
    match json {
        true => "--json",
        false => "--format",
    }
}

/// A second word one of the three grouped verbs does not carry.
///
/// `taxonomy` says "yet" where these two do not, and the difference is old
/// enough to be a promise nobody made. Both forms are preserved rather than
/// unified here, because a message a caller reads is not this change's subject.
/// `headwater help`, `headwater help <verb>` and `headwater help <verb> <word>`.
///
/// # Why this is a verb of this binary rather than the one `clap` injects
///
/// `clap` adds a `help` subcommand of its own during `Command::build()`, and it
/// adds a copy of the whole command tree underneath it: `headwater help sweep
/// plan`, `headwater probe help grade`, forty-four command lines in all. The
/// dispatch table carries none of them, so
/// `engine/crates/cli/tests/verbs.rs` would report every one — and the only way
/// to keep the injected subcommand is to write an exclusion into that walk,
/// which is the guard trading itself for a feature. One variant with one
/// positional adds `headwater help` and nothing else, and the walk stays exact.
///
/// # It prints the same bytes the flag prints
///
/// `headwater help check`, `headwater check --help` and `headwater check -h`
/// render one command of one tree, byte for byte, and
/// `engine/crates/cli/tests/help.rs` asserts that rather than asserting each
/// carries something. Two declarations are what make it hold: no `long_about`
/// and no `long_help` exists anywhere in this parser, so the two spellings of
/// the flag render one text, and this prints through `print_help` rather than
/// `print_long_help` for the same reason — the long renderer puts a blank line
/// between arguments and the flag does not. That is what clause 4 of
/// [#321](https://github.com/headwater-ai/headwater/issues/321) asks for,
/// stated as one route rather than three.
fn print_help_for(words: &[String]) -> ExitCode {
    let mut command = headwater_cli::command();
    command.build();

    // The refusal first, with shared borrows, so the descent below is known to
    // land. Each failure names the word the caller got wrong and what may
    // stand in that position, which is what every other refusal here does.
    let mut cursor = &command;
    for (at, word) in words.iter().enumerate() {
        let Some(next) = cursor.find_subcommand(word.as_str()) else {
            if at == 0 {
                return fail(&format!(
                    "`{word}` is not a verb this binary carries yet. It carries {}",
                    headwater_verbs::listed()
                ));
            }
            let verb = words[at - 1].as_str();
            return match headwater_verbs::words_of(verb).is_empty() {
                true => fail(&format!("`{verb}` takes no second word")),
                false => no_such_second_word(verb, &words[at..]),
            };
        };
        cursor = next;
    }

    // `headwater help` with no operand prints the same root screen `--help`
    // does, masthead included — `wants_root_help` does not see this path,
    // because there is no `-h`/`--help` token on a bare `help`.
    if words.is_empty() {
        print!(
            "{}",
            headwater_cli::paint::banner(
                headwater_resolve::release::ENGINE,
                headwater_cli::paint::stdout_color()
            )
        );
    }
    let target = descend(&mut command, words).expect("the walk above found every word");
    let _ = target.print_help();
    ExitCode::SUCCESS
}

/// The completion script of one shell, on standard output.
///
/// # The tree the script is written from is the tree that parses
///
/// `clap_complete` walks a [`clap::Command`], and the one handed to it is
/// [`headwater_cli::command`] — the same tree `main` parses with, the same tree
/// `headwater help <verb>` prints out of, and the same tree
/// `engine/crates/cli/tests/verbs.rs` walks against
/// [`headwater_verbs::VERBS`]. So a verb, a second word or a flag added to the
/// parser reaches every completion script with no edit here, and a script that
/// offered a verb this binary does not carry would be a discrepancy that walk
/// already fails on.
///
/// # A script carries no layout of this engine's
///
/// The property a caller depends on is that the bytes do not move with the
/// terminal of whoever asked for them, and
/// `engine/crates/cli/tests/completions.rs` asserts exactly that. This function
/// holds it in two steps. It builds at [`headwater_cli::paint::WIDTH`] rather
/// than at `paint::width()`, so no reading of `COLUMNS` reaches the tree; the
/// two are the same number on every run that gets here, because `--wide` is
/// refused on a run that prints no help and this run prints none. Then
/// [`headwater_cli::paint::flattened`] puts every string back on one line, so
/// no fold reaches the script either.
///
/// The second step is what a shell needs. `clap_complete` writes a `zsh`
/// positional as `'::name -- <help>:<action>'` and puts the help in the quotes
/// as the tree carries it, so a folded help string arrives as a description
/// broken across lines at a width the shell did not choose. A shell lays a
/// completion listing out itself and is the only party entitled to. Flattening
/// a folded string returns the source string, so the flag and subcommand
/// descriptions are unchanged and the positionals stop carrying a break.
///
/// # Standard output, and nothing else
///
/// Clause 11 of [#321](https://github.com/headwater-ai/headwater/issues/321)
/// holds that no escape byte reaches a machine artifact. A completion script is
/// a fifth such surface, and `clap_complete` writes to the sink it is handed
/// and nowhere else. `engine/crates/cli/tests/completions.rs` holds the two
/// halves that matter to a caller redirecting this into a file: standard error
/// is empty, and no `\x1b` byte is on either stream.
fn completions(shell: Option<headwater_cli::Shell>) -> ExitCode {
    let Some(shell) = shell else {
        let named: Vec<&str> = headwater_cli::Shell::ALL
            .iter()
            .map(|one| one.typed())
            .collect();
        return fail(&format!(
            "`completions <shell>` writes a completion script on standard output, for one of {}. \
             Where the script goes is the shell's own convention rather than this engine's, so \
             redirect it there: `{} completions bash > f && . f` loads one into the shell in \
             front of you",
            listed(&named),
            headwater_verbs::BINARY
        ));
    };
    let mut command =
        headwater_cli::paint::flattened(headwater_cli::command_at(headwater_cli::paint::WIDTH));
    clap_complete::generate(
        clap_complete::Shell::from(shell),
        &mut command,
        headwater_verbs::BINARY,
        &mut std::io::stdout(),
    );
    ExitCode::SUCCESS
}

/// `` `a`, `b` and `c` ``: the form every message in this binary uses.
///
/// `headwater_verbs::listed` writes the verbs this way and its `join` is
/// private to that crate, so this is the same rendering over a set that crate
/// does not carry. The shells are the parser's, not the dispatch table's.
fn listed(words: &[&str]) -> String {
    let quoted: Vec<String> = words.iter().map(|word| format!("`{word}`")).collect();
    match quoted.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

/// Standard input, whole, or `None` where it is not text.
///
/// The three `json` words each read one message, so this reads to the end
/// rather than by line. A harness writes one object and closes the stream.
fn stdin_text() -> Option<String> {
    use std::io::Read;
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).ok()?;
    Some(text)
}

/// `headwater json field <key>...`.
///
/// The artifact is the member, on standard output, with a newline after it. A
/// shell substitution strips that newline, and a person reading the value at a
/// terminal gets a line rather than a value with the prompt against it.
///
/// The refusal says which path reached nothing and stops there. Every way of
/// reaching no scalar is one answer, which `headwater_yaml::json::field`
/// states the reason for: a caller that told them apart would be acting on the
/// shape of a message it did not write.
fn json_field(path: &[String]) -> ExitCode {
    let Some(text) = stdin_text() else {
        return refuse("standard input is not text, so no JSON object was read from it");
    };
    match headwater_yaml::json::field(&text, path) {
        Some(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        None => refuse(&format!(
            "`{}` reaches no scalar of the object on standard input",
            path.join(".")
        )),
    }
}

/// `headwater json count [<key>...]`.
///
/// With no key it counts the object on standard input itself, which is the
/// reading a caller wants when the message is the collection.
fn json_count(path: &[String]) -> ExitCode {
    let Some(text) = stdin_text() else {
        return refuse("standard input is not text, so no JSON object was read from it");
    };
    match headwater_yaml::json::count(&text, path) {
        Some(count) => {
            println!("{count}");
            ExitCode::SUCCESS
        }
        None => refuse(&format!(
            "`{}` reaches no array and no object of the object on standard input",
            match path.is_empty() {
                true => ".".to_string(),
                false => path.join("."),
            }
        )),
    }
}

/// `headwater json quote`.
///
/// The artifact is one JSON string literal and nothing else, so a caller can
/// put it straight into the object it writes back to a harness. No newline
/// follows it, because a literal is a fragment of a message rather than a
/// message.
fn json_quote() -> ExitCode {
    let Some(text) = stdin_text() else {
        return refuse("standard input is not text, so nothing was quoted");
    };
    print!("{}", headwater_yaml::json::Json::string(text).render());
    ExitCode::SUCCESS
}

/// The command a sequence of words names, or `None` for a word that names none.
fn descend<'a>(command: &'a mut clap::Command, words: &[String]) -> Option<&'a mut clap::Command> {
    match words.split_first() {
        None => Some(command),
        Some((word, rest)) => descend(command.find_subcommand_mut(word.as_str())?, rest),
    }
}

fn no_such_second_word(verb: &str, words: &[String]) -> ExitCode {
    fail(&format!(
        "`{verb} {}` is not a verb this binary carries. It carries {}",
        words.first().map(String::as_str).unwrap_or_default(),
        headwater_verbs::words_of(verb)
    ))
}

/// `headwater taxonomy validate`.
///
/// It prints what it ran as well as what it found. A verdict with no statement
/// of what was checked is the silent pass that
/// [spec 4](../../../../docs/spec/04-assurance-model.md) exists to remove, and
/// eight of the rules on spec 2's list decide only part of what that list says.
fn validate(root: &Path) -> ExitCode {
    let repository = match headwater_resolve::repository(root) {
        Ok(repository) => repository,
        Err(errors) => {
            eprintln!("headwater: {}", err("the taxonomy did not resolve"));
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };

    println!("sources, in application order");
    for source in &repository.resolution.sources {
        println!("  {source}");
    }

    // Between the sources and the rules, because a founding is a fact about the
    // sources and the order they were applied in, and not about the resolved
    // taxonomy that the rules read. It prints at zero for the reason
    // `rules::render` prints its own count: a block that vanished when it
    // emptied would leave a reader unable to tell a quiet corpus from a reading
    // nobody ran. Nothing here refuses, and `founded.rs` carries why.
    println!();
    print!("{}", repository.resolution.foundings());

    // Beside the founding block and for the same reason: a reading of the
    // resolved taxonomy that refuses nothing, printed where a reader of the
    // verdict meets it. #538 is the issue, and `rules::display_names` carries
    // why a shelf with no display name is a decision rather than a defect.
    println!();
    print!(
        "{}",
        headwater_resolve::rules::display_names(&repository.resolution.taxonomy)
    );

    let findings = repository.resolution.validate();
    println!("\nrules");
    print!("{}", headwater_resolve::rules::render());

    if findings.is_empty() {
        println!("\n{} is valid", repository.consumer.package);
        return ExitCode::SUCCESS;
    }
    println!("\n{} is not valid", repository.consumer.package);
    eprint!("{}", indent(&err(&render_errors(&findings))));
    advise(root, &repository.consumer);
    ExitCode::FAILURE
}

/// Which bundle would have completed an incomplete selection, under the refusal
/// that reported it as a list of names nothing declares.
///
/// It prints nothing at all unless the refusal is that one and a bundle the
/// package ships would answer it, which is
/// [`headwater_resolve::selection::advice`]'s whole contract. It carries no
/// finding of its own and it moves no exit status: a refusal is what refuses,
/// and this is what a reader does about it.
///
/// Uncolored, under a red block, because it is guidance rather than a finding.
fn advise(root: &Path, consumer: &headwater_resolve::Consumer) {
    if let Some(advice) = headwater_resolve::selection::advice(root, consumer) {
        eprint!("{}", indent(&advice.render()));
    }
}

/// The same guidance for a publisher whose recipe named an incomplete selection.
///
/// The publisher's position is the consumer's position one step earlier, and
/// until [#582](https://github.com/headwater-ai/headwater/issues/582) they never
/// reached it: the publish exited 0 and the adopter met the refusal. Now the
/// publish refuses, and this is what the publisher reads under it.
///
/// It carries [`advise`]'s contract exactly. It prints nothing unless the
/// refusal was an incomplete bundle selection and a bundle the package ships
/// would answer it, it carries no finding, and it moves no exit status. So every
/// step here is best-effort: a `--package` that names nothing, a directory with
/// no manifest, a recipe that does not read — each one prints nothing, because
/// the reader is already holding the refusal that says what went wrong and a
/// second complaint about reading it again is noise.
fn advise_recipe(root: &Path, package: Option<&str>, from: Option<&Path>, assembly: Option<&str>) {
    let Some(assembly) = assembly else {
        return;
    };
    let directory = match from {
        Some(directory) => directory.to_path_buf(),
        None => {
            let name = match package {
                Some(name) => name.to_string(),
                None => match headwater_resolve::package::consumer(root) {
                    Ok(consumer) => consumer.package,
                    Err(_) => return,
                },
            };
            match headwater_resolve::package::located(root, &name) {
                Some((directory, _)) => directory,
                None => return,
            }
        }
    };
    if let Some(advice) = headwater_resolve::selection::for_recipe(root, &directory, assembly) {
        eprint!("{}", indent(&advice.render()));
    }
}

/// `headwater taxonomy resolve`, and `--check` over a committed lock.
fn resolve(root: &Path, check_only: bool) -> ExitCode {
    let repository = match headwater_resolve::repository(root) {
        Ok(repository) => repository,
        Err(errors) => {
            eprintln!(
                "headwater: {}",
                err("the taxonomy did not resolve, so no lock is possible")
            );
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    // One call site for both the write path and `--check`, placed before any
    // lock work so that neither can reach a verdict without it. Standard error,
    // on the precedent this file states for the hit count: a fact about a run
    // goes there so that a byte comparison of the verdict is untouched, and
    // `--check`'s one line of standard output stays one line. Only when the
    // count is non-zero, because a resolve that founds nothing has nothing to
    // say and `taxonomy validate` is where the accounting always prints.
    if !repository.resolution.founded.is_empty() {
        eprint!("{}", repository.resolution.foundings());
    }
    let sources = match headwater_resolve::package::sources(root, &repository.consumer) {
        Ok(sources) => sources,
        Err(errors) => {
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    // The payload is the one authored part of the lock, so a resolve reads it
    // off the committed file and writes it back. A resolve that dropped it
    // would delete an adopter's accounting as a side effect of a taxonomy edit,
    // and the run after it would report every pair the payload was holding.
    let authored = headwater_lock::authored_at(root);
    let text = match headwater_lock::write(
        &repository.consumer.package,
        &repository.consumer.version,
        &sources,
        &repository.resolution,
        authored.payload(),
    ) {
        Ok(text) => text,
        Err(findings) => {
            eprintln!(
                "headwater: {}",
                err(
                    "the taxonomy does not validate, so no lock is written. A lock is a \
                     validated taxonomy or it is nothing"
                )
            );
            eprint!("{}", indent(&err(&render_errors(&findings))));
            advise(root, &repository.consumer);
            return ExitCode::FAILURE;
        }
    };

    let path = root.join(headwater_lock::LOCK);
    if check_only {
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        // Which half of the file moved, rather than whether the file moved. The
        // two halves have opposite remedies and the byte comparison could name
        // neither: a person who quotes one scalar of the `adoption` block was
        // told their sources had moved, under a list of moved sources that was
        // empty, out of the one block this file's own header invites them to
        // edit.
        return match headwater_lock::diverged(&committed, &text) {
            headwater_lock::Divergence::Same => {
                println!("{} is what the sources resolve to", headwater_lock::LOCK);
                ExitCode::SUCCESS
            }
            headwater_lock::Divergence::Form { adoption } => {
                eprintln!(
                    "headwater: {}",
                    err(&format!(
                        "{} carries the taxonomy its sources resolve to, and is not written \
                         in the form `headwater taxonomy resolve` writes it. Nothing about \
                         your sources changed",
                        headwater_lock::LOCK
                    ))
                );
                match adoption {
                    true => eprintln!(
                        "  {}",
                        err(
                            "The `adoption` block is where the two differ. That block is \
                             authored, and this file's header invites a person to edit it, so \
                             what moved is its form and not the debt it declares. Run \
                             `headwater taxonomy resolve`: every task, owner, expiry and pair \
                             is carried through"
                        )
                    ),
                    false => eprintln!(
                        "  {}",
                        err(
                            "The difference is not inside the `adoption` block. Run `headwater \
                             taxonomy resolve` and commit the result"
                        )
                    ),
                }
                ExitCode::FAILURE
            }
            // A lock whose generated half moved is the case this verb was
            // written for.
            headwater_lock::Divergence::Generated => {
                eprintln!(
                    "headwater: {}",
                    err(&format!(
                        "{} is not what the sources resolve to. Run `headwater taxonomy \
                         resolve` and commit the result",
                        headwater_lock::LOCK
                    ))
                );
                // The lock that is there says which source moved, which is the
                // line an author acts on.
                if let Ok(lock) = headwater_lock::read(&committed) {
                    for moved in lock.moved(root) {
                        eprintln!("  {moved} has changed since the lock was written");
                    }
                }
                ExitCode::FAILURE
            }
            // A lock that will not read says nothing about its sources, so this
            // run says nothing about them either. The remedy splits, because
            // `resolve` refuses one of these two states and repairs the other,
            // and the state is already read: `Authored::Opaque` is exactly the
            // one the write path below returns on.
            headwater_lock::Divergence::Unreadable(why) => {
                eprintln!(
                    "headwater: {}",
                    err(&format!("{} did not read: {why}", headwater_lock::LOCK))
                );
                match &authored {
                    headwater_lock::Authored::Opaque { .. } => eprintln!(
                        "  Nothing can be seen of its adoption block, so `headwater taxonomy \
                         resolve` refuses this file rather than replacing it. Repair the file, \
                         or delete it to resolve from the sources alone and write the block again"
                    ),
                    _ => eprintln!(
                        "  Nothing here can say whether a source moved, because the file that \
                         records them will not read. Run `headwater taxonomy resolve` and commit \
                         the result. The digest covers the resolution and has never covered the \
                         adoption block, so that block is carried through"
                    ),
                }
                ExitCode::FAILURE
            }
        };
    }

    // This is the write path, so this is where the authored block is decided
    // rather than read. Every state names the line the run will print, and the
    // one state in which nothing is known about the block returns instead: a
    // rewrite there replaces a file whose authored half this engine cannot see,
    // and it would discard an owner, an expiry and every pair, without being
    // able to say what it discarded. The other five proceed and report, because
    // the caller was sent here by another verb's remedy and reads exit 0 as
    // "nothing happened but success".
    let note = match &authored {
        headwater_lock::Authored::NoLock => {
            "no lock was there, so there was no adoption block to carry".to_string()
        }
        headwater_lock::Authored::Nothing => {
            "the lock that was there declared no adoption block".to_string()
        }
        headwater_lock::Authored::Payload(payload) => carried(payload),
        headwater_lock::Authored::Salvaged { why, payload } => format!(
            "{}, out of a lock that did not read. The digest covers the resolution \
             and has never covered the adoption block\n  the lock said: {why}",
            carried(payload)
        ),
        headwater_lock::Authored::NothingBehind { why } => format!(
            "the lock that was there declared no adoption block, and it did not read \
             either\n  the lock said: {why}"
        ),
        headwater_lock::Authored::Opaque { why } => {
            eprintln!(
                "headwater: {}",
                err(&format!("{} did not read: {why}", headwater_lock::LOCK))
            );
            eprintln!(
                "  {}",
                err(
                    "Nothing can be seen of its adoption block, which is the one authored part \
                     of the file, so this run will not replace it. Repair the file, or delete it \
                     to resolve from the sources alone and write the block again"
                )
            );
            return ExitCode::FAILURE;
        }
    };

    if let Some(parent) = path.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return refuse(&format!("cannot create {}: {error}", parent.display()));
        }
    }
    if let Err(error) = std::fs::write(&path, &text) {
        return refuse(&format!("cannot write {}: {error}", path.display()));
    }
    println!("wrote {}", headwater_lock::LOCK);
    for source in &repository.resolution.sources {
        println!("  from {source}");
    }
    println!("  {note}");
    ExitCode::SUCCESS
}

/// What a run says it carried, counted in tasks rather than in pairs.
///
/// A task is the unit a person owns and dates, so the count that says the
/// authored block survived is the count of those.
fn carried(payload: &headwater_yaml::Mapping) -> String {
    let tasks = payload
        .get("tasks")
        .and_then(|node| node.value.as_seq())
        .map(|items| items.len())
        .unwrap_or_default();
    format!(
        "carried the adoption block through, {tasks} task{}",
        if tasks == 1 { "" } else { "s" }
    )
}

/// `headwater taxonomy audit`.
///
/// The ABox half of the pair `taxonomy validate` opens. It reads the same lock
/// and walks the same corpus every other verb does, and it decides nothing: it
/// exits 0 with findings, and no gate and no hook runs it.
///
/// **It exits 0 whatever it finds, and that is the constraint rather than a
/// default.** [Spec 6](../../../../docs/spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit)
/// makes these findings "advisory by construction, because a young or small
/// corpus fails differentiation for reasons that are not defects". The
/// strongest form of that promise is an exit status that no reading can move,
/// which is the shape `sweep` already has for its own reason. There is no
/// `--strict`.
///
/// **The clock is injected here for the reason it is injected into a check.** A
/// staleness reading and a dwell reading are both taken against a date, so two
/// runs over one tree agree only when the date is the same value. `--now` is
/// where a caller fixes it, and the report states the date it used.
///
/// **`--record` is the one thing here that writes, and it is opt-in for a
/// committed reason.** The `--now` help text a caller reads promises that "two
/// audits of one tree at one date write the same bytes", and a verb that
/// appended on every run would falsify that on its second run. The store also
/// refuses a duplicate `(lock, date)`, so the promise holds under the flag too.
/// The report is rendered from the store as it stands after the append, which
/// is what makes the section a function of the file rather than of the flag.
fn audit(root: &Path, now: Option<Date>, record: bool) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(context) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!(
            "headwater: {}",
            err("this host has no readable clock. Pass `--now <YYYY-MM-DD>`")
        );
        return ExitCode::FAILURE;
    };

    // The adoption reading is the one reading of this verb whose input is a run
    // of the check layer rather than the census and the graph beside it. The
    // ledger is what a check run says about the payload, and re-deriving it here
    // would be the second account of one tree that `take` exists to refuse.
    let reading = headwater_audit::reading::Reading::of(
        &run_of(&loaded, &context).adoption,
        &loaded.bound.digest,
        context.now(),
    );
    if record {
        match headwater_audit::reading::append(root, &reading) {
            Err(error) => {
                return refuse(&format!(
                    "the adoption reading did not append to {}: {error}",
                    headwater_audit::reading::STORE
                ))
            }
            Ok(headwater_audit::reading::Appended::Held) => eprintln!(
                "headwater: {} already holds a reading at {} under {}, and nothing was appended",
                headwater_audit::reading::STORE,
                context.now(),
                loaded.bound.digest
            ),
            Ok(headwater_audit::reading::Appended::Written) => eprintln!(
                "headwater: appended one adoption reading to {}",
                headwater_audit::reading::STORE
            ),
        }
    }
    // After the append, so the section reports the file a reader will open.
    let (recorded, unreadable) = match headwater_audit::reading::load(root) {
        Ok(held) => held,
        Err(error) => {
            return refuse(&format!(
                "{} did not read: {error}",
                headwater_audit::reading::STORE
            ))
        }
    };

    let audit = headwater_audit::take(
        headwater_audit::Subject {
            package: loaded.bound.package.clone(),
            version: loaded.bound.version.clone(),
            lock: loaded.bound.digest.clone(),
            now: context.now(),
        },
        &loaded.census,
        &loaded.graph,
        &loaded.taxonomy,
        &loaded.shape,
        &loaded.relations,
        // The resolved taxonomy, for the one member of a shelf that no typed
        // reader carries. See `headwater_scaffold::declared`.
        &loaded.bound.taxonomy,
        headwater_audit::Series {
            reading,
            recorded,
            unreadable,
        },
    );
    print!("{}", audit.render());
    ExitCode::SUCCESS
}

/// `headwater conformance`.
///
/// Whether this repository wired the method, rather than only copied it. The
/// rules come from the package and the readings come from this engine, and
/// [spec 7](../../../../docs/spec/07-distribution-and-federation.md#conformance)
/// argues both halves.
///
/// **Three refusals, and each one keeps a report from claiming more than it
/// measured.** A rule this engine holds no reading for ends the run, on the
/// `requires_engine` precedent. A waiver naming a rule the package does not
/// declare ends it, because no report could list that deviation. A waiver
/// missing any of its four fields ends it, because a deviation with no owner or
/// no expiry is one nobody closes.
///
/// **`--level` is the one thing that moves the exit status, and it moves only
/// that.** Without it the verb exits 0 whatever it finds, on the terms
/// `taxonomy audit` exits 0: it measures an adoption and it gates nothing. With
/// it the verb asks one question — is this repository at that rung — and a live
/// waiver answers for the rule it covers. The level the report states is
/// computed from met rules alone and no waiver reaches it.
fn conformance(root: &Path, level: Option<&str>, now: Option<Date>, json: bool) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(context) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!(
            "headwater: {}",
            err("this host has no readable clock. Pass `--now <YYYY-MM-DD>`")
        );
        return ExitCode::FAILURE;
    };

    let set = match headwater_conformance::at(root, &loaded.consumer) {
        Ok(set) => set,
        Err(error) => return refuse(&error.to_string()),
    };
    let waivers = match headwater_conformance::waivers(root) {
        Ok(waivers) => waivers,
        Err(refusals) => {
            eprintln!(
                "headwater: {}",
                err("the `conformance` block of the consumer declaration did not read")
            );
            for refusal in &refusals {
                eprintln!("  {}", err(refusal));
            }
            return ExitCode::FAILURE;
        }
    };

    // `lock.current` asks whether the committed lock matches the sources on
    // disk, so this verb reads the whole value rather than the taxonomy inside
    // it. Nothing reaches here with a candidate, and the arm says so rather
    // than assuming it.
    let Some(lock) = &loaded.bound.lock else {
        return refuse(
            "`headwater conformance` reads `.headwater/taxonomy.lock`, and this run holds a \
             taxonomy that came out of a published artifact instead",
        );
    };

    // The projection plan, built the one way `generate` builds one. A second
    // builder here would be a second answer to the question that rule asks.
    let projections = match headwater_generate::Projections::read(&loaded.bound.taxonomy) {
        Ok(projections) => projections,
        Err(errors) => return refused("the projections", &errors),
    };
    let surface = loaded.surface();
    let plan = headwater_generate::plan(
        &surface,
        &loaded.census,
        &projections,
        &loaded.identity(),
        &loaded.runs(root),
        headwater_verbs::VERBS,
    );

    let report = match headwater_conformance::evaluate(
        &set,
        &waivers,
        &headwater_conformance::Subject {
            root,
            consumer: &loaded.consumer,
            lock,
            census: &loaded.census,
            plan: &plan,
            now: context.now(),
        },
    ) {
        Ok(report) => report,
        Err(refusals) => {
            eprintln!(
                "headwater: {}",
                err("a waiver names no rule this package declares")
            );
            for refusal in &refusals {
                eprintln!("  {refusal}");
            }
            return ExitCode::FAILURE;
        }
    };
    // The text report is printed before the gate is asked, which is the order a
    // reader of this verb has always had: a refusal naming an undeclared rung
    // arrives under the gaps it is about. The document cannot take that order,
    // because the gate is a member of it, so it is written after.
    if !json {
        print!("{}", report.render());
    }
    let gated = match level {
        None => None,
        Some(level) => match report.gate(level) {
            Err(why) => return fail(&why),
            Ok(passes) => Some((level, passes)),
        },
    };
    if json {
        print!("{}", headwater_conformance::json::report(&report, gated));
    }

    // One reading of `Report::gate` per run. The exit status below reads the
    // value the document carries rather than asking the report a second time.
    let Some((level, passes)) = gated else {
        return ExitCode::SUCCESS;
    };
    match passes {
        true => {
            // Under `--json` the whole of standard output is the document, so
            // this sentence is its `gate` member rather than a line after it.
            if !json {
                println!(
                    "\n{level} passes, with every gap under it covered by a live waiver or met"
                );
            }
            ExitCode::SUCCESS
        }
        false => {
            eprintln!(
                "headwater: {}",
                err(&format!(
                    "{level} is not passed. Each gap above states the remediation the package \
                     wrote for it"
                ))
            );
            ExitCode::FAILURE
        }
    }
}

/// `headwater taxonomy publish`.
///
/// The publisher's half. It writes the artifact and prints the digest, which is
/// what the release notes carry and what a consumer writes into its own
/// declaration. The number is printed rather than filed anywhere, because a
/// digest that travels inside the artifact it describes checks nothing.
///
/// **`nothing was published` is printed for every error and it is a statement
/// about the disk.** Nothing here establishes it: this arm never learns whether
/// the library reached a write. What makes it true is
/// [`headwater_resolve::package::publish`], which reads every path the manifest
/// declares before it creates `--out` and returns `--out` to the state it found
/// it in when a write fails. That was false until [#271], where a failing run
/// left three files and an empty `bundles/` under a directory it said it had not
/// written to, and the next run was refused by the `--out` precondition catching
/// the first run's leftovers. `engine/crates/cli/tests/publish.rs` holds this
/// line to the disk from the state an adopter is in.
///
/// **`--json` writes the record as one document, because the handoff is a step
/// somebody scripts.** The digest has to be told to a consumer out of band, so
/// passing it on is automated, and a paragraph of English is what that
/// automation had to read until [#353]. The document is
/// [`headwater_resolve::release::document`], and it names its own shape rather
/// than this engine's version. A refusal writes no document and stays on
/// standard error, which is [HW-DR-0043]'s rule for every `--json` this binary
/// takes.
///
/// [#271]: https://github.com/headwater-ai/headwater/issues/271
/// [#353]: https://github.com/headwater-ai/headwater/issues/353
/// [HW-DR-0043]: ../../../../docs/decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md
fn publish(
    root: &Path,
    package: Option<&str>,
    from: Option<&Path>,
    assembly: Option<&str>,
    out: Option<&Path>,
    json: bool,
) -> ExitCode {
    let Some(out) = out else {
        return fail("`taxonomy publish` writes into a directory. Name it with `--out <dir>`");
    };
    if package.is_some() && from.is_some() {
        return fail(
            "`--package <name>` and `--from <dir>` name the same thing two ways: the first finds \
             a directory under `packages/` by the name its manifest declares, and the second \
             reads a directory the caller names directly. Pass one or the other",
        );
    }

    // Both halves of the pair come from the run that produced them. A plain
    // publish drops no `contents` key, so its half is empty by construction
    // rather than by a second reading of anything.
    let published = match from {
        Some(directory) => match assembly {
            Some(name) => {
                headwater_resolve::package::publish_assembly_from(root, directory, name, out)
                    .map(|done| (done.release, done.dropped))
            }
            None => headwater_resolve::package::publish_from(root, directory, out)
                .map(|release| (release, Vec::new())),
        },
        None => {
            let name = match package {
                Some(name) => name.to_string(),
                // The package this repository takes, where no flag names one. A
                // publisher usually publishes the package it also consumes.
                None => match headwater_resolve::package::consumer(root) {
                    Ok(consumer) => consumer.package,
                    Err(errors) => {
                        eprintln!(
                            "headwater: {}",
                            err(
                                "no `--package`, no `--from`, and this repository's declaration \
                                 does not read, so nothing says what to publish"
                            )
                        );
                        eprint!("{}", indent(&err(&render_errors(&errors))));
                        return ExitCode::FAILURE;
                    }
                },
            };
            match assembly {
                Some(assembly) => {
                    headwater_resolve::package::publish_assembly(root, &name, assembly, out)
                        .map(|done| (done.release, done.dropped))
                }
                None => headwater_resolve::package::publish(root, &name, out)
                    .map(|release| (release, Vec::new())),
            }
        }
    };

    let (record, dropped) = match published {
        Ok(pair) => pair,
        Err(errors) => {
            eprintln!("headwater: {}", err("nothing was published"));
            eprint!("{}", indent(&err(&render_errors(&errors))));
            advise_recipe(root, package, from, assembly);
            return ExitCode::FAILURE;
        }
    };

    // On standard error, and before the `--json` return, so a publisher reads it
    // in both output modes and the JSON document on standard output stays one
    // document. A key here is something the publisher asked for that nothing in
    // the artifact represents, which is the loss #581 refuses in the other
    // direction, so it is said rather than left for whoever opens the artifact.
    // `flatten::DROPPED` decides what reaches this: the keys flattening absorbs
    // rather than discards are not here, because a line printed by every publish
    // is a line nobody reads on the publish that loses something.
    if !dropped.is_empty() {
        eprintln!(
            "headwater: the source declared {}, and nothing in the flattened package carries it",
            dropped
                .iter()
                .map(|key| format!("`contents.{key}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        eprintln!(
            "{}",
            indent(
                "A migration payload states how one version line moves to the next, and a \
                 flattened package takes a new identity and a new version, so a payload written \
                 for the source package has no reader here. Publish the source package to ship \
                 it, or take the key out of the source manifest."
            )
        );
    }

    // #619, and beside `dropped` for the same two reasons: on standard error, so
    // a publisher reads it in both output modes and the `--json` document stays
    // one document. This half is the report and the refusal is the gate — the
    // publish only reached here because every reference a carried document
    // writes either resolves inside the artifact or is one of these. The list is
    // read out of the manifest the artifact carries rather than out of a second
    // copy of the key, so what is printed is what shipped.
    let recorded = headwater_resolve::package::recorded_references(out);
    if !recorded.is_empty() {
        eprintln!(
            "headwater: the artifact records {} references that resolve nowhere inside it, and \
             this publish carries no other",
            recorded.len()
        );
        eprintln!(
            "{}",
            indent(&format!(
                "`{}` in the manifest names each one as a carried document and a target the \
                 artifact does not hold, and the manifest ships with the artifact, so a consumer \
                 reads the same list. A reference this record does not hold is refused rather \
                 than reported. A pair the artifact stops dangling is a line to delete from it.",
                headwater_resolve::package::RECORDED_REFERENCES
            ))
        );
    }

    if json {
        print!(
            "{}",
            headwater_resolve::release::document(&record, out).render_pretty()
        );
        return ExitCode::SUCCESS;
    }

    println!("published {} {}", record.package, record.version);
    println!("  into {}", out.display());
    println!("  {} files", record.members.len());
    if let Some(range) = &record.requires_engine {
        println!("  for an engine in {range}");
    }
    println!("  digest {}", record.digest);
    println!(
        "\nState that digest where a consumer reads it, and never only inside the artifact. \
         A consumer pins it as `taxonomy.digest` in `.headwater/taxonomy.yml`, and \
         `headwater taxonomy vendor` checks a fetched copy against the pin."
    );
    ExitCode::SUCCESS
}

/// `headwater taxonomy vendor`.
///
/// The consumer's half, and the reason it takes a path is the guarantee it
/// keeps. Spec 0 forbids a network dependency and no crate of this engine
/// carries one. So the fetch is the caller's, by whatever the organization uses,
/// and this checks the bytes that arrived.
fn vendor(root: &Path, fetched: &Path, expect: Option<&str>) -> ExitCode {
    let declared = headwater_resolve::package::consumer(root)
        .ok()
        .and_then(|consumer| consumer.digest);
    let Some(pinned) = expect.map(str::to_string).or(declared) else {
        // Stays at `fail` (#455): `--expect` reaches past this on the command
        // line, which is the whole test, and a caller who does not know the
        // flag exists is the caller the grammar pointer is for. The consumer
        // declaration being incomplete pulls the other way and loses to that.
        return fail(
            "nothing pins this artifact. A digest the engine took from the artifact in front of \
             it is a pin against itself, so this refuses rather than records what it received. \
             Write the publisher's digest as `taxonomy.digest` in `.headwater/taxonomy.yml`, or \
             pass it with `--expect`",
        );
    };

    let record = match headwater_resolve::package::vendor(root, fetched, &pinned) {
        Ok(record) => record,
        Err(errors) => {
            eprintln!("headwater: {}", err("nothing was vendored"));
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };

    println!("vendored {} {}", record.package, record.version);
    println!("  from {}", fetched.display());
    println!(
        "  {} files, all of them the pinned bytes",
        record.members.len()
    );
    println!("  digest {}", record.digest);

    // The path is read back off the installed manifest rather than returned by
    // `package::vendor`, because `Release` is the release record and a doctrine
    // path is not one of its fields. That verb has already held the same key to
    // the artifact and refused every value it could not open, so this reads a
    // key one line above proved readable.
    let flattened = record.package.replace('/', "-");
    let installed = root
        .join(headwater_resolve::package::PACKAGES)
        .join(&flattened);
    let doctrine = headwater_resolve::package::manifest_at(&installed)
        .ok()
        .and_then(|manifest| headwater_resolve::package::doctrine(&manifest));
    if let Some(at) = doctrine {
        println!(
            "  doctrine at {}/{}/{}",
            headwater_resolve::package::PACKAGES,
            flattened,
            at.display()
        );
        println!(
            "\nThe doctrine directory is prose the publisher wrote for a person to read. It is \
             not schema, nothing resolves it, and no check reads it."
        );
    }

    println!(
        "\nThe digest says these are the bytes the pin was written for. It is not a signature, \
         so it says nothing about who published them. Run `headwater taxonomy resolve` to write \
         the lock this package produces."
    );
    ExitCode::SUCCESS
}

/// `headwater taxonomy diff`: measured compatibility between two versions, over
/// this corpus.
///
/// **It takes a directory, for the reason [`vendor`] does.** Spec 7 writes the
/// invocation as `taxonomy diff --to 4.0.0`, and no crate of this engine opens
/// a socket, so the artifact is one the caller already fetched. `--to` is
/// therefore the assertion rather than the address: the artifact says which
/// version it is, and the flag holds it to what the caller expected. The range
/// reader is [`headwater_resolve::release::satisfies`], which is the one piece
/// of version arithmetic this engine has, so `--to 4.0.0` and `--to ">=4 <5"`
/// are read by the same code that reads `requires_engine`.
///
/// **The candidate is resolved under this repository's own overlays**, which is
/// what makes the report about this corpus rather than about the base. Spec 7:
/// the consumer's run "verifies that claim against documents that the publisher
/// never saw".
///
/// **The comparison runs each phase twice and compares nothing else.** Two
/// publishes of an unchanged package differ in a timestamp, a path and a
/// digest, and a report that fired on those would fire on every release. See
/// [`headwater_compat`].
/// A published artifact, held to its own release record and to `--to`.
///
/// One reader, because `taxonomy diff` and `taxonomy migrate` both take a
/// directory somebody fetched and both have to know that it is the version it
/// says it is. Two readings would be two answers about one directory, and the
/// one that migrates is the one that writes.
fn artifact(
    fetched: &Path,
    to: Option<&str>,
) -> Result<headwater_resolve::release::Release, ExitCode> {
    let record = match headwater_resolve::release::at(fetched) {
        Ok(record) => record,
        Err(error) => {
            eprintln!(
                "headwater: {}",
                err(&format!(
                    "{} is not a published artifact this engine can read",
                    fetched.display()
                ))
            );
            eprintln!("{}", indent(&err(&error.to_string())));
            return Err(ExitCode::FAILURE);
        }
    };
    if let Err(error) = headwater_resolve::release::diverged(fetched, &record)
        .map_err(|error| error.to_string())
        .and_then(|diverged| match diverged.is_empty() {
            true => Ok(()),
            false => Err(diverged
                .iter()
                .map(|entry| entry.to_string())
                .collect::<Vec<_>>()
                .join("\n")),
        })
    {
        eprintln!(
            "headwater: {}",
            err("the artifact is not what its own release record says it is")
        );
        eprintln!("{}", indent(&err(&error)));
        return Err(ExitCode::FAILURE);
    }

    if let Some(range) = to {
        match headwater_resolve::release::satisfies(range, &record.version) {
            Err(why) => {
                return Err(fail(&format!(
                    "`--to {range}` states no version comparison this engine reads: {why}"
                )))
            }
            Ok(false) => {
                return Err(fail(&format!(
                    "`--to {range}` and the artifact declares {}. The flag names the version the \
                     caller expected, and this engine fetches nothing, so a mismatch is a wrong \
                     directory rather than a wrong number",
                    record.version
                )))
            }
            Ok(true) => {}
        }
    }
    Ok(record)
}

/// `taxonomy migrate`: the half of a migration that writes.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
/// splits a payload into "what the engine can apply mechanically
/// (`headwater migrate --apply`) and what needs human or agent judgment
/// (emitted as a task list with the affected documents attached)". Both halves
/// are below, and the split is not this function's to make: it is derived from
/// the target list by
/// [`headwater_resolve::migration::Application::over`], which is the only
/// constructor of it.
///
/// # A judgment step reaches no writer, structurally
///
/// The mechanical half is built by matching `Application::Mechanical` and
/// taking the one target off that arm. A choice and a re-statement carry no
/// such field, so there is no value a write could be composed from and no arm
/// in which one is. That is the whole guard, and it is worth more than a check
/// that a task list is non-empty.
///
/// # `--apply` writes `adoption.from` and `adoption.to`, and that is a decision
///
/// [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)
/// records the migration state in the lock. [#61 left the seam under
/// `adoption:` unstated](../../../../docs/spec/13-open-obligations.md): no
/// document said whether `taxonomy resolve --check` may pass while the
/// authored half of the lock is stale, or whether a part of it that no digest
/// covers may sit in a reviewed artifact.
/// [HW-DR-0046](../../../../docs/decisions/0046-migrating-from-version-carries-a-semver-and-a-release-digest-kept-as-separate-fields.md)
/// answers the second question for this one field: `from` is a semver and the
/// release digest [`headwater_resolve::package::consumer`] finds still pinned
/// in `.headwater/taxonomy.yml`, read before this run moves that pin, and `to`
/// is the version of the artifact this run applies. Both are written through
/// [`headwater_scaffold::tree::Reserved`], in the same all-or-nothing batch as
/// the mechanical document and overlay writes below.
///
/// A corpus pinned to no digest — one that takes its package from source
/// rather than through `vendor` — has nothing this run can honestly write as
/// `from.digest`. This run then writes every other file and says why the lock
/// is untouched, rather than refusing the whole migration over a field it
/// cannot verify.
///
/// Every other question `adoption:` still has to answer stays where #61 left
/// it. This run merges `from` and `to` into whatever the block already
/// declares and carries every other key, `tasks` above all, through
/// unchanged. Where there is no `tasks` key to carry, it writes an empty one,
/// for the reason [`migrated`] states: the next verb spec 7 names refuses a
/// block that declares none.
fn migrate(
    root: &Path,
    fetched: &Path,
    to: Option<&str>,
    now: Option<Date>,
    applying: bool,
) -> ExitCode {
    let Some(_ctx) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!(
            "headwater: {}",
            err("this host has no readable clock. Pass `--now <YYYY-MM-DD>`")
        );
        return ExitCode::FAILURE;
    };
    let record = match artifact(fetched, to) {
        Ok(record) => record,
        Err(code) => return code,
    };

    let lock = match headwater_lock::at(root) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("headwater: {}", err(&format!("{error}")));
            return ExitCode::FAILURE;
        }
    };
    // Cloned before `lock` moves into `Bound::of` below. `rewrite_adoption`
    // needs the committed lock as it stood before this run, the same reason
    // `infer` reads `loaded.bound.adoption` before it runs a check over it.
    let committed_lock = lock.clone();
    if record.package != lock.package {
        return fail(&format!(
            "this repository takes `{}` and the artifact publishes `{}`. Two packages are not \
             two versions of one, and a migration between them is not a rename of anything",
            lock.package, record.package
        ));
    }
    let from = lock.version.clone();
    let to = record.version.clone();

    let manifest = match headwater_resolve::package::manifest_at(fetched) {
        Ok(manifest) => manifest,
        Err(errors) => {
            eprintln!("headwater: {}", err("the artifact manifest did not read"));
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    let payloads = match headwater_resolve::migration::at(fetched, &manifest) {
        Ok(payloads) => payloads,
        Err(refusals) => {
            eprintln!(
                "headwater: {}",
                err("the artifact carries a migration payload this engine cannot read")
            );
            eprint!(
                "{}",
                indent(&err(&render_errors(
                    &headwater_resolve::migration::as_errors(
                        &fetched.display().to_string(),
                        &refusals,
                    )
                )))
            );
            return ExitCode::FAILURE;
        }
    };

    let mut selected: Vec<&headwater_resolve::migration::Payload> = Vec::new();
    for carried in &payloads {
        match carried.covers(&from, &to) {
            Err(why) => {
                // `refuse` (#455): the package already matched, so this is the
                // artifact the caller meant, and a payload inside it is
                // malformed. No spelling of this command line makes those
                // bytes readable.
                return refuse(&format!(
                    "{} states a version range this engine cannot read: {why}",
                    carried.at
                ));
            }
            Ok(false) => {}
            Ok(true) => selected.push(carried),
        }
    }
    let payload = match selected.len() {
        1 => selected[0],
        0 => {
            // `refuse` (#455): the publisher shipped an artifact this
            // transition is not covered by, which the message says by naming
            // spec 2. A different `--to` is a different migration rather than
            // a repair of this one.
            // The second sentence is unconditional, and it is unconditional
            // because nothing has been measured yet: this arm is reached
            // before the two check runs that would say which rules broke. So
            // it names the condition rather than asserting it, and `taxonomy
            // diff` — which has measured — says which of the two sentences
            // applies to this artifact.
            return refuse(&format!(
                "the artifact ships no migration payload for {from} to {to}. Spec 2 makes a major \
                 version ship one, and this verb applies a payload rather than deriving one. \
                 `headwater taxonomy diff {}` reports what moved. Where every break is a facet \
                 that became required, no payload can cover it — that is a break no subject of \
                 this vocabulary reaches — and `headwater infer --owner <name> --write` records \
                 it as adoption debt instead",
                fetched.display()
            ));
        }
        count => {
            // `refuse` (#455): the publisher shipped an ambiguous artifact.
            // Same reason as the arm above it, and no flag picks a route.
            return refuse(&format!(
                "{count} payloads of the artifact cover {from} to {to}, and a migration is not a \
                 choice of route: {}",
                selected
                    .iter()
                    .map(|payload| payload.at.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    };

    // The census of the taxonomy this repository takes, which is the taxonomy
    // every `from` of the payload was written against. A census under the
    // candidate would answer no documents for every step.
    let taking = match load_against(root, Bound::of(lock)) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };

    // This repository's own overlay, which is the second file a step can name
    // and the only one outside the corpus that this verb writes.
    let consumer = match headwater_resolve::package::consumer(root) {
        Ok(consumer) => consumer,
        Err(errors) => {
            eprintln!(
                "headwater: {}",
                err("the consumer declaration did not read")
            );
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    let overlay = match headwater_resolve::package::adopted(root, &consumer) {
        Ok(overlay) => overlay,
        Err(errors) => {
            eprintln!(
                "headwater: {}",
                err("this repository's overlay did not read")
            );
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };

    println!(
        "\nmigration  {}, from {from} to {to}\n  payload  {}\n",
        record.package, payload.at
    );

    // --- the half the engine writes ---------------------------------------
    let mut moves: Vec<headwater_scaffold::migrate::Move> = Vec::new();
    let mut readdressed: Vec<headwater_scaffold::overlay::Move> = Vec::new();
    let mut placed: Vec<String> = Vec::new();
    let mut mechanical = 0;
    for step in &payload.steps {
        let headwater_resolve::migration::Application::Mechanical { to } = &step.apply else {
            continue;
        };
        mechanical += 1;
        let sites = headwater_compat::migrate::sites(step, &taking.census, &overlay);
        println!("  {}  becomes `{to}`", step.at());
        match sites.is_empty() {
            true => println!("    {}", step.subject.reached_nothing()),
            false => {
                for site in &sites {
                    match site {
                        headwater_compat::migrate::Site::Front { path, key } => {
                            println!("    {path}  `{key}`");
                            moves.push(headwater_scaffold::migrate::Move {
                                path: path.clone(),
                                key: key.clone(),
                                from: step.from.clone(),
                                to: to.clone(),
                            });
                        }
                        headwater_compat::migrate::Site::Placement { .. } => {
                            let why = site.why().expect("a placement site states one");
                            println!("    {why}");
                            placed.push(why);
                        }
                        headwater_compat::migrate::Site::Overlay {
                            path,
                            at,
                            address,
                            span,
                        } => {
                            // The step's `from` is a prefix of this address, or
                            // `sites` would not have named the entry. The
                            // refusal is here anyway, because the alternative to
                            // an answer is a rewrite of somebody's overlay into
                            // a path nobody wrote. That argument may make this
                            // branch unreachable, and no case drives it for
                            // that reason.
                            //
                            // `refuse` and not `defect` (#455): somebody
                            // authored the overlay entry this reads, and
                            // `defect` is for a value this run's own code
                            // built. Nothing on the command line reaches
                            // `.headwater/overlay.yml`.
                            let Some(moved) =
                                headwater_resolve::migration::readdressed(address, &step.from, to)
                            else {
                                return refuse(&format!(
                                    "{at} in {path} is not addressed under `{}`, so this run \
                                     cannot say what it becomes",
                                    step.from
                                ));
                            };
                            println!("    {path}  {at}  becomes `{moved}`");
                            readdressed.push(headwater_scaffold::overlay::Move {
                                at: at.clone(),
                                address: address.clone(),
                                to: moved,
                                span: *span,
                            });
                        }
                    }
                }
            }
        }
        println!();
    }
    if mechanical == 0 {
        println!("  no step of this payload applies mechanically\n");
    }

    // --- the half an author settles ---------------------------------------
    let judgment: Vec<&headwater_resolve::migration::Step> = payload
        .steps
        .iter()
        .filter(|step| !step.apply.mechanical())
        .collect();
    println!(
        "  {} task{} for an author, and no run writes any of them",
        judgment.len(),
        match judgment.len() {
            1 => "",
            _ => "s",
        }
    );
    for step in &judgment {
        println!("\n    {}  {}", step.at(), step.apply.sentence());
        println!(
            "      task  {}",
            step.apply.task().expect("a judgment step carries one")
        );
        let sites = headwater_compat::migrate::sites(step, &taking.census, &overlay);
        match sites.is_empty() {
            true => println!("      {}", step.subject.reached_nothing()),
            false => {
                for site in &sites {
                    println!("      {}", site.key());
                }
            }
        }
    }
    println!();

    // --- what the lock records, and what it still does not ----------------
    match &consumer.digest {
        Some(digest) => println!(
            "  the lock records this migration on `--apply`: `adoption.from` becomes \
             {{version: {from}, digest: {digest}}} and `adoption.to` becomes {to} \
             (HW-DR-0046). The open task set, if any, is carried through unchanged"
        ),
        // The remedy names hand authorship, and it named `taxonomy vendor`
        // until #516 was adjudicated. That verb pins nothing: `--expect`
        // verifies the artifact against a digest the caller supplied, installs
        // it, exits 0 and leaves the declaration byte-identical, and the
        // identical next `vendor` refuses with "nothing pins this artifact".
        // So the sentence sent an adopter with no pin to a verb that refuses
        // them and sends them back here. The pin is authored (spec 7), and
        // `apply_with_no_pinned_digest_writes_no_migration_state` performs
        // every clause of what stands here now.
        None => println!(
            "  `.headwater/taxonomy.yml` pins no digest, so this run cannot write a verifiable \
             `adoption.from` (HW-DR-0046). The pin is authored: take the digest the publisher \
             states and write it as `taxonomy.digest` in `.headwater/taxonomy.yml` by hand, and \
             a later run of this verb records the migration. Every other file below is still \
             written on `--apply`"
        ),
    }

    if !placed.is_empty() {
        println!(
            "\n  {} document{} take{} a kind this payload renames from a homogeneous shelf, so no \
             byte of the document holds it and the remedy is a file move",
            placed.len(),
            match placed.len() {
                1 => "",
                _ => "s",
            },
            match placed.len() {
                1 => "s",
                _ => "",
            }
        );
    }

    let mut written = headwater_scaffold::migrate::compose(root, &moves);
    if !written.refused.is_empty() {
        eprintln!(
            "\nheadwater: {} document{} did not compose, and this run writes nothing",
            written.refused.len(),
            match written.refused.len() {
                1 => "",
                _ => "s",
            }
        );
        for refused in &written.refused {
            eprintln!("{}", indent(&err(&refused.to_string())));
        }
        return ExitCode::FAILURE;
    }

    // The overlay joins the same set. An overlay re-addressed while a document
    // it types keeps the old value is a corpus in neither state, so the two are
    // one write or they are neither, which is what `tree::Reserved` takes.
    let overlay_at = overlay.at().unwrap_or_default().to_string();
    match headwater_scaffold::overlay::compose(root, &overlay_at, &readdressed) {
        Err(refused) => {
            eprintln!(
                "\nheadwater: this repository's overlay did not compose, and this run \
                       writes nothing"
            );
            eprintln!("{}", indent(&err(&refused.to_string())));
            return ExitCode::FAILURE;
        }
        Ok(None) => {}
        Ok(Some((composed, count))) => {
            written.files.push(composed);
            written.replaced += count;
        }
    }

    // Counted here, ahead of the branch that builds it, so the preview below
    // and the write further down agree on how many files are in play.
    let files = written.files.len() + usize::from(consumer.digest.is_some());
    if !applying {
        println!(
            "\n  {} value{} in {} file{} would be written. Nothing was: pass `--apply`",
            written.replaced,
            match written.replaced {
                1 => "",
                _ => "s",
            },
            files,
            match files {
                1 => "",
                _ => "s",
            }
        );
        return ExitCode::SUCCESS;
    }

    // The lock: `adoption.from` and `adoption.to` (HW-DR-0046). Joins the same
    // reserved set as the document and overlay writes above, so the migration
    // state and the files it describes land in one write or none of it does.
    // Absent where `.headwater/taxonomy.yml` pins no digest — this run states
    // that above and writes every other file regardless.
    if let Some(digest) = &consumer.digest {
        let mut state = String::new();
        state.push_str("from:\n");
        state.push_str(&format!("  version: {}\n", quoted(&from)));
        state.push_str(&format!("  digest: {}\n", quoted(digest)));
        state.push_str(&format!("to: {}\n", quoted(&to)));
        // The empty standing list, which is what makes this block one the next
        // verb can add to. `headwater infer --write` merges into `tasks` and
        // refuses a block that declares no such key, because replacing an
        // authored block is the defect that refusal exists for. A block this
        // run generated carried `from` and `to` and no `tasks`, so the two
        // verbs spec 7 puts in order refused each other on every first
        // migration. `migrated` takes this entry only where nothing else
        // carried one through, so a standing task list is never touched.
        state.push_str("tasks: []\n");
        let fresh =
            match headwater_yaml::load(&state) {
                Ok(node) => match node.value.as_map() {
                    Some(map) => map.clone(),
                    None => return defect(
                        "the migration state this run built is not a mapping, which is a defect",
                    ),
                },
                Err(errors) => {
                    return defect(&format!(
                        "the migration state this run built does not load: {}",
                        headwater_yaml::error::render(&errors)
                    ))
                }
            };
        let block = migrated(committed_lock.adoption.as_ref(), &fresh);
        // Not `headwater_resolve::repository(root)`: that resolves
        // `packages/<name>` fresh, and a migration is run exactly when that
        // directory can already disagree with what this lock committed to.
        // `rewrite_adoption` stands on the resolution the lock already carries
        // and changes only the block this run computed.
        let text = headwater_lock::rewrite_adoption(&committed_lock, Some(&block));
        written.files.push(headwater_scaffold::tree::Composed {
            path: headwater_lock::LOCK.to_string(),
            text,
        });
    }

    let replaced = written.replaced;
    let reserved = match headwater_scaffold::tree::Reserved::over(root, written.files) {
        Ok(reserved) => reserved,
        Err(unopened) => {
            eprintln!("\nheadwater: a file of this migration cannot be written, so none was");
            eprintln!("{}", indent(&err(&unopened.to_string())));
            return ExitCode::FAILURE;
        }
    };
    match reserved.commit() {
        Ok(paths) => {
            println!(
                "\n  wrote {replaced} value{} in {} file{}",
                match replaced {
                    1 => "",
                    _ => "s",
                },
                paths.len(),
                match paths.len() {
                    1 => "",
                    _ => "s",
                }
            );
            for path in &paths {
                println!("    {path}");
            }
            ExitCode::SUCCESS
        }
        Err(halted) => {
            eprintln!("\nheadwater: the migration stopped part way");
            eprintln!("{}", indent(&err(&halted.to_string())));
            ExitCode::FAILURE
        }
    }
}

fn diff(root: &Path, fetched: &Path, to: Option<&str>, now: Option<Date>) -> ExitCode {
    // The clock, read once and before anything is walked, on the same terms
    // `check` reads it: two windowed expectations evaluated a second apart
    // would be a difference this verb attributed to the taxonomy.
    let Some(ctx) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!(
            "headwater: {}",
            err("this host has no readable clock. Pass `--now <YYYY-MM-DD>`")
        );
        return ExitCode::FAILURE;
    };

    // The artifact, held to its own release record before a declaration inside
    // it is read. A directory somebody edited after it was published is not the
    // version it says it is, and every dimension below would then measure
    // against a taxonomy that no publisher shipped.
    let record = match artifact(fetched, to) {
        Ok(record) => record,
        Err(code) => return code,
    };

    // What this repository takes today. The lock and not the sources: spec 6
    // fixes the lock as the one thing downstream reads, and a comparison
    // against unresolved sources would report the state of a working tree.
    let lock = match headwater_lock::at(root) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("headwater: {}", err(&format!("{error}")));
            return ExitCode::FAILURE;
        }
    };
    let consumer = match headwater_resolve::package::consumer(root) {
        Ok(consumer) => consumer,
        Err(errors) => {
            eprintln!(
                "headwater: {}",
                err("the consumer declaration did not read")
            );
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    // This repository's own overlay, read on its own. A step over an overlay
    // address names entries of this file, and no census row covers it.
    let overlay = match headwater_resolve::package::adopted(root, &consumer) {
        Ok(overlay) => overlay,
        Err(errors) => {
            eprintln!(
                "headwater: {}",
                err("this repository's overlay did not read")
            );
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    if record.package != lock.package {
        return fail(&format!(
            "this repository takes `{}` and the artifact publishes `{}`. Two packages are not \
             two versions of one, and none of the six dimensions is a question about them",
            lock.package, record.package
        ));
    }

    let manifest = match headwater_resolve::package::manifest_at(fetched) {
        Ok(manifest) => manifest,
        Err(errors) => {
            eprintln!("headwater: {}", err("the artifact manifest did not read"));
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };

    // The candidate, resolved under this repository's overlays. Every refusal
    // that names an overlay address is the `addressability` dimension, and it
    // is the one dimension whose subject is the schema rather than the corpus.
    let candidate = headwater_resolve::package::sources_at(root, fetched, &manifest, &consumer)
        .and_then(|sources| headwater_resolve::resolve(&sources));
    // The third element is the judgment task an `add` collision owes a
    // consumer, empty where no refusal is one. See `headwater_resolve::error`.
    // The founding record of the release the lock names, which is the previous
    // side of the quiet half. The lock is moved into `Bound::of` further down,
    // so the record is taken here. An older lock carries none and reads back as
    // an empty list, which reports every founding the candidate records — the
    // reading every lock had before `headwater_lock::Lock::founded` existed.
    let carried = lock.founded.clone();
    let (resolution, addressability, tasks) = match candidate {
        Ok(resolution) => {
            // The quiet half. The candidate resolved, and it may have resolved
            // because an overlay `add` created the declaration the new base
            // removed. A founding the previous release already carried is a
            // property of the consumer's bundle order rather than of anything
            // this release did, so only the ones this release introduces are
            // breaks. See `headwater_compat::addressability`.
            let outcome = headwater_compat::addressability(
                &carried,
                &resolution.founding_records(),
                Vec::new(),
            );
            (Some(resolution), outcome, String::new())
        }
        Err(errors) => {
            let breaks: Vec<headwater_compat::Break> = errors
                .iter()
                .filter(|error| error.kind.names_an_address())
                .map(|error| headwater_compat::Break {
                    at: format!("{} in {}", error.at, error.source),
                    was: "an address this overlay resolves against".to_string(),
                    now: error.to_string(),
                })
                .collect();
            let outcome = match breaks.is_empty() {
                true => headwater_compat::Outcome::NotMeasured(format!(
                    "the candidate did not resolve, and no refusal named an overlay address: {}",
                    render_errors(&errors).trim()
                )),
                false => headwater_compat::Outcome::over(breaks),
            };
            (None, outcome, headwater_resolve::error::collisions(&errors))
        }
    };

    let Some(resolution) = resolution else {
        let report = headwater_compat::Report {
            package: record.package.clone(),
            from: lock.version.clone(),
            to: record.version.clone(),
            base: headwater_compat::Base::Unresolved,
            measured: headwater_compat::Measured::against_nothing(
                addressability,
                "the candidate taxonomy did not resolve under this repository's overlays, so no \
                 phase ran against it",
            ),
        };
        print!("{}", report.render());
        print!("{tasks}");
        eprintln!(
            "headwater: {}",
            err(
                "the candidate did not resolve, so five of the six dimensions were not \
                 measured. The lines above are what this run does know"
            )
        );
        return ExitCode::FAILURE;
    };

    // Every rule of `taxonomy validate`, over the resolved candidate. Spec 7
    // asks the upgrade report for "whether the new base still satisfies the
    // core under the local overlay", and this is that question.
    let refusals = resolution.validate();

    let base = match headwater_lock::digest(&resolution.render()) == lock.digest {
        true => headwater_compat::Base::Same,
        false => headwater_compat::Base::Moved,
    };
    let package = record.package.clone();
    let from = lock.version.clone();
    let lock_version = lock.version.clone();
    let to = record.version.clone();

    let taking = match load_against(root, Bound::of(lock)) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let against = match load_against(
        root,
        Bound::candidate(
            &package,
            &to,
            &resolution,
            taking.bound.adoption.as_ref(),
            &fetched.display().to_string(),
        ),
    ) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };

    // One run per side, and the two dimensions that read a verdict read these.
    // A second run of either side would be a second reading of one question,
    // and `consequence` and `instance_validity` would then be able to disagree
    // about a corpus that changed between them.
    let before = run_of(&taking, &ctx);
    let after = run_of(&against, &ctx);
    let identity = taking.identity();

    // The two dimensions a migration step is a remedy for, and each one answers
    // with the documents it moved beside its verdict. The union is the
    // denominator the payload is accounted against, and it is built from the
    // comparisons that decided the dimensions rather than from a second reading
    // of them.
    let (classification, reclassified) =
        headwater_compat::classification(&taking.census, &against.census);
    let (validity, invalidated) = headwater_compat::instance_validity(&before, &after);
    let moved = Movement {
        documents: reclassified.union(&invalidated).cloned().collect(),
        // The rules behind the movement, and not a second reading of it. Read
        // off `instance_validity` alone, because that is the one dimension
        // whose breaks are keyed by a rule: a `classification` break is keyed
        // by a document path and it is a `kind` step's business, which is a
        // step the vocabulary does have. So a change that reclassified
        // anything answers the empty set here and keeps the sentence spec 2
        // states.
        rules: match &classification {
            headwater_compat::Outcome::Broken(_) => std::collections::BTreeSet::new(),
            _ => headwater_compat::broken_rules(&validity),
        },
    };
    let measured = headwater_compat::Measured {
        classification,
        instance_validity: validity,
        consequence: headwater_compat::consequence(&before, &after),
        // Both plans are built under one identity, and it is the identity of
        // the lock. Two projections carry the package, the version and the
        // taxonomy digest of the run that wrote them: the corpus descriptor
        // states all three and a probe result states the digest. Those are
        // injected values rather than consequences of a taxonomy, so a
        // comparison that let them move would report the version number as a
        // change that the version number caused. It would then fire on every
        // release, which is the one failure this dimension has to avoid.
        // Everything a taxonomy decides about a projection still moves: the
        // exclusions, the entry points, the exports and every declared output.
        projection: match (
            plan_of(root, &taking, &identity),
            plan_of(root, &against, &identity),
        ) {
            (Ok(before), Ok(after)) => headwater_compat::projection(&before, &after),
            (before, after) => headwater_compat::Outcome::NotMeasured(format!(
                "a projection declaration did not read: {}",
                [before.err(), after.err()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("; ")
            )),
        },
        identifier: headwater_compat::identifier(&taking.graph, &against.graph),
        addressability,
    };

    let report = headwater_compat::Report {
        package,
        from,
        to,
        base,
        measured,
    };
    print!("{}", report.render());

    if let Err(code) = payload(
        fetched,
        &manifest,
        &taking,
        &overlay,
        &lock_version,
        &record.version,
        &moved,
    ) {
        return code;
    }

    if !refusals.is_empty() {
        println!(
            "\nthe candidate resolves under this overlay and {} rule{} of `taxonomy validate` \
             refuses the result",
            refusals.len(),
            match refusals.len() {
                1 => "",
                _ => "s",
            }
        );
        print!("{}", indent(&render_errors(&refusals)));
    }

    // A broken dimension is a report and never a failure. The verb measures a
    // version that nobody has taken yet, and an upgrade that needs a migration
    // is the ordinary case rather than an error ([spec 7](../../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)).
    // What does fail is a run that could not measure, which is above.
    ExitCode::SUCCESS
}

/// What the artifact's migration payload says about the breaks measured above.
///
/// **It reports and it never gates.** The verb measures a version that nobody
/// has taken yet, and the payload is the publisher's account of what the
/// upgrade costs. A missing payload for a major upgrade is a finding a consumer
/// acts on, and [spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
/// makes it a hard failure of `migrate` rather than of the measurement that
/// precedes it.
///
/// **A step whose source this repository does not hold is reported, not
/// refused.** The old taxonomy a consumer holds is the base under its own
/// overlays, and an overlay may have removed the value a step renames. What is
/// refused, and at the other end where the publisher can act on it, is a step
/// whose source is still declared in the taxonomy it ships. See
/// [`headwater_resolve::migration`].
///
/// The error arm is a payload the artifact carries and this engine cannot read,
/// which is the one state that says nothing about the corpus at all.
///
/// # When "ship a payload" is not the remedy
///
/// `moved.rules` is the rules the measured dimensions report, and it decides
/// which
/// closing sentence a reader gets. A break every rule of which is
/// [`FACET_REQUIRED`] is a break no step of this vocabulary reaches: spec 7
/// names three subjects, a facet that became required is none of them, and
/// there is no old value for a `facet_value` step to move. Telling a publisher
/// to ship a payload for that is telling them to write a file the format
/// cannot express, so the sentence names the route the engine does reach
/// instead — `headwater infer --write`, which derives the pair set, and
/// `headwater check`, which then reports it as migration-pending. Every other
/// break keeps the sentence spec 2 makes true of it.
fn payload(
    fetched: &Path,
    manifest: &headwater_yaml::Mapping,
    taking: &Loaded,
    overlay: &headwater_resolve::Adopted,
    from: &str,
    to: &str,
    moved: &Movement,
) -> Result<(), ExitCode> {
    let payloads = match headwater_resolve::migration::at(fetched, manifest) {
        Ok(payloads) => payloads,
        Err(refusals) => {
            eprintln!(
                "headwater: {}",
                err("the artifact carries a migration payload this engine cannot read")
            );
            eprint!(
                "{}",
                indent(&err(&render_errors(
                    &headwater_resolve::migration::as_errors(
                        &fetched.display().to_string(),
                        &refusals,
                    )
                )))
            );
            return Err(ExitCode::FAILURE);
        }
    };

    let taxonomy = &taking.bound.taxonomy;
    let declares = |step: &headwater_resolve::migration::Step| {
        headwater_resolve::migration::declares(taxonomy, &step.subject, &step.from)
    };

    let mut selected = 0;
    for carried in &payloads {
        match carried.covers(from, to) {
            Err(why) => {
                println!(
                    "\n{} states a version range this engine cannot read, so nothing selected it: \
                     {why}",
                    carried.at
                );
            }
            Ok(false) => {}
            Ok(true) => {
                selected += 1;
                print!(
                    "{}",
                    headwater_compat::payload::account(
                        carried,
                        &taking.census,
                        overlay,
                        declares,
                        &moved.documents
                    )
                    .render()
                );
            }
        }
    }

    if selected == 0 && !moved.documents.is_empty() {
        let documents = format!(
            "{} document{}",
            moved.documents.len(),
            match moved.documents.len() {
                1 => "",
                _ => "s",
            }
        );
        match unreachable_by_a_step(&moved.rules) {
            true => println!(
                "\nthe artifact ships no migration payload for {from} to {to}, and {documents} \
                 stopped validating. No step can express this break: a facet that became \
                 required is none of the three subjects spec 7 declares, and there is no old \
                 value for a `facet_value` step to move. `headwater infer --owner <name> \
                 --write` records the breakage as adoption debt, and `headwater check` then \
                 reports it as migration-pending"
            ),
            false => println!(
                "\nthe artifact ships no migration payload for {from} to {to}, and {documents} \
                 stopped validating. Spec 2 makes a major version ship one"
            ),
        }
    }
    Ok(())
}

/// What the two dimensions a subject names measured, as the payload reader
/// needs it.
///
/// Both halves fall out of the same two comparisons that decided the
/// dimensions, which is what [`payload`] states about the denominator and holds
/// for the rules as well: a second pass over the instances could disagree with
/// the pass that decided the verdict.
struct Movement {
    /// Every document that stopped resolving to its kind or stopped
    /// validating, which is the denominator a payload is accounted against.
    documents: std::collections::BTreeSet<String>,
    /// The rules whose instances stopped agreeing. Empty where a document was
    /// reclassified, because a `classification` break is keyed by a path and
    /// names no rule.
    rules: std::collections::BTreeSet<String>,
}

/// The one rule a break can consist entirely of and still be unreachable by any
/// step of the migration vocabulary.
///
/// Spec 7 (*The migration payload*) declares three subjects — a facet value, a
/// kind, and an overlay address — and each one names an old thing that a step
/// moves. A facet that became required names no old thing: the documents never
/// carried the facet, so there is no value for a step to move and no address
/// that changed. See [`headwater_resolve::migration::Subject`].
const FACET_REQUIRED: &str = "facet.required.missing";

/// Whether every rule that broke is one no step of the vocabulary reaches.
///
/// Non-empty is required, and it is the whole reason this is a function rather
/// than an `iter().all()` at the call site: `all` answers `true` for an empty
/// set, and a dimension that broke on nothing is not a break this sentence
/// should describe.
fn unreachable_by_a_step(broken: &std::collections::BTreeSet<String>) -> bool {
    !broken.is_empty() && broken.iter().all(|rule| rule == FACET_REQUIRED)
}

/// One run of the check layer, with the cache off.
///
/// Off deliberately, and not as a precaution. The cache is keyed on the lock
/// digest, so a candidate would take no hit and would write entries under a
/// taxonomy that nobody committed. A later `headwater check` would then read a
/// cache whose contents no lock accounts for.
///
/// `taxonomy audit` reuses it for a second reason that reaches the same answer:
/// that verb gates nothing and exits 0 whatever it reads, so a report of it
/// should not leave cache entries behind as a side effect of being run.
fn run_of(loaded: &Loaded, ctx: &Context) -> headwater_check::Run {
    let mut cache = Cache::disabled();
    headwater_check::run(
        &loaded.census,
        &loaded.graph,
        &loaded.declared(),
        &loaded.claims,
        ctx,
        &mut cache,
    )
}

/// The projection plan of one side, under an identity the caller supplies.
///
/// The identity is a parameter here and nowhere else. Every other caller builds
/// the plan of the run it is in and takes the identity of that run, which is
/// what makes a written projection a statement about the lock beside it. This
/// caller is comparing two taxonomies over one repository, and the repository
/// has one identity for the length of the comparison.
fn plan_of(
    root: &Path,
    loaded: &Loaded,
    identity: &headwater_generate::Identity,
) -> Result<headwater_generate::Plan, String> {
    let projections =
        headwater_generate::Projections::read(&loaded.bound.taxonomy).map_err(|errors| {
            errors
                .iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
    Ok(headwater_generate::plan(
        &loaded.surface(),
        &loaded.census,
        &projections,
        identity,
        &loaded.runs(root),
        headwater_verbs::VERBS,
    ))
}

/// Phase A, once, for every verb that reads a corpus.
///
/// The taxonomy comes from the lock and the corpus block comes from the
/// consumer declaration. The two are different questions: the lock says what
/// the schema is, and `corpus:` says what to walk. Spec 6 keeps them apart too,
/// because a run reports "the corpus tree, the taxonomy lock hash" as two facts.
///
/// One function rather than one per verb. A read that walked a different tree
/// from the one `check` walks would answer about a corpus no run evaluated, and
/// nothing in either report would say so.
/// The taxonomy one load runs against, and where it came from.
///
/// Every verb but one takes this from `.headwater/taxonomy.lock`, which spec 6
/// fixes as the one thing downstream reads. `taxonomy diff` takes a second one
/// out of a published artifact, so that the same phases run twice over one tree
/// and a difference is attributable to the schema. The type exists so that the
/// second case is not a lock value that no lock file produced: a candidate
/// carries no `lock` and therefore reaches no rule that reads one.
struct Bound {
    package: String,
    version: String,
    /// The digest of the canonical taxonomy text, which is a component of every
    /// cache key a run writes.
    digest: String,
    taxonomy: headwater_yaml::Mapping,
    adoption: Option<headwater_yaml::Mapping>,
    /// Where the taxonomy came from, as a path a reader can open. It reaches a
    /// finding, because two rules of the register are about the taxonomy rather
    /// than about the corpus and a finding carries a path.
    source: String,
    /// The lock this was read from, and `None` for a candidate.
    ///
    /// One rule of `headwater conformance` asks whether the lock is current
    /// against the sources on disk, which is a question about a committed file
    /// and not about a taxonomy. So that verb needs the whole value and this is
    /// where it says so.
    lock: Option<headwater_lock::Lock>,
}

impl Bound {
    /// What this repository committed.
    fn of(lock: headwater_lock::Lock) -> Bound {
        Bound {
            package: lock.package.clone(),
            version: lock.version.clone(),
            digest: lock.digest.clone(),
            taxonomy: lock.taxonomy.clone(),
            adoption: lock.adoption.clone(),
            source: headwater_lock::LOCK.to_string(),
            lock: Some(lock),
        }
    }

    /// A taxonomy out of a published artifact, carrying this repository's own
    /// adoption payload.
    ///
    /// The payload rides along so that the two loads of a comparison differ in
    /// the taxonomy and in nothing else. It reclassifies a finding and never an
    /// instance, and the comparison reads instances, so it changes no dimension
    /// either way. Passing it anyway means that no future reader of it has to
    /// discover which of the two sides carried one.
    fn candidate(
        package: &str,
        version: &str,
        resolution: &headwater_resolve::Resolution,
        adoption: Option<&headwater_yaml::Mapping>,
        source: &str,
    ) -> Bound {
        Bound {
            package: package.to_string(),
            version: version.to_string(),
            digest: headwater_lock::digest(&resolution.render()),
            taxonomy: resolution.taxonomy.clone(),
            adoption: adoption.cloned(),
            source: source.to_string(),
            lock: None,
        }
    }
}

struct Loaded {
    bound: Bound,
    /// What this repository takes and what it walks. Held because the corpus
    /// descriptor states the root and the exclusions, and re-reading the
    /// declaration to build one would be a second read that a later edit can
    /// pull apart from this one.
    consumer: headwater_resolve::package::Consumer,
    census: headwater_census::census::Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    register: Register,
    /// The front-matter keys the graph phase reads by name. Held here, and
    /// built once, so the index and the identifier rule read an identifier from
    /// the same key. Two `Config::default()` calls would be two guesses that a
    /// future adopter setting could pull apart.
    config: Config,
    /// The identifier claim store, read off the tree beside the corpus rather
    /// than out of it. Held here, and read once, for the reason `config` is:
    /// every verb that runs the checks reads the same store, and two reads
    /// would be two answers that a mint between them could pull apart. See
    /// [`headwater_check::claim`].
    claims: headwater_check::claim::Claims,
}

fn load(root: &Path) -> Result<Loaded, ExitCode> {
    let lock = match headwater_lock::at(root) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("headwater: {}", err(&format!("{error}")));
            return Err(ExitCode::FAILURE);
        }
    };
    load_against(root, Bound::of(lock))
}

/// The same load, against a taxonomy the caller already holds.
///
/// One function rather than one per source of a taxonomy. `taxonomy diff` runs
/// this twice over one tree, and a second walker built for the candidate would
/// answer about a corpus that no run of `headwater check` evaluates.
fn load_against(root: &Path, bound: Bound) -> Result<Loaded, ExitCode> {
    let consumer = match headwater_resolve::package::consumer(root) {
        Ok(consumer) => consumer,
        Err(errors) => {
            eprintln!(
                "headwater: {}",
                err("the consumer declaration did not read")
            );
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return Err(ExitCode::FAILURE);
        }
    };
    let corpus = Corpus::declared(root, &consumer.corpus_root, &consumer.exclusions);
    let resolved = &bound.taxonomy;
    let taxonomy = match Taxonomy::read(resolved) {
        Ok(taxonomy) => taxonomy,
        Err(errors) => return Err(refused("the taxonomy", &errors)),
    };
    let relations = match Declarations::read(resolved) {
        Ok(declarations) => declarations,
        Err(errors) => return Err(refused("the relation declarations", &errors)),
    };
    let register = match Register::read(resolved) {
        Ok(register) => register,
        Err(errors) => return Err(refused("the obligations and controls", &errors)),
    };
    let shape = match Shape::read(resolved) {
        Ok(shape) => shape,
        Err(errors) => return Err(refused("the facet and kind declarations", &errors)),
    };

    // The resolver set, in the one place every verb that builds a graph reaches
    // it. `Resolvers::over` builds what a corpus supplies, and `with` adds what
    // it cannot: `headwater-import` reads a committed snapshot and depends on
    // `headwater-graph`, so the graph crate cannot name the resolver and this is
    // where the two meet. A repository that declares no import adds nothing and
    // the set is what it was.
    let mut resolvers = Resolvers::over(&corpus);
    let imports = match headwater_import::declared(root) {
        Ok(imports) => imports,
        Err(why) => {
            eprintln!("headwater: {}", err("the import declarations did not read"));
            eprintln!("{}", indent(&err(&why)));
            return Err(ExitCode::FAILURE);
        }
    };
    for items in headwater_import::anchors::over(root, &imports) {
        resolvers = match resolvers.with(Box::new(items)) {
            Ok(resolvers) => resolvers,
            Err(why) => {
                eprintln!("headwater: {}", err("the resolver set is ambiguous"));
                eprintln!("{}", indent(&err(&why)));
                return Err(ExitCode::FAILURE);
            }
        };
    }

    let census = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(&census, &relations, &resolvers, &corpus, &config);
    Ok(Loaded {
        bound,
        consumer,
        census,
        graph,
        shape,
        taxonomy,
        relations,
        register,
        config,
        claims: headwater_check::claim::Claims::at(root),
    })
}

impl Loaded {
    /// What a run of the checks is held against. One constructor, because a
    /// second one is where two runs over one tree start to differ, and
    /// `check --fix` runs the checks twice on purpose.
    fn declared(&self) -> Declared<'_> {
        Declared {
            lock: &self.bound.digest,
            taxonomy: &self.taxonomy,
            shape: &self.shape,
            relations: &self.relations,
            config: &self.config,
            register: &self.register,
            adoption: self.bound.adoption.as_ref(),
            source: &self.bound.source,
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }

    /// What the corpus descriptor states about this repository.
    ///
    /// Assembled here because this is where the lock and the consumer
    /// declaration are both in hand, and passed to the generator as strings so
    /// that the generator keeps no dependency on either crate.
    /// The two inputs a `probe_result` projection is generated from.
    ///
    /// Read here and not inside the generator, because that crate opens no file
    /// of its own: one plan over one tree holds one set of bytes, and a
    /// generator that read the disk could not promise that.
    ///
    /// Each of the three ways this returns less than everything is a state the
    /// projection reports rather than works around. A corpus with no budget
    /// declaration composes no selection, because the selection a transcript is
    /// graded against is the one the plan composed. A transcript this walk
    /// cannot read supplies no source. A corpus that holds no transcript
    /// supplies nothing at all, which is this repository today.
    ///
    /// # The tier below prices a run that already happened, so it reaches
    /// nothing
    ///
    /// `Plan::over` takes a tier and this one is fixed. The tier decides the
    /// arms, the repetitions and the ceiling, and it never filters the
    /// selection, so the probes composed under `regression` are the probes
    /// composed under `campaign`. What a tier can change is a refusal, and the
    /// three refusals it reaches are the three about cost — which
    /// [`headwater_probe::plan::Refusal::stops_a_grade`] drops, because a grade
    /// of a recorded run spends nothing. So a transcript that recorded a
    /// campaign run is graded against the same probes as one that recorded a
    /// regression run, and the fixed tier here states no fact about either.
    fn runs(&self, root: &Path) -> headwater_generate::Runs {
        let mut runs = headwater_generate::Runs::default();
        for row in &self.census.rows {
            let headwater_census::census::Outcome::Typed { kind, .. } = &row.outcome else {
                continue;
            };
            if kind != headwater_probe::intake::KIND {
                continue;
            }
            if let Ok(source) = std::fs::read_to_string(root.join(&row.path)) {
                runs.transcripts.push(headwater_generate::Transcript {
                    path: row.path.clone(),
                    source,
                });
            }
        }
        let Ok(declaration) = std::fs::read_to_string(root.join(headwater_probe::budget::PATH))
        else {
            return runs;
        };
        let Ok(budgets) = headwater_probe::Budgets::read(&declaration) else {
            return runs;
        };
        runs.graded_against(&headwater_probe::Plan::over(
            &self.census,
            &self.graph,
            &self.config,
            &budgets,
            &self.bound.digest,
            headwater_probe::Tier::Regression,
            &headwater_probe::plan::Narrowing::default(),
        ));
        runs
    }

    fn identity(&self) -> headwater_generate::Identity {
        headwater_generate::Identity {
            corpus_root: self.consumer.corpus_root.clone(),
            exclusions: self.consumer.exclusions.clone(),
            package: self.bound.package.clone(),
            version: self.bound.version.clone(),
            lock: self.bound.digest.clone(),
        }
    }
}

/// `headwater route`.
///
/// It exits 0 whether or not it offers a pointer. Spec 5 makes silence a
/// result: "below the threshold it says nothing", and a non-zero exit would
/// make an agent's shell treat a considered silence as a failure.
fn route(root: &Path, task: &str, budget: Option<usize>, json: bool) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let budget = match budget {
        Some(pointers) => Budget { pointers },
        None => Budget::default(),
    };
    let route = loaded.surface().route(task, budget);
    // One route, rendered two ways, and the JSON document carries the text form
    // inside it. `.claude/hooks/intent.sh` is the caller that needs both out of
    // one run: it decides on the pointer set and then puts the report a person
    // reads into an agent's context.
    match json {
        true => print!("{}", headwater_query::json::route(&route)),
        false => print!("{}", route.render()),
    }
    ExitCode::SUCCESS
}

/// `headwater explain`.
///
/// A target that names no document exits non-zero. That is not a finding about
/// a corpus, it is a question about a document that is not there, and a caller
/// who mistyped a path needs to know from the exit status.
///
/// # A target with no document still classifies
///
/// [#319](https://github.com/headwater-ai/headwater/issues/319): the census
/// only ever answers for a path it walked, so a target with no document behind
/// it used to get one sentence whether it named a place this corpus owns, a
/// place it excludes, or nowhere this corpus has ever heard of. `.claude/hooks/write.sh`
/// asks exactly this question of a path that does not exist yet, and it used
/// to answer it with a matcher of its own rather than waiting on this verb.
/// [`Corpus::classify`] is the one matcher now, reached here and by
/// `.claude/hooks/write.sh` alike, over `Corpus::declared` rebuilt from the
/// same [`headwater_resolve::package::Consumer`] `load` already read — no
/// second read of the declaration, because nothing in it is re-read from
/// disk.
///
/// The exit status stays non-zero and `--json` still writes nothing
/// ([HW-DR-0043](../../../../docs/decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md)):
/// this remains a refusal, and what changes is only the English sentence a
/// caller, or a hook, reads on standard error.
fn explain(root: &Path, target: &str, json: bool) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    match loaded.surface().explain(target) {
        Some(explanation) => {
            let explanation: headwater_query::Explanation = explanation;
            // A target that names no document is refused below, on standard
            // error and with the same exit status either way. `--json` selects
            // the artifact and never the status: a refusal is not a document
            // with a member missing from it.
            match json {
                true => print!("{}", headwater_query::json::explain(&explanation)),
                false => print!(
                    "{}",
                    explanation.render(headwater_cli::paint::stdout_color())
                ),
            }
            ExitCode::SUCCESS
        }
        None => {
            let corpus = Corpus::declared(
                root,
                &loaded.consumer.corpus_root,
                &loaded.consumer.exclusions,
            );
            eprintln!(
                "headwater: {}",
                err(&classification_text(
                    target,
                    &corpus.classify(Path::new(target))
                ))
            );
            ExitCode::FAILURE
        }
    }
}

/// The sentence [`explain`]'s refusal prints for a target with no document,
/// one per state of [`headwater_census::walk::Classification`].
///
/// A closed match rather than a `Display` on the type itself: the type lives
/// in `headwater_census`, which states facts about a corpus and states them
/// to every crate that reads it, and the words a caller reads belong to the
/// one binary that owns a caller.
fn classification_text(
    target: &str,
    classification: &headwater_census::walk::Classification,
) -> String {
    use headwater_census::walk::Classification;
    match classification {
        Classification::Corpus => {
            format!("`{target}` is a path of this corpus, with no document written there yet")
        }
        Classification::Excluded(pattern) => {
            format!("`{target}` is excluded by `{pattern}`, so it is not corpus content")
        }
        Classification::Outside => {
            format!("`{target}` is outside every corpus root this repository declares")
        }
        Classification::Unclassifiable => {
            format!("`{target}` is not a path this repository can classify")
        }
    }
}

/// `headwater new <kind>`: the scaffolder.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
/// makes this a correctness root, and `headwater_scaffold` carries the argument
/// for the three properties that follow from it. This function is the shell:
/// it loads, it proposes, it prints, and it writes.
///
/// # The report states where every value came from
///
/// A scaffolder that printed only what it wrote would ask a reader to trust the
/// output because a tool produced it, which is the sentence the issue exists to
/// refuse. So every field names the declaration behind it, and the assisted
/// fraction at the end is the count of those origins rather than a number this
/// verb keeps beside them.
///
/// **What the fraction measures.** One run. It is not a property of the corpus
/// and it cannot become one:
/// [Q4](../../../../docs/decisions/0004-relation-storage.md) keeps `created_by`
/// on the relation type, so a later reader of a committed corpus cannot tell a
/// scaffolded edge from a hand-typed one.
/// [HW-OBL-0001](../../../../docs/obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md)
/// holds the debt that nothing trends this number yet.
fn new(
    root: &Path,
    kind: &str,
    title: Option<String>,
    relates: &[(String, String)],
    given: &[(String, String)],
    now: Option<Date>,
) -> ExitCode {
    let Some(title) = title else {
        return fail(
            "`new` takes `--title <text>`. The file name and the document's own name both come \
             from it, and this engine invents neither",
        );
    };
    match scaffold(
        root,
        kind,
        &title,
        relates,
        given,
        now,
        EntryPoint::Terminal,
    ) {
        Err(why) => refuse(&why),
        Ok(written) => {
            print!("{}", written.artifact);
            eprint!("{}", written.account);
            match written.ok {
                true => ExitCode::SUCCESS,
                false => ExitCode::FAILURE,
            }
        }
    }
}

/// One run of the scaffolder, held rather than printed.
///
/// The verb above is the shell that prints it, and the `new` tool of the MCP
/// server is the second caller. Two callers and one function, because a
/// scaffolded document that differed by the surface that asked for it would be
/// the second authoring path this repository rules against.
///
/// **The surface is an argument here and a decision at each boundary.** It is
/// the term [HW-OBL-0004](../../../../docs/obligations/0004-working-tree-write-tools-have-no-measured-effect.md)
/// asks the capture-cost store for, and it is the one input that is a fact
/// about the caller rather than about the corpus.
fn scaffold(
    root: &Path,
    kind: &str,
    title: &str,
    relates: &[(String, String)],
    given: &[(String, String)],
    now: Option<Date>,
    surface: EntryPoint,
) -> Result<Written, String> {
    // `load` writes its own account to standard error, which is where a
    // terminal reads it and where the process serving a protocol call keeps it.
    let loaded = load(root).map_err(|_| "the corpus did not load".to_string())?;
    let now = match now {
        Some(now) => now,
        None => match Context::from_system_clock() {
            Some(context) => context.now(),
            None => {
                return Err(
                    "the host clock is before the epoch, and this engine will not guess a date"
                        .to_string(),
                )
            }
        },
    };

    let index = headwater_graph::index::Index::build(&loaded.census, &loaded.config);
    let sources = headwater_scaffold::Sources {
        resolved: &loaded.bound.taxonomy,
        shape: &loaded.shape,
        shelves: &loaded.taxonomy,
        relations: &loaded.relations,
        census: &loaded.census,
        index: &index,
        config: &loaded.config,
        claims: &loaded.claims,
    };
    let request = headwater_scaffold::Request {
        kind,
        title,
        now,
        relates,
        given,
    };

    // Nothing below this line has written anything yet, which is why every
    // refusal here is a refusal with an unchanged tree behind it.
    let plan = headwater_scaffold::propose(&sources, &request).map_err(|why| why.to_string())?;
    let composed =
        headwater_scaffold::write::compose(root, &plan).map_err(|why| why.to_string())?;
    // The claim, then the document. Nothing above this line has written a byte,
    // so a claim that cannot be made refuses with the tree untouched. See
    // `headwater_scaffold::claim` for why this order and not the other one.
    let claimed = headwater_scaffold::claim::write(root, &plan).map_err(|why| why.to_string())?;
    headwater_scaffold::write::apply(root, &composed).map_err(|why| why.to_string())?;

    let reading =
        headwater_scaffold::reading::Reading::of(&plan, &loaded.bound.digest, now, surface);
    let recorded = headwater_scaffold::reading::append(root, &reading);
    Ok(Written {
        artifact: scaffold_report(&plan, &composed, claimed.as_deref(), recorded.is_ok()),
        // The document landed and its reading did not, which is the one outcome
        // a store of this shape cannot report later: a run with no reading and a
        // corpus that never ran the verb are the same file. So the run says so
        // and fails, rather than leaving a silent hole in a denominator that
        // `headwater capture` would then report as reach.
        account: match &recorded {
            Ok(()) => String::new(),
            Err(why) => format!(
                "headwater: the document landed and its capture-cost reading did not. {why}. \
                 Append this line to `{}` by hand, or the run is invisible to `headwater \
                 capture`:\n{}\n",
                headwater_scaffold::reading::STORE,
                reading.render()
            ),
        },
        // The document is on disk either way, and that is what the seal reads.
        landed: true,
        ok: recorded.is_ok(),
    })
}

/// What `headwater new` writes to standard output.
///
/// Held apart from the verb so that a test reads the report of a plan without
/// a process and without a tree.
fn scaffold_report(
    plan: &headwater_scaffold::Plan,
    composed: &[headwater_scaffold::write::Composed],
    claimed: Option<&str>,
    recorded: bool,
) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    for file in composed {
        let verb = match file.created {
            true => "wrote",
            false => "edited",
        };
        let _ = writeln!(out, "{verb} {}", file.path);
    }
    // The claim, named where the reader is already reading what this run
    // wrote. It is not a document, so it is not in `composed`.
    if let Some(claim) = claimed {
        let _ = writeln!(out, "claimed {claim}");
    }

    let _ = writeln!(out, "\nwhat the taxonomy decided");
    let _ = writeln!(out, "  kind {} on the shelf `{}`", plan.kind, plan.shelf);
    if let Some(minting) = &plan.minting {
        let _ = writeln!(
            out,
            "  identifier {} under `{}`, allocation {}",
            minting.id,
            minting.scheme,
            minting.allocation.as_deref().unwrap_or("unstated")
        );
        if let Some(highest) = minting.reconciled_from {
            let _ = writeln!(
                out,
                "    reconciled against {highest}, which is the highest value this tree and \
                 the identifier claim store carry between them. A document that was deleted is \
                 not on the tree, and the store outlives it, so the pair is a lower bound only \
                 on a value that no claim recorded"
            );
        }
    } else {
        let _ = writeln!(
            out,
            "  no identifier: `{}` names no scheme, and no relation may name a document of it",
            plan.kind
        );
    }
    for field in &plan.fields {
        let _ = writeln!(out, "  {} — {}", field.key, field.origin.reason());
    }
    for section in &plan.sections {
        let _ = writeln!(
            out,
            "  section `{}` — the kind requires it",
            section.heading
        );
    }

    if !plan.edges.is_empty() {
        let _ = writeln!(out, "\nthe edges it proposed");
        for edge in &plan.edges {
            let _ = writeln!(
                out,
                "  {} {} — `created_by: {}`, so a scaffold pays for it",
                edge.relation, edge.target, edge.created_by
            );
            match &edge.reciprocal {
                Some(half) => {
                    let _ = writeln!(
                        out,
                        "    the far half `{}` went into {}, because reciprocity is required",
                        half.relation, half.path
                    );
                }
                None => {
                    let _ = writeln!(out, "    the relation asks for no far half");
                }
            }
        }
        let _ = writeln!(
            out,
            "  no facet of another document moved. `on_target` is a lifecycle event, and no \
             rule of this engine reads a transition"
        );
    }

    if !plan.expected.is_empty() {
        let _ = writeln!(out, "\nwhat this document may also declare, and nobody did");
        for expected in &plan.expected {
            let _ = writeln!(
                out,
                "  {} to {} — `created_by: {}`",
                expected.relation,
                expected.to.join(", "),
                expected.created_by
            );
        }
    }

    let assisted = plan.assisted();
    let _ = writeln!(out, "\nassisted fraction of this run");
    let _ = writeln!(
        out,
        "  {} of {} — front matter {}/{}, sections {}/{}, identifier {}/{}, edge halves {}/{}",
        assisted.supplied(),
        assisted.total(),
        assisted.fields.0,
        assisted.fields.1,
        assisted.sections.0,
        assisted.sections.1,
        assisted.identifier.0,
        assisted.identifier.1,
        assisted.edge_halves.0,
        assisted.edge_halves.1,
    );
    let _ = writeln!(
        out,
        "  It counts a section heading and never its prose, and it counts one run rather than \
         this corpus. Q4 keeps `created_by` on the relation type, so no reader of a committed \
         corpus can tell a scaffolded edge from a hand-typed one"
    );
    if recorded {
        let _ = writeln!(
            out,
            "  recorded in `{}`, which is where it trends. It names no person and no agent, \
             and `headwater capture` reads it back",
            headwater_scaffold::reading::STORE
        );
    }
    let _ = writeln!(
        out,
        "\nRun `headwater check` over the result. Nothing this verb wrote is exempt from a rule"
    );
    out
}

/// `headwater capture`: the capture-cost store, read back.
///
/// # It reports a population before it reports a number
///
/// A capture-cost number with no population beside it is a number nobody should
/// cite, and this verb is the reader of a store that starts empty on every
/// corpus. So the report leads with what the store holds and from when, and the
/// reach figure names the date of the first reading. Every document written
/// before that date carries no reading and never could, which is a fact about
/// the store rather than about the authoring of those documents.
///
/// # What it refuses to do
///
/// **It does not average across taxonomies.** The denominator is a count of
/// declarations, so a required facet that the scaffolder can fill raises the
/// fraction with no change in what an author typed. The report states how many
/// lock digests the readings span, and a run that spans more than one says so
/// on the same line as the aggregate.
///
/// **It does not silently absorb a document that arrived by another route.** A
/// document written by any means other than `headwater new` is classified by
/// the census and named by no reading, so it raises the reach denominator and
/// nothing else. The number falls, which is the honest direction.
///
/// **It does not count a reading whose document is gone.** Such a reading is
/// listed by name, and a reader then knows whether the store is describing a
/// tree that still exists.
fn capture(root: &Path, format: Option<String>) -> ExitCode {
    let wants_json = match format.as_deref() {
        None | Some("text") => false,
        Some("json") => true,
        Some(other) => {
            return refuse(&format!(
                "`capture --format {other}` names no target. It writes `text` and `json`"
            ))
        }
    };
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let (readings, unreadable) = match headwater_scaffold::reading::load(root) {
        Ok(held) => held,
        Err(why) => {
            return refuse(&format!(
                "`{}` would not read: {why}",
                headwater_scaffold::reading::STORE
            ))
        }
    };

    // What the corpus says, in the two forms the join needs. The census decides
    // what is classified and the index decides what an identifier resolves to,
    // so this verb re-derives neither and the join itself is one function that a
    // test states a corpus to.
    let index = headwater_graph::index::Index::build(&loaded.census, &loaded.config);
    let classified = headwater_scaffold::reading::Classified {
        paths: loaded
            .census
            .rows
            .iter()
            .filter(|row| matches!(row.outcome, headwater_census::census::Outcome::Typed { .. }))
            .map(|row| row.path.clone())
            .collect(),
        identified: index
            .typed
            .iter()
            .map(|node| (node.id.clone(), node.path.clone()))
            .collect(),
    };
    let reach = headwater_scaffold::reading::reach(&readings, &classified);

    // One reading and two readings, because a report that says "1 readings" is
    // a report a person stops reading.
    let readings_of = |count: usize| match count {
        1 => "1 reading".to_string(),
        other => format!("{other} readings"),
    };
    let total = headwater_scaffold::reading::total(&readings);
    let locks = headwater_scaffold::reading::locks(&readings);
    let first = readings.iter().map(|reading| reading.date).min();
    let last = readings.iter().map(|reading| reading.date).max();

    if wants_json {
        println!(
            "{}",
            headwater_scaffold::json::render(&readings, &unreadable, &classified, &reach)
        );
        return ExitCode::SUCCESS;
    }

    println!("capture-cost store");
    println!(
        "  {}, which is outside the corpus root. No census row covers it, no language regime \
         binds it, and no rule reads it",
        headwater_scaffold::reading::STORE
    );
    match first {
        None => println!(
            "  no reading. This corpus has not run `headwater new` since the store existed, and \
             every number below is empty rather than zero"
        ),
        Some(first) => println!(
            "  {}, {} to {}",
            readings_of(readings.len()),
            first,
            last.unwrap_or(first)
        ),
    }
    for line in &unreadable {
        println!(
            "  line {} is not a reading and is counted nowhere: {}",
            line.line, line.why
        );
    }

    if !readings.is_empty() {
        println!("\nassisted fraction over every reading");
        println!(
            "  {} of {} — front matter {}/{}, sections {}/{}, identifier {}/{}, edge halves \
             {}/{}",
            total.supplied(),
            total.total(),
            total.fields.0,
            total.fields.1,
            total.sections.0,
            total.sections.1,
            total.identifier.0,
            total.identifier.1,
            total.edge_halves.0,
            total.edge_halves.1,
        );
        println!(
            "  It counts a section heading and never its prose, which is the convention spec 3 \
             fixes and this aggregate inherits"
        );
        match locks.len() {
            1 => println!(
                "  every reading was taken under {}, so they share a denominator",
                locks[0]
            ),
            many => {
                println!(
                    "  {many} taxonomies produced these readings, so the aggregate above is \
                     across two denominators and is not a trend"
                );
                for lock in &locks {
                    println!("    {lock}");
                }
            }
        }

        println!("\nby kind");
        for (kind, taken, assisted) in headwater_scaffold::reading::by_kind(&readings) {
            println!(
                "  {kind} — {}, {} of {}",
                readings_of(taken),
                assisted.supplied(),
                assisted.total()
            );
        }

        // The two arms of HW-OBL-0004, as a grouping and never as a
        // comparison. Q7 claims that a write tool raises the fraction, and a
        // claim of that shape needs a powered comparison rather than two rows
        // that differ.
        println!("\nby surface");
        for (surface, taken, assisted) in headwater_scaffold::reading::by_surface(&readings) {
            println!(
                "  {} — {}, {} of {}",
                match surface {
                    Some(surface) => surface.name(),
                    None => "no surface stated",
                },
                readings_of(taken),
                assisted.supplied(),
                assisted.total()
            );
        }
        if readings.iter().any(|reading| reading.surface.is_none()) {
            println!(
                "  a reading that states no surface was taken before the term existed. It is not \
                 the terminal arm under another name, and this report counts it as neither"
            );
        }
        println!(
            "  the surface is the entry point a run was made at, and never who drove it. A person \
             at a client and an agent at the same client are one reading, which HW-OBL-0111 \
             records"
        );
    }

    println!("\nreach of the authoring verb");
    println!(
        "  {} of {} classified documents carry a reading",
        reach.reached.len(),
        classified.paths.len()
    );
    match first {
        None => println!(
            "  the store holds no reading, so the reach is zero by construction rather than by \
             measurement"
        ),
        Some(first) => println!(
            "  the first reading is dated {first}. Every document written before that day carries \
             none and never could, so read this against the store's own age"
        ),
    }
    println!(
        "  a document written by any other route is classified here and named by no reading, so \
         it lowers this number rather than being absorbed by it"
    );
    for (at, path) in &reach.moved {
        println!(
            "  {} was written to {} and is now at {path}, joined by its identifier",
            readings[*at].id.as_deref().unwrap_or("a reading"),
            readings[*at].document
        );
    }
    match reach.lost.is_empty() {
        true => println!("  no reading names a document this corpus does not classify"),
        false => {
            println!(
                "  {} name a document this corpus does not classify, and none of them is counted \
                 above",
                readings_of(reach.lost.len())
            );
            for at in &reach.lost {
                println!("    {}", readings[*at].document);
            }
        }
    }

    println!("\nwhat this store does not hold, and why");
    println!(
        "  no person and no agent. Spec 3 aims the remedy for a falling fraction at the taxonomy \
         rather than at the author, and a per-author number is a performance measure"
    );
    println!("  no wall-clock time. A duration is not reproducible under `--now`");
    println!(
        "  no run that refused. A refusal wrote no document, so there is nothing to attribute a \
         reading to"
    );
    println!(
        "  nothing a hook or a skill did. Spec 5 says a hook binds nothing, so a disabled hook \
         and a hook that stayed silent would be one reading"
    );
    ExitCode::SUCCESS
}

/// `headwater sweep plan`.
///
/// The briefing an agent reads. It writes to standard output, reads the same
/// lock every other verb reads, and touches nothing.
///
/// It is deterministic, and that is testable: two runs over one tree write the
/// same bytes. The sweep's unreproducible part is what an agent does between
/// this verb and the next one, and neither verb performs it.
fn sweep_plan(root: &Path, under: Option<String>) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let plan = headwater_sweep::Plan::over(
        &loaded.census,
        &loaded.graph,
        &loaded.config,
        &loaded.bound.digest,
        under.as_deref().unwrap_or(""),
    );
    print!("{}", plan.render(headwater_cli::paint::stdout_color()));
    ExitCode::SUCCESS
}

/// `headwater sweep report <path>`.
///
/// # It exits 0 on every report it can produce, and that is the constraint
///
/// A sweep never gates
/// ([spec 12](../../../../docs/spec/12-check-layer.md#where-the-llm-coherence-sweep-fits)).
/// The strongest form of that promise is an exit status that no output of a
/// model can move, so this verb exits 0 with findings, exits 0 with every
/// finding refused, and exits 0 when it refuses the whole file. There is no
/// `--strict`.
///
/// The two non-zero exits are a caller's rather than a model's: a path this
/// process cannot read, and a `--format` that names no target. Both are true
/// before any file is parsed.
///
/// # It writes nothing
///
/// The report prints the front matter that would declare a proposed edge and
/// never writes it. A proposal an agent applies to itself is the same act as an
/// agent accepting its own draft, which is what
/// [HW-OBL-0108](../../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md)
/// records. So there is no `--write`, and this is the one verb of the write
/// path that has none.
fn sweep_report(root: &Path, path: &Path, format: Option<String>) -> ExitCode {
    let wants_json = match format.as_deref() {
        None | Some("text") => false,
        Some("json") => true,
        Some(other) => {
            return refuse(&format!(
                "`sweep report --format {other}` names no target. It writes `text` and `json`"
            ))
        }
    };
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return fail(&format!(
                "the sweep file at {} did not read: {error}",
                path.display()
            ))
        }
    };
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let tree = headwater_sweep::Tree {
        root,
        census: &loaded.census,
        graph: &loaded.graph,
        relations: &loaded.relations,
        shape: &loaded.shape,
        lock: &loaded.bound.digest,
    };
    let report = headwater_sweep::Report::read(&source, &tree);
    match wants_json {
        true => println!("{}", headwater_sweep::json::render(&report)),
        false => print!("{}", report.render(headwater_cli::paint::stdout_color())),
    }
    ExitCode::SUCCESS
}

/// `headwater probe plan`.
///
/// # It exits 0 on a refusal, and that is the point rather than a leniency
///
/// A probe never gates
/// ([spec 5](../../../../docs/spec/05-ai-integration.md#two-tiers-and-the-cadence-follows-the-purpose)),
/// so no exit status of this binary may carry a fact about a probe run. A plan
/// that refuses its own run prints the refusal and exits 0, exactly as `sweep
/// report` does with a refused file. A caller that wants the refusal reads the
/// text, which is what a person does.
///
/// The non-zero exits are a caller's rather than a run's: a tier, an arm or a
/// category that names nothing, and a budget declaration this engine cannot
/// read. All four are true before a corpus is walked.
///
/// # It writes nothing, and it reaches nothing
///
/// The plan goes to standard output. No socket is opened here, no crate below
/// can open one, and the run this plan describes is performed by a recorder
/// that is not in this repository.
fn probe_plan(
    root: &Path,
    tier: Option<&str>,
    arm: Option<&str>,
    category: Option<&str>,
    seed: u64,
) -> ExitCode {
    let tier = match tier {
        None => headwater_probe::Tier::Regression,
        Some(name) => match headwater_probe::Tier::read(name) {
            Some(tier) => tier,
            None => {
                return refuse(&format!(
                    "`--tier {name}` names no tier. The tiers are `regression` and `campaign`"
                ))
            }
        },
    };
    let narrowing = headwater_probe::plan::Narrowing {
        category: match category {
            None => None,
            Some(name) => match headwater_probe::Category::read(name) {
                Some(category) => Some(category),
                None => {
                    return refuse(&format!(
                        "`--category {name}` names no probe category. They are: {}",
                        headwater_probe::Category::ALL
                            .iter()
                            .map(|category| category.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                }
            },
        },
        arm: match arm {
            None => None,
            Some(name) => match headwater_probe::Arm::read(name) {
                Some(arm) => Some(arm),
                None => {
                    return refuse(&format!(
                        "`--arm {name}` names no arm. The arms are `present` and `absent`"
                    ))
                }
            },
        },
        seed,
    };

    let path = root.join(headwater_probe::budget::PATH);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            return refuse(&format!(
                "{} did not read: {error}. A harness with no declared ceiling cannot fail closed, \
                 so no run is planned without one",
                headwater_probe::budget::PATH
            ))
        }
    };
    let budgets = match headwater_probe::Budgets::read(&source) {
        Ok(budgets) => budgets,
        Err(unreadable) => return refuse(&unreadable.to_string()),
    };

    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let plan = headwater_probe::Plan::over(
        &loaded.census,
        &loaded.graph,
        &loaded.config,
        &budgets,
        &loaded.bound.digest,
        tier,
        &narrowing,
    );
    print!("{}", plan.render());
    ExitCode::SUCCESS
}

/// `headwater probe grade <path>`.
///
/// The one verb of this binary that returns a verdict, and the exit status
/// still carries none. A probe never gates
/// ([spec 5](../../../../docs/spec/05-ai-integration.md#two-tiers-and-the-cadence-follows-the-purpose)),
/// so a run where every expectation was refuted exits 0 exactly as a run where
/// every one was satisfied does. A caller that wants the rate reads the text,
/// which is what a person does.
///
/// It reads the budget declaration for the same reason `plan` does: the
/// selection a transcript is graded against is the selection the plan composed,
/// and re-deriving it here from the corpus rather than trusting the transcript
/// is what holds a result to the probes this tree declares.
///
/// It writes nothing and it reaches nothing. No socket is opened here, no crate
/// below can open one, and no gate, hook or CI step calls this verb.
fn probe_grade(root: &Path, path: &Path) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return fail(&format!(
                "the transcript at {} did not read: {error}",
                path.display()
            ))
        }
    };
    let declaration = root.join(headwater_probe::budget::PATH);
    let budgets = match std::fs::read_to_string(&declaration) {
        Ok(source) => match headwater_probe::Budgets::read(&source) {
            Ok(budgets) => budgets,
            Err(unreadable) => return refuse(&unreadable.to_string()),
        },
        Err(error) => {
            return refuse(&format!(
                "`{}` did not read: {error}. A grade names the selection it was taken over, and \
                 the selection comes from the plan",
                headwater_probe::budget::PATH
            ))
        }
    };
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let plan = headwater_probe::Plan::over(
        &loaded.census,
        &loaded.graph,
        &loaded.config,
        &budgets,
        &loaded.bound.digest,
        headwater_probe::Tier::Regression,
        &headwater_probe::plan::Narrowing::default(),
    );
    // A plan returns from inside the loop that composes its selection, so a
    // refusal leaves the probes it had read and none of the rest. Grading
    // against that part reports a rate over a denominator no document declares,
    // and one that moves with the order the paths sort in. `Plan::gradable` is
    // the one route to a selection that anything may grade against, and it
    // holds `Refusal::stops_a_grade` for every caller. Testing the selection
    // for emptiness here instead let every late refusal through with a partial
    // one.
    let selected = match plan.gradable() {
        Ok(selected) => selected,
        Err(refusal) => {
            println!("Nothing was graded. `headwater probe plan` refuses this corpus: {refusal}");
            return ExitCode::SUCCESS;
        }
    };
    let tree = headwater_probe::intake::Tree {
        census: &loaded.census,
        config: &loaded.config,
        lock: &loaded.bound.digest,
    };
    let record = headwater_probe::Record::read(&source, &tree);
    let results = headwater_probe::Results::over(&record, selected);
    print!("{}", results.render());
    ExitCode::SUCCESS
}

/// `headwater probe record <path>`.
///
/// It reads a transcript back and reports what this engine could confirm about
/// it. It grades nothing: a verdict is a function of the transcript, the
/// expectations and a grader version, and `headwater probe grade` is where that
/// version lives.
///
/// It exits 0 on every record it can produce, including a transcript it refuses
/// whole, for the reason `sweep report` does. The two non-zero exits are a
/// caller's: no path, and a path this process cannot read.
fn probe_record(root: &Path, path: &Path) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return fail(&format!(
                "the transcript at {} did not read: {error}",
                path.display()
            ))
        }
    };
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let tree = headwater_probe::intake::Tree {
        census: &loaded.census,
        config: &loaded.config,
        lock: &loaded.bound.digest,
    };
    let record = headwater_probe::Record::read(&source, &tree);
    print!("{}", record.render());
    ExitCode::SUCCESS
}

/// `headwater probe stale`.
///
/// # It reads transcripts, and a transcript is what the committed result is
///
/// The question is which committed results a change voided, and this walk reads
/// transcripts rather than result documents. The two are one artifact: a probe
/// result is a projection of the transcript, the expectations and the grader
/// version, and `generate --check` holds every committed result to that
/// derivation on every pull request. So a transcript names the read set of the
/// result derived from it, and reading the transcript avoids parsing prose that
/// this engine wrote.
///
/// # No exit status of this verb carries an answer
///
/// [`headwater_probe::read_set::Staleness::render`] returns a string, and this
/// function returns [`ExitCode::SUCCESS`] after every answer it can produce.
/// The two non-zero exits are a caller's and both are true before a read set is
/// composed: a corpus this binary cannot load, and a budget declaration it
/// cannot read. A probe never gates
/// ([spec 5](../../../../docs/spec/05-ai-integration.md#two-tiers-and-the-cadence-follows-the-purpose)),
/// and a result going stale is the fact a gate over this verb would carry.
fn probe_stale(root: &Path) -> ExitCode {
    let declaration = root.join(headwater_probe::budget::PATH);
    let budgets = match std::fs::read_to_string(&declaration) {
        Ok(source) => match headwater_probe::Budgets::read(&source) {
            Ok(budgets) => budgets,
            Err(unreadable) => return refuse(&unreadable.to_string()),
        },
        Err(error) => {
            return refuse(&format!(
                "`{}` did not read: {error}. A read set covers the probes of a selection, and the \
                 selection comes from the plan",
                headwater_probe::budget::PATH
            ))
        }
    };
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let plan = headwater_probe::Plan::over(
        &loaded.census,
        &loaded.graph,
        &loaded.config,
        &budgets,
        &loaded.bound.digest,
        headwater_probe::Tier::Regression,
        &headwater_probe::plan::Narrowing::default(),
    );
    // A plan returns from inside the loop that composes its selection, so a
    // refusal leaves a part of one behind: the probes read before the offending
    // one and none of the rest. A read set over that part covers a population
    // no document declares, and it moves with the order the paths sort in.
    // `Plan::gradable` is the one route to a selection anything may grade
    // against, and this verb answers about a grade that already happened.
    if let Err(refusal) = plan.gradable() {
        println!(
            "No read set is composed over this corpus, so nothing here is stale or fresh: \
             {refusal}"
        );
        return ExitCode::SUCCESS;
    }

    let tree = headwater_probe::intake::Tree {
        census: &loaded.census,
        config: &loaded.config,
        lock: &loaded.bound.digest,
    };
    let mut seen = 0usize;
    let mut stale = 0usize;
    for row in &loaded.census.rows {
        let headwater_census::census::Outcome::Typed { kind, .. } = &row.outcome else {
            continue;
        };
        if kind != headwater_probe::intake::KIND {
            continue;
        }
        seen += 1;
        println!("## The result of {}", row.path);
        println!();
        let source = match std::fs::read_to_string(root.join(&row.path)) {
            Ok(source) => source,
            Err(error) => {
                println!(
                    "The transcript did not read, so nothing here decides whether it is stale: \
                     {error}"
                );
                println!();
                continue;
            }
        };
        let record = headwater_probe::Record::read(&source, &tree);
        let staleness = headwater_probe::read_set::Staleness::over(&record, &plan, &loaded.census);
        if !staleness.verdict().stands() {
            stale += 1;
        }
        print!("{}", staleness.render());
        println!();
    }

    match seen {
        0 => println!(
            "This corpus holds no `{}` document, so no result has been recorded and a change \
             voids nothing. A transcript is written by a recorder that observes a session from \
             outside it, and no verb of this engine writes one.",
            headwater_probe::intake::KIND
        ),
        seen => println!(
            "Of {}, this tree moved the read set of {}.",
            headwater_probe::plural(seen, "committed transcript"),
            stale
        ),
    }
    ExitCode::SUCCESS
}

/// `headwater import`, and `--write` over the documents it names.
///
/// A dry run and a real one differ by one call, because the plan is what both
/// print and the write is a loop over what the plan composed.
///
/// It exits non-zero on any refusal, and there is no advisory posture to fall
/// back on. [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
/// says why: a wrong imported edge produces a correct check result over a wrong
/// graph, so a finding is what a later run cannot make. Either an import is
/// refused here or nothing downstream is going to notice.
fn import(root: &Path, name: Option<&str>, expect: Option<&str>, writing: bool) -> ExitCode {
    // `refuse` (#455, the ninth site): `declared` reads
    // `.headwater/taxonomy.yml`, which is a fixed location this engine reads on
    // every run, and every error it returns is a fact about that file. No
    // spelling of `headwater import` gets past one. The sibling call in `load`
    // already reports without the pointer, so this was the odd one out.
    let declarations = match headwater_import::declared(root) {
        Ok(declarations) => declarations,
        Err(why) => return refuse(&why),
    };
    let names: Vec<String> = declarations
        .iter()
        .map(|declaration| declaration.name.clone())
        .collect();
    let declaration = match name {
        Some(name) => declarations
            .iter()
            .find(|declaration| declaration.name == name),
        // One declared import needs no name on the command line, and two do.
        // Choosing for the caller where there are two would import whichever
        // one the file happened to list first.
        None => match declarations.len() {
            1 => declarations.first(),
            _ => None,
        },
    };
    let Some(declaration) = declaration else {
        return match name {
            Some(name) => refuse(
                &headwater_import::Refusal::Undeclared {
                    name: name.to_string(),
                    declared: names,
                }
                .to_string(),
            ),
            None if names.is_empty() => refuse(
                "this repository declares no import. An import is a block under `imports` in \
                 `.headwater/taxonomy.yml` naming where a committed snapshot sits, the digest it \
                 is pinned to, and the channel that digest arrived on",
            ),
            None => fail(&format!(
                "this repository declares {} imports, so `import` takes the name of one: {}",
                names.len(),
                names.join(", ")
            )),
        };
    };

    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let corpus = headwater_import::Corpus {
        index: &loaded.graph.index,
        relations: &loaded.relations,
        shape: &loaded.shape,
    };
    let plan = match headwater_import::plan(root, declaration, expect, &corpus) {
        Ok(plan) => plan,
        Err(refusals) => {
            eprintln!("headwater: {}", err("nothing was imported"));
            eprint!("{}", indent(&headwater_import::render(&refusals)));
            return ExitCode::FAILURE;
        }
    };

    print!("{}", plan.render());
    let pending = plan.to_write();
    if !writing {
        println!(
            "\n{} to write, and nothing was written. Run it again with `--write`.",
            headwater_import::plural(pending.len(), "edge half", "edge halves")
        );
        return ExitCode::SUCCESS;
    }

    let composed = match headwater_import::write::compose(root, &pending) {
        Ok(composed) => composed,
        Err(why) => {
            eprintln!("headwater: {}", err("nothing was written"));
            eprintln!("{}", indent(&err(&why)));
            return ExitCode::FAILURE;
        }
    };
    // The headline comes off the refusal rather than out of this line. A run
    // refused at the reservation wrote nothing, and this call site printed *the
    // write stopped part way* over it for as long as one sentence covered both
    // phases.
    if let Err(unwritten) = headwater_import::write::apply(root, &composed) {
        eprintln!("headwater: {}", err(unwritten.headline()));
        eprintln!("{}", indent(&err(&unwritten.to_string())));
        return ExitCode::FAILURE;
    }
    println!(
        "\nwrote {} into {}",
        headwater_import::plural(pending.len(), "edge half", "edge halves"),
        headwater_import::plural(composed.len(), "document", "documents")
    );
    println!(
        "The digest says these are the bytes the pin was written for. What stands behind them is \
         the channel above, and `headwater check` reads the result as it reads any other edge."
    );
    ExitCode::SUCCESS
}

/// `headwater generate`, and `--check` over what is committed.
///
/// Advisory is not the default here, and that is deliberate. Spec 6 fixes the
/// advisory posture for `check`, which reports findings about prose a person
/// wrote. A projection that differs from its source is not a finding about a
/// document, it is an artifact that is out of date, and the remedy is one
/// command rather than a judgment. So a difference exits non-zero, in the way
/// `taxonomy resolve --check` does over a stale lock.
fn generate(root: &Path, check_only: bool) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let projections = match headwater_generate::Projections::read(&loaded.bound.taxonomy) {
        Ok(projections) => projections,
        Err(errors) => return refused("the projections", &errors),
    };
    let surface = loaded.surface();
    let plan = headwater_generate::plan(
        &surface,
        &loaded.census,
        &projections,
        &loaded.identity(),
        &loaded.runs(root),
        headwater_verbs::VERBS,
    );
    let report = match check_only {
        true => headwater_generate::check(root, &plan),
        false => headwater_generate::write(root, &plan),
    };
    print!("{}", report.render());
    if report.has_errors() {
        // A projection that drifted and a marked file this run did not write
        // are two failures with two remedies, and printing the first remedy
        // for the second tells a reader to run the verb that cannot help.
        let drifted = report.wrote.iter().any(|wrote| wrote.verdict.is_error());
        match (check_only, drifted) {
            (true, true) => eprintln!(
                "headwater: {}",
                err(
                    "a projection is not what this corpus and this lock produce. Run \
                     `headwater generate` and commit the result"
                )
            ),
            (true, false) => eprintln!(
                "headwater: {}",
                err(
                    "a marked file is committed that this run does not write. Running this \
                     verb again writes it no more, and the line under it above says why"
                )
            ),
            (false, _) => eprintln!("headwater: {}", err("a projection did not write")),
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// `headwater export`.
///
/// Two modes, and the flag that separates them is `--format`. Spec 6 gives the
/// reason for both in one sentence: export "carries its own verb because a
/// consumer outside the repository asks for one format at a time".
///
/// **With a target named**, the artifact goes to standard output. That consumer
/// holds no clone, wants one vocabulary, and takes bytes on a pipe. No declared
/// output path is involved, so a target the taxonomy never declared is still
/// emittable, which is what makes the flag worth having.
///
/// **With none**, it writes what the taxonomy declared, to the paths the
/// taxonomy names, and `--check` holds them to regeneration. That is the same
/// comparison `generate --check` performs, over the subset one profile names.
///
/// The census goes to standard error in the first mode and to standard output in
/// the second, so that a redirected artifact is the artifact and nothing else.
fn export(
    root: &Path,
    profile: Option<String>,
    format: Option<String>,
    typed: &str,
    generated_at: Option<String>,
    check_only: bool,
) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let projections = match headwater_generate::Projections::read(&loaded.bound.taxonomy) {
        Ok(projections) => projections,
        Err(errors) => return refused("the projections", &errors),
    };
    let surface = loaded.surface();

    let Some(target) = format else {
        if generated_at.is_some() {
            return fail(
                "--at states the time an artifact that leaves this repository was generated, \
                 and it is refused for a declared output. A committed export is held to \
                 regeneration by byte, so a clock reading inside one would fail the gate on a \
                 morning when nothing changed. Name a target with --format",
            );
        }
        let plan = match headwater_generate::export_plan(&surface, &projections, profile.as_deref())
        {
            Ok(plan) => plan,
            Err(message) => return fail(&message),
        };
        let report = match check_only {
            true => headwater_generate::check(root, &plan),
            false => headwater_generate::write(root, &plan),
        };
        print!("{}", report.render());
        if report.has_errors() {
            eprintln!(
                "headwater: {}",
                err("a declared export is not what this corpus and this lock produce")
            );
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    };

    let Some(emitter) = headwater_generate::Emitter::parse(&target) else {
        return fail(&format!(
            "`{target}` is not an emitter target. Spec 6 names {}",
            headwater_generate::Emitter::ALL
                .iter()
                .map(|one| format!("`{}`", one.name()))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    };
    if check_only {
        return fail(&format!(
            "--check compares a committed artifact against what a run produces, and {typed} \
             writes to standard output where nothing is committed. Run `headwater export \
             --check` over the declared outputs instead"
        ));
    }

    // One artifact on a pipe is one profile. With several declared and none
    // named, the engine asks rather than concatenate two JSON documents into a
    // stream that no reader parses.
    let selected: Vec<&headwater_generate::Profile> = match &profile {
        Some(name) => match projections.profile(name) {
            Some(profile) => vec![profile],
            None => return fail(&format!("no profile is called `{name}`")),
        },
        None => projections.profiles.iter().collect(),
    };
    let profile = match selected.as_slice() {
        [one] => *one,
        [] => {
            return refuse(
                "this taxonomy declares no projection, so it declares no export profile. \
                 Spec 6 makes a profile an entry under `projections`",
            )
        }
        several => {
            return fail(&format!(
                "{typed} writes one artifact to standard output and this taxonomy declares {} \
                 profiles. Name one with --profile: {}",
                several.len(),
                several
                    .iter()
                    .map(|profile| format!("`{}`", profile.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    };

    match headwater_generate::export::emit(&surface, profile, emitter, generated_at.as_deref()) {
        Ok(emission) => {
            print!("{}", emission.bytes);
            eprint!("{}", headwater_generate::export::render(&emission.census));
            if emission.census.is_defective() {
                eprintln!(
                    "headwater: {}",
                    err(
                        "the projection census found an omission that no declared loss reason \
                         covers, which is a defect in this emitter rather than in the corpus"
                    )
                );
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("headwater: {}", err("nothing was exported"));
            eprintln!("  {}", refusal.reason());
            ExitCode::FAILURE
        }
    }
}

/// `headwater mcp`: the reads above and one run of the checks, served to an
/// agent.
///
/// Every decision the `check` tool needs is taken here, at the same boundary
/// `check` takes it at, and handed over. A protocol call has no boundary of its
/// own, so a tool that read a clock or opened a cache would be this file's
/// defaults written a second time behind a wire.
///
/// # `--write` is an argument somebody typed, and that is the whole of the
/// reason it is a flag
///
/// [Spec 5](../../../../docs/spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it)
/// puts the working-tree write class off by default and makes the opt-in per
/// server, "because a client may connect to a checkout that the user did not
/// intend to change". Two other shapes were available and both are worse.
///
/// A setting in the checkout would let a repository grant the write class to
/// every client that ever opens it, and the person who started the server would
/// not have said anything. The consent would then be a fact about a file that
/// somebody else committed. A second verb would be a second server to hold in
/// step with this one, and spec 6's grammar names verbs rather than modes.
///
/// So the switch is a word in the command line that started the process, which
/// is the same place `--now` is: the two inputs a caller is answerable for, at
/// the boundary where a caller speaks.
///
/// The write class is [`headwater_query::mcp::Writing`], and it holds the two
/// verbs as functions rather than as parts. A tool call therefore runs the verb
/// this file runs, and there is no second composition of a scaffolder or a
/// fixer behind the protocol.
fn mcp(root: &Path, now: Option<Date>, writing: bool) -> ExitCode {
    // The clock, read once for the life of the server, by the two lines
    // `check` reads it with. A server that guessed the date would answer a
    // windowed expectation wrong for as long as it ran, so a host that cannot
    // say what day it is gets no server.
    let Some(ctx) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!(
            "headwater: {}",
            err("this host has no readable clock. Pass `--now <YYYY-MM-DD>`")
        );
        return ExitCode::FAILURE;
    };
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    // The two verbs of the write class, as this file runs them. Each closure
    // takes the arguments its tool declares and nothing else: the root and the
    // clock are the server's, and no tool may name either.
    // The clock alone, because the two closures below outlive the borrow of
    // the context and a context is no longer a `Copy` value: it may carry the
    // change a run is scoped to. No tool of this server names either one.
    let now = ctx.now();
    let scaffolding = move |kind: &str, title: &str, relates: &[(String, String)]| {
        scaffold(
            root,
            kind,
            title,
            relates,
            // No facet values. The write tool declares a kind, a title and
            // relations, and nothing else, so a kind that requires a facet no
            // declaration determines is refused over the protocol and written
            // from a terminal. Widening the tool is a change to the write
            // class that [Q7](../../../../docs/spec/09-decisions.md#q7--scope-of-the-mcp-surface)
            // fixed, and not a change to this call.
            &[],
            Some(now),
            EntryPoint::Protocol,
        )
    };
    let fixing = move |format: Format| fix_over(root, &Context::at(now), format);
    // The corpus is read once, at startup, and every tool answers from it.
    // That is the same posture every other verb takes, and it is what makes two
    // reads in one session answer the same bytes. A write ends the session,
    // because it ends the tree that walk described.
    let server = headwater_query::mcp::Server {
        surface: loaded.surface(),
        census: &loaded.census,
        graph: &loaded.graph,
        declared: loaded.declared(),
        claims: &loaded.claims,
        package: &loaded.bound.package,
        version: &loaded.bound.version,
        now: ctx,
        writing: match writing {
            false => None,
            true => Some(headwater_query::mcp::Writing {
                scaffold: &scaffolding,
                fix: &fixing,
            }),
        },
    };
    headwater_query::mcp::serve(&server, std::io::stdin().lock(), std::io::stdout().lock());
    ExitCode::SUCCESS
}

/// `headwater gate`: the read set of an earlier run, held against this tree.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
/// rules what a gate reads, and [`headwater_check::gate`] is that ruling. This
/// function is the shell around it: it opens the artifact, asks the filesystem
/// for the bytes at each listed path, and prints what the decision was.
///
/// It walks no corpus and it resolves no taxonomy. That is the whole economy of
/// the artifact: the answer costs one hash per listed input, and it costs no
/// run. It is also the limit of the answer, which the report states every time.
fn gate(root: &Path, read_set: Option<PathBuf>, now: Option<Date>, json: bool) -> ExitCode {
    let Some(path) = read_set else {
        return fail(
            "`gate` holds a read set against this tree and takes the file that carries one. \
             Try `headwater check --read-set run.readset` on one tree, then `headwater gate \
             --read-set run.readset` on another",
        );
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return fail(&format!("cannot read {}: {error}", path.display())),
    };
    let recorded = match headwater_check::Recorded::parse(&text) {
        Ok(recorded) => recorded,
        Err(refusal) => {
            return fail(&format!(
                "{} does not read as a read set. {}",
                path.display(),
                refusal.render()
            ))
        }
    };
    // The same clock rule `check` follows, and for the same reason: a gate that
    // guessed the day would carry a windowed verdict across the day it expired.
    let Some(asked) = now.or_else(|| Context::from_system_clock().map(|ctx| ctx.now())) else {
        eprintln!(
            "headwater: {}",
            err("this host has no readable clock. Pass `--now <YYYY-MM-DD>`")
        );
        return ExitCode::FAILURE;
    };
    // The lock, and nothing else of the taxonomy. A lock that moved voids every
    // result at once, so its digest is the one component a gate compares that is
    // not a document.
    let lock = match headwater_lock::at(root) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("headwater: {}", err(&format!("{error}")));
            return ExitCode::FAILURE;
        }
    };
    let verdict = headwater_check::gate::decide(&recorded, &lock.digest, asked, |listed| {
        // The one input of a published read set that is not a file. The claim
        // store is a directory, so `read` refuses it and the gate would report
        // a store that is right there as a path that is gone. Its digest is
        // over the canonical listing, which is the same value the run that
        // wrote this read set recorded.
        if listed == headwater_check::claim::STORE {
            return Some(headwater_check::claim::Claims::at(root).digest());
        }
        std::fs::read(root.join(listed))
            .ok()
            .map(|bytes| headwater_hash::digest(&bytes))
    });
    // The sentence about what a read set cannot say is in both forms: the
    // report ends on it and the document carries it as `limit`, out of the one
    // constant both read.
    match json {
        true => print!("{}", verdict.render_json()),
        false => print!("{}", verdict.render()),
    }
    match verdict.carries() {
        true => ExitCode::SUCCESS,
        // Spec 12: "a false invalidation costs one run. A false survival ships
        // an invalid corpus with a green report." A non-zero exit is the signal
        // to run the checks again, and it is the cheaper of the two errors.
        false => ExitCode::FAILURE,
    }
}

/// `headwater check --fix`: write the patches, then run the checks again.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#fixability) fixes what may
/// be written and [`headwater_scaffold::fix`] carries the guards. This function
/// is the shell: it takes the patches of one run, composes them, writes what
/// composed, and hands back what refused.
///
/// **The report a caller reads is the run after the fix**, which is why this
/// happens before the run below rather than after it. A fix that produced a
/// document the engine's own checks reject is a defect in the fix, and a report
/// from before the write would hide it for exactly one run.
///
/// It writes to standard error. `--format` puts one artifact on standard
/// output, and a line about a file this run wrote is not part of that artifact.
fn fix(root: &Path, ctx: &Context, cached: bool) -> Result<Fixed, ExitCode> {
    let loaded = load(root)?;
    let mut cache = match cached {
        true => Cache::at(root, &loaded.bound.digest),
        false => Cache::disabled(),
    };
    let run = headwater_check::run(
        &loaded.census,
        &loaded.graph,
        &loaded.declared(),
        &loaded.claims,
        ctx,
        &mut cache,
    );
    cache.write(root);

    // A suppressed finding is not in this list, which is the author asking for
    // the text to stand. The runner filters, and a fix reads what a reader
    // reads.
    let patches: Vec<headwater_check::Patch> = run
        .findings
        .iter()
        .filter_map(|finding| finding.patch.clone())
        .collect();
    let composed = headwater_scaffold::fix::compose(root, &patches);
    if let Err(refusal) = headwater_scaffold::fix::apply(root, &composed.files) {
        eprintln!("headwater: {}", err(&format!("{refusal}")));
        return Err(ExitCode::FAILURE);
    }
    // After the edits, and never before them. A create that cannot happen
    // leaves every edited file already written, which is the ordering the
    // scaffolder argues for: the reservation refuses with an unchanged tree,
    // and the create is the step that can meet a path somebody else took.
    if let Err(refusal) = headwater_scaffold::fix::make(root, &composed.created) {
        eprintln!("headwater: {}", err(&format!("{refusal}")));
        return Err(ExitCode::FAILURE);
    }
    let mut account = String::new();
    for file in &composed.files {
        use std::fmt::Write;
        let _ = writeln!(
            account,
            "headwater: fixed {} ({} patch{})",
            file.path,
            file.applied,
            match file.applied {
                1 => "",
                _ => "es",
            }
        );
    }
    // Counted rather than listed one line at a time. The bootstrap of a corpus
    // that has minted for years writes hundreds of these in one run, and a
    // caller reading standard error needs the number and the directory rather
    // than every path.
    if !composed.created.is_empty() {
        use std::fmt::Write;
        let _ = writeln!(
            account,
            "headwater: made {} file{} under `{}`",
            composed.created.len(),
            match composed.created.len() {
                1 => "",
                _ => "s",
            },
            headwater_check::claim::STORE
        );
    }
    if composed.is_empty() {
        account.push_str("headwater: no finding of this run carries a patch\n");
    }
    Ok(Fixed {
        account,
        // The seal of a writing MCP server reads this, and so does nothing
        // else. A run that composed no file left the tree as it found it.
        landed: !composed.files.is_empty() || !composed.created.is_empty(),
        refused: composed.refused,
    })
}

/// What one run of the fixer did, held rather than printed.
///
/// The account is what a terminal reads on standard error, and the write tool
/// of the MCP server puts the same bytes in its first content block. One
/// composition, because a client and a terminal reading different accounts of
/// one write is the drift this repository spends its comments on.
struct Fixed {
    account: String,
    landed: bool,
    refused: Vec<headwater_scaffold::fix::Refused>,
}

/// The account of every file that refused the patch it was offered.
///
/// A refusal is not a finding, so no `--strict` softens it. It says this verb
/// was asked to write and did not, and a run that swallowed that would leave a
/// caller believing a corpus was fixed.
fn refusal_account(refused: &[headwater_scaffold::fix::Refused]) -> String {
    use std::fmt::Write;
    if refused.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let _ = writeln!(
        out,
        "headwater: {} file{} refused the patch it was offered, and nothing was written to any \
         of them:",
        refused.len(),
        match refused.len() {
            1 => "",
            _ => "s",
        }
    );
    for refusal in refused {
        let _ = writeln!(out, "  {refusal}");
    }
    out
}

/// One run of `headwater check --fix --format <format>`, held rather than
/// printed.
///
/// This is the `fix` tool of the MCP server, and it is the terminal's verb with
/// the two streams captured. The artifact is what
/// [`headwater_adapter::render`] wrote, so it is byte for byte what a terminal
/// reads on standard output. No cache, on the rule the whole server follows.
fn fix_over(root: &Path, ctx: &Context, format: Format) -> Result<Written, String> {
    let fixed = fix(root, ctx, false).map_err(|_| {
        "the fixer refused a file, and the account is on the standard error of the process \
         serving this"
            .to_string()
    })?;
    let loaded = load(root).map_err(|_| "the corpus did not load".to_string())?;
    let mut cache = Cache::disabled();
    let run = headwater_check::run(
        &loaded.census,
        &loaded.graph,
        &loaded.declared(),
        &loaded.claims,
        ctx,
        &mut cache,
    );
    let subject = Subject {
        package: &loaded.bound.package,
        version: &loaded.bound.version,
        lock: &loaded.bound.digest,
        now: &ctx.now().render(),
    };
    // Plain, unconditionally: this is the MCP `fix` tool's byte-for-byte
    // record of a terminal run, and an MCP process is never a terminal, so a
    // real `headwater check` run piped the same way would sense the same
    // mode.
    let artifact = headwater_adapter::render(&run, &loaded.census, &loaded.graph, &subject, format);
    // The same audit the verb fails a run on. A caller here holds one artifact
    // rather than a terminal, so a finding that reached no output is invisible
    // to it.
    let audited = headwater_adapter::census(&run, format, &artifact);
    if audited.is_defective() {
        return Err(audited.complaint(format));
    }
    Ok(Written {
        account: format!("{}{}", fixed.account, refusal_account(&fixed.refused)),
        artifact,
        landed: fixed.landed,
        ok: fixed.refused.is_empty(),
    })
}

/// What one invocation of `check` was asked for.
///
/// One value rather than eight parameters. The flags decide what a run reads,
/// what it writes and what it exits with, and a caller that passed two of them
/// in the wrong order would compile.
struct Asked {
    strict: bool,
    cached: bool,
    fixing: bool,
    now: Option<Date>,
    read_set: Option<PathBuf>,
    register_out: Option<PathBuf>,
    format: Option<String>,
    /// The manifest of the change this run is scoped to, where a caller named
    /// one. See `headwater_check::change`.
    change: Option<PathBuf>,
}

fn check(root: &Path, asked: Asked) -> ExitCode {
    let Asked {
        strict,
        cached,
        fixing,
        now,
        read_set,
        register_out,
        format,
        change,
    } = asked;
    // The vocabulary is decided before anything is read, so a run that would
    // refuse the flag refuses it before it walks a corpus. `export --format`
    // names an emitter target and this names an output format: two lists, two
    // enums, and the word `json` in both means a different artifact.
    let format = match format.as_deref().map(Format::parse) {
        None => Format::Text,
        Some(Some(format)) => format,
        Some(None) => {
            let names: Vec<&str> = Format::ALL.iter().map(|format| format.name()).collect();
            return fail(&format!(
                "`check --format` takes one of {}. An emitter target of `export` is not one of \
                 them: that flag names a vocabulary for the graph and this one names a \
                 vocabulary for the findings",
                names.join(", ")
            ));
        }
    };
    // The one clock read of the whole engine, and it is here rather than in a
    // check. Spec 12: "`ctx.now` is a bound value, never a syscall." A run
    // whose host cannot say what day it is refuses rather than guesses, because
    // a windowed expectation evaluated against a guess is a wrong verdict.
    let Some(ctx) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!(
            "headwater: {}",
            err("this host has no readable clock. Pass `--now <YYYY-MM-DD>`")
        );
        return ExitCode::FAILURE;
    };
    // The second injected value, read here and bound after the walk below. A
    // manifest this engine cannot read is a refusal rather than a shorter
    // change: a line that was dropped reads as a document that did not move,
    // which is a transition nothing reports.
    let unbound = match &change {
        None => None,
        Some(path) => match headwater_check::change::Unbound::at(path) {
            Ok(unbound) => Some(unbound),
            Err(why) => {
                eprintln!("headwater: {}", err("the change manifest did not read"));
                eprintln!("{}", indent(&err(&why)));
                return ExitCode::FAILURE;
            }
        },
    };
    // The write, and then the read. See `fix` above: the report below is the
    // state after the patches landed, so a fix that produced a document these
    // checks reject reports it on the same run rather than on the next one.
    let refused = match fixing {
        false => Vec::new(),
        true => match fix(root, &ctx, cached) {
            Ok(fixed) => {
                eprint!("{}", fixed.account);
                fixed.refused
            }
            Err(code) => return code,
        },
    };

    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded {
        bound,
        consumer: _,
        census: taken,
        graph,
        ..
    } = &loaded;

    // The manifest, held against the corpus this run walked. A path that
    // reaches no row of it binds to nothing, and the report names it: a caller
    // who mistyped one character would otherwise read a run that counted
    // nothing and said it succeeded.
    let ctx = match unbound {
        None => ctx,
        Some(unbound) => {
            ctx.scoped_to(unbound.bind(|path| taken.rows.iter().any(|row| row.path == path)))
        }
    };

    // Phase B. The cache is keyed on the lock digest among other things, so a
    // taxonomy that moved invalidates every entry without anyone clearing a
    // directory.
    let mut cache = match cached {
        true => Cache::at(root, &bound.digest),
        false => Cache::disabled(),
    };
    let run = headwater_check::run(
        taken,
        graph,
        &loaded.declared(),
        &loaded.claims,
        &ctx,
        &mut cache,
    );
    cache.write(root);

    // The run, in the vocabulary the caller asked for. Spec 6 lists four
    // formats and `headwater_adapter::render` writes every one of them, so this
    // verb composes no report of its own and the MCP `check` tool composes none
    // either. A run reports the state it evaluated, and the report is not
    // optional (spec 4): `Subject` is that statement, and the corpus tree is
    // the half of it that nothing computes yet.
    //
    // What the flag does not touch: the exit status, the cache accounting on
    // standard error, and the two files below. A flag that moved a verdict
    // would be the second input to it that no reviewer sees.
    let subject = Subject {
        package: &bound.package,
        version: &bound.version,
        lock: &bound.digest,
        now: &ctx.now().render(),
    };
    // The width the report is laid out at. `paint::width` is 80 unless the
    // command line carries `--wide`, and it is the one reader of `COLUMNS` in
    // this binary, so a run with no flag writes the same bytes into a pipe, a
    // file and a terminal. The three machine formats ignore the number.
    let width = headwater_cli::paint::width();
    // Color is a property of the text format alone: `--no-color` and a
    // terminal's own state decide it, and a machine format renders the same
    // bytes regardless of either, so this reads the stream only where it
    // would otherwise matter.
    let mode = match format {
        Format::Text => headwater_cli::paint::stdout_color(),
        _ => headwater_cli::paint::ColorMode::Plain,
    };
    let artifact = headwater_adapter::render_at(&run, taken, graph, &subject, format, width, mode);
    print!("{artifact}");
    // The census over what was written, in the shape spec 6 fixes for the
    // graph emitters. A finding that reached no output and that no loss
    // reason covers is a defect in the adapter, and it fails the run the
    // way a defective projection census does.
    let audited = headwater_adapter::census(&run, format, &artifact);
    if audited.is_defective() {
        eprint!("headwater: {}", err(&audited.complaint(format)));
        return ExitCode::FAILURE;
    }

    if let Some(path) = read_set {
        if let Err(error) = std::fs::write(&path, run.read_set.render()) {
            eprintln!(
                "headwater: {}",
                err(&format!("cannot write {}: {error}", path.display()))
            );
            return ExitCode::FAILURE;
        }
    }

    // The register. It is already in the report above, because spec 4 makes it
    // mandatory and inspectable rather than a flag. What the flag adds is a
    // file, and the content is the same content for the reason the read set's
    // is: a projection a consumer regenerates and one a reader reads are one
    // artifact or they are two truths.
    //
    // Not the same bytes, since #340. The report lays its register block out at
    // the width of the run and this file is written as the register composed it,
    // because a file is read by whoever opens it and a report is read at a
    // width. Nothing parses this file — `Register::read` reads the resolved
    // taxonomy and never a rendered one — so the layout costs no consumer
    // anything, which is the difference from the read set above.
    if let Some(path) = register_out {
        if let Err(error) = std::fs::write(&path, run.register.render()) {
            eprintln!(
                "headwater: {}",
                err(&format!("cannot write {}: {error}", path.display()))
            );
            return ExitCode::FAILURE;
        }
    }

    // The cache accounting goes to standard error, because it is a fact about
    // this machine's disk and the report above is a fact about the corpus.
    // `--no-cache` and a cached run write the same bytes to standard output,
    // and a line here would be the one thing that made them differ.
    eprint!("{}", run.cache.render());

    if !refused.is_empty() {
        eprint!("{}", refusal_account(&refused));
        return ExitCode::FAILURE;
    }

    if strict && run.has_errors() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// How long a task holds when nobody says.
///
/// A quarter. Spec 7 wants the expiry to force a conversation, and a date the
/// adopter did not choose should arrive while the people who ran `infer` are
/// still the people who own the corpus. It is a default rather than a rule: a
/// payload states its own dates and `--until` writes them.
const DEFAULT_WINDOW: i64 = 90;

/// `headwater infer`: the payload, computed from the diff between nothing and
/// one.
///
/// [Q12](../../../../docs/spec/09-decisions.md#q12--migration-path-for-an-existing-corpus):
/// before the first taxonomy a corpus is governed by nothing, so every document
/// in it is trivially valid, and the findings the first taxonomy raises are that
/// taxonomy's migration payload.
///
/// **The run that computes a payload reads the payload, and the write adds to
/// it.** It computed the diff between nothing and one until
/// [#249](https://github.com/headwater-ai/headwater/issues/249), on the stated
/// reasoning that a run which read the committed block would propose an empty
/// payload on the second run, "and the second run would report the whole of the
/// debt as new findings". That last step only follows where the write is a
/// *replacement*: an empty payload written over the block is the block deleted,
/// and the check after it reports everything the block was holding. The
/// replacement was the defect. Once the write adds, an empty proposal writes
/// nothing and the declared debt stands, which is what makes a second run and a
/// third run cost nothing.
///
/// So this run holds three things it did not hold before, and each one is one of
/// the three failures #249 separates:
///
/// - The checks run **against the declared block**, so a finding an open task
///   already accounts for is not proposed a second time.
/// - An identifier is minted **past every identifier the block declares**, by
///   the same code in the run that prints a proposal and the run that writes
///   one, so the two cannot disagree.
/// - The write **merges** into the declared `tasks` and says on standard output
///   what it did to the block that was there. `taxonomy resolve` is the
///   precedent and prints `carried the adoption block through, 1 task`.
///
/// A block this engine cannot merge into stops the write rather than replacing
/// it, for the reason `resolve` refuses an unreadable lock: a rewrite there
/// discards an owner, an expiry and every pair, and cannot say what it
/// discarded.
///
/// It writes nothing without `--write`. A payload is a commitment with a name
/// and a date on it, so it is printed for a person to read before it is a file
/// they have to review in a diff.
fn infer(
    root: &Path,
    owner: Option<String>,
    until: Option<Date>,
    write: bool,
    now: Option<Date>,
) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(ctx) = now.map(Context::at).or_else(Context::from_system_clock) else {
        return refuse("the host clock is before 1970, and this engine will not guess a date");
    };
    let until = until.unwrap_or_else(|| ctx.now().plus_days(DEFAULT_WINDOW));
    if until < ctx.now() {
        return fail(&format!(
            "--until {until} is in the past, and a task that has already lapsed accounts for nothing"
        ));
    }

    // The one authored part of the lock, read before anything is computed. It
    // decides three things below: which findings are already accounted for,
    // which identifiers are taken, and what the write adds to.
    let declared = loaded.bound.adoption.clone();

    let run = headwater_check::run(
        &loaded.census,
        &loaded.graph,
        &Declared {
            lock: &loaded.bound.digest,
            taxonomy: &loaded.taxonomy,
            shape: &loaded.shape,
            relations: &loaded.relations,
            config: &loaded.config,
            register: &loaded.register,
            adoption: declared.as_ref(),
            source: headwater_lock::LOCK,
        },
        &loaded.claims,
        &ctx,
        &mut Cache::disabled(),
    );

    // One task per rule. A rule is the unit an adopter works down, because the
    // fix for every pair under it is the same fix, and it is the unit the
    // coverage report already counts by.
    //
    // `run.findings` is what a reader of the report sees, so a finding an
    // author already suppressed does not enter the payload. The precedence spec
    // 4 fixes runs in one direction only when it is applied once, and a pair in
    // both inventories would be the double count that precedence exists to
    // prevent.
    let mut tasks: Vec<(&'static str, Vec<&headwater_check::Finding>)> = Vec::new();
    for finding in &run.findings {
        match tasks.iter_mut().find(|(rule, _)| *rule == finding.rule) {
            Some((_, held)) => held.push(finding),
            None => tasks.push((finding.rule, vec![finding])),
        }
    }

    let owner =
        match (&owner, write) {
            (Some(owner), _) => owner.clone(),
            // Refused rather than defaulted. The owner is the field spec 4 ranks a
            // migration state above a suppression for, and an owner this engine
            // invented would rank it above nothing.
            (None, true) => return fail(
                "--write needs --owner. An owner is the field spec 4 ranks declared debt above a \
                 suppression for, and this engine will not invent one",
            ),
            (None, false) => "TODO name a person or a team".to_string(),
        };

    // Identifiers the block already declares, read off the block as written
    // rather than through `adoption::read`. A task that fails to parse still
    // holds its identifier, and a run that minted over it would write a lock
    // naming one task twice.
    let mut taken = claimed(declared.as_ref());
    // Held before anything is minted into `taken`, because the report names
    // what was there and `mint` appends to the same list.
    let named = taken.clone();
    // Counted off the sequence rather than off the identifiers, because a task
    // that names no `id` is still a task in the block and still something a
    // write adds beside. The two numbers differ exactly where a task is
    // malformed, and reporting the identifier count as the task count would
    // undercount what is at risk.
    let standing = standing(declared.as_ref());

    let mut payload = String::new();
    payload.push_str("tasks:\n");
    for (rule, held) in &tasks {
        payload.push_str(&format!("  - id: {}\n", mint(&mut taken)));
        payload.push_str(&format!(
            "    statement: {}\n",
            quoted(&format!(
                "{} {} of {rule}, {}",
                held.len(),
                match held.len() {
                    1 => "finding",
                    _ => "findings",
                },
                match standing {
                    // The wording of first contact, which is what Q12 makes
                    // this verb about.
                    0 => "raised when this taxonomy first reached this corpus",
                    // And what is true instead once a block is there. The
                    // corpus has met this taxonomy already, and these are the
                    // findings no task the lock declares accounts for.
                    _ => "held by no task this lock declared",
                }
            ))
        ));
        // Quoted, because an owner is whatever a person typed and a team name
        // holding a colon would otherwise write a payload that does not load.
        payload.push_str(&format!("    owner: {}\n", quoted(&owner)));
        payload.push_str(&format!("    until: {until}\n"));
        payload.push_str("    pairs:\n");
        // The cell, once. Two findings of one rule on one document are one
        // pair, which is the grain spec 7 fixes.
        let mut cells: Vec<&str> = held.iter().map(|finding| finding.path.as_str()).collect();
        cells.sort_unstable();
        cells.dedup();
        // Both values quoted, for the reason `quoted` records: this was an
        // unquoted flow mapping, and a `}` in a filename closed it early while
        // a comma in one truncated the value and exited 0.
        for path in cells {
            payload.push_str(&format!(
                "      - {{path: {}, rule: {}}}\n",
                quoted(path),
                quoted(rule)
            ));
        }
    }

    let pairs: usize = payload.matches("      - {path: ").count();
    // Said in every run, before anything else this verb prints, and said the
    // same whether the run writes or not. A reader who is about to hand this
    // verb a `--write` is a reader who needs to know there is something there
    // to write beside.
    match standing {
        0 => println!("the lock declares no adoption block, so this run writes the first one"),
        _ => {
            println!(
                "the lock declares {standing} adoption {}, and a run with --write adds beside {}",
                match standing {
                    1 => "task",
                    _ => "tasks",
                },
                match standing {
                    1 => "it",
                    _ => "them",
                }
            );
            if !named.is_empty() {
                println!("  {}", named.join(", "));
            }
            if named.len() < standing {
                println!(
                    "  and {} that name no `id`, which `headwater check` refuses and this run \
                     carries through as it found them",
                    standing - named.len()
                );
            }
        }
    }
    match tasks.is_empty() {
        true => match standing {
            0 => println!("no finding, so no debt to declare"),
            _ => println!(
                "no finding this run raised is outside those tasks, so there is no new debt to \
                 declare"
            ),
        },
        false => println!(
            "{pairs} pairs of debt, in {} {}, expiring {until}",
            tasks.len(),
            match tasks.len() {
                1 => "task",
                _ => "tasks",
            }
        ),
    }

    // What the tree holds that the proposal does not explain. Spec 7 makes this
    // one of the three artifacts of one read, and it is the half a payload
    // cannot carry: an unclassified file raises no finding to declare.
    let unexplained: Vec<&headwater_census::census::Row> = loaded
        .census
        .rows
        .iter()
        .filter(|row| matches!(row.outcome.class(), "untyped" | "unreadable"))
        .collect();
    println!("\nwhat this taxonomy does not explain");
    match unexplained.is_empty() {
        true => println!("  every file the census walked classified"),
        false => {
            println!(
                "  {} files classified as nothing, and no payload can hold them",
                unexplained.len()
            );
            for row in unexplained.iter().take(10) {
                println!("    {} {}", row.path, row.outcome.class());
            }
            if unexplained.len() > 10 {
                println!("    and {} more", unexplained.len() - 10);
            }
        }
    }

    // The debt that is not a rule violation. Spec 5 ranks a route on the scent
    // facet, so a document with none is reachable by name and by nothing else.
    // A payload holds the finding where a facet contract requires a summary. It
    // cannot hold this, because where no contract requires one there is no
    // finding to declare, and the corpus is still unsearchable.
    let surface = loaded.surface();
    let documents = surface.documents();
    let mute: Vec<&str> = documents
        .iter()
        .filter(|document| surface.summary(document).is_none())
        .map(|document| document.path)
        .collect();
    println!("\nwhat nothing will route to");
    if documents.is_empty() {
        println!("  no document classified, so a route has nothing to reach whatever it matches");
    } else {
        match mute.is_empty() {
            true => println!("  every classified document states a summary"),
            false => println!(
                "  {} of {} classified documents state no summary, so a task matches them on \
                 their path and their anchors alone",
                mute.len(),
                documents.len()
            ),
        }
    }
    // Routing matches a task against declared purposes before it matches any
    // text, so a taxonomy with none is silent whatever the corpus says.
    let purposes = &surface.shape().purposes;
    match purposes.is_empty() {
        true => println!(
            "  the taxonomy declares no purposes, so a task matches nothing. `headwater init` \
             writes the question that fills them in"
        ),
        false => println!(
            "  {} purposes are declared, and a task is matched against their `answers` phrases",
            purposes.len()
        ),
    }

    // The `answers` phrases decide what routes where, and no check reads them.
    // A purpose is matched on the terms that separate it from the others, so
    // two purposes whose phrases share every term separate nothing and every
    // task matches both equally. This measures it and reports it. It is not a
    // finding: a rule would need an obligation and a control, and what is
    // missing first is the measurement.
    let phrases: Vec<(&str, Vec<String>)> = purposes
        .iter()
        .map(|purpose| {
            (
                purpose.name.as_str(),
                headwater_query::terms(&purpose.answers.join(" ")),
            )
        })
        .collect();
    let mut mute_purposes: Vec<&str> = Vec::new();
    let mut collisions: Vec<(&str, &str)> = Vec::new();
    for (index, (name, terms)) in phrases.iter().enumerate() {
        if terms.is_empty() {
            mute_purposes.push(name);
            continue;
        }
        for (other, others) in phrases.iter().skip(index + 1) {
            if others.is_empty() {
                continue;
            }
            // Separating terms in either direction. None in both is the case
            // where a task cannot tell the two apart.
            let apart = terms.iter().any(|term| !others.contains(term))
                || others.iter().any(|term| !terms.contains(term));
            if !apart {
                collisions.push((name, other));
            }
        }
    }
    for name in &mute_purposes {
        println!(
            "  the purpose {name} states no `answers`, so only its one-sentence intent is matched"
        );
    }
    for (left, right) in &collisions {
        println!(
            "  the purposes {left} and {right} answer the same terms, so no task separates them"
        );
    }
    if !purposes.is_empty() && mute_purposes.is_empty() && collisions.is_empty() {
        println!("  every purpose answers a term no other purpose answers");
    }

    // Reported after the two sections above and never instead of them. A corpus
    // whose files classify as nothing raises no finding, and an `infer` that
    // answered "no debt" and stopped would report a taxonomy that fits as the
    // same result as a taxonomy that touches nothing. Those are the two
    // outcomes of first contact and they are opposite ones.
    if tasks.is_empty() {
        match standing {
            0 => {
                println!(
                    "\nthis corpus raises no finding against this taxonomy, so it declares no debt"
                );
                if !unexplained.is_empty() {
                    println!(
                        "  read that with the {} unclassified files above. A taxonomy that \
                         classifies nothing raises nothing",
                        unexplained.len()
                    );
                }
            }
            // The idempotent run. There is nothing to add, so nothing is
            // written and the declared block is reported untouched rather than
            // left unmentioned. A run that said nothing here is a run a reader
            // cannot tell from one that wrote.
            _ => println!(
                "\nevery finding this run raised is held by a task the lock declares, so there is \
                 nothing to add. {} is left as it was, with its {standing} {}",
                headwater_lock::LOCK,
                match standing {
                    1 => "task",
                    _ => "tasks",
                }
            ),
        }
        return ExitCode::SUCCESS;
    }

    if !write {
        println!("\nthe payload, which --write puts in the lock\n");
        print!("{}", indent(&payload));
        println!(
            "\nRun again with --write --owner <name> to commit it. Until it is in the lock, \
             `headwater check` reports every pair above as a finding"
        );
        return ExitCode::SUCCESS;
    }

    // Both arms re-read a string this same function assembled above. If either
    // fires, this engine emitted YAML it cannot read back, so neither the
    // caller nor anything on disk is at fault and `defect` is the helper
    // (#455).
    let fresh = match headwater_yaml::load(&payload) {
        Ok(node) => match node.value.as_map() {
            Some(map) => map.clone(),
            None => {
                return defect("the payload this run built is not a mapping, which is a defect")
            }
        },
        Err(errors) => {
            return defect(&format!(
                "the payload this run built does not load: {}",
                headwater_yaml::error::render(&errors)
            ))
        }
    };

    // The merge, and the one state it refuses. Everything above this line is a
    // proposal; this is where a file that somebody authored is at risk.
    let block = match merged(declared.as_ref(), &fresh) {
        Ok(block) => block,
        Err(why) => {
            eprintln!(
                "headwater: {}",
                err(&format!(
                    "{} declares an adoption block this run cannot add to",
                    headwater_lock::LOCK
                ))
            );
            eprintln!("  {}", err(&why));
            eprintln!(
                "  {}",
                err(
                    "A payload written over it would discard an owner, an expiry and every \
                     pair, and this run cannot say what it discarded. Repair the block, or \
                     remove it to write a first payload"
                )
            );
            return ExitCode::FAILURE;
        }
    };

    // Written through the resolver, so the lock a payload lands in is the lock
    // the sources produce. A payload written into a stale lock would be debt
    // declared against a taxonomy nobody committed.
    let repository = match headwater_resolve::repository(root) {
        Ok(repository) => repository,
        Err(errors) => {
            eprintln!(
                "headwater: {}",
                err("the taxonomy did not resolve, so no payload can be written")
            );
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    let sources = match headwater_resolve::package::sources(root, &repository.consumer) {
        Ok(sources) => sources,
        Err(errors) => {
            eprint!("{}", indent(&err(&render_errors(&errors))));
            return ExitCode::FAILURE;
        }
    };
    let text = match headwater_lock::write(
        &repository.consumer.package,
        &repository.consumer.version,
        &sources,
        &repository.resolution,
        Some(&block),
    ) {
        Ok(text) => text,
        Err(findings) => {
            eprint!("{}", indent(&err(&render_errors(&findings))));
            return ExitCode::FAILURE;
        }
    };
    let path = root.join(headwater_lock::LOCK);
    if let Err(error) = std::fs::write(&path, &text) {
        return refuse(&format!("cannot write {}: {error}", path.display()));
    }
    println!("\nwrote the payload into {}", headwater_lock::LOCK);
    println!("  {pairs} pairs, owner {owner}, until {until}");
    // What became of the block that was there, in the run that did it. The
    // count is of tasks rather than of pairs, because a task is the unit a
    // person owns and dates.
    match declared.as_ref() {
        Some(block) => println!(
            "  {}, and added {} beside {}",
            carried(block),
            match tasks.len() {
                1 => "1 task".to_string(),
                other => format!("{other} tasks"),
            },
            match standing {
                1 => "it",
                _ => "them",
            }
        ),
        None => println!("  there was no adoption block, and this payload is the whole of it"),
    }
    ExitCode::SUCCESS
}

/// The task identifiers an authored block declares, as written.
///
/// Read off the block rather than through [`headwater_check::adoption::read`],
/// which refuses a task it cannot parse. A refused task still occupies its
/// identifier, and a run that minted over it would write a lock naming one task
/// twice — which is the defect this function exists to stop, one layer down.
fn claimed(block: Option<&headwater_yaml::Mapping>) -> Vec<String> {
    block
        .and_then(|block| block.get("tasks"))
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_map())
                .filter_map(|task| task.get("id"))
                .filter_map(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// How many tasks an authored block declares, whatever state each one is in.
///
/// The count a write reports as carried, and the count [`carried`] renders. A
/// task that fails to parse is one of these and is not one of [`claimed`].
fn standing(block: Option<&headwater_yaml::Mapping>) -> usize {
    block
        .and_then(|block| block.get("tasks"))
        .and_then(|node| node.value.as_seq())
        .map(|items| items.len())
        .unwrap_or_default()
}

/// The next `AD-` identifier no name in `taken` holds, added to `taken`.
///
/// One function, called by the run that prints a proposal and by the run that
/// writes one, which is what makes #249's fourth clause true by construction
/// rather than by two implementations agreeing.
fn mint(taken: &mut Vec<String>) -> String {
    let mut counter = 1;
    loop {
        let id = format!("AD-{counter}");
        if !taken.iter().any(|held| held == &id) {
            taken.push(id.clone());
            return id;
        }
        counter += 1;
    }
}

/// The declared block with this run's tasks added to it.
///
/// The declared items are carried as they were loaded rather than re-rendered
/// from a reading of them, so a key this engine does not know about survives a
/// merge. Where there is no block, the fresh payload is the whole of it.
///
/// The error is the state where a merge is not possible: a block that declares
/// no `tasks` sequence. It is returned rather than resolved by replacing the
/// block, because replacing it is the defect.
fn merged(
    declared: Option<&headwater_yaml::Mapping>,
    fresh: &headwater_yaml::Mapping,
) -> Result<headwater_yaml::Mapping, String> {
    let Some(declared) = declared else {
        return Ok(fresh.clone());
    };
    let Some(entry) = declared.entry("tasks") else {
        return Err("it declares no `tasks` key".to_string());
    };
    let Some(standing) = entry.value.value.as_seq() else {
        return Err(format!(
            "its `tasks` is {} rather than a sequence",
            entry.value.value.kind_name()
        ));
    };
    let added = fresh
        .get("tasks")
        .and_then(|node| node.value.as_seq())
        .ok_or_else(|| "the payload this run built declares no `tasks` sequence".to_string())?;

    let mut items = standing.to_vec();
    items.extend(added.iter().cloned());
    let entries = declared
        .entries()
        .iter()
        .map(|entry| match entry.key.value == "tasks" {
            true => headwater_yaml::Entry {
                key: entry.key.clone(),
                value: headwater_yaml::Spanned::new(
                    headwater_yaml::Value::Seq(items.clone()),
                    entry.value.span,
                ),
            },
            false => entry.clone(),
        })
        .collect();
    Ok(headwater_yaml::Mapping::new(entries))
}

/// The declared block with `from` and `to` set to this run's measured values.
///
/// Unlike [`merged`], this never fails: `from` and `to` are set or replaced
/// unconditionally, this run states what is true now, and it does not need a
/// declared shape to add to. Every other entry — `tasks` above all — is
/// carried exactly as it was loaded, so a key this engine does not know about
/// survives. `from` and `to` are placed first, which is the order [spec
/// 7](../../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)
/// states them in.
///
/// # The one entry this adds rather than carries
///
/// A block that declares no `tasks` key at all gets an empty one. Carrying
/// through what is not there wrote a block naming `from` and `to` and nothing
/// else, and [`merged`] refuses exactly that block, so `headwater infer
/// --write` refused this verb's own output on every migration of a corpus with
/// no prior adoption block. That refusal protects an authored task list from
/// being replaced, and an absent key holds no owner, no expiry and no pair, so
/// adding an empty sequence discards nothing and leaves the guard standing. A
/// declared list, malformed or not, is carried and never rewritten.
fn migrated(
    declared: Option<&headwater_yaml::Mapping>,
    fresh: &headwater_yaml::Mapping,
) -> headwater_yaml::Mapping {
    let mut entries: Vec<headwater_yaml::Entry> = Vec::new();
    for key in ["from", "to"] {
        if let Some(entry) = fresh.entry(key) {
            entries.push(entry.clone());
        }
    }
    if let Some(declared) = declared {
        for entry in declared.entries() {
            if entry.key.value != "from" && entry.key.value != "to" {
                entries.push(entry.clone());
            }
        }
    }
    // The empty `tasks` sequence, taken from the fresh state only where the
    // declared block carried none. A block with a standing list keeps it
    // exactly as it was loaded, which is the line above; a block with no such
    // key — the one this verb wrote before, and the one an adopter has on a
    // first migration — gets an empty one. That is additive: no entry is
    // dropped, and the block [`merged`] refuses becomes one it can add to.
    if !entries.iter().any(|entry| entry.key.value == "tasks") {
        if let Some(entry) = fresh.entry("tasks") {
            entries.push(entry.clone());
        }
    }
    headwater_yaml::Mapping::new(entries)
}

/// `headwater init`: the consumer declaration and the overlay, scaffolded.
///
/// Spec 7 makes `init` and [`infer`] one command with two evidence sources: the
/// tree, and an interview. This is the half that asks. It emits an overlay and
/// never a resolved taxonomy, which is what keeps a bundle selection add-only.
///
/// **The interview is conducted through the file rather than through a
/// terminal.** Every answer it collects ends up committed and reviewed, and a
/// prompt that produced the same file would make the verb non-deterministic and
/// untestable for the sake of asking the same questions in a worse place. So
/// the questions are written where the answers go, each one marked, and the
/// verb prints the list of what is unanswered.
fn init(root: &Path, corpus_root: Option<String>, package: Option<String>) -> ExitCode {
    let declaration = root.join(headwater_resolve::package::CONSUMER);
    if declaration.exists() {
        return refuse(&format!(
            "{} is already there, so this repository is already bound. \
             `headwater infer` is the verb that reads an existing binding",
            headwater_resolve::package::CONSUMER
        ));
    }

    // The corpus root, proposed from the tree. The directory holding the most
    // Markdown, because that is the evidence a tree offers about where its
    // documentation is, and the adopter overrides it with one word.
    let proposed = corpus_root.or_else(|| busiest_directory(root));
    let Some(corpus_root) = proposed else {
        return refuse(
            "no directory under this repository holds a Markdown file, so nothing here \
             proposes a corpus root. Pass --corpus <dir> to name one",
        );
    };

    // The package, found rather than assumed. Nothing in this engine fetches
    // one, so a name that resolves to no package on disk is reported here
    // instead of by `taxonomy resolve` two commands later.
    let package = package.unwrap_or_else(|| "headwater/standard".to_string());
    let found = headwater_resolve::package::find_version(root, &package);

    let mut declaration_text = String::new();
    declaration_text.push_str(&format!(
        "\
# The consumer declaration, written by `headwater init`. It says two things,
# and they are different questions: what schema this repository takes, and what
# tree it walks.
#
# `headwater taxonomy resolve` reads this and writes {}. Everything after that
# reads the lock and never these sources.

taxonomy:
  package: {package}
",
        headwater_lock::LOCK
    ));
    match &found {
        Some(version) => declaration_text.push_str(&format!("  version: {version}\n")),
        // **The same ruling as the printed line below, and for the same reason.**
        // This arm writes a second message under the identical condition, and it
        // said "Vendor the package, then pin the version it declares" until
        // [#276](https://github.com/headwater-ai/headwater/issues/276). That was
        // the single-route wording, and it was worse here than in the report: an
        // adopter who copied a package directory cannot vendor, and the version
        // they pin comes from the package they copied rather than from an
        // artifact. So it named a dead end in the file they then edit and commit.
        // It names both routes now, and says where the number comes from either
        // way.
        None => declaration_text.push_str(
            "  # INTERVIEW: no package of this name is under `packages/`, and nothing in this\n\
             \x20 # engine fetches one. Copy a package directory into `packages/`, or run\n\
             \x20 # `headwater taxonomy vendor <dir>` on a published artifact. Either way, pin\n\
             \x20 # the version that the package itself declares.\n\
             \x20 version: 0.0.0\n",
        ),
    }
    declaration_text.push_str(&format!(
        "\
  # A bundle is an optional part of the package, and a selection is add-only.
  # INTERVIEW: which traditions does this corpus already follow?
  bundles: []
  overlay: .headwater/overlay.yml

corpus:
  # Proposed from this tree: the directory holding the most Markdown.
  root: {corpus_root}
  # An exclusion states a reason. A pattern with none is a silent pass with a
  # configuration file in front of it, so the reason is not optional.
  # exclude:
  #   - path: {corpus_root}/vendor/**
  #     reason: vendored copies of documents another team owns
"
    ));

    let overlay_text = format!(
        "\
# The adopter overlay, written by `headwater init`. It is an overlay and never a
# resolved taxonomy, so nothing here can weaken the package it sits on: a
# bundle selection is add-only, and an add-only overlay carries no operation
# that removes a base rule.
#
# Every block below is a question this engine cannot answer from a tree. It is
# prose about what this corpus is for, and a corpus does not state it.
#
# INTERVIEW 1 --- what does each purpose answer?
#
# A task is matched against declared purposes before it is matched against any
# text, and it is matched on the `answers` phrases first. Two purposes whose
# phrases share every term separate nothing, and every task then matches both
# equally. Read `{package}`'s purposes, and add the phrases a person here would
# actually type.
#
#   add:
#     purposes.rationale.answers: [\"why is it this way\", \"what was rejected\"]
#
# INTERVIEW 2 --- what identifies a document, and what does the prefix mean?
#
# A relation names its target by identifier. A corpus whose documents carry none
# has no edges, and no check about an edge can say anything about it.
#
#   add:
#     identifier_schemes.doc_id: {{pattern: \"{{namespace}}-DOC-{{slug}}\", namespace: ACME, allocation: minted-once}}
#     kinds.<kind>.identifier: {{scheme: doc_id}}
#
# INTERVIEW 3 --- what does this corpus already write?
#
# Run `headwater infer` once this file resolves. It reports the files that
# classify as nothing, which is the half a payload cannot carry, and the
# documents that state no summary, which nothing will route to.

add: {{}}
"
    );

    if let Some(parent) = declaration.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return refuse(&format!("cannot create {}: {error}", parent.display()));
        }
    }
    if let Err(error) = std::fs::write(&declaration, &declaration_text) {
        return refuse(&format!("cannot write {}: {error}", declaration.display()));
    }
    let overlay = root.join(".headwater/overlay.yml");
    if let Err(error) = std::fs::write(&overlay, &overlay_text) {
        return refuse(&format!("cannot write {}: {error}", overlay.display()));
    }

    println!("wrote {}", headwater_resolve::package::CONSUMER);
    println!("wrote .headwater/overlay.yml");
    println!("\nwhat this read off the tree");
    println!("  corpus root {corpus_root}");
    match &found {
        Some(version) => println!("  package {package} {version}, under `packages/`"),
        // **Two routes, because this verb cannot know which one the reader
        // holds.** The line said "Vendor it before resolving" until
        // [#276](https://github.com/headwater-ai/headwater/issues/276) ruled on
        // it. `vendor` in this engine takes a published artifact, and
        // `HW-OBL-0085` records that nothing here fetches one, so the old line
        // named the single action a reader on a fresh tree cannot perform — in
        // the first verb an adopter runs. A newcomer read it as *copy*, which is
        // the misreading #271 was filed from and which the tutorial spent a
        // paragraph repairing. So the copy is named, because it is what an
        // adopter can do, and `vendor` keeps its own sense with the artifact
        // beside it.
        None => println!(
            "  package {package} is not under `packages/`, and nothing here fetches one. Copy a \
             package directory into `packages/` to resolve against it, or run `headwater taxonomy \
             vendor <dir>` on a published artifact to reach `pin.current` too"
        ),
    }
    println!("\nwhat it cannot read off a tree, and asked instead");
    println!("  the phrases each purpose answers, which decide what a task routes to");
    println!("  the identifier scheme, and what its prefix discriminates");
    println!("  which bundles this corpus already follows");
    println!(
        "\nAnswer them in .headwater/overlay.yml, then run `headwater taxonomy resolve` and \
         `headwater infer`"
    );
    ExitCode::SUCCESS
}

/// The directory under `root` holding the most Markdown files.
///
/// One level down, and never `root` itself. A repository whose Markdown is at
/// the top is a repository whose corpus root is the whole of it, and proposing
/// that would put the census over `target/` and `node_modules/`.
/// A scalar as a double-quoted YAML string.
///
/// Everything this verb emits goes through it rather than only the fields that
/// look dangerous today. A payload that does not load is a payload the next
/// command refuses, and the failure would be reported against the lock rather
/// than against the text that produced it.
///
/// # The three routes a review found, and what each one cost
///
/// That paragraph was a claim and not a fact until a review of
/// [#455](https://github.com/headwater-ai/headwater/issues/455) tested it. This
/// escaped `"` and `\` alone, and the `pairs` entries reached the payload as an
/// unquoted flow mapping that never came here at all. Three routes followed,
/// and each one was reached by running the binary rather than by reading it:
///
/// - A newline in `--owner` wrote a raw line break inside a double-quoted
///   scalar. The payload stopped loading, and the run told the caller that
///   neither their corpus nor their command line caused it.
/// - A `}` in a document's filename closed the flow mapping early, with the
///   same false sentence under it.
/// - A comma in a filename was worse than either, because nothing failed.
///   `docs/spec/a,b.md` was read as the value `docs/spec/a`, the run exited 0,
///   and the lock declared debt against a document that does not exist.
///
/// So the escaping is by category rather than by the two characters that were
/// noticed first: a control character is what a raw line break is a member of,
/// and [`char::is_control`] is the whole C0 and C1 range. `U+2028` and `U+2029`
/// are outside it and YAML reads both as line breaks, so they are named.
///
/// The escaping itself is [`headwater_resolve::render::quoted`] rather than a
/// second copy here. This one was the second copy, and being a second copy is
/// how it fell three characters behind the writer that puts the same strings
/// into the lock. Repairing it exposed a fourth route in that writer, which is
/// recorded there.
///
/// The invariant this holds is what makes `defect`'s two call sites in
/// [`infer`] unreachable, and
/// `tests/wiring.rs::a_payload_this_verb_writes_loads_whatever_a_path_or_an_owner_holds`
/// drives every route.
fn quoted(text: &str) -> String {
    headwater_resolve::render::quoted(text)
}

fn busiest_directory(root: &Path) -> Option<String> {
    let mut best: Option<(String, usize)> = None;
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        let count = markdown_under(&entry.path(), 0);
        if count == 0 {
            continue;
        }
        if best.as_ref().is_none_or(|(_, most)| count > *most) {
            best = Some((name, count));
        }
    }
    best.map(|(name, _)| name)
}

fn markdown_under(directory: &Path, depth: usize) -> usize {
    if depth > 6 {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(directory) else {
        return 0;
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count += markdown_under(&path, depth + 1);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            count += 1;
        }
    }
    count
}

/// Two spaces in front of each line of a section, so that three reports in one
/// stream stay tellable apart. The reports themselves stay unindented, because
/// a fixture records what a library renders and not what a CLI wrapped.
fn indent(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                String::from("\n")
            } else {
                format!("  {line}\n")
            }
        })
        .collect()
}

fn refused(what: &str, errors: &[headwater_census::shelves::DeclarationError]) -> ExitCode {
    eprintln!(
        "headwater: {}",
        err(&format!("{what} did not read, so no run is possible"))
    );
    for error in errors {
        eprintln!("  {}", err(&error.to_string()));
    }
    ExitCode::FAILURE
}

/// A refusal a caller's command line earned. The message, and one line saying
/// where the grammar is.
///
/// # The ruling this carries, which reverses the one it used to carry
///
/// The whole of `USAGE` printed here, under every message. The reason it did is
/// the sentence under [`refuse`]: a caller who wrote the wrong flag is reading
/// the grammar, so the grammar went where that caller already was. That reason
/// was written against a `USAGE` of about a hundred lines. It reached 359
/// source lines and 25,415 bytes on the wire, so the one sentence a caller
/// needed arrived above a screenful that scrolled it away, and anything
/// recording this stream recorded the whole manual once per typo. The reason
/// `refuse` gives for existing had grown into a reason against this function,
/// which is #306's argument and it needs nothing else to stand.
///
/// So the pointer replaces the body. Nothing a caller could do with the grammar
/// here is lost: `headwater --help` writes it to standard output and exits 0,
/// and it is named on the line under every refusal.
///
/// # Every call site moved, at once and on purpose
///
/// `USAGE` was this function's suffix rather than anything a call site passed,
/// so all 98 of them moved together on one edit. #306 asks which of "all of
/// them changed deliberately" and "none of them did" was meant, and this is the
/// answer: **all of them**, and not one message text changed.
///
/// The parser migration ([#321](https://github.com/headwater-ai/headwater/issues/321))
/// then took the sites the argument loop and dispatch owned: one caller now
/// hands over the line `clap` wrote for all of them. What was left is 68
/// sites, not 45: four more joined after #321 landed, as this binary grew a
/// `Help` verb, a `Completions` verb and two more flags.
///
/// [#331](https://github.com/headwater-ai/headwater/issues/331) read every
/// one of the 68 and moved fifteen to [`refuse`]: a fixed location this
/// engine reads or writes regardless of the command line, a fact the corpus
/// itself declares wrong, and a fact about the host. What stays here is
/// answerable by one question — would retyping the command line differently
/// change the answer? — and that includes a caller-supplied value this run
/// went on to check against something the corpus declares (a level, a
/// profile, an import, a directory) and found wanting, because the value that
/// was wrong is still the one the caller typed.
///
/// # The eight sites #331 left, and the line that settles them
///
/// [#455](https://github.com/headwater-ai/headwater/issues/455) settled the
/// eight sites #331 left here, and the line it drew is the one this function
/// now states. A caller naming a location — a `--root`, a fetched artifact, a
/// directory — does not make the refusal a command-line fact: [`refuse`]'s own
/// case in `tests/wiring.rs` is reached through an explicit `--root`. What
/// decides it is whether a spelling of this request gets past the refusal.
///
/// An artifact that is whole and simply is not this repository's package stays
/// here (the two package-mismatch sites, in `migrate` and in `diff`), because
/// a different artifact is a different command
/// line and the grammar names the argument that was wrong. An artifact whose
/// own payload is unreadable, absent for the transition, or ambiguous does not,
/// because no spelling of this command line reads it. A refusal a flag repairs
/// stays here even where a file edit is the other remedy: `taxonomy vendor`
/// with no pin names `--expect` beside the declaration, and `--expect` is
/// grammar.
///
/// # Why the prefix is written per line
///
/// `clap` puts a `tip:` line under some of its messages, and every other line
/// this binary writes to standard error opens with its own name. A message with
/// no newline in it prints exactly the two lines it printed before this loop.
fn fail(message: &str) -> ExitCode {
    for line in message.lines() {
        eprintln!("headwater: {}", err(line));
    }
    eprintln!("headwater: run `headwater --help` for the grammar");
    ExitCode::FAILURE
}

/// `text`, in the `error` role when standard error is a terminal, and
/// unchanged otherwise.
///
/// [`fail`] colored its own message this way from the day
/// [HW-DR-0045](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
/// landed. This is that rule, pulled out once [`refuse`], [`defect`],
/// [`refused`] and every refusal this binary prints inline needed it too,
/// rather than a second hand-written wrap at each of them. It reads the
/// stream itself rather than taking a `mode`, because every call site here
/// is standard error and none of them is a pure function under test the way
/// `headwater_check::paint`'s own callers are — see that module's comment for
/// the boundary.
fn err(text: &str) -> String {
    headwater_cli::paint::paint(
        headwater_cli::paint::Role::Error,
        text,
        headwater_cli::paint::stderr_color(),
    )
}

/// A refusal no command line reaches past: a fact about the corpus, the
/// filesystem, the host, a fetched artifact's own content, or a wait this
/// engine declares — true regardless of what command line reached it.
///
/// [`fail`] names where the grammar is, because a caller who wrote the wrong
/// flag is looking for it. A caller who hit one of these is not: a fixed
/// location this engine reads or writes on every run
/// (`.headwater/taxonomy.lock`, `.headwater/overlay.yml`, the consumer
/// declaration, a probe budget declaration) refusing to exist, to parse, or to
/// accept a write; a fact the package, the corpus, or a fetched artifact itself
/// declares wrong or incomplete, independent of any flag; a fact about the host
/// that no flag repairs, such as a clock reading before 1970; or a verb this
/// binary parses and this engine has never implemented, where no other spelling
/// of the request exists. A line pointing at the grammar under one of these
/// points away from what the sentence says.
///
/// The boundary with [`defect`] is who wrote the value. Everything here was
/// authored by somebody — a corpus, an overlay, a published artifact — and a
/// value this run's own code built belongs there instead.
fn refuse(message: &str) -> ExitCode {
    eprintln!("headwater: {}", err(message));
    ExitCode::FAILURE
}

/// A refusal that is a defect in this engine: a value this run's own code built
/// failed a check this run's own code makes.
///
/// Neither [`fail`] nor [`refuse`] fits. The command line was correct and no
/// grammar helps, so the pointer [`fail`] adds is wrong. The corpus, the
/// filesystem and the host are all sound, so a caller sent here by [`refuse`]
/// would search their own documents for a fault that is ours.
///
/// The line under the message says whose fault it is, and names the engine
/// version, which is what a report of it needs. It names no address to send
/// that report to: this repository has no published home yet, and a line
/// naming one would be a line that stops being true.
///
/// # What the line asserts, and what had to become true before it could
///
/// "Neither your corpus nor your command line caused it" is a strong claim, and
/// it was **false when it was first written**. A review of #455 reached both
/// call sites from outside: a newline in `--owner`, and a `}` in a document's
/// filename. In each case the run printed that sentence to a caller whose
/// command line or whose corpus was exactly the cause.
///
/// What makes it true is [`quoted`] rather than this function. Every scalar the
/// payload carries now goes through it, control characters included, and the
/// `pairs` entries are quoted rather than written into a bare flow mapping. Its
/// doc comment records the three routes and the third one, which failed
/// silently rather than loudly.
///
/// So the two call sites are unreachable, and they are unreachable for a reason
/// a reader can check rather than by luck. That is the point rather than an
/// excuse: nothing drives them, so nothing executes them, and the invariant
/// they hold is the last thing saying the payload this run wrote is the payload
/// it meant to write. Read this claim as conditional on that one, and re-test
/// it rather than trusting it if `quoted` ever stops being the one route out.
///
/// Two cases hold this pair, and neither drives the sites, because after the
/// repair nothing can.
/// `tests/wiring.rs::every_refusal_about_a_value_this_run_built_goes_out_through_defect`
/// holds which helper carries each message, and
/// `tests/wiring.rs::the_defect_helper_names_the_engine_version_and_no_address`
/// holds what this body prints. The second exists because the review rewrote
/// this body to print an address and drop the version constant, and the whole
/// suite stayed green.
fn defect(message: &str) -> ExitCode {
    eprintln!("headwater: {}", err(message));
    eprintln!(
        "headwater: {}",
        err(&format!(
            "this is a defect in engine {}, and neither your corpus nor your command line caused it",
            headwater_resolve::release::ENGINE
        ))
    );
    ExitCode::FAILURE
}
