// SPDX-License-Identifier: Apache-2.0
//! Spec 7's export section, held against [`headwater_generate::Grain`], and
//! held as the one home of the export rules.
//!
//! # Why this file exists
//!
//! The export rules used to live in spec 6, and #1572 moved them to spec 7
//! (`docs/spec/07-distribution-and-federation.md#what-leaves-a-corpus`).
//! A rule that two specification parts state drifts in one of them, which is
//! the defect the move removed. So this file holds these things:
//!
//! - the tombstone grain table of spec 7 names exactly the grains the engine
//!   reads, in both directions, and each of its rows is held whole;
//! - spec 6 states neither the grain table nor the filter rule list again,
//!   shares no sentence of six words or more with spec 7's export section,
//!   and shares no run of eight words in a row with it;
//! - spec 7 states the filter rules as a list of exactly six items, each held
//!   whole;
//! - spec 7 states, once and under the subsection that owns each, that
//!   emitters never chain and that no profile presents as total, and spec 6
//!   names neither;
//! - no engine comment quotes a sentence of spec 7's export section and
//!   credits it to spec 6.
//!
//! The private `_exhaustive` match makes a third `Grain` variant fail to
//! compile until [`ENGINE_GRAINS`] and the table change with it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use headwater_generate::Grain;

/// The lead sentence of the filter rule list in spec 7. The list that
/// follows it is the list of the rules.
const FILTER_RULES_LEAD: &str = "rules make the filter honest";

/// The six filter rules, each list item whole and in order. A rule that is
/// inverted, dropped, split or reworded in spec 7 goes red here until the
/// same edit is made in this file.
const FILTER_RULES: [&str; 6] = [
    "- **Carried and withheld partition the corpus, and the engine generates both.** This is \
     the [partition rule](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) that \
     `exportable_as` obeys, applied to documents instead of to checks. Neither list is \
     authored, so neither can drift from the other.",
    "- **A withholding is a loss reason.** The projection census already accounts for every \
     node and edge that the output does not carry. A withheld document is one more accounted \
     absence.",
    "- **A document is withheld whole.** The unit is the document, and no filter reaches \
     inside a body. A redaction inside prose is how a reader ends up with a rectangle drawn \
     over text that is still there.",
    "- **The filter is default-deny over classes.** A node class, an edge class, or an \
     attribute that no profile names does not travel. So a later release that adds a class \
     does not widen a profile that nobody re-read. A filter stated as a list of exclusions \
     grows a hole every time the schema grows.",
    "- **Every projection inside a profile regenerates from the filtered graph.** Take a shelf \
     index, a lineage view, or a navigation file. Built at full visibility and then shipped \
     inside a filtered profile, each one carries what the filter removed. A count, a sort \
     order, or an index of terms is enough. That failure is observed, and it is the one that \
     survives a correct redaction.",
    "- **The declaration travels with the artifact.** A filtered export states that it is \
     filtered, and it states when it was generated. A copy of an artifact carries neither of \
     those unless the artifact does.",
];

/// The header line of the tombstone grain table.
const GRAIN_TABLE_HEADER: &str = "| Grain |";

/// The rows of the tombstone grain table, each whole and in order, so that
/// what each grain tells a reader is held and not only its name.
const GRAIN_ROWS: [&str; 2] = [
    "| `counted` | A placeholder sits where each withheld node or edge would have been, and it \
     carries the identifier of the rule that withheld it | The default. The reader is a tier \
     under a contract, and the existence of the item is not the secret |",
    "| `sealed` | The view is filtered. Nothing else | The existence of the item is itself the \
     disclosure |",
];

/// Every grain the engine reads, in declaration order.
const ENGINE_GRAINS: [Grain; 2] = [Grain::Counted, Grain::Sealed];

/// A third variant fails to compile here until [`ENGINE_GRAINS`] and spec 7's
/// table name it.
fn _exhaustive(g: Grain) {
    match g {
        Grain::Counted | Grain::Sealed => {}
    }
}

fn spec(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/spec")
        .join(name)
}

fn spec_six() -> String {
    std::fs::read_to_string(spec("06-engine-architecture.md")).expect("read spec 6")
}

fn spec_seven() -> String {
    std::fs::read_to_string(spec("07-distribution-and-federation.md")).expect("read spec 7")
}

/// The backticked first-column values of the `| Grain |` table.
fn grain_table(text: &str) -> BTreeSet<String> {
    let mut lines = text
        .lines()
        .skip_while(|l| !l.starts_with(GRAIN_TABLE_HEADER));
    assert!(
        lines.next().is_some(),
        "spec 7 has no `{GRAIN_TABLE_HEADER}` table"
    );
    lines
        .skip(1) // the separator row
        .take_while(|l| l.starts_with('|'))
        .map(|row| {
            let cell = row
                .trim_start_matches('|')
                .split('|')
                .next()
                .unwrap_or("")
                .trim();
            assert!(
                cell.starts_with('`') && cell.ends_with('`') && cell.len() > 2,
                "a grain cell is one backticked name, found {cell:?}"
            );
            cell.trim_matches('`').to_owned()
        })
        .collect()
}

/// The top-level list items that follow the paragraph holding the lead.
fn filter_rules(text: &str) -> Vec<String> {
    let mut lines = text.lines().skip_while(|l| !l.contains(FILTER_RULES_LEAD));
    assert!(
        lines.next().is_some(),
        "spec 7 holds no sentence with {FILTER_RULES_LEAD:?}"
    );
    lines
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| l.starts_with("- "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_tombstone_grain_table_of_spec_7_names_exactly_the_grains_the_engine_reads() {
    let table = grain_table(&spec_seven());
    let engine: BTreeSet<String> = ENGINE_GRAINS
        .map(|g| g.name().to_owned())
        .into_iter()
        .collect();
    assert_eq!(table, engine, "spec 7's grain table and `Grain` disagree");
}

#[test]
fn the_tombstone_grain_rows_of_spec_7_state_what_each_grain_tells_a_reader() {
    let seven = spec_seven();
    let rows: Vec<&str> = seven
        .lines()
        .skip_while(|l| !l.starts_with(GRAIN_TABLE_HEADER))
        .skip(2) // the header and the separator row
        .take_while(|l| l.starts_with('|'))
        .collect();
    assert_eq!(
        rows, GRAIN_ROWS,
        "spec 7's grain rows are not the rows this file holds"
    );
}

/// The first cell of a table row, with its spacing and backticks gone, so
/// that `| \`counted\` |` and `|\`counted\`|` read the same.
fn first_cell(row: &str) -> Option<String> {
    let row = row.trim();
    let rest = row.strip_prefix('|')?;
    let cell = rest.split('|').next()?.trim().trim_matches('`').trim();
    Some(cell.to_lowercase())
}

#[test]
fn spec_6_states_no_grain_table_and_no_filter_rule_list() {
    let six = spec_six();
    let grains: Vec<&str> = ENGINE_GRAINS.iter().map(|g| g.name()).collect();
    for line in six.lines() {
        if let Some(cell) = first_cell(line) {
            assert!(
                !grains.contains(&cell.as_str()),
                "spec 6 states a grain row again: {line}"
            );
            assert!(
                cell != "grain",
                "spec 6 states the grain table again: {line}"
            );
        }
    }
    assert!(
        !six.contains(FILTER_RULES_LEAD),
        "spec 6 states the filter rule list again; spec 7 is its one home"
    );
}

/// The heading of the spec 7 section that holds every export rule.
const EXPORT_SECTION: &str = "## What leaves a corpus";

/// The fewest words a sentence fragment carries before a copy of it counts.
/// Shorter fragments, such as "Nothing else", recur in unrelated prose.
const MIN_FRAGMENT_WORDS: usize = 6;

/// Markdown reduced to the words a reader sees: no emphasis, no backticks,
/// link text kept and link targets dropped, lower case, one space between
/// words.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..i]);
        rest = &rest[i + 2..];
        rest = rest.find(')').map_or("", |j| &rest[j + 1..]);
    }
    out.push_str(rest);
    let out = out.replace(['*', '`', '['], "").to_lowercase();
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The body of spec 7's export section, from its heading to the next `## `.
fn export_section(text: &str) -> Vec<&str> {
    let mut lines = text.lines().skip_while(|l| *l != EXPORT_SECTION);
    assert!(lines.next().is_some(), "spec 7 has no `{EXPORT_SECTION}`");
    lines.take_while(|l| !l.starts_with("## ")).collect()
}

/// Every sentence fragment of the section that is long enough to count: a
/// line is cut at each table cell, each sentence end and each colon.
fn fragments(lines: &[&str]) -> BTreeSet<String> {
    lines
        .iter()
        .filter(|l| !l.starts_with('#') && !l.starts_with("|---"))
        .flat_map(|l| l.split('|'))
        .map(plain)
        .flat_map(|l| {
            l.split(['.', ':', ';'])
                .map(|s| s.trim().trim_start_matches("- ").to_owned())
                .collect::<Vec<_>>()
        })
        .filter(|s| s.split_whitespace().count() >= MIN_FRAGMENT_WORDS)
        .collect()
}

#[test]
fn spec_6_shares_no_sentence_with_the_export_section_of_spec_7() {
    let seven = spec_seven();
    let section = export_section(&seven);
    let six = plain(&spec_six().lines().collect::<Vec<_>>().join(" "));
    let copies: Vec<String> = fragments(&section)
        .into_iter()
        .filter(|f| six.contains(f.as_str()))
        .collect();
    assert!(
        copies.is_empty(),
        "spec 6 states again what spec 7's export section states: {copies:#?}"
    );
}

/// The fewest words in a row that spec 6 may not share with spec 7's export
/// section. A whole-fragment match misses a copy that drops the last clause
/// of a sentence. A run of this many words catches it.
const RUN_WORDS: usize = 8;

/// Plain text cut into words, with punctuation gone.
fn words(text: &str) -> Vec<String> {
    plain(text)
        .split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-' || c == '_'))
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn spec_6_shares_no_run_of_eight_words_with_the_export_section_of_spec_7() {
    let seven = spec_seven();
    let six = format!(" {} ", words(&spec_six()).join(" "));
    let mut runs = BTreeSet::new();
    // Headings are left out: spec 6 keeps the three export headings on
    // purpose, so that their inbound links land.
    for line in export_section(&seven)
        .iter()
        .filter(|l| !l.starts_with('#'))
    {
        for cell in line.split('|') {
            let w = words(cell);
            for run in w.windows(RUN_WORDS) {
                let run = run.join(" ");
                if six.contains(&format!(" {run} ")) {
                    runs.insert(run);
                }
            }
        }
    }
    assert!(
        runs.is_empty(),
        "spec 6 shares these runs of {RUN_WORDS} words with spec 7's export section: {runs:#?}"
    );
}

/// A moved rule that no table or list holds: the bold sentence that states
/// it, the whole paragraph that it opens, the spec 7 subsection that owns it,
/// and the words that name it, which spec 6 may not use at all. The paragraph
/// is held whole, so an edit that keeps the bold sentence and changes what the
/// rule says goes red here until the edit is made in this file too.
struct Rule {
    rule: &'static str,
    paragraph: &'static str,
    subsection: &'static str,
    name: &'static str,
}

const EMITTERS_NEVER_CHAIN: Rule = Rule {
    rule: "**Emitters never chain.**",
    paragraph: "**Emitters never chain.** Every emitter reads the resolved lock and the graph \
                directly. A pipeline that routes one standard format through another inherits \
                every loss of every hop, and declares none of them. LinkML's own SHACL \
                generator is the observed case, because it drops constructs that LinkML itself \
                expresses ([Q13](09-decisions.md#q13--linkml-and-shacl-as-substrate)).",
    subsection: "### An export is a projection, and it declares what it dropped",
    name: "never chain",
};

const NO_VIEW_PRESENTS_AS_TOTAL: Rule = Rule {
    rule: "**No profile may produce a view that presents as total.**",
    paragraph: "**No profile may produce a view that presents as total.** That is the \
                invariant, and it holds under both grains because it leaks nothing. Under \
                `sealed` a reader still knows to stop drawing conclusions from absence, which \
                is the harm that the rule exists to prevent. An agent that traverses a \
                filtered graph, finds nothing, and reports absence is the failure that \
                [spec 5](05-ai-integration.md) names at its start. Here our own filter causes \
                it.",
    subsection: "### An export profile carries a filter",
    name: "presents as total",
};

/// Spec 7 states the rule once, as one whole paragraph under the subsection
/// that owns it. Spec 6 does not name it.
fn has_one_home(r: &Rule) {
    assert!(
        r.paragraph.starts_with(r.rule),
        "{:?} opens its paragraph",
        r.rule
    );
    let seven = spec_seven();
    let owned: Vec<&str> = export_section(&seven)
        .into_iter()
        .skip_while(|l| *l != r.subsection)
        .skip(1)
        .take_while(|l| !l.starts_with("### "))
        .collect();
    assert_eq!(
        owned.iter().filter(|l| **l == r.paragraph).count(),
        1,
        "spec 7's `{}` does not state this paragraph once: {:?}",
        r.subsection,
        r.paragraph
    );
    assert_eq!(
        seven.matches(r.rule).count(),
        1,
        "spec 7 states {:?} more than once",
        r.rule
    );
    let six = plain(&spec_six().lines().collect::<Vec<_>>().join(" "));
    assert!(
        !six.contains(r.name),
        "spec 6 states {:?} again; spec 7 is its one home",
        r.name
    );
}

#[test]
fn spec_7_alone_states_that_emitters_never_chain() {
    has_one_home(&EMITTERS_NEVER_CHAIN);
}

#[test]
fn spec_7_alone_states_that_no_profile_presents_as_total() {
    has_one_home(&NO_VIEW_PRESENTS_AS_TOTAL);
}

fn engine_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read an engine directory") {
        let path = entry.expect("read a directory entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                engine_sources(&path, out);
            }
        } else if name.ends_with(".rs") || name.ends_with(".yml") {
            out.push(path);
        }
    }
}

/// The text inside the first pair of double quotes after `at` in `text`,
/// where the quote opens within 80 bytes of `at`.
fn quote_after(text: &str, at: usize) -> Option<&str> {
    let rest = &text[at..];
    let open = rest.find('"')?;
    if open > 80 {
        return None;
    }
    let body = &rest[open + 1..];
    let close = body.find('"')?;
    Some(&body[..close])
}

/// A comment that quotes a sentence of spec 7's export section and credits it
/// to spec 6 points a reader at a part that no longer holds it. This reads
/// every comment that names spec 6 and then opens a quote, and fails when the
/// quote is text of that section. A quote of text spec 6 keeps is not this
/// test's business.
#[test]
fn no_comment_quotes_spec_7_export_text_as_spec_6() {
    let engine = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let seven = spec_seven();
    let export = plain(&export_section(&seven).join(" "));
    let mut files = Vec::new();
    engine_sources(&engine.join("crates"), &mut files);
    let mut missing = Vec::new();
    let mut quotes = 0;
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read an engine source");
        let rel = file
            .strip_prefix(&engine)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let Some(at) = line.to_lowercase().find("spec 6") else {
                continue;
            };
            // Only comments credit a sentence. A string literal that names
            // spec 6 is a message or a test input.
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("//") || trimmed.starts_with('#')) {
                continue;
            }
            let joined = lines[i..lines.len().min(i + 6)]
                .iter()
                .map(|l| l.trim().trim_start_matches(['/', '!', '#']).trim())
                .collect::<Vec<_>>()
                .join(" ");
            let at = at - (line.len() - line.trim().len()).min(at);
            let at = joined.to_lowercase().find("spec 6").unwrap_or(at);
            let Some(quote) = quote_after(&joined, at) else {
                continue;
            };
            let quote = plain(quote);
            let quote = quote.trim_end_matches(['.', ',']);
            if quote.split_whitespace().count() < 4 {
                continue;
            }
            quotes += 1;
            if export.contains(quote) {
                missing.push(format!("{rel}:{}: {quote:?}", i + 1));
            }
        }
    }
    assert!(quotes > 0, "no engine comment quotes spec 6 at all");
    assert!(
        missing.is_empty(),
        "these lines credit to spec 6 a sentence of spec 7's export section: {missing:#?}"
    );
}

#[test]
fn spec_7_states_six_filter_rules() {
    let rules = filter_rules(&spec_seven());
    assert_eq!(
        rules.len(),
        6,
        "spec 7 states {} filter rules, not six: {rules:#?}",
        rules.len()
    );
    for (i, (found, held)) in rules.iter().zip(FILTER_RULES).enumerate() {
        assert_eq!(
            found,
            held,
            "spec 7 states filter rule {} otherwise than this file holds it",
            i + 1
        );
    }
}
