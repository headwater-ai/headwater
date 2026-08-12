// SPDX-License-Identifier: Apache-2.0
//! `headwater` — one verb of it.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#cli) lists ten
//! verbs. This binary carries `check`, in the shape
//! [M1](https://github.com/headwater-ai/headwater/milestone/1) needs: run the
//! pipeline over a corpus, print what it found, and exit.
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
//! # Advisory is the default, and it comes from here
//!
//! Spec 6: "The CLI is advisory by default (exit 0 with findings on stdout).
//! Use `--strict` for gates. The default is deliberate: a tool that blocks on
//! first contact is removed, and a removed tool catches nothing." So the
//! workflow that runs this binary passes no flag and needs no `continue-on-error`:
//! the posture is a property of the engine, visible in one place, and a
//! promotion to blocking is one word in a workflow file with an audit trail.
//!
//! # Where the taxonomy comes from, and why that is temporary
//!
//! From [`headwater_census::standin`], which puts the base package, the bundle
//! and the overlay together by applying `add` operations at dotted addresses.
//! It is not a resolver, it validates nothing, and it dies with
//! [#50](https://github.com/headwater-ai/headwater/issues/50). Until then this
//! binary runs in this repository and would need a real resolver to run
//! anywhere else, which is the honest state of M1 rather than a defect in the
//! CLI.

use headwater_census::census::{self, Detail as CensusDetail};
use headwater_census::shelves::Taxonomy;
use headwater_census::standin;
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Detail as GraphDetail, Graph};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
headwater check [--strict] [--root <path>]

  --strict       exit non-zero when a finding is an error. Without it the run
                 is advisory and always exits 0, which is the default spec 6
                 fixes.
  --root <path>  the repository to read. Defaults to the working directory.
";

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let mut strict = false;
    let mut root: Option<PathBuf> = None;
    let mut verb: Option<String> = None;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--strict" => strict = true,
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
            other => verb = Some(other.to_string()),
        }
    }

    match verb.as_deref() {
        Some("check") => {}
        None => return fail("no verb. Try `headwater check`"),
        Some(other) => {
            return fail(&format!(
                "`{other}` is not a verb this binary carries yet. It carries `check`"
            ))
        }
    }

    let root = match root {
        Some(path) => path,
        None => match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => return fail(&format!("no working directory: {error}")),
        },
    };

    check(&root, strict)
}

fn check(root: &Path, strict: bool) -> ExitCode {
    // Phase A. The census fixes the denominator before any check runs, and the
    // graph is built from the census rather than from a second walk.
    let corpus = standin::corpus(root);
    let resolved = standin::resolved(root);
    let taxonomy = match Taxonomy::read(&resolved) {
        Ok(taxonomy) => taxonomy,
        Err(errors) => return refused("the taxonomy", &errors),
    };
    let declarations = match Declarations::read(&resolved) {
        Ok(declarations) => declarations,
        Err(errors) => return refused("the relation declarations", &errors),
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
    let run = headwater_check::run(&taken, &graph, &taxonomy, &declarations);

    println!("census");
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
