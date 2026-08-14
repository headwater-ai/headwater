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

const USAGE: &str = "\
headwater check              [--strict] [--fix] [--no-cache] [--now <date>]
                             [--read-set <path>] [--register <path>]
                             [--format text|json|sarif|markdown] [--root <path>]
headwater gate               --read-set <path> [--now <date>] [--root <path>]
headwater route              <task description> [--budget <n>] [--root <path>]
headwater explain            <path|identifier> [--root <path>]
headwater mcp                [--now <date>] [--write] [--root <path>]
headwater new                <kind> --title <text> [--relates <relation>=<identifier>]
                             [--now <date>] [--root <path>]
headwater capture            [--format text|json] [--root <path>]
headwater sweep plan         [--under <path>] [--root <path>]
headwater sweep report       <path> [--format text|json] [--root <path>]
headwater generate           [--check] [--root <path>]
headwater import             [<name>] [--expect <digest>] [--write] [--root <path>]
headwater export             [--profile <name>] [--format json|jsonschema] [--at <date>]
                             [--check] [--root <path>]
headwater init               [--corpus <dir>] [--package <name>] [--root <path>]
headwater infer              [--owner <name>] [--until <date>] [--write]
                             [--now <date>] [--root <path>]
headwater conformance        [--level <name>] [--now <date>] [--root <path>]
headwater taxonomy validate  [--root <path>]
headwater taxonomy resolve   [--check] [--root <path>]
headwater taxonomy audit     [--now <date>] [--root <path>]
headwater taxonomy publish   [--package <name>] --out <dir> [--root <path>]
headwater taxonomy vendor    <dir> [--expect <digest>] [--root <path>]

  check              run the pipeline over the corpus, against the taxonomy in
                     the committed lock.
  gate               hold the read set of an earlier run against the tree in
                     front of it, and report whether the verdicts of that run
                     carry to this one. It reads only what the set lists, so it
                     reports the reach of its own answer and never reports that
                     a corpus is green. It exits non-zero on a verdict that does
                     not carry, which is the signal to run the checks again.
  route              resolve a task description to the documents that govern it,
                     as pointers. It is silent when nothing matches.
  explain            why a document is the kind it is, what it serves, and what
                     is consequently required of it.
  mcp                serve the reads above, and one run of the checks, to an
                     agent over the Model Context Protocol, on standard input
                     and output. It registers spec 5's query class, and with
                     `--write` the working-tree write class beside it. The
                     corpus is walked once before it starts and the clock is
                     read once, so every call answers about the same tree at the
                     same date, and the `check` tool returns the bytes `check
                     --format` returns. A call that moves a byte of that tree
                     ends the server rather than answering from a walk it made
                     stale.
  new                scaffold a document of a kind: the placement its shelf
                     dictates, the front matter its facets require, the sections
                     its contract requires, an identifier under its scheme, and
                     the edges the taxonomy assigns to a scaffold. It writes no
                     generated-file marker, because what it writes is an
                     authored document from the moment it lands and every check
                     reads it. It decides everything before it writes anything,
                     and it never overwrites a document. Every run that writes a
                     document appends one capture-cost reading to the store, and
                     a run whose reading did not land exits non-zero.
  capture            read the capture-cost store back: the assisted fraction
                     over every reading it holds, the same by kind, and how far
                     the authoring verb reaches into the corpus. It names no
                     person and no agent, and it never averages readings taken
                     under two taxonomies.
  generate           write every projection the taxonomy declares, and report
                     every one it does not write with the reason. It refuses to
                     overwrite a file that carries no generated-file marker.
  import             read a snapshot that somebody already fetched and
                     committed, and write the edges it declares into the
                     documents at their near ends. The snapshot is checked
                     against a digest and a channel that a person wrote into
                     `.headwater/taxonomy.yml`, and an import with neither is
                     refused rather than recorded. Without `--write` it reports
                     the edges and touches nothing. A wrong imported edge would
                     produce a correct check result over a wrong graph, so every
                     link is refused whole rather than reported as a finding.
  export             emit one declared export profile through one emitter
                     target, with the loss set the target declares and the
                     projection census that holds the output against the graph.
                     With `--format` it writes the artifact to standard output,
                     which is what a consumer outside this repository asks for.
                     Without one it writes every declared export to the path its
                     taxonomy names, and `--check` holds those to regeneration.
  sweep              the two deterministic halves of the coherence sweep, which
                     is a sampler and never a check. `plan` writes the briefing
                     an agent reads: the slice, what the graph already declares
                     about it, and the file to write back. `report` reads that
                     file and says what this engine could confirm about it —
                     that every quotation is in the document it names, that
                     every path is a classified document, and that no proposed
                     edge is one the graph already carries. No model is reached
                     from this binary, both halves exit 0 whatever they find,
                     and neither writes a byte of the corpus.
  init               scaffold the consumer declaration and the overlay for a
                     repository that has neither, and print the questions that
                     no tree answers. It refuses to overwrite a binding.
  infer              report the debt this taxonomy raises over this corpus as
                     an adoption payload: `(document, rule)` pairs under tasks
                     that each carry an owner and an expiry. It prints the
                     payload and writes nothing without `--write`.
  conformance        evaluate this repository against the conformance rules the
                     taxonomy package ships, and report the level that the
                     passing rules reach. A level states what this repository
                     wired up: it measures nothing about the corpus, no key
                     declares one, and a waiver moves the exit status and never
                     the level. A rule this engine holds no reading for ends the
                     run rather than being skipped. Without `--level` it exits 0
                     whatever it finds.
  taxonomy validate  resolve the sources and report every rule of spec 2's
                     list, and what each one did not decide. Writes nothing.
  taxonomy resolve   write `.headwater/taxonomy.lock`. It is written only when
                     the taxonomy validates, so a lock is a validated taxonomy.
  taxonomy audit     measure the taxonomy against the corpus: edge counts and
                     staleness by the creator each relation declares, relation
                     drift by family, facet differentiation, the discriminator
                     distribution of a heterogeneous shelf, and state dwell. It
                     reports findings about the schema and never about a
                     document, it gates nothing, and it always exits 0. One bar
                     is declared and the rest of the readings are distributions
                     with no verdict beside them.
  taxonomy publish   write the artifact of a package into a directory, with a
                     release record over it: every file, the digest of its
                     bytes, and one digest over that list. It prints the digest,
                     which is the number the release notes state and a consumer
                     pins.
  taxonomy vendor    check an artifact that somebody already fetched against the
                     digest this repository pinned, and install it under
                     `packages/`. It refuses an artifact that is not the pinned
                     one, and it names every file that moved. Nothing here
                     fetches: no crate of this engine depends on the network, so
                     the verb takes the path of a directory and never a
                     location.

  --strict       `check` only: exit non-zero when a finding is an error. Without
                 it the run is advisory and always exits 0, which is the default
                 spec 6 fixes.
  --fix          `check` only: write the patch that rides with a finding, in
                 this working tree. A finding carries one only when the fix is
                 mechanical and total, and a finding an author suppressed
                 carries none. Every patch is held against the bytes it names
                 and the result is read back before it lands, so a file whose
                 shape this engine guessed wrong is refused with nothing
                 written. The report that follows is the run after the write,
                 and the account of what was written goes to standard error.
                 It exits non-zero on a refusal.
  --no-cache     `check` only: read and write no cache, and evaluate every
                 instance. This run and a cached one write the same bytes to
                 standard output, and a difference between them is a defect in
                 the cache rather than a result.
  --now <date>   `check`, `gate`, `new`, `mcp`, `conformance` and
                 `taxonomy audit`: the date to
                 evaluate against, as `YYYY-MM-DD`. Defaults to today. On
                 `taxonomy audit` it is what a staleness reading and a dwell
                 reading are taken at, so two audits of one tree at one date
                 write the same bytes. Spec 12 makes the
                 clock an injected value rather than a syscall inside a check,
                 and this flag is where it is injected: same corpus, same lock,
                 same date, same bytes. On `gate` it is the day the question is
                 asked about, and a run that read the clock is void on any
                 other day. On `mcp` it is read once and fixed for the life of
                 the server, and every result states it.
  --read-set <path>
                 `check`: write the read set of this run to a file as well as
                 to the report. `gate`: the file to hold against this tree, and
                 the flag is required there. The artifact is what decides
                 whether a verdict survives a merge without running the checks
                 again.
  --register <path>
                 `check` only: write the register of this run to a file as well
                 as to the report. Spec 4 makes it a projection of the
                 `obligations` and `controls` declarations, generated and never
                 authored: every obligation with its disposition, every control
                 with its health, and what escaped under each.
  --title <text> `new` only: what the document is called. Required, because the
                 file name and the facet in the `name` role both come from it.
  --relates <relation>=<identifier>
                 `new` only, and repeatable: an edge to propose, as a relation
                 and the identifier of the document at the other end. It is
                 refused unless the taxonomy declares `created_by: scaffold` on
                 the relation, unless both ends are kinds the relation permits,
                 and unless the target resolves. Where reciprocity is required
                 the far half is written into the target document.
  --budget <n>   `route` only: how many pointers it may offer. Five by default.
  --owner <name>
                 `infer` only: who owns the debt it proposes. Required with
                 `--write`, because an owner is the field that ranks declared
                 debt above a suppression and this engine will not invent one.
  --until <date> `infer` only: the last day the tasks it proposes hold, as
                 `YYYY-MM-DD`. Ninety days out by default.
  --level <name> `conformance` only: the rung to ask about, by the name the
                 package declares. It exits non-zero on a gap under that rung
                 that no live waiver covers. It never moves the level the report
                 states, which is computed from met rules alone.
  --write        `infer`: put the payload in the lock, which is committed and
                 reviewed. Without it nothing is written. `mcp`: register the
                 working-tree write class, which is `new` and `fix`. Spec 5
                 keeps it off by default, because a client may connect to a
                 checkout that the user did not intend to change, so the
                 consent is a word somebody typed rather than a setting a tree
                 carries. A tool that lands a change is registered by no
                 switch. The first call that moves a byte ends the server: it
                 walked the corpus once, so every later answer would be about a
                 tree that is gone.
  --corpus <dir> `init` only: the corpus root to declare. Proposed from the
                 tree by default.
  --package <name>
                 `init`: the package to take. `headwater/standard` by default.
                 `taxonomy publish`: the package to publish. The one this
                 repository's own declaration takes, by default, because a
                 publisher usually publishes what it also consumes.
  --check        `taxonomy resolve` and `generate`: write nothing and exit
                 non-zero when what is committed is not what a run produces. The
                 two read different things. `taxonomy resolve --check` reads the
                 taxonomy sources, so it answers whether the lock is current.
                 `generate --check` reads the corpus through the lock, so it
                 answers whether a derived artifact is.
  --profile <name>
                 `export` only: which declared export profile to emit. Every
                 declared profile by default, so a filtered audience is never
                 omitted by accident.
  --format <target>
                 `check`: which vocabulary to write the run in. `text` is the
                 report a person reads and the default. `sarif` is what a forge
                 ingests as a check run, `markdown` is a job summary or a review
                 comment, and `json` is the finding shape spec 4 declares, for an
                 adapter nobody here wrote. Each names what it could not carry.

                 `export`: the emitter target. `json` is the native
                 property graph with no loss and `jsonschema` constrains front
                 matter. The other five targets of spec 6 parse and report the
                 consumer each one waits on. With this flag the artifact goes to
                 standard output and no declared output path is touched.

                 `capture`: `text` is the report a person reads and the default,
                 and `json` is the same numbers for a program. Neither carries a
                 reading the store does not hold.

                 `sweep report`: `text` is the report a person reads and the
                 default, and `json` is the finding shape spec 4 declares with
                 the provenance and the evidence a sweep adds.
  --under <path> `sweep plan` only: the slice, as a path prefix under the
                 repository root. The whole corpus by default. There is no
                 sampling rule here: a slice this engine picked would be an
                 unreproducible sample dressed as a reproducible one, and the
                 plan reports its own extent instead.
  --at <date>    `export` only: the generation time the artifact states, as
                 `YYYY-MM-DD`. Absent by default, because an artifact that
                 `--check` compares by byte cannot carry a clock reading. Spec 6
                 asks a filtered export that leaves the repository to state one,
                 and this is where it is injected.
  --out <dir>    `taxonomy publish` only: where to write the artifact. The
                 directory must be empty or absent, because a published artifact
                 is every file under its root and a stray one would be a member
                 the publisher never shipped.
  --expect <d>   `taxonomy vendor` only: the digest to check the artifact
                 against. It defaults to `taxonomy.digest` in
                 `.headwater/taxonomy.yml`, and the verb refuses when neither is
                 there. A pin the engine took from the artifact in front of it
                 would be a pin against itself.
  --root <path>  the repository to read. Defaults to the working directory.
";

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let mut strict = false;
    let mut check_only = false;
    let mut fixing = false;
    let mut cached = true;
    let mut now: Option<Date> = None;
    let mut read_set: Option<PathBuf> = None;
    let mut register_out: Option<PathBuf> = None;
    let mut root: Option<PathBuf> = None;
    let mut budget: Option<usize> = None;
    let mut write = false;
    let mut owner: Option<String> = None;
    let mut until: Option<Date> = None;
    let mut corpus_root: Option<String> = None;
    let mut package: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut expect: Option<String> = None;
    let mut profile: Option<String> = None;
    let mut format: Option<String> = None;
    let mut generated_at: Option<String> = None;
    let mut title: Option<String> = None;
    let mut under: Option<String> = None;
    let mut relates: Vec<(String, String)> = Vec::new();
    let mut level: Option<String> = None;
    let mut words: Vec<String> = Vec::new();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--strict" => strict = true,
            "--check" => check_only = true,
            "--fix" => fixing = true,
            "--no-cache" => cached = false,
            "--now" => match arguments.next().as_deref().map(Date::parse) {
                Some(Some(date)) => now = Some(date),
                Some(None) => return fail("--now takes a date written `YYYY-MM-DD`"),
                None => return fail("--now names a date and none followed it"),
            },
            "--budget" => match arguments.next().as_deref().map(str::parse::<usize>) {
                Some(Ok(value)) if value > 0 => budget = Some(value),
                Some(_) => return fail("--budget takes a whole number above zero"),
                None => return fail("--budget names a number and none followed it"),
            },
            "--write" => write = true,
            "--profile" => match arguments.next() {
                Some(name) => profile = Some(name),
                None => return fail("--profile names an export profile and none followed it"),
            },
            "--format" => match arguments.next() {
                Some(name) => format = Some(name),
                None => return fail("--format names an emitter target and none followed it"),
            },
            "--at" => match arguments.next() {
                Some(text) => match Date::parse(&text) {
                    Some(_) => generated_at = Some(text),
                    None => return fail("--at takes a date written `YYYY-MM-DD`"),
                },
                None => return fail("--at names a date and none followed it"),
            },
            "--under" => match arguments.next() {
                Some(path) => under = Some(path),
                None => return fail("--under names a path prefix and none followed it"),
            },
            "--title" => match arguments.next() {
                Some(text) => title = Some(text),
                None => return fail("--title names the document and none followed it"),
            },
            "--relates" => match arguments.next() {
                Some(pair) => match pair.split_once('=') {
                    Some((relation, target)) if !relation.is_empty() && !target.is_empty() => {
                        relates.push((relation.to_string(), target.to_string()))
                    }
                    _ => {
                        return fail(
                            "--relates takes `<relation>=<identifier>`, as in \
                             `--relates supersedes=DR-repo-0007`",
                        )
                    }
                },
                None => {
                    return fail("--relates names a relation and a target, and none followed it")
                }
            },
            "--owner" => match arguments.next() {
                Some(name) => owner = Some(name),
                None => return fail("--owner names a person or a team and none followed it"),
            },
            "--corpus" => match arguments.next() {
                Some(directory) => corpus_root = Some(directory),
                None => return fail("--corpus names a directory and none followed it"),
            },
            "--package" => match arguments.next() {
                Some(name) => package = Some(name),
                None => return fail("--package names a package and none followed it"),
            },
            "--level" => match arguments.next() {
                Some(name) => level = Some(name),
                None => return fail("--level names a level and none followed it"),
            },
            "--out" => match arguments.next() {
                Some(path) => out = Some(PathBuf::from(path)),
                None => return fail("--out names a directory and none followed it"),
            },
            "--expect" => match arguments.next() {
                Some(text) => expect = Some(text),
                None => return fail("--expect names a digest and none followed it"),
            },
            "--until" => match arguments.next().as_deref().map(Date::parse) {
                Some(Some(date)) => until = Some(date),
                Some(None) => return fail("--until takes a date written `YYYY-MM-DD`"),
                None => return fail("--until names a date and none followed it"),
            },
            "--register" => match arguments.next() {
                Some(path) => register_out = Some(PathBuf::from(path)),
                None => return fail("--register names a file and none followed it"),
            },
            "--read-set" => match arguments.next() {
                Some(path) => read_set = Some(PathBuf::from(path)),
                None => return fail("--read-set names a path and none followed it"),
            },
            "--root" => match arguments.next() {
                Some(path) => root = Some(PathBuf::from(path)),
                None => return fail("--root names a path and none followed it"),
            },
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                return fail(&format!("`{other}` is not a flag this binary knows"));
            }
            other => words.push(other.to_string()),
        }
    }

    let root = match root {
        Some(path) => path,
        None => match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => return fail(&format!("no working directory: {error}")),
        },
    };

    let verb: Vec<&str> = words.iter().map(String::as_str).collect();
    match verb.as_slice() {
        ["check"] => check(
            &root,
            Asked {
                strict,
                cached,
                fixing,
                now,
                read_set,
                register_out,
                format,
            },
        ),
        ["gate"] => gate(&root, read_set, now),
        ["route"] => fail("`route` takes a task description. Try `headwater route \"add rate limiting to the ingest API\"`"),
        ["route", task @ ..] => route(&root, &task.join(" "), budget),
        ["explain"] => fail("`explain` takes a path or an identifier"),
        ["explain", target] => explain(&root, target),
        ["mcp"] => mcp(&root, now, write),
        ["new"] => fail(
            "`new` takes a kind. Try `headwater new decision --title \"Adopt an overlay\"`",
        ),
        ["new", kind] => new(&root, kind, title, &relates, now),
        ["capture"] => capture(&root, format),
        ["sweep"] => fail("`sweep` takes a second word: `plan` or `report`"),
        ["sweep", "plan"] => sweep_plan(&root, under),
        ["sweep", "report"] => fail(
            "`sweep report` takes the path of the file an agent wrote back. \
             `headwater sweep plan` prints the shape of it",
        ),
        ["sweep", "report", path] => sweep_report(&root, Path::new(path), format),
        ["sweep", other, ..] => fail(&format!(
            "`sweep {other}` is not a verb this binary carries. It carries `plan` and `report`"
        )),
        ["generate"] => generate(&root, check_only),
        ["import"] => import(&root, None, expect.as_deref(), write),
        ["import", name] => import(&root, Some(name), expect.as_deref(), write),
        ["export"] => export(&root, profile, format, generated_at, check_only),
        ["init"] => init(&root, corpus_root, package),
        ["infer"] => infer(&root, owner, until, write, now),
        ["conformance"] => conformance(&root, level.as_deref(), now),
        // Spec 6 lists this verb and no document of the specification states
        // what an expression is. The engine names the gap rather than invent a
        // form, which is the posture the resolver takes over a `$package`
        // reference for the same reason.
        ["query", ..] => fail(
            "`query <expression>` is listed in spec 6 and no document states what an expression \
             is, so this engine implements none. See `docs/spec/13-open-obligations.md`. \
             `headwater route` and `headwater explain` are the reads that exist",
        ),
        ["taxonomy", "validate"] => validate(&root),
        ["taxonomy", "resolve"] => resolve(&root, check_only),
        ["taxonomy", "audit"] => audit(&root, now),
        ["taxonomy", "publish"] => publish(&root, package.as_deref(), out.as_deref()),
        ["taxonomy", "vendor"] => fail(
            "`taxonomy vendor` takes the path of a package somebody already fetched. \
             This engine opens no socket, so it checks a directory it is handed",
        ),
        ["taxonomy", "vendor", fetched] => {
            vendor(&root, Path::new(fetched), expect.as_deref())
        }
        ["taxonomy"] => fail(
            "`taxonomy` takes a second word: `validate`, `resolve`, `audit`, `publish` or \
             `vendor`",
        ),
        // `diff` and `migrate` are named in spec 6's grammar and neither one
        // runs. The grammar holds a declared name to running or to a named
        // wait, so each one says what it waits on rather than reading as a
        // verb this binary forgot.
        ["taxonomy", "diff", ..] => fail(
            "`taxonomy diff` waits on a second taxonomy to compare against. A release record \
             names every file of a published artifact and its digest, and nothing yet reads two \
             of them as one comparison. See `docs/spec/07-distribution-and-federation.md`",
        ),
        ["taxonomy", "migrate", ..] => fail(
            "`taxonomy migrate` waits on `taxonomy diff`. A migration payload names the version \
             it came from, and no run can name one without a measured comparison against it. \
             See `docs/spec/07-distribution-and-federation.md`",
        ),
        ["taxonomy", other, ..] => fail(&format!(
            "`taxonomy {other}` is not a verb this binary carries yet. \
             It carries `validate`, `resolve`, `audit`, `publish` and `vendor`"
        )),
        [] => fail("no verb. Try `headwater check`"),
        // The list below is hand-maintained beside the arms above, and nothing
        // holds the two together, so a new verb needs an edit in both places.
        // Spec 6 keeps a third copy as the CLI grammar. #138 removed a name
        // from that grammar which no arm here carries, and found this message
        // one verb short of the arms in the same reading.
        [other, ..] => fail(&format!(
            "`{other}` is not a verb this binary carries yet. \
             It carries `check`, `gate`, `route`, `explain`, `mcp`, `new`, `capture`, \
             `sweep`, `generate`, `import`, `export`, `init`, `infer`, `conformance` and \
             `taxonomy`"
        )),
    }
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
            eprintln!("headwater: the taxonomy did not resolve");
            eprint!("{}", indent(&render_errors(&errors)));
            return ExitCode::FAILURE;
        }
    };

    println!("sources, in application order");
    for source in &repository.resolution.sources {
        println!("  {source}");
    }

    let findings = repository.resolution.validate();
    println!("\nrules");
    print!("{}", headwater_resolve::rules::render());

    if findings.is_empty() {
        println!("\n{} is valid", repository.consumer.package);
        return ExitCode::SUCCESS;
    }
    println!("\n{} is not valid", repository.consumer.package);
    eprint!("{}", indent(&render_errors(&findings)));
    ExitCode::FAILURE
}

/// `headwater taxonomy resolve`, and `--check` over a committed lock.
fn resolve(root: &Path, check_only: bool) -> ExitCode {
    let repository = match headwater_resolve::repository(root) {
        Ok(repository) => repository,
        Err(errors) => {
            eprintln!("headwater: the taxonomy did not resolve, so no lock is possible");
            eprint!("{}", indent(&render_errors(&errors)));
            return ExitCode::FAILURE;
        }
    };
    let sources = match headwater_resolve::package::sources(root, &repository.consumer) {
        Ok(sources) => sources,
        Err(errors) => {
            eprint!("{}", indent(&render_errors(&errors)));
            return ExitCode::FAILURE;
        }
    };
    // The payload is the one authored part of the lock, so a resolve reads it
    // off the committed file and writes it back. A resolve that dropped it
    // would delete an adopter's accounting as a side effect of a taxonomy edit,
    // and the run after it would report every pair the payload was holding.
    let adoption = headwater_lock::adoption_at(root);
    let text = match headwater_lock::write(
        &repository.consumer.package,
        &repository.consumer.version,
        &sources,
        &repository.resolution,
        adoption.as_ref(),
    ) {
        Ok(text) => text,
        Err(findings) => {
            eprintln!(
                "headwater: the taxonomy does not validate, so no lock is written. \
                 A lock is a validated taxonomy or it is nothing"
            );
            eprint!("{}", indent(&render_errors(&findings)));
            return ExitCode::FAILURE;
        }
    };

    let path = root.join(headwater_lock::LOCK);
    if check_only {
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        if committed == text {
            println!("{} is what the sources resolve to", headwater_lock::LOCK);
            return ExitCode::SUCCESS;
        }
        eprintln!(
            "headwater: {} is not what the sources resolve to. \
             Run `headwater taxonomy resolve` and commit the result",
            headwater_lock::LOCK
        );
        // The lock that is there says which source moved, which is the line an
        // author acts on. A lock that will not even read is its own message.
        if let Ok(lock) = headwater_lock::read(&committed) {
            for moved in lock.moved(root) {
                eprintln!("  {moved} has changed since the lock was written");
            }
        }
        return ExitCode::FAILURE;
    }

    if let Some(parent) = path.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return fail(&format!("cannot create {}: {error}", parent.display()));
        }
    }
    if let Err(error) = std::fs::write(&path, &text) {
        return fail(&format!("cannot write {}: {error}", path.display()));
    }
    println!("wrote {}", headwater_lock::LOCK);
    for source in &repository.resolution.sources {
        println!("  from {source}");
    }
    ExitCode::SUCCESS
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
fn audit(root: &Path, now: Option<Date>) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(context) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!("headwater: this host has no readable clock. Pass `--now <YYYY-MM-DD>`");
        return ExitCode::FAILURE;
    };

    let audit = headwater_audit::take(
        headwater_audit::Subject {
            package: loaded.lock.package.clone(),
            version: loaded.lock.version.clone(),
            lock: loaded.lock.digest.clone(),
            now: context.now(),
        },
        &loaded.census,
        &loaded.graph,
        &loaded.taxonomy,
        &loaded.shape,
        &loaded.relations,
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
fn conformance(root: &Path, level: Option<&str>, now: Option<Date>) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(context) = now.map(Context::at).or_else(Context::from_system_clock) else {
        eprintln!("headwater: this host has no readable clock. Pass `--now <YYYY-MM-DD>`");
        return ExitCode::FAILURE;
    };

    let set = match headwater_conformance::at(root, &loaded.consumer) {
        Ok(set) => set,
        Err(error) => return fail(&error.to_string()),
    };
    let waivers = match headwater_conformance::waivers(root) {
        Ok(waivers) => waivers,
        Err(refusals) => {
            eprintln!(
                "headwater: the `conformance` block of the consumer declaration did not read"
            );
            for refusal in &refusals {
                eprintln!("  {refusal}");
            }
            return ExitCode::FAILURE;
        }
    };

    // The projection plan, built the one way `generate` builds one. A second
    // builder here would be a second answer to the question that rule asks.
    let projections = match headwater_generate::Projections::read(&loaded.lock.taxonomy) {
        Ok(projections) => projections,
        Err(errors) => return refused("the projections", &errors),
    };
    let surface = loaded.surface();
    let plan = headwater_generate::plan(&surface, &loaded.census, &projections, &loaded.identity());

    let report = match headwater_conformance::evaluate(
        &set,
        &waivers,
        &headwater_conformance::Subject {
            root,
            consumer: &loaded.consumer,
            lock: &loaded.lock,
            census: &loaded.census,
            plan: &plan,
            now: context.now(),
        },
    ) {
        Ok(report) => report,
        Err(refusals) => {
            eprintln!("headwater: a waiver names no rule this package declares");
            for refusal in &refusals {
                eprintln!("  {refusal}");
            }
            return ExitCode::FAILURE;
        }
    };
    print!("{}", report.render());

    let Some(level) = level else {
        return ExitCode::SUCCESS;
    };
    match report.gate(level) {
        Err(why) => fail(&why),
        Ok(true) => {
            println!("\n{level} passes, with every gap under it covered by a live waiver or met");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!(
                "headwater: {level} is not passed. Each gap above states the remediation the \
                 package wrote for it"
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
fn publish(root: &Path, package: Option<&str>, out: Option<&Path>) -> ExitCode {
    let Some(out) = out else {
        return fail("`taxonomy publish` writes into a directory. Name it with `--out <dir>`");
    };
    let name = match package {
        Some(name) => name.to_string(),
        // The package this repository takes, where no flag names one. A
        // publisher usually publishes the package it also consumes.
        None => match headwater_resolve::package::consumer(root) {
            Ok(consumer) => consumer.package,
            Err(errors) => {
                eprintln!(
                    "headwater: no `--package` and this repository's declaration does not read, \
                     so nothing says what to publish"
                );
                eprint!("{}", indent(&render_errors(&errors)));
                return ExitCode::FAILURE;
            }
        },
    };

    let record = match headwater_resolve::package::publish(root, &name, out) {
        Ok(record) => record,
        Err(errors) => {
            eprintln!("headwater: nothing was published");
            eprint!("{}", indent(&render_errors(&errors)));
            return ExitCode::FAILURE;
        }
    };

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
            eprintln!("headwater: nothing was vendored");
            eprint!("{}", indent(&render_errors(&errors)));
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
    println!(
        "\nThe digest says these are the bytes the pin was written for. It is not a signature, \
         so it says nothing about who published them. Run `headwater taxonomy resolve` to write \
         the lock this package produces."
    );
    ExitCode::SUCCESS
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
struct Loaded {
    lock: headwater_lock::Lock,
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
}

fn load(root: &Path) -> Result<Loaded, ExitCode> {
    let lock = match headwater_lock::at(root) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("headwater: {error}");
            return Err(ExitCode::FAILURE);
        }
    };
    let consumer = match headwater_resolve::package::consumer(root) {
        Ok(consumer) => consumer,
        Err(errors) => {
            eprintln!("headwater: the consumer declaration did not read");
            eprint!("{}", indent(&render_errors(&errors)));
            return Err(ExitCode::FAILURE);
        }
    };
    let corpus = Corpus::declared(root, &consumer.corpus_root, &consumer.exclusions);
    let resolved = &lock.taxonomy;
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
            eprintln!("headwater: the import declarations did not read");
            eprintln!("{}", indent(&why));
            return Err(ExitCode::FAILURE);
        }
    };
    for items in headwater_import::anchors::over(root, &imports) {
        resolvers = match resolvers.with(Box::new(items)) {
            Ok(resolvers) => resolvers,
            Err(why) => {
                eprintln!("headwater: the resolver set is ambiguous");
                eprintln!("{}", indent(&why));
                return Err(ExitCode::FAILURE);
            }
        };
    }

    let census = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(&census, &relations, &resolvers, &corpus, &config);
    Ok(Loaded {
        lock,
        consumer,
        census,
        graph,
        shape,
        taxonomy,
        relations,
        register,
        config,
    })
}

impl Loaded {
    /// What a run of the checks is held against. One constructor, because a
    /// second one is where two runs over one tree start to differ, and
    /// `check --fix` runs the checks twice on purpose.
    fn declared(&self) -> Declared<'_> {
        Declared {
            lock: &self.lock.digest,
            taxonomy: &self.taxonomy,
            shape: &self.shape,
            relations: &self.relations,
            config: &self.config,
            register: &self.register,
            adoption: self.lock.adoption.as_ref(),
            source: headwater_lock::LOCK,
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
    fn identity(&self) -> headwater_generate::Identity {
        headwater_generate::Identity {
            corpus_root: self.consumer.corpus_root.clone(),
            exclusions: self.consumer.exclusions.clone(),
            package: self.lock.package.clone(),
            version: self.lock.version.clone(),
            lock: self.lock.digest.clone(),
        }
    }
}

/// `headwater route`.
///
/// It exits 0 whether or not it offers a pointer. Spec 5 makes silence a
/// result: "below the threshold it says nothing", and a non-zero exit would
/// make an agent's shell treat a considered silence as a failure.
fn route(root: &Path, task: &str, budget: Option<usize>) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let budget = match budget {
        Some(pointers) => Budget { pointers },
        None => Budget::default(),
    };
    print!("{}", loaded.surface().route(task, budget).render());
    ExitCode::SUCCESS
}

/// `headwater explain`.
///
/// A target that names no document exits non-zero. That is not a finding about
/// a corpus, it is a question about a document that is not there, and a caller
/// who mistyped a path needs to know from the exit status.
fn explain(root: &Path, target: &str) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    match loaded.surface().explain(target) {
        Some(explanation) => {
            let explanation: headwater_query::Explanation = explanation;
            print!("{}", explanation.render());
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("headwater: `{target}` is neither a path of this corpus nor an identifier it carries");
            ExitCode::FAILURE
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
/// [OBL-repo-0001](../../../../docs/obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md)
/// holds the debt that nothing trends this number yet.
fn new(
    root: &Path,
    kind: &str,
    title: Option<String>,
    relates: &[(String, String)],
    now: Option<Date>,
) -> ExitCode {
    let Some(title) = title else {
        return fail(
            "`new` takes `--title <text>`. The file name and the document's own name both come \
             from it, and this engine invents neither",
        );
    };
    match scaffold(root, kind, &title, relates, now, EntryPoint::Terminal) {
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
/// the term [OBL-repo-0004](../../../../docs/obligations/0004-working-tree-write-tools-have-no-measured-effect.md)
/// asks the capture-cost store for, and it is the one input that is a fact
/// about the caller rather than about the corpus.
fn scaffold(
    root: &Path,
    kind: &str,
    title: &str,
    relates: &[(String, String)],
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
        resolved: &loaded.lock.taxonomy,
        shape: &loaded.shape,
        shelves: &loaded.taxonomy,
        relations: &loaded.relations,
        census: &loaded.census,
        index: &index,
        config: &loaded.config,
    };
    let request = headwater_scaffold::Request {
        kind,
        title,
        now,
        relates,
    };

    // Nothing below this line has written anything yet, which is why every
    // refusal here is a refusal with an unchanged tree behind it.
    let plan = headwater_scaffold::propose(&sources, &request).map_err(|why| why.to_string())?;
    let composed =
        headwater_scaffold::write::compose(root, &plan).map_err(|why| why.to_string())?;
    headwater_scaffold::write::apply(root, &composed).map_err(|why| why.to_string())?;

    let reading =
        headwater_scaffold::reading::Reading::of(&plan, &loaded.lock.digest, now, surface);
    let recorded = headwater_scaffold::reading::append(root, &reading);
    Ok(Written {
        artifact: scaffold_report(&plan, &composed, recorded.is_ok()),
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
                "    reconciled against {highest}, which is the highest value on this tree. \
                 A document that was deleted is not on the tree, so this is a lower bound on \
                 what was ever allocated"
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
        use headwater_yaml::json::Json;
        let pair = |value: (usize, usize)| {
            Json::Array(vec![
                Json::Raw(value.0.to_string()),
                Json::Raw(value.1.to_string()),
            ])
        };
        let count = |value: usize| Json::Raw(value.to_string());
        let day = |value: Option<Date>| match value {
            Some(date) => Json::string(date.render()),
            None => Json::Array(vec![]),
        };
        println!(
            "{}",
            Json::object([
                ("store", Json::string(headwater_scaffold::reading::STORE)),
                ("readings", count(readings.len())),
                (
                    "unreadable_lines",
                    Json::Array(
                        unreadable
                            .iter()
                            .map(|line| Json::Raw(line.line.to_string()))
                            .collect()
                    )
                ),
                (
                    "locks",
                    Json::Array(locks.iter().map(Json::string).collect())
                ),
                ("first_reading", day(first)),
                ("last_reading", day(last)),
                ("fields", pair(total.fields)),
                ("sections", pair(total.sections)),
                ("identifier", pair(total.identifier)),
                ("edge_halves", pair(total.edge_halves)),
                ("supplied", count(total.supplied())),
                ("denominator", count(total.total())),
                (
                    "by_kind",
                    Json::Array(
                        headwater_scaffold::reading::by_kind(&readings)
                            .into_iter()
                            .map(|(kind, taken, assisted)| {
                                Json::object([
                                    ("kind", Json::string(kind)),
                                    ("readings", count(taken)),
                                    ("supplied", count(assisted.supplied())),
                                    ("denominator", count(assisted.total())),
                                ])
                            })
                            .collect()
                    )
                ),
                (
                    "by_surface",
                    Json::Array(
                        headwater_scaffold::reading::by_surface(&readings)
                            .into_iter()
                            .map(|(surface, taken, assisted)| {
                                Json::object([
                                    (
                                        "surface",
                                        match surface {
                                            Some(surface) => Json::string(surface.name()),
                                            // Absent rather than a name, on the
                                            // rule the store itself follows: a
                                            // reading that states no surface is
                                            // not one of the two arms.
                                            None => Json::Array(vec![]),
                                        },
                                    ),
                                    ("readings", count(taken)),
                                    ("supplied", count(assisted.supplied())),
                                    ("denominator", count(assisted.total())),
                                ])
                            })
                            .collect()
                    )
                ),
                ("reached", count(reach.reached.len())),
                ("classified", count(classified.paths.len())),
                (
                    "resolving_to_nothing",
                    Json::Array(
                        reach
                            .lost
                            .iter()
                            .map(|at| Json::string(&readings[*at].document))
                            .collect()
                    )
                ),
            ])
            .render_pretty()
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

        // The two arms of OBL-repo-0004, as a grouping and never as a
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
             at a client and an agent at the same client are one reading, which OBL-repo-0111 \
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
        &loaded.lock.digest,
        under.as_deref().unwrap_or(""),
    );
    print!("{}", plan.render());
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
/// [OBL-repo-0108](../../../../docs/obligations/0108-every-agent-drafted-document-carries-an-accepted-by-the-drafting-agent-typed.md)
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
        lock: &loaded.lock.digest,
    };
    let report = headwater_sweep::Report::read(&source, &tree);
    match wants_json {
        true => println!("{}", headwater_sweep::json::render(&report)),
        false => print!("{}", report.render()),
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
    let declarations = match headwater_import::declared(root) {
        Ok(declarations) => declarations,
        Err(why) => return fail(&why),
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
            eprintln!("headwater: nothing was imported");
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
            eprintln!("headwater: nothing was written");
            eprintln!("{}", indent(&why));
            return ExitCode::FAILURE;
        }
    };
    if let Err(why) = headwater_import::write::apply(root, &composed) {
        eprintln!("headwater: the write stopped part way");
        eprintln!("{}", indent(&why));
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
    let projections = match headwater_generate::Projections::read(&loaded.lock.taxonomy) {
        Ok(projections) => projections,
        Err(errors) => return refused("the projections", &errors),
    };
    let surface = loaded.surface();
    let plan = headwater_generate::plan(&surface, &loaded.census, &projections, &loaded.identity());
    let report = match check_only {
        true => headwater_generate::check(root, &plan),
        false => headwater_generate::write(root, &plan),
    };
    print!("{}", report.render());
    if report.has_errors() {
        match check_only {
            true => eprintln!(
                "headwater: a projection is not what this corpus and this lock produce. \
                 Run `headwater generate` and commit the result"
            ),
            false => eprintln!("headwater: a projection did not write"),
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
    generated_at: Option<String>,
    check_only: bool,
) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let projections = match headwater_generate::Projections::read(&loaded.lock.taxonomy) {
        Ok(projections) => projections,
        Err(errors) => return refused("the projections", &errors),
    };
    let surface = loaded.surface();

    let Some(target) = format else {
        if generated_at.is_some() {
            return fail(
                "--at states the time an artifact that leaves this repository was generated,                  and it is refused for a declared output. A committed export is held to                  regeneration by byte, so a clock reading inside one would fail the gate on a                  morning when nothing changed. Name a target with --format",
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
            eprintln!("headwater: a declared export is not what this corpus and this lock produce");
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
        return fail(
            "--check compares a committed artifact against what a run produces, and --format              writes to standard output where nothing is committed. Run `headwater export              --check` over the declared outputs instead",
        );
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
            return fail(
                "this taxonomy declares no projection, so it declares no export profile.                  Spec 6 makes a profile an entry under `projections`",
            )
        }
        several => {
            return fail(&format!(
                "--format writes one artifact to standard output and this taxonomy declares {}                  profiles. Name one with --profile: {}",
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
                    "headwater: the projection census found an omission that no declared loss                      reason covers, which is a defect in this emitter rather than in the corpus"
                );
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("headwater: nothing was exported");
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
    let ctx = match now.map(Context::at).or_else(Context::from_system_clock) {
        Some(ctx) => ctx,
        None => {
            eprintln!("headwater: this host has no readable clock. Pass `--now <YYYY-MM-DD>`");
            return ExitCode::FAILURE;
        }
    };
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    // The two verbs of the write class, as this file runs them. Each closure
    // takes the arguments its tool declares and nothing else: the root and the
    // clock are the server's, and no tool may name either.
    let scaffolding = move |kind: &str, title: &str, relates: &[(String, String)]| {
        scaffold(
            root,
            kind,
            title,
            relates,
            Some(ctx.now()),
            EntryPoint::Protocol,
        )
    };
    let fixing = move |format: Format| fix_over(root, &ctx, format);
    // The corpus is read once, at startup, and every tool answers from it.
    // That is the same posture every other verb takes, and it is what makes two
    // reads in one session answer the same bytes. A write ends the session,
    // because it ends the tree that walk described.
    let server = headwater_query::mcp::Server {
        surface: loaded.surface(),
        census: &loaded.census,
        graph: &loaded.graph,
        declared: loaded.declared(),
        package: &loaded.lock.package,
        version: &loaded.lock.version,
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
fn gate(root: &Path, read_set: Option<PathBuf>, now: Option<Date>) -> ExitCode {
    let path =
        match read_set {
            Some(path) => path,
            None => return fail(
                "`gate` holds a read set against this tree and takes the file that carries one. \
                 Try `headwater check --read-set run.readset` on one tree, then `headwater gate \
                 --read-set run.readset` on another",
            ),
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
    let asked = match now.or_else(|| Context::from_system_clock().map(|ctx| ctx.now())) {
        Some(asked) => asked,
        None => {
            eprintln!("headwater: this host has no readable clock. Pass `--now <YYYY-MM-DD>`");
            return ExitCode::FAILURE;
        }
    };
    // The lock, and nothing else of the taxonomy. A lock that moved voids every
    // result at once, so its digest is the one component a gate compares that is
    // not a document.
    let lock = match headwater_lock::at(root) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("headwater: {error}");
            return ExitCode::FAILURE;
        }
    };
    let verdict = headwater_check::gate::decide(&recorded, &lock.digest, asked, |listed| {
        std::fs::read(root.join(listed))
            .ok()
            .map(|bytes| headwater_hash::digest(&bytes))
    });
    print!("{}", verdict.render());
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
        true => Cache::at(root, &loaded.lock.digest),
        false => Cache::disabled(),
    };
    let run = headwater_check::run(
        &loaded.census,
        &loaded.graph,
        &loaded.declared(),
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
        eprintln!("headwater: {refusal}");
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
    if composed.is_empty() {
        account.push_str("headwater: no finding of this run carries a patch\n");
    }
    Ok(Fixed {
        account,
        // The seal of a writing MCP server reads this, and so does nothing
        // else. A run that composed no file left the tree as it found it.
        landed: !composed.files.is_empty(),
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
        ctx,
        &mut cache,
    );
    let subject = Subject {
        package: &loaded.lock.package,
        version: &loaded.lock.version,
        lock: &loaded.lock.digest,
        now: &ctx.now().render(),
    };
    let artifact = headwater_adapter::render(&run, &loaded.census, &loaded.graph, &subject, format);
    // The same audit the verb fails a run on. A caller here holds one artifact
    // rather than a terminal, so a finding that reached no output is invisible
    // to it.
    let audited = headwater_adapter::census(&run, &artifact);
    if audited.is_defective() {
        return Err(format!(
            "the {} adapter dropped {} of {} findings with no declared loss reason: {}",
            format.name(),
            audited.unaccounted.len(),
            audited.findings,
            audited.unaccounted.join(", ")
        ));
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
    let ctx = match now.map(Context::at).or_else(Context::from_system_clock) {
        Some(ctx) => ctx,
        None => {
            eprintln!("headwater: this host has no readable clock. Pass `--now <YYYY-MM-DD>`");
            return ExitCode::FAILURE;
        }
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
        lock,
        consumer: _,
        census: taken,
        graph,
        ..
    } = &loaded;

    // Phase B. The cache is keyed on the lock digest among other things, so a
    // taxonomy that moved invalidates every entry without anyone clearing a
    // directory.
    let mut cache = match cached {
        true => Cache::at(root, &lock.digest),
        false => Cache::disabled(),
    };
    let run = headwater_check::run(taken, graph, &loaded.declared(), &ctx, &mut cache);
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
        package: &lock.package,
        version: &lock.version,
        lock: &lock.digest,
        now: &ctx.now().render(),
    };
    let artifact = headwater_adapter::render(&run, taken, graph, &subject, format);
    print!("{artifact}");
    // The census over what was written, in the shape spec 6 fixes for the
    // graph emitters. A finding that reached no output and that no loss
    // reason covers is a defect in the adapter, and it fails the run the
    // way a defective projection census does.
    let audited = headwater_adapter::census(&run, &artifact);
    if audited.is_defective() {
        eprintln!(
            "headwater: the {} adapter dropped {} of {} findings with no declared loss reason:",
            format.name(),
            audited.unaccounted.len(),
            audited.findings
        );
        for missing in &audited.unaccounted {
            eprintln!("  {missing}");
        }
        return ExitCode::FAILURE;
    }

    if let Some(path) = read_set {
        if let Err(error) = std::fs::write(&path, run.read_set.render()) {
            eprintln!("headwater: cannot write {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
    }

    // The register. It is already in the report above, because spec 4 makes it
    // mandatory and inspectable rather than a flag. What the flag adds is a
    // file, and the bytes are the same bytes for the reason the read set's are:
    // a projection a consumer regenerates and one a reader reads are one
    // artifact or they are two truths.
    if let Some(path) = register_out {
        if let Err(error) = std::fs::write(&path, run.register.render()) {
            eprintln!("headwater: cannot write {}: {error}", path.display());
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
/// **The run that computes a payload ignores the payload.** It is the diff
/// between nothing and one, and a run that read the committed block would
/// compute the diff between the debt already declared and one. Re-running would
/// then propose an empty payload, and the second run would report the whole of
/// the debt as new findings.
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
    let ctx = match now.map(Context::at).or_else(Context::from_system_clock) {
        Some(ctx) => ctx,
        None => {
            return fail("the host clock is before 1970, and this engine will not guess a date")
        }
    };
    let until = until.unwrap_or_else(|| ctx.now().plus_days(DEFAULT_WINDOW));
    if until < ctx.now() {
        return fail(&format!(
            "--until {until} is in the past, and a task that has already lapsed accounts for nothing"
        ));
    }

    let run = headwater_check::run(
        &loaded.census,
        &loaded.graph,
        &Declared {
            lock: &loaded.lock.digest,
            taxonomy: &loaded.taxonomy,
            shape: &loaded.shape,
            relations: &loaded.relations,
            config: &loaded.config,
            register: &loaded.register,
            adoption: None,
            source: headwater_lock::LOCK,
        },
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

    let mut payload = String::new();
    payload.push_str("tasks:\n");
    for (index, (rule, held)) in tasks.iter().enumerate() {
        payload.push_str(&format!("  - id: AD-{}\n", index + 1));
        payload.push_str(&format!(
            "    statement: {}\n",
            quoted(&format!(
                "{} {} of {rule}, raised when this taxonomy first reached this corpus",
                held.len(),
                match held.len() {
                    1 => "finding",
                    _ => "findings",
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
        for path in cells {
            payload.push_str(&format!("      - {{path: {path}, rule: {rule}}}\n"));
        }
    }

    let pairs: usize = payload.matches("      - {path: ").count();
    match tasks.is_empty() {
        true => println!("no finding, so no debt to declare"),
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
        println!("\nthis corpus raises no finding against this taxonomy, so it declares no debt");
        if !unexplained.is_empty() {
            println!(
                "  read that with the {} unclassified files above. A taxonomy that classifies \
                 nothing raises nothing",
                unexplained.len()
            );
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

    let block = match headwater_yaml::load(&payload) {
        Ok(node) => match node.value.as_map() {
            Some(map) => map.clone(),
            None => return fail("the payload this run built is not a mapping, which is a defect"),
        },
        Err(errors) => {
            return fail(&format!(
                "the payload this run built does not load: {}",
                headwater_yaml::error::render(&errors)
            ))
        }
    };

    // Written through the resolver, so the lock a payload lands in is the lock
    // the sources produce. A payload written into a stale lock would be debt
    // declared against a taxonomy nobody committed.
    let repository = match headwater_resolve::repository(root) {
        Ok(repository) => repository,
        Err(errors) => {
            eprintln!("headwater: the taxonomy did not resolve, so no payload can be written");
            eprint!("{}", indent(&render_errors(&errors)));
            return ExitCode::FAILURE;
        }
    };
    let sources = match headwater_resolve::package::sources(root, &repository.consumer) {
        Ok(sources) => sources,
        Err(errors) => {
            eprint!("{}", indent(&render_errors(&errors)));
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
            eprint!("{}", indent(&render_errors(&findings)));
            return ExitCode::FAILURE;
        }
    };
    let path = root.join(headwater_lock::LOCK);
    if let Err(error) = std::fs::write(&path, &text) {
        return fail(&format!("cannot write {}: {error}", path.display()));
    }
    println!("\nwrote the payload into {}", headwater_lock::LOCK);
    println!("  {pairs} pairs, owner {owner}, until {until}");
    ExitCode::SUCCESS
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
        return fail(&format!(
            "{} is already there, so this repository is already bound. \
             `headwater infer` is the verb that reads an existing binding",
            headwater_resolve::package::CONSUMER
        ));
    }

    // The corpus root, proposed from the tree. The directory holding the most
    // Markdown, because that is the evidence a tree offers about where its
    // documentation is, and the adopter overrides it with one word.
    let proposed = corpus_root.or_else(|| busiest_directory(root));
    let corpus_root = match proposed {
        Some(root) => root,
        None => {
            return fail(
                "no directory under this repository holds a Markdown file, so nothing here \
                 proposes a corpus root. Pass --corpus <dir> to name one",
            )
        }
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
        None => declaration_text.push_str(
            "  # INTERVIEW: no package of this name is under `packages/`, and nothing in this\n\
             \x20 # engine fetches one. Vendor the package, then pin the version it declares.\n\
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
#     identifier_schemes.doc_id: {{pattern: \"DOC-{{namespace}}-{{slug}}\", namespace: ACME, allocation: minted-once}}
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
            return fail(&format!("cannot create {}: {error}", parent.display()));
        }
    }
    if let Err(error) = std::fs::write(&declaration, &declaration_text) {
        return fail(&format!("cannot write {}: {error}", declaration.display()));
    }
    let overlay = root.join(".headwater/overlay.yml");
    if let Err(error) = std::fs::write(&overlay, &overlay_text) {
        return fail(&format!("cannot write {}: {error}", overlay.display()));
    }

    println!("wrote {}", headwater_resolve::package::CONSUMER);
    println!("wrote .headwater/overlay.yml");
    println!("\nwhat this read off the tree");
    println!("  corpus root {corpus_root}");
    match &found {
        Some(version) => println!("  package {package} {version}, under `packages/`"),
        None => println!(
            "  package {package} is not under `packages/`, and nothing here fetches one. \
             Vendor it before resolving"
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
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
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
    eprintln!("headwater: {what} did not read, so no run is possible");
    for error in errors {
        eprintln!("  {error}");
    }
    ExitCode::FAILURE
}

fn fail(message: &str) -> ExitCode {
    eprintln!("headwater: {message}\n\n{USAGE}");
    ExitCode::FAILURE
}

/// A refusal that is a fact about the corpus rather than a mistyped command.
///
/// [`fail`] prints the grammar, because a caller who wrote the wrong flag is
/// reading it. A caller whose taxonomy declares no shelf for a kind is not, and
/// a hundred lines of grammar under that sentence buries the sentence.
fn refuse(message: &str) -> ExitCode {
    eprintln!("headwater: {message}");
    ExitCode::FAILURE
}
