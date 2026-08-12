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

use headwater_census::census::{self, Detail as CensusDetail};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_graph::anchors::Resolvers;
use headwater_check::Register;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Detail as GraphDetail, Graph};
use headwater_resolve::render_errors;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
headwater check              [--strict] [--root <path>]
headwater taxonomy validate  [--root <path>]
headwater taxonomy resolve   [--check] [--root <path>]

  check              run the pipeline over the corpus, against the taxonomy in
                     the committed lock.
  taxonomy validate  resolve the sources and report every rule of spec 2's
                     list, and what each one did not decide. Writes nothing.
  taxonomy resolve   write `.headwater/taxonomy.lock`. It is written only when
                     the taxonomy validates, so a lock is a validated taxonomy.

  --strict       `check` only: exit non-zero when a finding is an error. Without
                 it the run is advisory and always exits 0, which is the default
                 spec 6 fixes.
  --check        `taxonomy resolve` only: write nothing and exit non-zero when
                 the committed lock is not what the sources resolve to.
  --root <path>  the repository to read. Defaults to the working directory.
";

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let mut strict = false;
    let mut check_only = false;
    let mut root: Option<PathBuf> = None;
    let mut words: Vec<String> = Vec::new();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--strict" => strict = true,
            "--check" => check_only = true,
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
        ["check"] => check(&root, strict),
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
             It carries `check` and `taxonomy`"
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

fn check(root: &Path, strict: bool) -> ExitCode {
    // The taxonomy comes from the lock, and the corpus block comes from the
    // consumer declaration. The two are different questions: the lock says what
    // the schema is, and `corpus:` says what to walk. Spec 6 keeps them apart
    // too, because a run reports "the corpus tree, the taxonomy lock hash" as
    // two facts.
    let lock = match headwater_lock::at(root) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("headwater: {error}");
            return ExitCode::FAILURE;
        }
    };
    let consumer = match headwater_resolve::package::consumer(root) {
        Ok(consumer) => consumer,
        Err(errors) => {
            eprintln!("headwater: the consumer declaration did not read");
            eprint!("{}", indent(&render_errors(&errors)));
            return ExitCode::FAILURE;
        }
    };

    // Phase A. The census fixes the denominator before any check runs, and the
    // graph is built from the census rather than from a second walk.
    let corpus = Corpus::declared(root, &consumer.corpus_root, &consumer.exclusions);
    let resolved = lock.taxonomy;
    let taxonomy = match Taxonomy::read(&resolved) {
        Ok(taxonomy) => taxonomy,
        Err(errors) => return refused("the taxonomy", &errors),
    };
    let declarations = match Declarations::read(&resolved) {
        Ok(declarations) => declarations,
        Err(errors) => return refused("the relation declarations", &errors),
    };
    let register = match Register::read(&resolved) {
        Ok(register) => register,
        Err(errors) => return refused("the obligations and controls", &errors),
    };

    let taken = census::take(&corpus, &taxonomy);
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    );

    // Phase B.
    let run = headwater_check::run(&taken, &graph, &taxonomy, &declarations, &register);

    // A run reports the state it evaluated, and the report is not optional
    // (spec 4). The lock hash is half of that statement, and the corpus tree is
    // the other half, which nothing computes yet.
    println!("taxonomy");
    println!("  {} {}", lock.package, lock.version);
    println!("  {}", lock.digest);
    println!("\ncensus");
    print!("{}", indent(&taken.render(CensusDetail::Exceptions)));
    println!("\ngraph");
    print!("{}", indent(&graph.render(GraphDetail::Exceptions)));
    println!("\nchecks");
    print!("{}", indent(&run.render(headwater_check::Detail::Findings)));

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
