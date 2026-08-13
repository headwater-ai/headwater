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
use headwater_census::census::{self, Detail as CensusDetail};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Detail as GraphDetail, Graph};
use headwater_query::{Budget, Surface};
use headwater_resolve::render_errors;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
headwater check              [--strict] [--no-cache] [--now <date>] [--read-set <path>]
                             [--register <path>] [--format text|json|sarif|markdown]
                             [--root <path>]
headwater route              <task description> [--budget <n>] [--root <path>]
headwater explain            <path|identifier> [--root <path>]
headwater mcp                [--root <path>]
headwater generate           [--check] [--root <path>]
headwater export             [--profile <name>] [--format json|jsonschema] [--at <date>]
                             [--check] [--root <path>]
headwater init               [--corpus <dir>] [--package <name>] [--root <path>]
headwater infer              [--owner <name>] [--until <date>] [--write]
                             [--now <date>] [--root <path>]
headwater taxonomy validate  [--root <path>]
headwater taxonomy resolve   [--check] [--root <path>]

  check              run the pipeline over the corpus, against the taxonomy in
                     the committed lock.
  route              resolve a task description to the documents that govern it,
                     as pointers. It is silent when nothing matches.
  explain            why a document is the kind it is, what it serves, and what
                     is consequently required of it.
  mcp                serve the reads above to an agent over the Model Context
                     Protocol, on standard input and output. It registers no
                     tool that writes.
  generate           write every projection the taxonomy declares, and report
                     every one it does not write with the reason. It refuses to
                     overwrite a file that carries no generated-file marker.
  export             emit one declared export profile through one emitter
                     target, with the loss set the target declares and the
                     projection census that holds the output against the graph.
                     With `--format` it writes the artifact to standard output,
                     which is what a consumer outside this repository asks for.
                     Without one it writes every declared export to the path its
                     taxonomy names, and `--check` holds those to regeneration.
  init               scaffold the consumer declaration and the overlay for a
                     repository that has neither, and print the questions that
                     no tree answers. It refuses to overwrite a binding.
  infer              report the debt this taxonomy raises over this corpus as
                     an adoption payload: `(document, rule)` pairs under tasks
                     that each carry an owner and an expiry. It prints the
                     payload and writes nothing without `--write`.
  taxonomy validate  resolve the sources and report every rule of spec 2's
                     list, and what each one did not decide. Writes nothing.
  taxonomy resolve   write `.headwater/taxonomy.lock`. It is written only when
                     the taxonomy validates, so a lock is a validated taxonomy.

  --strict       `check` only: exit non-zero when a finding is an error. Without
                 it the run is advisory and always exits 0, which is the default
                 spec 6 fixes.
  --no-cache     `check` only: read and write no cache, and evaluate every
                 instance. This run and a cached one write the same bytes to
                 standard output, and a difference between them is a defect in
                 the cache rather than a result.
  --now <date>   `check` only: the date to evaluate against, as `YYYY-MM-DD`.
                 Defaults to today. Spec 12 makes the clock an injected value
                 rather than a syscall inside a check, and this flag is where it
                 is injected: same corpus, same lock, same date, same bytes.
  --read-set <path>
                 `check` only: write the read set of this run to a file as well
                 as to the report. It is what a gate compares against a later
                 tree to decide whether this verdict survives a merge, without
                 running the checks again.
  --register <path>
                 `check` only: write the register of this run to a file as well
                 as to the report. Spec 4 makes it a projection of the
                 `obligations` and `controls` declarations, generated and never
                 authored: every obligation with its disposition, every control
                 with its health, and what escaped under each.
  --budget <n>   `route` only: how many pointers it may offer. Five by default.
  --owner <name>
                 `infer` only: who owns the debt it proposes. Required with
                 `--write`, because an owner is the field that ranks declared
                 debt above a suppression and this engine will not invent one.
  --until <date> `infer` only: the last day the tasks it proposes hold, as
                 `YYYY-MM-DD`. Ninety days out by default.
  --write        `infer` only: put the payload in the lock, which is committed
                 and reviewed. Without it nothing is written.
  --corpus <dir> `init` only: the corpus root to declare. Proposed from the
                 tree by default.
  --package <name>
                 `init` only: the package to take. `headwater/standard` by
                 default.
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
  --at <date>    `export` only: the generation time the artifact states, as
                 `YYYY-MM-DD`. Absent by default, because an artifact that
                 `--check` compares by byte cannot carry a clock reading. Spec 6
                 asks a filtered export that leaves the repository to state one,
                 and this is where it is injected.
  --root <path>  the repository to read. Defaults to the working directory.
";

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let mut strict = false;
    let mut check_only = false;
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
    let mut profile: Option<String> = None;
    let mut format: Option<String> = None;
    let mut generated_at: Option<String> = None;
    let mut words: Vec<String> = Vec::new();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--strict" => strict = true,
            "--check" => check_only = true,
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
        ["check"] => check(&root, strict, cached, now, read_set, register_out, format),
        ["route"] => fail("`route` takes a task description. Try `headwater route \"add rate limiting to the ingest API\"`"),
        ["route", task @ ..] => route(&root, &task.join(" "), budget),
        ["explain"] => fail("`explain` takes a path or an identifier"),
        ["explain", target] => explain(&root, target),
        ["mcp"] => mcp(&root),
        ["generate"] => generate(&root, check_only),
        ["export"] => export(&root, profile, format, generated_at, check_only),
        ["init"] => init(&root, corpus_root, package),
        ["infer"] => infer(&root, owner, until, write, now),
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
        ["taxonomy"] => fail("`taxonomy` takes a second word: `validate` or `resolve`"),
        ["taxonomy", other] => fail(&format!(
            "`taxonomy {other}` is not a verb this binary carries yet. \
             It carries `validate` and `resolve`"
        )),
        [] => fail("no verb. Try `headwater check`"),
        [other, ..] => fail(&format!(
            "`{other}` is not a verb this binary carries yet. \
             It carries `check`, `route`, `explain`, `mcp`, `generate`, `export`, \
             `init`, `infer` and `taxonomy`"
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

    let census = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &census,
        &relations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
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
    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
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

/// `headwater mcp`: the reads above, served to an agent.
fn mcp(root: &Path) -> ExitCode {
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    // The corpus is read once, at startup, and the surface answers from it.
    // That is the same posture every other verb takes, and it is what makes two
    // calls in one session answer the same bytes.
    headwater_query::mcp::serve(
        &loaded.surface(),
        std::io::stdin().lock(),
        std::io::stdout().lock(),
    );
    ExitCode::SUCCESS
}

fn check(
    root: &Path,
    strict: bool,
    cached: bool,
    now: Option<Date>,
    read_set: Option<PathBuf>,
    register_out: Option<PathBuf>,
    format: Option<String>,
) -> ExitCode {
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
    let loaded = match load(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded {
        lock,
        consumer: _,
        census: taken,
        graph,
        shape,
        taxonomy,
        relations: declarations,
        register,
        config,
    } = &loaded;

    // Phase B. The cache is keyed on the lock digest among other things, so a
    // taxonomy that moved invalidates every entry without anyone clearing a
    // directory.
    let mut cache = match cached {
        true => Cache::at(root, &lock.digest),
        false => Cache::disabled(),
    };
    let run = headwater_check::run(
        taken,
        graph,
        &Declared {
            lock: &lock.digest,
            taxonomy,
            shape,
            relations: declarations,
            config,
            register,
            adoption: lock.adoption.as_ref(),
            source: headwater_lock::LOCK,
        },
        &ctx,
        &mut cache,
    );
    cache.write(root);

    // A run in a vocabulary that is not the terminal's. Spec 6 lists four
    // formats and `headwater-adapter` writes three of them: the text report
    // below is composed from the census and the graph as well as the run, and
    // the adapters receive neither.
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
    let translated = headwater_adapter::render(&run, &subject, format);
    if let Some(artifact) = &translated {
        print!("{artifact}");
        // The census over what was written, in the shape spec 6 fixes for the
        // graph emitters. A finding that reached no output and that no loss
        // reason covers is a defect in the adapter, and it fails the run the
        // way a defective projection census does.
        let audited = headwater_adapter::census(&run, artifact);
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
    }

    // A run reports the state it evaluated, and the report is not optional
    // (spec 4). The lock hash is half of that statement, and the corpus tree is
    // the other half, which nothing computes yet.
    if translated.is_none() {
        println!("taxonomy");
        println!("  {} {}", lock.package, lock.version);
        println!("  {}", lock.digest);
        // The injected values are part of the state a verdict is about, so a run
        // that does not report them cannot be reproduced from its own output.
        println!("\nclock");
        println!("  {}", ctx.now());
        println!("\ncensus");
        print!("{}", indent(&taken.render(CensusDetail::Exceptions)));
        println!("\ngraph");
        print!("{}", indent(&graph.render(GraphDetail::Exceptions)));
        println!("\nchecks");
        print!("{}", indent(&run.render(headwater_check::Detail::Findings)));

        // The read set, which spec 12 asks a run to report beside its coverage
        // numbers. It is the same bytes `--read-set` writes, so a gate reading the
        // file and a reader of the report are looking at one artifact.
        println!("\nread set");
        print!("{}", indent(&run.read_set.render()));
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
