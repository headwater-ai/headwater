// SPDX-License-Identifier: Apache-2.0
//! The command line of `headwater`, declared once and derived.
//!
//! # What this replaced, and the contract that went with it
//!
//! Until [HW-DR-0033](../../../../docs/decisions/0033-adopt-clap-for-the-command-line-and-withdraw-the-flat-flag-namespace.md)
//! the parse was a loop over `std::env::args()` in `main`, with thirty-three
//! arms that read a flag and twenty that read a verb. Every flag of the binary
//! was admitted before the verb was decided, so a flag belonging to another
//! verb was accepted and did nothing: `headwater check --level L0` exited 0 and
//! wrote the report that `headwater check` writes. Two interface contracts and
//! [spec 12](../../../../docs/spec/12-check-layer.md) stated that as a promise.
//! The decision record withdraws it. A flag belongs to the verb that reads it,
//! and a verb refuses a flag it does not.
//!
//! # Why the types are a library and not a module of the binary
//!
//! An integration test cannot reach an item of a `[[bin]]` target, which is why
//! `tests/verbs.rs` used to read `main.rs` as **source text** and scrape the
//! `["word", …]` patterns out of it. A scrape is a parser of Rust that nothing
//! holds, and it would go blind the moment the arms stopped being written by
//! hand. With the surface here, that test holds `Cli::command()` against
//! [`headwater_verbs::VERBS`] in both directions, over the tree `clap` itself
//! builds.
//!
//! # What is deliberately absent
//!
//! No `about`, no `long_about` and no help text on any argument. The one-line
//! summary of a verb is a field on [`headwater_verbs::Verb`] rather than a
//! string here, because a summary written at this layer would be the fifth
//! hand-kept copy of the verb list that
//! [#257](https://github.com/headwater-ai/headwater/issues/257) was filed
//! about. [#321](https://github.com/headwater-ai/headwater/issues/321) carries
//! the help layout, the flag descriptions and the color, and this file carries
//! the parse alone.
//!
//! Color is declared off. Nothing here emits an escape sequence, which is the
//! state the binary was already in and the state its recorded fixtures read.

use clap::{Parser, Subcommand};
use headwater_check::Date;
use std::path::PathBuf;

// Every command line this binary answers to.
//
// The subcommand is optional because `headwater` with no verb has a message of
// its own, and because `--version` answers from outside a corpus with no verb
// in front of it.
#[derive(Parser, Debug)]
#[command(
    name = headwater_verbs::BINARY,
    bin_name = headwater_verbs::BINARY,
    color = clap::ColorChoice::Never,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(long, global = true, value_name = "path")]
    pub root: Option<PathBuf>,

    // `-V` and `--version`, held here rather than by `clap`.
    //
    // `clap` prints `{name} {version}`, and this binary prints the version
    // alone: the value is `headwater_resolve::release::ENGINE`, which is what
    // a `requires_engine` range is read against, so a caller pastes one line
    // into a bug report and a reader compares it to a range.
    #[arg(short = 'V', long, global = true)]
    pub version: bool,

    #[command(subcommand)]
    pub verb: Option<Verb>,
}

// The first word.
//
// The order is [`headwater_verbs::VERBS`]' order, which is the order the help
// prints and the order the generated verb index carries.
#[derive(Subcommand, Debug)]
pub enum Verb {
    Check {
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        fix: bool,
        #[arg(long = "no-cache")]
        no_cache: bool,
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
        #[arg(long, value_name = "manifest")]
        change: Option<PathBuf>,
        #[arg(long = "read-set", value_name = "path")]
        read_set: Option<PathBuf>,
        #[arg(long, value_name = "path")]
        register: Option<PathBuf>,
        #[arg(long, value_name = "text|json|sarif|markdown")]
        format: Option<String>,
    },
    Gate {
        // Optional here and required by the verb, so that the refusal a caller
        // reads is the one the verb wrote: it names what a read set is and how
        // to produce one, which a missing-argument message cannot.
        #[arg(long = "read-set", value_name = "path")]
        read_set: Option<PathBuf>,
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
    },
    Route {
        task: Vec<String>,
        #[arg(long, value_name = "n", value_parser = a_budget)]
        budget: Option<usize>,
    },
    Explain {
        target: Option<String>,
    },
    Mcp {
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
        #[arg(long)]
        write: bool,
    },
    New {
        kind: Option<String>,
        #[arg(long, value_name = "text")]
        title: Option<String>,
        #[arg(long, value_name = "relation=identifier", value_parser = a_pair)]
        relates: Vec<(String, String)>,
        #[arg(long, value_name = "facet=value", value_parser = a_pair)]
        facet: Vec<(String, String)>,
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
    },
    Capture {
        #[arg(long, value_name = "text|json")]
        format: Option<String>,
    },
    Sweep {
        #[command(subcommand)]
        word: Option<SweepWord>,
    },
    Probe {
        #[command(subcommand)]
        word: Option<ProbeWord>,
    },
    Generate {
        #[arg(long)]
        check: bool,
    },
    Import {
        name: Option<String>,
        #[arg(long, value_name = "digest")]
        expect: Option<String>,
        #[arg(long)]
        write: bool,
    },
    Export {
        #[arg(long, value_name = "name")]
        profile: Option<String>,
        #[arg(long, value_name = "json|jsonschema")]
        format: Option<String>,
        #[arg(long, value_name = "date", value_parser = a_date_as_written)]
        at: Option<String>,
        #[arg(long)]
        check: bool,
    },
    Init {
        #[arg(long, value_name = "dir")]
        corpus: Option<String>,
        #[arg(long, value_name = "name")]
        package: Option<String>,
    },
    Infer {
        #[arg(long, value_name = "name")]
        owner: Option<String>,
        #[arg(long, value_name = "date", value_parser = a_date)]
        until: Option<Date>,
        #[arg(long)]
        write: bool,
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
    },
    Conformance {
        #[arg(long, value_name = "name")]
        level: Option<String>,
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
    },
    // Listed in spec 6, and no document states what an expression is. The verb
    // states that wait when a caller types it, which is what #146 requires of
    // a declared name.
    Query {
        expression: Vec<String>,
    },
    Taxonomy {
        #[command(subcommand)]
        word: Option<TaxonomyWord>,
    },
    // A first word this binary does not carry.
    //
    // It reaches the message that names every word it does carry, which is the
    // message this binary printed before the migration and the reason the
    // external form is declared at all: `clap` would otherwise say
    // `unrecognized subcommand` and name at most one near miss.
    #[command(external_subcommand)]
    Other(Vec<String>),
}

// The second word of `sweep`.
#[derive(Subcommand, Debug)]
pub enum SweepWord {
    Plan {
        #[arg(long, value_name = "path")]
        under: Option<String>,
    },
    Report {
        path: Option<String>,
        #[arg(long, value_name = "text|json")]
        format: Option<String>,
    },
    #[command(external_subcommand)]
    Other(Vec<String>),
}

// The second word of `probe`.
#[derive(Subcommand, Debug)]
pub enum ProbeWord {
    Plan {
        #[arg(long, value_name = "regression|campaign")]
        tier: Option<String>,
        #[arg(long, value_name = "present|absent")]
        arm: Option<String>,
        #[arg(long, value_name = "name")]
        category: Option<String>,
        // Zero is the default and it is a value like any other. The seed is
        // the caller's, so a run that states none states zero, and a run that
        // repeats a seed repeats a selection.
        #[arg(long, value_name = "n", default_value_t = 0)]
        seed: u64,
    },
    Record {
        path: Option<String>,
    },
    Grade {
        path: Option<String>,
    },
    Stale,
    #[command(external_subcommand)]
    Other(Vec<String>),
}

// The second word of `taxonomy`.
#[derive(Subcommand, Debug)]
pub enum TaxonomyWord {
    Validate,
    Resolve {
        #[arg(long)]
        check: bool,
    },
    Audit {
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
    },
    Publish {
        #[arg(long, value_name = "name")]
        package: Option<String>,
        #[arg(long, value_name = "dir")]
        out: Option<PathBuf>,
    },
    Vendor {
        path: Option<String>,
        #[arg(long, value_name = "digest")]
        expect: Option<String>,
    },
    Diff {
        path: Option<String>,
        #[arg(long, value_name = "version")]
        to: Option<String>,
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
    },
    Migrate {
        path: Option<String>,
        #[arg(long, value_name = "version")]
        to: Option<String>,
        #[arg(long)]
        apply: bool,
        #[arg(long, value_name = "date", value_parser = a_date)]
        now: Option<Date>,
    },
    #[command(external_subcommand)]
    Other(Vec<String>),
}

/// A date the engine compares against, as `YYYY-MM-DD`.
fn a_date(text: &str) -> Result<Date, String> {
    Date::parse(text).ok_or_else(|| "a date written `YYYY-MM-DD`".to_string())
}

/// The same date, kept as the caller wrote it.
///
/// `export --at` puts the value into an artifact rather than comparing it, so
/// it is checked here and carried on unparsed.
fn a_date_as_written(text: &str) -> Result<String, String> {
    a_date(text).map(|_| text.to_string())
}

/// A pointer budget, which is a count and never zero.
fn a_budget(text: &str) -> Result<usize, String> {
    match text.parse::<usize>() {
        Ok(value) if value > 0 => Ok(value),
        _ => Err("a whole number above zero".to_string()),
    }
}

/// `<left>=<right>`, with neither half empty.
///
/// `--relates supersedes=HW-DR-0007` and `--facet probe_category=discovery` are
/// the two callers, and `clap` prints the flag it was refusing in front of
/// whatever this returns.
fn a_pair(text: &str) -> Result<(String, String), String> {
    match text.split_once('=') {
        Some((left, right)) if !left.is_empty() && !right.is_empty() => {
            Ok((left.to_string(), right.to_string()))
        }
        _ => Err(
            "`<left>=<right>`, as in `--relates supersedes=HW-DR-0007` or \
                  `--facet probe_category=discovery`"
                .to_string(),
        ),
    }
}

/// What a caller reads when `clap` refuses a command line.
///
/// `clap` renders a refusal as the message, then a usage block, then a line
/// telling the caller to try `--help`. Two of those three are the grammar and a
/// pointer to it, and [#306](https://github.com/headwater-ai/headwater/issues/306)
/// already settled what this binary does with both: a refusal names where the
/// grammar is rather than reprinting it, and the pointer names the binary so a
/// caller can paste it. So the message is what is taken here, with any `tip:`
/// line under it, and `fail` supplies the prefix and the pointer.
///
/// The message text is `clap`'s, which is the whole reason for taking the
/// crate: it names the offending word, and it enumerates the legal values of a
/// flag that has a closed set.
pub fn headline(error: &clap::Error) -> String {
    let rendered = error.render().to_string();
    let head: Vec<String> = rendered
        .lines()
        .take_while(|line| !line.starts_with("Usage:") && !line.starts_with("For more information"))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.strip_prefix("error: ").unwrap_or(line).to_string())
        .collect();
    match head.is_empty() {
        true => rendered.split_whitespace().collect::<Vec<_>>().join(" "),
        false => head.join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::{a_budget, a_date, a_pair, headline, Cli};
    use clap::{CommandFactory, Parser};

    #[test]
    fn the_declared_parse_is_a_command_clap_can_build() {
        Cli::command().debug_assert();
    }

    #[test]
    fn a_value_a_flag_cannot_take_is_named_rather_than_defaulted() {
        assert!(a_date("2026-13-45").is_err());
        assert!(a_date("2026-01-01").is_ok());
        assert!(a_budget("0").is_err());
        assert!(a_budget("x").is_err());
        assert_eq!(a_budget("3"), Ok(3));
        assert!(a_pair("nope").is_err());
        assert!(a_pair("=right").is_err());
        assert!(a_pair("left=").is_err());
        assert_eq!(
            a_pair("supersedes=HW-DR-0007"),
            Ok(("supersedes".to_string(), "HW-DR-0007".to_string()))
        );
    }

    /// The refusal a caller reads carries the message and never the usage block.
    ///
    /// The marker is `--root <path>`, which is a line of the help body and of no
    /// refusal. `engine/crates/cli/tests/wiring.rs` holds the same marker over
    /// the running binary; this holds it over the string this function returns,
    /// where a failure names the line rather than a process.
    #[test]
    fn a_refusal_carries_the_message_and_not_the_grammar() {
        let error = Cli::try_parse_from(["headwater", "check", "--nonsense"])
            .expect_err("`--nonsense` is not a flag `check` reads");
        let headline = headline(&error);
        assert!(
            headline.contains("--nonsense"),
            "the refusal names the offending word: {headline}"
        );
        assert!(
            !headline.contains("--root <path>"),
            "the refusal does not reprint the grammar: {headline}"
        );
        assert!(
            !headline.contains("Usage:"),
            "the refusal does not reprint the usage block: {headline}"
        );
        assert!(
            !headline.contains("For more information"),
            "the pointer is `fail`'s and is written once: {headline}"
        );
    }

    /// A flag of another verb is refused rather than accepted and ignored.
    ///
    /// This is the reversal HW-DR-0033 records, at the parse rather than at the
    /// exit status.
    #[test]
    fn a_flag_of_another_verb_does_not_reach_this_one() {
        assert!(Cli::try_parse_from(["headwater", "check", "--level", "L0"]).is_err());
        assert!(Cli::try_parse_from(["headwater", "conformance", "--level", "L0"]).is_ok());
        assert!(Cli::try_parse_from(["headwater", "sweep", "report", "f", "--strict"]).is_err());
        assert!(Cli::try_parse_from(["headwater", "check", "--strict"]).is_ok());
    }

    /// `--root` is the one flag every verb reads, so it is the one flag declared
    /// global. A global flag is a declaration per flag, which is the opposite of
    /// the namespace that admitted all of them everywhere.
    #[test]
    fn the_corpus_flag_reaches_every_verb_from_either_side_of_it() {
        for arguments in [
            ["headwater", "check", "--root", "/tmp"],
            ["headwater", "--root", "/tmp", "check"],
        ] {
            let cli = Cli::try_parse_from(arguments).expect("`--root` is global");
            assert_eq!(cli.root.as_deref(), Some(std::path::Path::new("/tmp")));
        }
    }
}
