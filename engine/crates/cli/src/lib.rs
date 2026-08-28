// SPDX-License-Identifier: Apache-2.0
//! The command line of `headwater`, declared once and derived.
//!
//! # What this replaced, and the contract that went with it
//!
//! Until [HW-DR-0033](../../../../docs/decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md)
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
//! hand. With the surface here, that test holds [`command`] against
//! [`headwater_verbs::VERBS`] in both directions, over the tree `clap` itself
//! builds.
//!
//! # Where the words come from, and what is deliberately absent
//!
//! No `about` on a verb and no summary is written in this file. [`command`]
//! reads both off [`headwater_verbs::VERBS`] and puts them on the tree, because
//! a summary written here would be the fifth hand-kept copy of the verb list
//! that [#257](https://github.com/headwater-ai/headwater/issues/257) was filed
//! about. The correspondence is by command line rather than by a name repeated
//! at each variant: [`command`] walks the table and calls `mut_subcommand`, so
//! a verb renamed in one place and not the other is a verb the walk in
//! `tests/verbs.rs` reports.
//!
//! **A flag is the other way round, and for the reason `HW-DR-0033` gives.** A
//! flag belongs to the verb that reads it, so its description is written at the
//! declaration of that flag, here. `engine/crates/cli/tests/help.rs` holds every
//! argument of every command in the tree to carrying one, which is what stops
//! the next flag arriving undescribed the way `--facet`, `--tier`, `--arm`,
//! `--category` and `--seed` did.
//!
//! **A `///` comment on a derived item becomes help text.** The commentary on
//! the types below is `//` for that reason, and the module documentation you
//! are reading is `//!`, which `clap` does not read either. A house-style doc
//! comment on a variant or a field would be printed to a caller.
//!
//! Color is declared off. Nothing here emits an escape sequence, which is the
//! state the binary was already in and the state its recorded fixtures read.
//! `--no-color` is declared anyway, and it is declared as what it is: a caller
//! who writes it out of habit is answered rather than refused, and its help
//! says outright that this binary has no color to turn off. That is the
//! opposite of a flag whose name implies an effect it does not have.
//!
//! **The width is decided in [`paint`] and never by the terminal.** Every string
//! below is folded before `clap` sees it, because `clap` cannot fold at all in
//! this workspace and the feature that would let it reads the terminal. So the
//! strings here are written as one long line each and reach a caller folded.

pub mod paint;

/// What `--json` says on a verb that also declares `--format`.
///
/// One constant with four readers rather than four literals that agree until
/// somebody edits one of them. It is not the copy of the verb list that
/// [#257](https://github.com/headwater-ai/headwater/issues/257) rules against:
/// it is one sentence about one flag, and the flag means the same thing at
/// every declaration of it because [`Verb`]'s dispatch maps all four onto the
/// one value `--format json` already named.
///
/// **Stating both is refused and never resolved.** `conflicts_with` is what
/// refuses it, so the refusal is `clap`'s message under this binary's exit 1.
/// The alternative was a precedence rule, and a precedence rule is how a caller
/// states a value and the engine substitutes its own — which is the defect
/// [#337](https://github.com/headwater-ai/headwater/issues/337) and
/// [#338](https://github.com/headwater-ai/headwater/issues/338) are open about.
const JSON_BESIDE_FORMAT: &str = "write this run as one JSON document on standard output. It is \
    the artifact `--format json` writes, byte for byte. A run that states both is refused rather \
    than resolved, because two names for one target is a question answered twice";

/// What `--json` says on a verb that declares no `--format`.
///
/// These four have two renderings and not four, so the flag is a boolean rather
/// than a second `--format` whose closed set would hold two values. `--format`
/// stays where it is on the four verbs that have it, because #321 asks that
/// `--json` be accepted where `--format json` already is and never that it
/// replace anything.
const JSON_ALONE: &str = "write this run as one JSON document on standard output, instead of the \
    report a person reads. The document names its own shape in a `version` member, so a consumer \
    pins that rather than the version of this engine. It moves no exit status";

use clap::{Command, CommandFactory, FromArgMatches, Parser, Subcommand};
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
    #[arg(
        long,
        global = true,
        value_name = "path",
        help = "the repository to read. Defaults to the working directory"
    )]
    pub root: Option<PathBuf>,

    // `-V` and `--version`, held here rather than by `clap`.
    //
    // `clap` prints `{name} {version}`, and this binary prints the version
    // alone: the value is `headwater_resolve::release::ENGINE`, which is what
    // a `requires_engine` range is read against, so a caller pastes one line
    // into a bug report and a reader compares it to a range.
    #[arg(
        short = 'V',
        long,
        global = true,
        help = "the version of this engine. It is the number a package's `requires_engine` range \
                is read against, and it is the number to quote in a bug report. One line on \
                standard output, and no repository is needed to ask"
    )]
    pub version: bool,

    // `--wide`, which is the one reader of `COLUMNS` in this binary.
    //
    // It is declared here so that a caller meets it in the help and so that a
    // verb refuses it in the one place it means nothing. `paint::width` reads
    // the raw arguments for it rather than this field, because the answer is
    // needed to build the tree that produces this field.
    #[arg(
        long,
        global = true,
        help = "lay the help out at the width `COLUMNS` states, held to the range 80 to 120. A \
                reading that is absent or is not a number gives 80, which is what a run with no \
                flag gives. Without it the help is 80 columns wide, nothing reads `COLUMNS`, and \
                a run piped \
                into a file and a run under a terminal write the same bytes. A shell keeps \
                `COLUMNS` to itself, so the form that carries it is `COLUMNS=100 headwater --wide \
                --help`. It lays out the help and nothing else, so a run that prints no help \
                refuses it rather than accepting a flag that would do nothing"
    )]
    pub wide: bool,

    // `--no-color`, which is a flag this binary has nothing to turn off with.
    //
    // Declaring a flag that changes no byte is the defect
    // [#337](https://github.com/headwater-ai/headwater/issues/337) and
    // [#338](https://github.com/headwater-ai/headwater/issues/338) are filed
    // about, and this is the case those two are not: there the name implies a
    // narrowing the code does not perform and the caller is told nothing, and
    // here the help states the whole truth in its first sentence. What it buys
    // is that `headwater check --no-color` runs, where it exited 1 before, and
    // a caller who writes the near-universal spelling meets an answer rather
    // than a refusal about a flag every other tool carries.
    #[arg(
        long = "no-color",
        global = true,
        help = "write no color, which is what every run of this binary already does. Nothing here \
                emits an escape sequence on any stream, in any format, under any terminal or for \
                any value of `NO_COLOR`, so this flag confirms the state rather than changing it. \
                It is declared so that a caller who writes it out of habit is answered rather \
                than refused"
    )]
    pub no_color: bool,

    #[command(subcommand)]
    pub verb: Option<Verb>,
}

// The first word.
//
// The order is [`headwater_verbs::VERBS`]' order, which is the order the first
// screen prints and the order the generated verb index carries.
#[derive(Subcommand, Debug)]
pub enum Verb {
    Check {
        #[arg(
            long,
            help = "exit non-zero when a finding is an error. Without it the run is advisory and \
                    always exits 0, which is the default spec 6 fixes"
        )]
        strict: bool,
        #[arg(
            long,
            help = "write the patch that rides with a finding, in this working tree. A finding \
                    carries one only when the fix is mechanical and total, and a finding an author \
                    suppressed carries none. Every patch is held against the bytes it names and \
                    the result is read back before it lands, so a file whose shape this engine \
                    guessed wrong is refused with nothing written. The report that follows is the \
                    run after the write, and the account of what was written goes to standard \
                    error. It exits non-zero on a refusal"
        )]
        fix: bool,
        #[arg(
            long = "no-cache",
            help = "read and write no cache, and evaluate every instance. This run and a cached \
                    one write the same bytes to standard output, and a difference between them is \
                    a defect in the cache rather than a result"
        )]
        no_cache: bool,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date to evaluate against, as `YYYY-MM-DD`. Defaults to today. Spec 12 \
                    makes the clock an injected value rather than a syscall inside a check, and \
                    this flag is where it is injected: same corpus, same lock, same date, same \
                    bytes"
        )]
        now: Option<Date>,
        #[arg(
            long,
            value_name = "manifest",
            help = "the manifest of the change this run is scoped to. Each line names one document \
                    the change carries, as `added<tab><path>` or `prior<tab><path><tab><file>`, \
                    and the second form names a file holding the bytes that stood before the \
                    change. A document the manifest does not name did not change. It is what a \
                    rule that reads a transition needs, and without it every instance of such a \
                    rule is reported as skipped rather than passed. This engine walks no history: \
                    the caller anchors the prior version to the state on the branch where the \
                    change lands, which spec 12 fixes as the merge base of a proposed change and \
                    the committed `HEAD` of a working-tree hook. Every path is held against the \
                    corpus this run walks, and one that reaches no row of it is counted and named \
                    in the report rather than absorbed. No path is normalized, so `./docs/a.md` \
                    reaches no row. What this engine cannot check is whether the manifest tells \
                    the truth: a line that says `added` for a document that already stood, and a \
                    document the change carried and the manifest omits, are both invisible without \
                    the history that spec 12 rules out as an input"
        )]
        change: Option<PathBuf>,
        #[arg(
            long = "read-set",
            value_name = "path",
            help = "write the read set of this run to a file as well as to the report. The \
                    artifact is what decides whether a verdict survives a merge without running \
                    the checks again, and `headwater gate` is what reads it"
        )]
        read_set: Option<PathBuf>,
        #[arg(
            long,
            value_name = "path",
            help = "write the register of this run to a file as well as to the report. Spec 4 \
                    makes it a projection of the `obligations` and `controls` declarations, \
                    generated and never authored: every obligation with its disposition, every \
                    control with its health, and what escaped under each"
        )]
        register: Option<PathBuf>,
        #[arg(
            long,
            value_name = "text|json|sarif|markdown",
            help = "which vocabulary to write the run in. `text` is the report a person reads and \
                    the default. `sarif` is what a forge ingests as a check run, `markdown` is a \
                    job summary or a review comment, and `json` is the finding shape spec 4 \
                    declares, for an adapter nobody here wrote. Each names what it could not carry"
        )]
        format: Option<String>,
        #[arg(long, conflicts_with = "format", help = JSON_BESIDE_FORMAT)]
        json: bool,
    },
    Gate {
        // Optional here and required by the verb, so that the refusal a caller
        // reads is the one the verb wrote: it names what a read set is and how
        // to produce one, which a missing-argument message cannot.
        #[arg(
            long = "read-set",
            value_name = "path",
            help = "the read set to hold against this tree, and it is required here. \
                    `headwater check --read-set <path>` is what writes one. The artifact is what \
                    decides whether a verdict survives a merge without running the checks again"
        )]
        read_set: Option<PathBuf>,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the day the question is asked about, as `YYYY-MM-DD`. Defaults to today. A run \
                    that read the clock is void on any other day"
        )]
        now: Option<Date>,
        #[arg(long, help = JSON_ALONE)]
        json: bool,
    },
    Conformance {
        #[arg(
            long,
            value_name = "name",
            help = "the rung to ask about, by the name the package declares. It exits non-zero on \
                    a gap under that rung that no live waiver covers. It never moves the level the \
                    report states, which is computed from met rules alone"
        )]
        level: Option<String>,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date to evaluate against, as `YYYY-MM-DD`. Defaults to today"
        )]
        now: Option<Date>,
        #[arg(long, help = JSON_ALONE)]
        json: bool,
    },
    Route {
        #[arg(
            value_name = "task description",
            help = "what you are about to do, in your own words. Every word after the verb is one \
                    description, so it needs no quoting to hold together"
        )]
        task: Vec<String>,
        #[arg(
            long,
            value_name = "n",
            value_parser = a_budget,
            help = "how many ranked pointers it may offer. It never removes a document that \
                    governs a path the task named, and it says how many it withheld. Five by \
                    default"
        )]
        budget: Option<usize>,
        #[arg(long, help = JSON_ALONE)]
        json: bool,
    },
    Explain {
        #[arg(
            value_name = "path|identifier",
            help = "the document to explain, as a path under the corpus root or as the identifier \
                    it declares"
        )]
        target: Option<String>,
        #[arg(long, help = JSON_ALONE)]
        json: bool,
    },
    Query {
        #[arg(
            value_name = "expression",
            help = "the expression to run, and no document of this repository states what one is"
        )]
        expression: Vec<String>,
    },
    Capture {
        #[arg(
            long,
            value_name = "text|json",
            help = "`text` is the report a person reads and the default, and `json` is the same \
                    numbers for a program. Neither carries a reading the store does not hold"
        )]
        format: Option<String>,
        #[arg(long, conflicts_with = "format", help = JSON_BESIDE_FORMAT)]
        json: bool,
    },
    Mcp {
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date to evaluate against, as `YYYY-MM-DD`. Defaults to today. It is read \
                    once and fixed for the life of the server, and every result states it"
        )]
        now: Option<Date>,
        #[arg(
            long,
            help = "register the working-tree write class, which is `new` and `fix`. Spec 5 keeps \
                    it off by default, because a client may connect to a checkout that the user \
                    did not intend to change, so the consent is a word somebody typed rather than \
                    a setting a tree carries. A tool that lands a change is registered by no \
                    switch. The first call that moves a byte ends the server: it walked the corpus \
                    once, so every later answer would be about a tree that is gone"
        )]
        write: bool,
    },
    New {
        #[arg(
            value_name = "kind",
            help = "the kind of document to scaffold, by the name the resolved taxonomy declares \
                    for it"
        )]
        kind: Option<String>,
        #[arg(
            long,
            value_name = "text",
            help = "what the document is called. Required, because the file name and the facet in \
                    the `name` role both come from it"
        )]
        title: Option<String>,
        #[arg(
            long,
            value_name = "relation=identifier",
            value_parser = a_pair,
            help = "an edge to propose, as a relation and the identifier of the document at the \
                    other end. Repeatable. It is refused unless the taxonomy declares \
                    `created_by: scaffold` on the relation, unless both ends are kinds the \
                    relation permits, and unless the target resolves. Where reciprocity is \
                    required the far half is written into the target document"
        )]
        relates: Vec<(String, String)>,
        #[arg(
            long,
            value_name = "facet=value",
            value_parser = a_pair,
            help = "a value for a facet this kind requires, as `<facet>=<value>`. Repeatable. A \
                    facet the kind does not require is refused, a facet in an engine role is \
                    refused because the role decides the value, and a value outside a closed set \
                    is refused with the set printed. The discriminator of a heterogeneous shelf \
                    is the exception: the shelf decides it, and a value stated for it is \
                    overwritten rather than refused"
        )]
        facet: Vec<(String, String)>,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date the document is stamped with, as `YYYY-MM-DD`. Defaults to today"
        )]
        now: Option<Date>,
    },
    Infer {
        #[arg(
            long,
            value_name = "name",
            help = "who owns the debt it proposes. Required with `--write`, because an owner is \
                    the field that ranks declared debt above a suppression and this engine will \
                    not invent one"
        )]
        owner: Option<String>,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the last day the tasks it proposes hold, as `YYYY-MM-DD`. Ninety days out by \
                    default"
        )]
        until: Option<Date>,
        #[arg(
            long,
            help = "put the payload in the lock, which is committed and reviewed. Without it \
                    nothing is written"
        )]
        write: bool,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date to evaluate against, as `YYYY-MM-DD`. Defaults to today"
        )]
        now: Option<Date>,
    },
    Generate {
        #[arg(
            long,
            help = "write nothing and exit non-zero when what is committed is not what a run \
                    produces. It reads the corpus through the lock, so it answers whether a \
                    derived artifact is current"
        )]
        check: bool,
    },
    Import {
        #[arg(
            value_name = "name",
            help = "which declared import to read, by the name its block carries in \
                    `.headwater/taxonomy.yml`. One declared import needs no name and two do, \
                    because choosing for the caller would import whichever the file listed first"
        )]
        name: Option<String>,
        #[arg(
            long,
            value_name = "digest",
            help = "the digest to check the snapshot against. It defaults to the `digest` of the \
                    import block in `.headwater/taxonomy.yml`, and the verb refuses when neither \
                    is there rather than reading an unpinned directory"
        )]
        expect: Option<String>,
        #[arg(
            long,
            help = "write the edge halves into the documents at their near ends. Without it the \
                    edges are reported and nothing is touched"
        )]
        write: bool,
    },
    Export {
        #[arg(
            long,
            value_name = "name",
            help = "which declared export profile to emit. Every declared profile by default, so \
                    a filtered audience is never omitted by accident"
        )]
        profile: Option<String>,
        #[arg(
            long,
            value_name = "json|jsonschema",
            help = "the emitter target. `json` is the native property graph with no loss and \
                    `jsonschema` constrains front matter. The other five targets of spec 6 parse \
                    and report the consumer each one waits on. With this flag the artifact goes to \
                    standard output and no declared output path is touched"
        )]
        format: Option<String>,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date_as_written,
            help = "the generation time the artifact states, as `YYYY-MM-DD`. Absent by default, \
                    because an artifact that `--check` compares by byte cannot carry a clock \
                    reading. Spec 6 asks a filtered export that leaves the repository to state \
                    one, and this is where it is injected"
        )]
        at: Option<String>,
        #[arg(
            long,
            help = "write nothing and exit non-zero when a declared output is not what a run \
                    produces. It holds every export the taxonomy names a path for to regeneration"
        )]
        check: bool,
        #[arg(long, conflicts_with = "format", help = JSON_BESIDE_FORMAT)]
        json: bool,
    },
    Sweep {
        #[command(subcommand)]
        word: Option<SweepWord>,
    },
    Probe {
        #[command(subcommand)]
        word: Option<ProbeWord>,
    },
    Init {
        #[arg(
            long,
            value_name = "dir",
            help = "the corpus root to declare. Proposed from the tree by default"
        )]
        corpus: Option<String>,
        #[arg(
            long,
            value_name = "name",
            help = "the package to take. `headwater/standard` by default"
        )]
        package: Option<String>,
    },
    Taxonomy {
        #[command(subcommand)]
        word: Option<TaxonomyWord>,
    },
    // `headwater help <verb>`, which is a variant here rather than the
    // subcommand `clap` injects during `build()`.
    //
    // The injected one carries a copy of the whole command tree under itself —
    // `headwater help sweep plan` and forty-two more — and the dispatch table
    // carries no such command line, so `tests/verbs.rs` would either fail or
    // need an exclusion written into it. One variant with one positional adds
    // the command line #321 asks for and leaves that walk exact.
    Help {
        #[arg(
            value_name = "verb",
            help = "the verb to describe, with its second word where it takes one: \
                    `headwater help taxonomy diff`. Without one this screen is printed"
        )]
        verb: Vec<String>,
    },
    // `headwater completions <shell>`, whose operand is optional to the parser
    // for the reason every other required operand here is: a bare
    // `headwater completions` names the four shells and says where a script
    // goes, and `clap`'s missing-argument message says neither.
    Completions {
        #[arg(
            value_name = "shell",
            help = "the shell to write a script for. A name outside the four is refused with the \
                    four printed, and no script is written"
        )]
        shell: Option<Shell>,
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

/// The shells `headwater completions` writes a script for.
///
/// # Four, where `clap_complete` offers five
///
/// `clap_complete::Shell` carries `Elvish` as well. It is not here, and the
/// reason is spec 6's own rule about the CLI grammar block: a name that block
/// declares either runs or states its wait. Clause 8 of
/// [#321](https://github.com/headwater-ai/headwater/issues/321) names four
/// shells, the grammar block names the same four, and each of the four is a
/// script this repository has run rather than a name passed through to a
/// generator. A fifth would be a name in the grammar that nothing here has
/// ever executed.
///
/// A name outside the four is refused by `clap` with the four printed, because
/// this is the value parser rather than a match arm underneath one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

impl From<Shell> for clap_complete::Shell {
    fn from(shell: Shell) -> Self {
        match shell {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
            Shell::Powershell => clap_complete::Shell::PowerShell,
        }
    }
}

impl Shell {
    /// The name a caller types, which is the name the refusal prints.
    pub fn typed(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Powershell => "powershell",
        }
    }

    /// The four, in the order a caller meets them in the help.
    pub const ALL: &'static [Shell] = &[Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Powershell];
}

// The second word of `sweep`.
#[derive(Subcommand, Debug)]
pub enum SweepWord {
    Plan {
        #[arg(
            long,
            value_name = "path",
            help = "the slice, as a path prefix under the repository root. The whole corpus by \
                    default. There is no sampling rule here: a slice this engine picked would be \
                    an unreproducible sample dressed as a reproducible one, and the plan reports \
                    its own extent instead"
        )]
        under: Option<String>,
    },
    Report {
        #[arg(
            value_name = "path",
            help = "the file an agent wrote back. `headwater sweep plan` prints the shape of it"
        )]
        path: Option<String>,
        #[arg(
            long,
            value_name = "text|json",
            help = "`text` is the report a person reads and the default, and `json` is the finding \
                    shape spec 4 declares with the provenance and the evidence a sweep adds"
        )]
        format: Option<String>,
        #[arg(long, conflicts_with = "format", help = JSON_BESIDE_FORMAT)]
        json: bool,
    },
    #[command(external_subcommand)]
    Other(Vec<String>),
}

// The second word of `probe`.
#[derive(Subcommand, Debug)]
pub enum ProbeWord {
    Plan {
        #[arg(
            long,
            value_name = "regression|campaign",
            help = "which tier of `.headwater/probe.yml` to plan against. A tier declares the \
                    ceiling, the session cost, the repetitions and the arms, and the plan is \
                    projected against all four. `regression` by default"
        )]
        tier: Option<String>,
        #[arg(
            long,
            value_name = "present|absent",
            help = "narrow the selection to one arm the tier declares. Every arm the tier \
                    declares by default, which is one for `regression` and two for `campaign`. \
                    An arm the tier does not declare narrows nothing: the plan runs the tier's \
                    own arms and prints them under `arms:`"
        )]
        arm: Option<String>,
        #[arg(
            long,
            value_name = "name",
            help = "narrow the selection to one probe category, by the name this engine declares \
                    for it. Every category by default, a name outside the closed set is refused \
                    with the set printed, and a category no probe of this corpus carries is \
                    refused rather than planned as a run of nothing"
        )]
        category: Option<String>,
        // Zero is the default and it is a value like any other. The seed is
        // the caller's, so a run that states none states zero, and a run that
        // repeats a seed repeats a selection.
        //
        // It is a member of the run identity and not an input to the selection.
        // `crates/probe/src/plan.rs` records it and prints it, and the selection
        // is every declared probe, narrowed by category and sorted by
        // identifier. Spec 5 asks for deterministic rotation and this engine
        // implements none, so the help says that rather than implying a draw.
        #[arg(
            long,
            value_name = "n",
            default_value_t = 0,
            help = "the rotation seed, which is a member of the run identity spec 5 declares. It \
                    is the caller's number: a run that states none states zero, and it is \
                    recorded as stated. No selection is drawn from it — every declared probe is \
                    selected — so it identifies a run rather than choosing one"
        )]
        seed: u64,
    },
    Record {
        #[arg(
            value_name = "path",
            help = "the transcript a recorder wrote. `headwater probe plan` prints the run \
                    identity it has to carry"
        )]
        path: Option<String>,
    },
    Grade {
        #[arg(
            value_name = "path",
            help = "the transcript a recorder wrote. It is graded against the probes this corpus \
                    declares, re-derived here rather than taken from the transcript"
        )]
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
        #[arg(
            long,
            help = "write nothing and exit non-zero when what is committed is not what a run \
                    produces. It reads the taxonomy sources, so it answers whether the lock is \
                    current"
        )]
        check: bool,
    },
    Audit {
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date a staleness reading and a dwell reading are taken at, as \
                    `YYYY-MM-DD`. Defaults to today, and two audits of one tree at one date write \
                    the same bytes"
        )]
        now: Option<Date>,
        #[arg(
            long,
            help = "append this run's adoption reading to `.headwater/adoption.jsonl`. Without it \
                    the verb writes nothing. A reading the store already holds at this lock and \
                    this date is not appended twice, so two recorded audits of one tree at one \
                    date still write the same bytes"
        )]
        record: bool,
    },
    Publish {
        #[arg(
            long,
            value_name = "name",
            help = "the package to publish. The one this repository's own declaration takes, by \
                    default, because a publisher usually publishes what it also consumes. Refused \
                    together with `--from`, which names the same thing by its directory instead"
        )]
        package: Option<String>,
        #[arg(
            long,
            value_name = "dir",
            help = "read the manifest at this directory directly, bypassing the lookup by name \
                    under `packages/` that `--package` drives. For a repository that both \
                    publishes a package and consumes it: `taxonomy vendor` refuses to install over \
                    a directory that carries no release record, so a maintained source cannot sit \
                    where its own artifact would be installed. This reads it from wherever it \
                    actually sits instead"
        )]
        from: Option<PathBuf>,
        #[arg(
            long,
            value_name = "dir",
            help = "where to write the artifact. The directory must be empty or absent, because a \
                    published artifact is every file under its root and a stray one would be a \
                    member the publisher never shipped. A run that cannot finish leaves it as it \
                    found it, so a second run meets the same precondition the first one did"
        )]
        out: Option<PathBuf>,
    },
    Vendor {
        #[arg(
            value_name = "dir",
            help = "the directory of an artifact somebody already fetched. This engine opens no \
                    socket, so the verb takes a path and never a location"
        )]
        path: Option<String>,
        #[arg(
            long,
            value_name = "digest",
            help = "the digest to check the artifact against. It defaults to `taxonomy.digest` in \
                    `.headwater/taxonomy.yml`, and the verb refuses when neither is there. A pin \
                    the engine took from the artifact in front of it would be a pin against itself"
        )]
        expect: Option<String>,
    },
    Diff {
        #[arg(
            value_name = "dir",
            help = "the directory of an artifact somebody already fetched. This engine opens no \
                    socket, so the verb takes a path and never a location"
        )]
        path: Option<String>,
        #[arg(
            long,
            value_name = "version",
            help = "the version the artifact is expected to be, written as a version or as a \
                    range: `4.0.0`, or `>=4 <5` with the quoting your shell needs. This engine \
                    fetches nothing, so the directory decides which artifact is compared and this \
                    flag holds it to what the caller meant. It is read by the one range reader \
                    the engine has, which is what reads `requires_engine`"
        )]
        to: Option<String>,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date to evaluate against, as `YYYY-MM-DD`. Defaults to today"
        )]
        now: Option<Date>,
    },
    Migrate {
        #[arg(
            value_name = "dir",
            help = "the directory of an artifact somebody already fetched. This engine opens no \
                    socket, so the verb takes a path and never a location"
        )]
        path: Option<String>,
        #[arg(
            long,
            value_name = "version",
            help = "the version the artifact is expected to be, written as a version or as a \
                    range: `4.0.0`, or `>=4 <5` with the quoting your shell needs"
        )]
        to: Option<String>,
        #[arg(
            long,
            help = "write the files each step names. Without it every file each step would write \
                    is reported and nothing is written"
        )]
        apply: bool,
        #[arg(
            long,
            value_name = "date",
            value_parser = a_date,
            help = "the date to evaluate against, as `YYYY-MM-DD`. Defaults to today"
        )]
        now: Option<Date>,
    },
    #[command(external_subcommand)]
    Other(Vec<String>),
}

/// The command tree this binary parses with, and the one every reader takes.
///
/// [`Cli::command`] is the derived half and carries the grammar alone. This
/// function is what puts the words on it, and every word it puts there comes
/// out of [`headwater_verbs::VERBS`]: the group headings and the one-line
/// summary of the first screen, the long description a verb prints for itself,
/// and the same pair for each second word.
///
/// `main` prints help through this and `tests/verbs.rs` walks it, so a reader
/// of the help and a reader of the test meet the same tree. A caller that used
/// [`Cli::command`] directly would meet a tree with no prose on it at all.
pub fn command() -> Command {
    command_at(paint::width())
}

/// The same tree, laid out at a width the caller states.
///
/// Every string it carries is folded to `width` before `clap` sees it, and
/// `clap` folds nothing, so this number and the strings are the whole of the
/// layout. [`command`] is this at [`paint::WIDTH`] unless the command line
/// carries `--wide`.
pub fn command_at(width: usize) -> Command {
    let mut root = Cli::command()
        .about(format!(
            "{} — {}",
            headwater_verbs::BINARY,
            headwater_verbs::TAGLINE
        ))
        .help_template(first_screen(width));
    for verb in headwater_verbs::VERBS {
        // A name the derive does not carry is skipped rather than added.
        //
        // `Command::mut_subcommand` panics on a name it cannot find, and
        // `Command::subcommand` would put a command in the tree with no variant
        // behind it and nothing to dispatch to. Either one would answer a
        // discrepancy between the table and the parser here, where a caller
        // running `--help` meets it. It is answered in
        // `engine/crates/cli/tests/verbs.rs` instead, which walks this tree
        // against the table in both directions and prints the command lines that
        // are on one side and not the other.
        if root.find_subcommand(verb.name).is_some() {
            root = root.mut_subcommand(verb.name, |one| described(one, verb, width));
        }
    }
    paint::painted(root, width)
}

/// The command line this process was started with, parsed through [`command`].
///
/// `Cli::parse` and `Cli::try_parse` build their own tree out of the derive
/// alone, which carries the grammar and none of the words. A binary that parsed
/// through one tree and printed help out of another would answer `--help` from
/// a command nothing had described, which is the state this returned before the
/// words were put on it. One entry point is what keeps the two the same tree.
pub fn parsed() -> Result<Cli, clap::Error> {
    let matches = command().try_get_matches()?;
    if let Some(message) = a_width_for_a_run_that_lays_nothing_out(&matches) {
        return Err(clap::Error::raw(
            clap::error::ErrorKind::ArgumentConflict,
            message,
        ));
    }
    Cli::from_arg_matches(&matches)
}

/// `--wide` on a run that prints no help, which is a run it would do nothing in.
///
/// # The rule is wider than clause 12 asks, and deliberately
///
/// Clause 12 of [#321](https://github.com/headwater-ai/headwater/issues/321)
/// asks that `--wide` be refused alongside `--format json|sarif|markdown`. The
/// rule here is that it is refused on **every** run that prints no help, and
/// the machine formats are one case of it. The reason is that the flag lays out
/// the help and lays out nothing else: the report of `headwater check` is
/// composed by `headwater_adapter` and is not laid out at any width, so
/// `headwater check --wide --format text` would be as inert as
/// `--format json` and would say so to nobody.
///
/// This repository has two open issues about flags accepted and silently
/// ignored — [#337](https://github.com/headwater-ai/headwater/issues/337) and
/// [#338](https://github.com/headwater-ai/headwater/issues/338) — and a third
/// would have been this one. When a report gains a layout the refusal narrows
/// to the machine formats, which is the clause as written.
///
/// # Why reaching this function is already the test
///
/// `clap` answers `--help` inside `try_get_matches` and returns before this
/// runs, so a run that printed help never arrives here. The one route that
/// prints help and does arrive is `headwater help <verb>`, which is a verb of
/// this binary rather than a flag, and it is the one command the check lets
/// through.
///
/// The `format` value is read off the matches rather than off the parsed
/// `Verb`, so every verb that declares one is named by the same two lines and a
/// verb that gains one later is named without an edit.
fn a_width_for_a_run_that_lays_nothing_out(matches: &clap::ArgMatches) -> Option<String> {
    let mut leaf = matches;
    while let Some((_, inner)) = leaf.subcommand() {
        leaf = inner;
    }
    if leaf.try_get_one::<bool>("wide").ok().flatten() != Some(&true) {
        return None;
    }
    if matches.subcommand_name() == Some("help") {
        return None;
    }
    // `text` is a report a person reads and it is still not laid out at a
    // width, so it falls to the general reason rather than to the machine-format
    // one. The narrower message is for the case clause 12 names.
    let format = leaf.try_get_one::<String>("format").ok().flatten();
    let says = match format.filter(|value| value.as_str() != "text") {
        Some(format) => format!("`--format {format}` writes an artifact that nothing lays out"),
        None => "this run prints no help".to_string(),
    };
    Some(format!(
        "`--wide` says how wide the help is laid out, and {says}. A run carrying it would carry \
         one flag that does nothing, so it is refused rather than run. The runs it widens are \
         `{0} --wide --help`, `{0} <verb> --wide --help` and `{0} --wide help <verb>`",
        headwater_verbs::BINARY
    ))
}

/// One verb of the tree, with the words the table carries for it.
fn described(command: Command, verb: &headwater_verbs::Verb, width: usize) -> Command {
    let mut one = command.about(verb.description);
    if !verb.words.is_empty() {
        one = one.help_template(second_words(verb, width));
        for word in verb.words {
            if one.find_subcommand(word.name).is_none() {
                continue;
            }
            one = one.mut_subcommand(word.name, |inner| inner.about(word.description));
        }
    }
    one
}

/// The column the second field of a printed list starts at.
const COLUMN: usize = 15;

/// The template `headwater --help` renders.
///
/// The literal parts of a `clap` template are written out as they stand, and
/// only the `{…}` tags are rendered, so this is where the layout of the first
/// screen is decided rather than in a `write!` somewhere else. `{subcommands}`
/// is deliberately absent: `clap` renders one flat list and the screen this
/// builds is grouped, and the groups come off
/// [`headwater_verbs::groups`] in the order the table first names each one.
///
/// The examples are the one part of this screen that no earlier version of the
/// binary carried. #321 measured the old help and found no example anywhere in
/// its 25,415 bytes, so these are written rather than recovered, and each one
/// is a command line that runs.
fn first_screen(width: usize) -> String {
    let mut out = String::from("{about}\n\n{usage-heading} {usage}\n\nExamples:\n");
    for (line, says) in [
        (
            "headwater check --strict",
            "run the checks, and fail on an error",
        ),
        (
            "headwater route \"add rate limiting\"",
            "the documents that govern a task",
        ),
        (
            "headwater explain HW-DR-0033",
            "why a document is the kind it is",
        ),
        (
            "headwater new decision --title \"Adopt an overlay\"",
            "scaffold a document of a kind",
        ),
        (
            "headwater help taxonomy diff",
            "the long description of one verb",
        ),
    ] {
        // The command line and what it does are stacked rather than columned.
        // The longest of the five is 48 columns, so a column wide enough to
        // hold it leaves 26 for a description and every one of the five is
        // longer than that. Two lines each is what 80 columns buys.
        out.push_str(&format!("  {line}\n"));
        out.push_str(&paint::fold_indented(says, width, 6));
    }
    for group in headwater_verbs::groups() {
        out.push_str(&format!("\n{group}:\n"));
        for verb in headwater_verbs::VERBS
            .iter()
            .filter(|one| one.group == group)
        {
            out.push_str(&paint::row(verb.name, verb.summary, COLUMN, width));
        }
    }
    out.push_str("\nGlobal flags:\n{options}\n\n");
    out.push_str(&paint::fold_indented(
        &format!(
            "Run `{0} help <verb>` for the long description of one verb, or `{0} <verb> --help`.",
            headwater_verbs::BINARY
        ),
        width,
        0,
    ));
    out
}

/// The template a verb with second words renders.
///
/// The same argument as [`first_screen`]: `clap`'s own subcommand list would
/// print each second word's `about`, which is its long description here, so a
/// caller who typed `headwater sweep` to find out what `plan` is would meet
/// both descriptions in full. This prints the summary the table carries and
/// names where the long one is.
fn second_words(verb: &headwater_verbs::Verb, width: usize) -> String {
    let mut out = String::from("{about}\n\n{usage-heading} {usage}\n\nSecond words:\n");
    for word in verb.words {
        out.push_str(&paint::row(word.name, word.summary, COLUMN, width));
    }
    out.push_str("\nFlags:\n{options}\n\n");
    out.push_str(&paint::fold_indented(
        &format!(
            "Run `{} help {} <word>` for the long description of one.",
            headwater_verbs::BINARY,
            verb.name
        ),
        width,
        0,
    ));
    out
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
    use super::{a_budget, a_date, a_pair, command, headline, Cli};
    use clap::{CommandFactory, Parser};

    #[test]
    fn the_declared_parse_is_a_command_clap_can_build() {
        Cli::command().debug_assert();
        command().debug_assert();
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
