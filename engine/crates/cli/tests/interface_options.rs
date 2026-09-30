// SPDX-License-Identifier: Apache-2.0
//! The twenty-one interface contracts that hand-state `--no-color`'s meaning,
//! held against the one sentence `headwater-help.md` and `NO_COLOR_TEXT`
//! already carry.
//!
//! # What #476 left true of only one document, and #475 owed to the rest
//!
//! [HW-DR-0045](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
//! rewrites `NO_COLOR_TEXT` (`engine/crates/cli/src/lib.rs`) because the
//! sentence it stated before the ruling — that no run of this binary ever
//! writes color — became false the moment any run does. [#476](https://github.com/headwater-ai/headwater/pull/476)
//! landed that rewrite in the source and in `docs/interfaces/headwater-help.md`
//! alone. Seventeen more documents under `docs/interfaces/` still hand-stated
//! the old sentence in their own Options table, each a hand-kept copy of a
//! fact the binary itself now decides. This file is the second direction
//! [#257](https://github.com/headwater-ai/headwater/issues/257) is about: a
//! table that fell out of step with what it describes.
//!
//! # Why the list grew after that sweep
//!
//! A document joins [`REMAINING`] when the binary keeps the promise its row
//! states, and not before. [#479](https://github.com/headwater-ai/headwater/issues/479)
//! wired `headwater derived`, which is the first verb whose report renders
//! below `headwater-check` and so the first that the palette could not reach
//! while it lived there. `headwater-derived.md` was rewritten to restate the
//! sentence and `headwater-sweep.md` already restated it, so both are held
//! here now. Eleven command lines had a row this file did not read on
//! `d6ac7c93`, and each one that gets wired is a line in this list.
//!
//! # Why the expected sentence is read rather than written twice
//!
//! [`crates/cli/tests/help.rs`](help.rs) states the reason its own cases read
//! `headwater_verbs::VERBS` at run time rather than pinning a string: "a
//! second copy of the verb list is what #257 was filed about." The sentence
//! this file holds every document to is read out of `headwater-help.md`
//! itself for the same reason — the alternative is a second hand-kept copy of
//! prose that could drift from the file it is supposed to match.
//!
//! # Why a `docs/decisions/` record joined a file scoped to `docs/interfaces/`
//!
//! The sweep #475 and #476 ran reached eighteen documents on one shelf and
//! stopped there. [HW-DR-0033](../../../../docs/decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md)
//! stated the same retired sentence in its own prose and was not among them,
//! which is what [#668](https://github.com/headwater-ai/headwater/issues/668)
//! is. So the omitted document is held here rather than in a file of its own:
//! this is the table already built for this one sentence, and a second table
//! asking the same question of a nineteenth document is the shape #257 is
//! about. The three cases at the end of this file are the only ones that read
//! outside `docs/interfaces/`, and none of them uses [`REMAINING`].
//!
//! # Why this is a file of its own rather than a case in `interface_contract.rs`
//!
//! `interface_contract.rs` holds the *kind* `interface_contract` declares —
//! the eight required headings, the `governs` edge, the rule against its
//! negation — against a taxonomy this repository resolves. Every case there
//! runs over a scratch corpus this file writes for the purpose. What is under
//! test here is different: it is the *content* of eighteen real documents
//! already on the shelf, and it reads them from `docs/interfaces/` rather than
//! writing anything. Mixing the two would make one file answer two different
//! questions about two different trees.
//!
//! # Why the flag-set cases at the end joined this file
//!
//! [#1487](https://github.com/headwater-ai/headwater/issues/1487) asked for
//! the second half of HW-OBL-0156: a test that compares the flags the parser
//! admits with the rows of each Options table. The last two cases are that
//! test. They read the same shelf as the cases above and write nothing, so
//! they belong here for the reason the section above gives. They hold the
//! name and the placeholder of each row, and never its words. On the day they
//! landed they found two tables that named a `--format` value the parser does
//! not print: `headwater-capture.md` and `headwater-export.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Every document under `docs/interfaces/` this file holds to the sentence, in
/// the order `docs/interfaces/README.md` lists the verbs.
///
/// Seventeen of them are the documents `--no-color`'s old sentence used to
/// reach, which #475 and #476 swept. `headwater-derived` and `headwater-sweep`
/// joined at #479, when `headwater derived` became the first verb below
/// `headwater-check` to render the palette at all. `headwater-sweep` needed no
/// document change to join: its row already restated the sentence and nothing
/// read it. `headwater-show` joined at birth, at #740: its standard output is
/// a document's own bytes and never painted, and its refusal on standard
/// error is painted the way `explain`'s is. `headwater-site` joined at birth,
/// at #978: its report paints nothing, and its row says so.
///
/// Two documents that carry a `--no-color` row are absent on purpose.
/// `headwater-help.md` is the one this file reads the expectation out of, so
/// holding it to itself would assert nothing. `headwater-json.md` states the
/// opposite of the sentence, and it is right to: `headwater json` writes a
/// machine format that stays plain on every stream, so a row promising that it
/// senses a terminal would be a promise the binary must not keep. An absence
/// here is a claim about a document rather than an oversight, which is why both
/// are named.
const REMAINING: [&str; 21] = [
    "headwater-capture",
    "headwater-check",
    "headwater-completions",
    "headwater-conformance",
    "headwater-derived",
    "headwater-explain",
    "headwater-export",
    "headwater-gate",
    "headwater-generate",
    "headwater-import",
    "headwater-infer",
    "headwater-init",
    "headwater-mcp",
    "headwater-new",
    "headwater-probe",
    "headwater-query",
    "headwater-route",
    "headwater-show",
    "headwater-site",
    "headwater-sweep",
    "headwater-taxonomy",
];

/// Two of the twenty-one, `headwater-probe` and `headwater-query`, name the
/// global flags in one prose sentence rather than in an Options table row —
/// see each document's own Options section. A row-shaped assertion over them
/// would fail on a document that was never wrong about the flag, so this file
/// holds them to a narrower bar: that `--no-banner` now sits beside the
/// global flags they already named, and that the paragraph never claims
/// `--no-color` changes no byte.
const PROSE_ONLY: [&str; 2] = ["headwater-probe", "headwater-query"];

fn interfaces_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/interfaces")
}

fn read(slug: &str) -> String {
    let path = interfaces_dir().join(format!("{slug}.md"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The `--no-color` row of an Options table, as the two cells between its
/// pipes. `None` for a document that states the flag in prose instead — see
/// [`PROSE_ONLY`].
fn no_color_row(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|line| line.strip_prefix("| `--no-color` | "))
        .map(|rest| rest.trim_end_matches(" |"))
}

/// The sentence every rewritten row restates, read out of
/// `headwater-help.md` rather than written a second time here — see the
/// module comment.
fn canonical_sentence() -> String {
    let help = read("headwater-help");
    no_color_row(&help)
        .expect("headwater-help.md carries the --no-color row this file reads the sentence from")
        .to_string()
}

/// Every rewritten Options table restates the sentence `headwater-help.md`
/// carries, word for word, whatever else its own row goes on to say about the
/// verb in front of it.
#[test]
fn every_rewritten_options_table_states_the_sentence_headwater_help_states() {
    let canonical = canonical_sentence();
    let mut wrong = Vec::new();
    for slug in REMAINING.iter().filter(|slug| !PROSE_ONLY.contains(slug)) {
        let text = read(slug);
        match no_color_row(&text) {
            Some(row) if row.contains(&canonical) => {}
            Some(row) => wrong.push(format!("{slug}.md: {row}")),
            None => wrong.push(format!(
                "{slug}.md: no `--no-color` row in its Options table"
            )),
        }
    }
    assert!(
        wrong.is_empty(),
        "these documents do not restate the sentence headwater-help.md carries:\n{}",
        wrong.join("\n")
    );
}

/// The sentence `--no-color`'s row used to carry, refused everywhere: this is
/// the reading the case above catches only when a row is present at all, and
/// this reads the whole document, catching it in a prose sentence too.
#[test]
fn no_document_still_claims_no_run_of_this_binary_ever_writes_color() {
    let retired = [
        "changes no byte",
        "Confirms color-free output",
        "Confirm the binary's color-free output",
        "Disable color output",
        "color-free output",
    ];
    let mut offenders = Vec::new();
    for slug in REMAINING {
        let text = read(slug);
        for phrase in retired {
            if text.contains(phrase) {
                offenders.push(format!("{slug}.md: {phrase:?}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "these documents still state the sentence this decision made false:\n{}",
        offenders.join("\n")
    );
}

/// Every one of the twenty-one gains a `--no-banner` mention: a row beside
/// `--no-color`'s for the eighteen documents that carry an Options table row,
/// and a place in the prose list of global flags for the two that do not.
#[test]
fn every_document_gains_a_no_banner_mention() {
    let mut missing = Vec::new();
    for slug in REMAINING {
        let text = read(slug);
        let carries = match PROSE_ONLY.contains(&slug) {
            true => text.contains("--no-banner"),
            false => text
                .lines()
                .any(|line| line.starts_with("| `--no-banner` |")),
        };
        if !carries {
            missing.push(slug);
        }
    }
    assert!(
        missing.is_empty(),
        "these documents name no `--no-banner`: {missing:?}"
    );
}

/// The two prose-only documents never claim `--no-color` changes no byte —
/// narrower than the row case above because neither one carries a row to
/// begin with, and this is the same refusal in the shape their own Options
/// section takes.
#[test]
fn the_two_prose_only_documents_carry_no_banner_beside_no_color() {
    for slug in PROSE_ONLY {
        let text = read(slug);
        assert!(
            text.contains("`--no-color`") && text.contains("`--no-banner`"),
            "{slug}.md names --no-color without --no-banner beside it"
        );
    }
}

// # HW-DR-0033's own paragraph, the nineteenth document
//
// See the module comment for why it is held here. Both cases below anchor on
// something that exists — the ruling record on disk, and the cases `width.rs`
// declares — rather than on the absence of a phrase alone. A bare "does not
// contain" passes just as well when the paragraph has been renamed away,
// deleted, or looked for at a path that no longer resolves, which is the
// silent-success shape this repository keeps meeting.

/// What HW-DR-0045 reversed, in the spellings HW-DR-0033 used for it.
///
/// One list, read by two cases: the paragraph-scoped one below and the
/// document-wide one after it. The document-wide case exists because the
/// first version of this file scoped its refusal to the `--no-color`
/// paragraph alone, and a second copy of the same falsehood sat 24 lines
/// above it, in the same `status: current` record, and survived. `The parse
/// sets ColorChoice::Never, so no escape sequence reaches either stream` is
/// true of what `clap` renders and false of everything `paint.rs` writes: a
/// pty measures 778 escape bytes on standard output and a pipe measures 0.
/// A phrase belongs here when it states the behavior rather than the
/// mechanism, so `ColorChoice::Never` itself is absent from the list — the
/// parser really does set it.
const REVERSED: [&str; 5] = [
    "changes no byte",
    "writes no color on either stream",
    "reaches either stream",
    "under any terminal",
    "is read by nothing",
];

/// The path of the record this file's last three cases read.
fn hw_dr_0033_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../../docs/decisions/\
         0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md",
    )
}

/// The `--no-color` paragraph of HW-DR-0033, as its one source line.
///
/// Markdown in this repository is never hard-wrapped, so a paragraph is a
/// line. This finds the one that opens in bold on `--no-color`, and panics
/// rather than returning `None`, because a missing paragraph is the failure
/// both cases below exist to report.
fn hw_dr_0033_no_color_paragraph() -> String {
    let path = hw_dr_0033_path();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.lines()
        .find(|line| line.starts_with("**`--no-color`"))
        .unwrap_or_else(|| {
            panic!(
                "{} carries no paragraph opening in bold on `--no-color`",
                path.display()
            )
        })
        .to_string()
}

/// HW-DR-0033's `--no-color` paragraph cites the ruling that moved it, and it
/// no longer states the behavior that ruling reversed.
///
/// The citation is held against the record on disk rather than against a
/// spelling of the identifier: the link must reach a file that declares
/// `id: HW-DR-0045`, so a typo'd path fails here instead of passing quietly.
#[test]
fn hw_dr_0033_cites_the_ruling_that_moved_it_and_drops_the_reversed_claim() {
    let paragraph = hw_dr_0033_no_color_paragraph();

    let link = "0045-coloring-the-cli-and-where-the-banner-goes.md";
    assert!(
        paragraph.contains(link),
        "the paragraph links nothing to the ruling that reversed it:\n{paragraph}"
    );
    let ruling = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/decisions")
        .join(link);
    let ruling = std::fs::read_to_string(&ruling)
        .unwrap_or_else(|e| panic!("the paragraph links {}: {e}", ruling.display()));
    assert!(
        ruling.contains("id: HW-DR-0045"),
        "the link the paragraph carries reaches a document that is not HW-DR-0045"
    );

    let still: Vec<&str> = REVERSED
        .into_iter()
        .filter(|phrase| paragraph.contains(phrase))
        .collect();
    assert!(
        still.is_empty(),
        "the paragraph still states what HW-DR-0045 reversed: {still:?}\n{paragraph}"
    );
}

/// No line of HW-DR-0033 states what HW-DR-0045 reversed, wherever it sits.
///
/// The case above reads one paragraph, which is how the clause at the record's
/// `Color is declared off` heading survived the first pass: it says the same
/// thing about the same behavior, 24 lines higher, in the same `status:
/// current` record. The scope of a refusal has to be the document a reader
/// meets, not the paragraph an issue happened to name.
///
/// It calls [`hw_dr_0033_no_color_paragraph`] first, and that panics on a
/// document with no such paragraph, so this cannot pass by reading a file that
/// has been emptied, renamed away, or moved out from under the path.
#[test]
fn nothing_in_hw_dr_0033_still_states_the_behavior_hw_dr_0045_reversed() {
    let _anchor = hw_dr_0033_no_color_paragraph();

    let path = hw_dr_0033_path();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut offenders = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for phrase in REVERSED {
            if line.contains(phrase) {
                offenders.push(format!("line {}: {phrase:?}", number + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "HW-DR-0033 still states what HW-DR-0045 reversed:\n{}",
        offenders.join("\n")
    );
}

/// The number of `no_escape_byte_*` cases the paragraph credits `width.rs`
/// with is counted out of `width.rs` at run time, and never written here.
///
/// The paragraph said three on a day a fourth had already landed. A number
/// pinned in this file would go stale the same way, one file further out.
#[test]
fn hw_dr_0033_states_the_number_of_escape_byte_cases_width_actually_holds() {
    let width = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/width.rs");
    let text =
        std::fs::read_to_string(&width).unwrap_or_else(|e| panic!("{}: {e}", width.display()));
    let held = text
        .lines()
        .filter(|line| line.starts_with("fn no_escape_byte_"))
        .count();
    let spelled = ["no", "one", "two", "three", "four", "five", "six", "seven"]
        .get(held)
        .copied()
        .unwrap_or_else(|| panic!("{held} cases is past the range this case spells"));

    let paragraph = hw_dr_0033_no_color_paragraph();
    assert!(
        paragraph.contains(&format!("in {spelled} places")),
        "`width.rs` holds {held} `no_escape_byte_*` cases, and the paragraph does not say \
         \"in {spelled} places\":\n{paragraph}"
    );
}

// ---------------------------------------------------------------------------
// The flag set each Options table names, held against the parser
// ---------------------------------------------------------------------------

/// One flag as a caller types it: the long name, and the placeholder for its
/// value where it takes one.
type Flag = (String, Option<String>);

/// Flags keyed by the command line they belong to, `taxonomy publish` for one.
type FlagsByLine = BTreeMap<String, BTreeSet<Flag>>;

/// The placeholder the parser prints after a flag, or `None` for a flag that
/// takes no value.
///
/// Where the parser carries no value name of its own, `clap` writes the
/// argument's identifier in capitals. The placeholder a reader needs then is
/// the list of values the parser accepts, so that list is what a table is held
/// to: `--view concrete|abstract` rather than `--view VIEW`.
fn placeholder(argument: &clap::Arg) -> Option<String> {
    let takes = argument
        .get_num_args()
        .is_some_and(|range| range.takes_values());
    if !takes {
        return None;
    }
    let named = argument
        .get_value_names()
        .and_then(|names| names.first())
        .map(ToString::to_string)
        .unwrap_or_default();
    let defaulted = named
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit());
    if defaulted {
        let values: Vec<String> = argument
            .get_possible_values()
            .iter()
            .map(|value| value.get_name().to_string())
            .collect();
        if !values.is_empty() {
            return Some(values.join("|"));
        }
    }
    Some(named)
}

/// The parser's tree, built, so that every argument carries what `clap` fills
/// in at build time.
fn built_command() -> clap::Command {
    let mut root = headwater_cli::command();
    root.build();
    root
}

/// The long flags that each command line of the parser admits, with the
/// global flags and `--help` left out.
///
/// # Why the global flags are left out
///
/// `--root`, `--no-color`, `--no-banner`, `--wide`, `--version` and `--help`
/// are declared once on the root and admitted on every command line.
/// `headwater-help.md` is the document that states them, and
/// [`the_help_document_names_the_global_flags_the_parser_declares`] holds it
/// to them. A verb's document may restate a global row, and many do. No case
/// here asks it to, and none refuses it.
///
/// The walk is the one `tests/help.rs` makes over the same tree: every
/// argument of every command line, read from `headwater_cli::command()` and
/// never from a list written here.
fn parser_flags() -> FlagsByLine {
    fn walk(line: &str, command: &clap::Command, out: &mut FlagsByLine) {
        let flags = out.entry(line.to_string()).or_default();
        for argument in command.get_arguments() {
            if argument.is_global_set() || argument.is_positional() {
                continue;
            }
            let Some(long) = argument.get_long() else {
                continue;
            };
            if long == "help" {
                continue;
            }
            flags.insert((format!("--{long}"), placeholder(argument)));
        }
        for word in command.get_subcommands() {
            let next = if line.is_empty() {
                word.get_name().to_string()
            } else {
                format!("{line} {}", word.get_name())
            };
            walk(&next, word, out);
        }
    }

    let mut out = BTreeMap::new();
    walk("", &built_command(), &mut out);
    out
}

/// The global flags the root declares, less `--help` and `--version`, which
/// the parser answers before any verb runs. `headwater-help.md` says so in the
/// sentence under its table.
fn global_flags() -> BTreeSet<Flag> {
    built_command()
        .get_arguments()
        .filter(|argument| argument.is_global_set())
        .filter_map(|argument| {
            let long = argument.get_long()?;
            if long == "help" || long == "version" {
                return None;
            }
            Some((format!("--{long}"), placeholder(argument)))
        })
        .collect()
}

/// The text of a document's `## Options` section, up to the next heading of
/// level two.
fn options_section(text: &str) -> String {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("## ") {
            inside = line == "## Options";
            continue;
        }
        if inside {
            lines.push(line);
        }
    }
    lines.join("\n")
}

/// The cells of one table row, split on the pipes a Markdown table reads as
/// separators. A pipe written `\|` is text inside a cell, and it is kept as a
/// bare `|`.
fn cells(row: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cell = String::new();
    let mut chars = row.trim().trim_start_matches('|').chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                cell.push('|');
                chars.next();
            }
            '|' => out.push(std::mem::take(&mut cell).trim().to_string()),
            _ => cell.push(c),
        }
    }
    if !cell.trim().is_empty() {
        out.push(cell.trim().to_string());
    }
    out
}

/// The code spans of one cell, in order.
fn code_spans(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .map(ToString::to_string)
        .collect()
}

/// The flags one code span names, each with the placeholder written after it.
///
/// A span is one flag, `--change <manifest>`, or a sub-word with its flags in
/// brackets, `publish [--package <name>] [--check]`. A token that closes a
/// bracket ends its flag, so `[--check]` takes no placeholder from the token
/// after it.
fn flags_in(span: &str) -> Vec<Flag> {
    let tokens: Vec<&str> = span.split_whitespace().collect();
    let mut out = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let bare = token.trim_start_matches('[');
        if !bare.starts_with("--") {
            continue;
        }
        let closed = bare.ends_with(']');
        let name = bare.trim_end_matches(']').to_string();
        let value = if closed {
            None
        } else {
            tokens
                .get(index + 1)
                .filter(|next| !next.starts_with('[') && !next.starts_with("--"))
                .map(|next| {
                    next.trim_end_matches(']')
                        .trim_start_matches('<')
                        .trim_end_matches('>')
                        .to_string()
                })
        };
        out.push((name, value));
    }
    out
}

/// The flags a document's Options tables name, keyed by the command line each
/// one belongs to.
///
/// A row belongs to the verb unless one of its code spans names a sub-word of
/// the verb, alone (`plan`, in the probe and sweep tables) or as the first word
/// of the span (`publish [--package <name>]`, in the taxonomy table). The last
/// cell of a row is its description and is never read, because a flag named in
/// a sentence there is prose about the row's flag.
fn document_flags(verb: &str, words: &BTreeSet<String>, text: &str) -> FlagsByLine {
    let mut out = FlagsByLine::new();
    for row in options_section(text).lines() {
        if !row.trim_start().starts_with('|') {
            continue;
        }
        let mut row_cells = cells(row);
        row_cells.pop();
        let spans: Vec<String> = row_cells.iter().flat_map(|cell| code_spans(cell)).collect();
        let word = spans.iter().find_map(|span| {
            let first = span.split_whitespace().next()?;
            words.contains(first).then(|| first.to_string())
        });
        let line = match word {
            Some(word) => format!("{verb} {word}"),
            None => verb.to_string(),
        };
        for span in &spans {
            for flag in flags_in(span) {
                out.entry(line.clone()).or_default().insert(flag);
            }
        }
    }
    out
}

/// Each Options table under `docs/interfaces/` names exactly the flags the
/// parser admits on its command line, with the placeholder the parser prints.
///
/// # What this holds, and what it does not
///
/// It holds a row's existence and its placeholder, in both directions: a flag
/// the parser admits with no row fails, and so does a row for a flag the
/// parser refuses. It reads no word of the description column. The table and
/// the `help` string of one flag say the same thing in different words, `check
/// --strict` among them, and an equality of words would fail on a table that
/// was never wrong. HW-OBL-0156 records that the words stay unread.
///
/// # Which documents
///
/// Every `headwater-<verb>.md` whose `<verb>` is a command line of the parser,
/// except `headwater-help.md`, which names the global flags and which the next
/// case holds. A document that states its flags in prose rather than in a
/// table, `headwater-query.md` among them, names no row. It passes only while
/// the parser admits no flag of its own on that verb.
#[test]
fn every_options_table_names_the_flags_the_parser_admits() {
    let parser = parser_flags();
    let globals: BTreeSet<String> = global_flags().into_iter().map(|(name, _)| name).collect();
    let mut wrong = Vec::new();
    let mut read_count = 0;
    for verb in parser
        .keys()
        .filter(|line| !line.is_empty() && !line.contains(' ') && *line != "help")
    {
        let slug = format!("headwater-{verb}");
        let Ok(text) = std::fs::read_to_string(interfaces_dir().join(format!("{slug}.md"))) else {
            continue;
        };
        read_count += 1;
        let prefix = format!("{verb} ");
        let words: BTreeSet<String> = parser
            .keys()
            .filter_map(|line| line.strip_prefix(&prefix))
            .filter(|rest| !rest.contains(' '))
            .map(ToString::to_string)
            .collect();
        let mut named = document_flags(verb, &words, &text);
        for flags in named.values_mut() {
            flags.retain(|(name, _)| !globals.contains(name));
        }
        let lines =
            std::iter::once(verb.clone()).chain(words.iter().map(|word| format!("{verb} {word}")));
        for line in lines {
            let admitted = parser.get(&line).cloned().unwrap_or_default();
            let stated = named.remove(&line).unwrap_or_default();
            for flag in admitted.difference(&stated) {
                wrong.push(format!(
                    "{slug}.md: `headwater {line}` admits {flag:?}, and no row names it"
                ));
            }
            for flag in stated.difference(&admitted) {
                wrong.push(format!(
                    "{slug}.md: a row names {flag:?}, and `headwater {line}` does not admit it"
                ));
            }
        }
    }
    assert!(
        read_count > 20,
        "read {read_count} interface documents, and docs/interfaces/ holds more than twenty"
    );
    assert!(
        wrong.is_empty(),
        "{} Options rows disagree with the parser:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// `headwater-help.md` names the global flags the root declares, with their
/// placeholders, and no other flag.
#[test]
fn the_help_document_names_the_global_flags_the_parser_declares() {
    let text = read("headwater-help");
    let stated: BTreeSet<Flag> = document_flags("help", &BTreeSet::new(), &text)
        .into_values()
        .flatten()
        .collect();
    assert_eq!(
        stated,
        global_flags(),
        "headwater-help.md's Options table and the global flags of the parser differ"
    );
}
