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
                             [--register <path>] [--root <path>]
headwater route              <task description> [--budget <n>] [--root <path>]
headwater explain            <path|identifier> [--root <path>]
headwater mcp                [--root <path>]
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
  --check        `taxonomy resolve` only: write nothing and exit non-zero when
                 the committed lock is not what the sources resolve to.
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
        ["check"] => check(&root, strict, cached, now, read_set, register_out),
        ["route"] => fail("`route` takes a task description. Try `headwater route \"add rate limiting to the ingest API\"`"),
        ["route", task @ ..] => route(&root, &task.join(" "), budget),
        ["explain"] => fail("`explain` takes a path or an identifier"),
        ["explain", target] => explain(&root, target),
        ["mcp"] => mcp(&root),
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
             It carries `check`, `route`, `explain`, `mcp` and `taxonomy`"
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
    let text = match headwater_lock::write(
        &repository.consumer.package,
        &repository.consumer.version,
        &sources,
        &repository.resolution,
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
    census: headwater_census::census::Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    register: Register,
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
    let graph = Graph::build(
        &census,
        &relations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    );
    Ok(Loaded {
        lock,
        census,
        graph,
        shape,
        taxonomy,
        relations,
        register,
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
) -> ExitCode {
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
        census: taken,
        graph,
        shape,
        taxonomy,
        relations: declarations,
        register,
    } = &loaded;

    // Phase B. The cache is keyed on the lock digest among other things, so a
    // taxonomy that moved invalidates every entry without anyone clearing a
    // directory.
    let mut cache = match cached {
        true => Cache::at(root, &lock.digest),
        false => Cache::disabled(),
    };
    let run = headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &lock.digest,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            register: &register,
            source: headwater_lock::LOCK,
        },
        &ctx,
        &mut cache,
    );
    cache.write(root);

    // A run reports the state it evaluated, and the report is not optional
    // (spec 4). The lock hash is half of that statement, and the corpus tree is
    // the other half, which nothing computes yet.
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
