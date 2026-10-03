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
//! - spec 7 states each of seven moved rules once, as a whole paragraph under
//!   the subsection that owns it (emitters never chain, no profile presents as
//!   total, the warrant rule, the transcription pin, an exporter fails closed,
//!   a withholding rule never ships advisory, and the claim), and no sentence
//!   of spec 6 states one again, in its words or in a paraphrase the table of
//!   forms names;
//! - a sentence that opens with a link and goes on "states that" is rule
//!   text, and only a short pointer is dropped from the comparison;
//! - no engine comment quotes a sentence of spec 7's export section and
//!   credits it to spec 6;
//! - every current decision that counts the non-claims of a filtered export
//!   counts the bullets that spec 7 lists under **Not claims**.
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

/// The three subsections of spec 7's export section. Each must be inside it,
/// so that a stray `## ` heading cannot take one out of what the tests below
/// read.
const SUBSECTIONS: [&str; 3] = [
    "### An export is a projection, and it declares what it dropped",
    "### An export profile carries a filter",
    "### What a filtered export claims, and what it does not",
];

/// The words that open what a pointer says is stated elsewhere: a noun phrase
/// ("the mechanics of the verb") or a question ("how the verb writes it").
/// "states that ..." is not among them, because what follows "that" is the
/// rule itself.
const POINTER_OBJECTS: [&str; 10] = [
    "the ", "how ", "which ", "what ", "why ", "when ", "where ", "whether ", "each ", "its ",
];

/// The most words a pointer names after "states". A longer run is prose that
/// says something, whatever its first word.
const POINTER_WORDS: usize = 25;

/// A sentence that opens with a link and goes on "states" and a short list of
/// what the linked document holds points at where something is stated, and
/// states no rule itself. Spec 6 is right to carry the same kind of sentence.
///
/// A sentence in the same shape that states a rule is rule text, and it is
/// compared: "[Spec 4](…) states that every emitter reads the lock" says what
/// every emitter does, and so does "[Spec 4](…) states the rule: …". Before
/// #1572's slice 4d any sentence that opened with a link and went on "states "
/// was dropped, so spec 6 could copy a rule in that shape with the suite
/// green.
fn is_pointer(sentence: &str) -> bool {
    let s = sentence.trim();
    if !s.starts_with('[') {
        return false;
    }
    let Some(target) = s.find("](") else {
        return false;
    };
    let Some(close) = s[target..].find(')') else {
        return false;
    };
    let Some(object) = s[target + close + 1..].trim_start().strip_prefix("states ") else {
        return false;
    };
    POINTER_OBJECTS.iter().any(|o| object.starts_with(o))
        && !object.contains(':')
        && object.split_whitespace().count() <= POINTER_WORDS
}

/// The rule text of spec 7's export section: every line from the first
/// subsection on, headings and table separators left out, and each pointer
/// sentence dropped. The opening paragraph above the first subsection says
/// what the section holds and where the verb is stated, and it holds no rule.
fn rule_lines(seven: &str) -> Vec<String> {
    let section = export_section(seven);
    for heading in SUBSECTIONS {
        assert!(
            section.contains(&heading),
            "spec 7's export section does not hold `{heading}`"
        );
    }
    section
        .into_iter()
        .skip_while(|l| *l != SUBSECTIONS[0])
        .filter(|l| !l.starts_with('#') && !l.starts_with("|---"))
        .map(|l| {
            l.split(". ")
                .filter(|s| !is_pointer(s))
                .collect::<Vec<_>>()
                .join(". ")
        })
        .collect()
}

/// Every sentence fragment of the rule text that is long enough to count: a
/// line is cut at each table cell, each sentence end and each colon.
fn fragments(lines: &[String]) -> BTreeSet<String> {
    lines
        .iter()
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

/// Every fragment of the rule text of `seven`'s export section that `six`
/// states again.
fn copied_fragments(six: &str, seven: &str) -> Vec<String> {
    let six = plain(&six.lines().collect::<Vec<_>>().join(" "));
    fragments(&rule_lines(seven))
        .into_iter()
        .filter(|f| six.contains(f.as_str()))
        .collect()
}

#[test]
fn spec_6_shares_no_sentence_with_the_export_section_of_spec_7() {
    let copies = copied_fragments(&spec_six(), &spec_seven());
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

/// Every run of [`RUN_WORDS`] words of the rule text of `seven`'s export
/// section that `six` holds.
fn shared_runs(six: &str, seven: &str) -> BTreeSet<String> {
    let six = format!(" {} ", words(six).join(" "));
    let mut runs = BTreeSet::new();
    // Headings are left out: spec 6 keeps the three export headings on
    // purpose, so that their inbound links land. So are pointer sentences.
    for line in rule_lines(seven) {
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
    runs
}

#[test]
fn spec_6_shares_no_run_of_eight_words_with_the_export_section_of_spec_7() {
    let runs = shared_runs(&spec_six(), &spec_seven());
    assert!(
        runs.is_empty(),
        "spec 6 shares these runs of {RUN_WORDS} words with spec 7's export section: {runs:#?}"
    );
}

/// One way spec 6 could state a moved rule again. A phrase is words in a row.
/// A set of words held together is a paraphrase. "Emitters do not chain."
/// holds "emitter" and "chain", so one sentence of spec 6 that holds every
/// word of the set states the rule in other words.
#[derive(Clone, Copy, Debug)]
enum Form {
    Phrase(&'static str),
    Together(&'static [&'static str]),
}

impl Form {
    /// Whether the plain text of one sentence states this form.
    fn in_sentence(self, sentence: &str) -> bool {
        match self {
            Form::Phrase(p) => sentence.contains(&plain(p)),
            Form::Together(ws) => ws.iter().all(|w| sentence.contains(&plain(w))),
        }
    }
}

/// A moved rule that no table or list holds: the bold sentence that states
/// it, the whole paragraph that it opens, the spec 7 subsection that owns it,
/// and the forms that name it, which spec 6 may not use at all. The paragraph
/// is held whole, so an edit that keeps the bold sentence and changes what the
/// rule says goes red here until the edit is made in this file too.
///
/// No form is a word that a pointer of spec 6 carries. Spec 6 says "why an
/// exporter fails closed" and "states the claim", so neither "fails closed" nor
/// "the claim" names a rule here.
struct Rule {
    rule: &'static str,
    paragraph: &'static str,
    subsection: &'static str,
    forms: &'static [Form],
}

const EMITTERS_NEVER_CHAIN: Rule = Rule {
    rule: "**Emitters never chain.**",
    paragraph: "**Emitters never chain.** Every emitter reads the resolved lock and the graph \
                directly. A pipeline that routes one standard format through another inherits \
                every loss of every hop, and declares none of them. LinkML's own SHACL \
                generator is the observed case, because it drops constructs that LinkML itself \
                expresses ([Q13](09-decisions.md#q13--linkml-and-shacl-as-substrate)).",
    subsection: "### An export is a projection, and it declares what it dropped",
    forms: &[
        Form::Phrase("never chain"),
        Form::Together(&["emitter", "chain"]),
    ],
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
    forms: &[
        Form::Phrase("presents as total"),
        Form::Together(&["view", "complete"]),
        Form::Together(&["filtered", "total"]),
    ],
};

/// G1 of #1572: the warrant rule.
const NO_WARRANT_NO_CONTENT: Rule = Rule {
    rule: "**An emitter that cannot carry the warrant does not carry the content.**",
    paragraph: "**An emitter that cannot carry the warrant does not carry the content.** Every \
                node carries a [warrant](01-conceptual-model.md#warrant), and the native export \
                carries it with no loss. A target vocabulary that has no place for it produces \
                an artifact in which unwarranted content is indistinguishable from accepted \
                content. A loss-set entry reaches the consumer who reads the loss set and \
                nobody else. The field also reports that ordinary tooling strips a mark which \
                travels beside content ([HW-EVAL-adjacent-work \
                §P](../evaluations/adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)). \
                So such an emitter **withholds** every `asserted` and `transcribed` node, at \
                the profile's declared tombstone grain, with its own inability to mark as the \
                reason. That is [principle 7](00-vision-and-scope.md#design-principles) read \
                the way that a filtered exporter reads it. An unmarked assertion is \
                unrecoverable, and a withholding is visible and cheap \
                ([Q15](09-decisions.md#q15--a-synthesized-content-tier)).",
    subsection: "### An export is a projection, and it declares what it dropped",
    forms: &[
        Form::Phrase("cannot carry the warrant"),
        Form::Together(&["emitter", "warrant"]),
    ],
};

/// G2 of #1572: the transcription pin.
const A_TRANSCRIPTION_CARRIES_ITS_PIN: Rule = Rule {
    rule: "**A transcription that leaves carries its pin.**",
    paragraph: "**A transcription that leaves carries its pin.** A `transcribed` node exports \
                the identity of the snapshot that it copies. A consumer who holds the copy can \
                then return to the authority and ask whether it is current. Scholarly \
                publishing solved the same problem in that direction, rather than by a flag \
                that has to survive every copy.",
    subsection: "### An export is a projection, and it declares what it dropped",
    forms: &[
        Form::Phrase("carries its pin"),
        Form::Together(&["transcri", "pin"]),
    ],
};

/// G3 of #1572: an exporter fails closed.
const AN_EXPORTER_FAILS_CLOSED: Rule = Rule {
    rule: "**An exporter fails closed, and that is [principle \
           7](00-vision-and-scope.md#design-principles) read correctly.**",
    paragraph: "**An exporter fails closed, and that is [principle \
                7](00-vision-and-scope.md#design-principles) read correctly.** An exporter \
                that cannot evaluate its filter emits nothing and fails the run. It never \
                emits an unfiltered artifact, and it never emits a partly filtered one. The \
                principle says \"fail open at the edges\", and its own gloss gives the rule \
                underneath: degrade toward the cheaper error. For an agent-facing hint, \
                silence is cheaper than a wrong pointer. For an exporter with a filter, an \
                empty output is cheaper than one document too many.",
    subsection: "### An export profile carries a filter",
    forms: &[
        Form::Phrase("emits nothing and fails the run"),
        Form::Phrase("unfiltered artifact"),
        Form::Together(&["exporter", "cannot evaluate"]),
    ],
};

/// G4 of #1572: a withholding rule is never advisory.
const A_WITHHOLDING_RULE_NEVER_SHIPS_ADVISORY: Rule = Rule {
    rule: "**A withholding rule never ships advisory.**",
    paragraph: "**A withholding rule never ships advisory.** Its two error classes are not \
                both recoverable, so the promotion machinery measures the wrong one \
                ([spec 4](04-assurance-model.md#promotion-advisory-to-blocking)). It is not \
                suppressible and [it is not waivable](#waivers).",
    subsection: "### An export profile carries a filter",
    forms: &[
        Form::Phrase("never ships advisory"),
        Form::Together(&["withholding rule", "advisory"]),
        Form::Together(&["withholding rule", "blocking"]),
    ],
};

/// G5 of #1572: the claim of a filtered export.
const THE_CLAIM: Rule = Rule {
    rule: "**The claim.**",
    paragraph: "**The claim.** A filtered export contains no document that its declared \
                filter withholds, and no artifact inside the profile derives from one.",
    subsection: "### What a filtered export claims, and what it does not",
    forms: &[
        Form::Phrase("no artifact inside the profile derives"),
        Form::Together(&["filtered export", "withh"]),
    ],
};

/// Every rule of spec 7's export section that a one-home test holds.
const RULES: [&Rule; 7] = [
    &EMITTERS_NEVER_CHAIN,
    &NO_VIEW_PRESENTS_AS_TOTAL,
    &NO_WARRANT_NO_CONTENT,
    &A_TRANSCRIPTION_CARRIES_ITS_PIN,
    &AN_EXPORTER_FAILS_CLOSED,
    &A_WITHHOLDING_RULE_NEVER_SHIPS_ADVISORY,
    &THE_CLAIM,
];

/// Every sentence of `text`, in plain text. A line is a paragraph in this
/// corpus, so no sentence runs across two lines, and a heading is a sentence
/// of its own.
fn plain_sentences(text: &str) -> Vec<String> {
    text.lines()
        .map(plain)
        .flat_map(|l| {
            l.split(". ")
                .map(|s| s.trim().to_owned())
                .collect::<Vec<_>>()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// What breaks the one home of `r`, given the text of spec 6 and of spec 7:
/// spec 7 must state the rule once, as one whole paragraph under the
/// subsection that owns it, and no sentence of spec 6 may state any of its
/// forms. Nothing found is an empty list.
fn one_home_findings(r: &Rule, six: &str, seven: &str) -> Vec<String> {
    let mut found = Vec::new();
    if !r.paragraph.starts_with(r.rule) {
        found.push(format!("{:?} does not open its paragraph", r.rule));
    }
    let owned: Vec<&str> = export_section(seven)
        .into_iter()
        .skip_while(|l| *l != r.subsection)
        .skip(1)
        .take_while(|l| !l.starts_with("### "))
        .collect();
    if owned.iter().filter(|l| **l == r.paragraph).count() != 1 {
        found.push(format!(
            "spec 7's `{}` does not state this paragraph once: {:?}",
            r.subsection, r.paragraph
        ));
    }
    if seven.matches(r.rule).count() != 1 {
        found.push(format!("spec 7 does not state {:?} exactly once", r.rule));
    }
    for sentence in plain_sentences(six) {
        for form in r.forms {
            if form.in_sentence(&sentence) {
                found.push(format!(
                    "spec 6 states {form:?} of {:?} again, and spec 7 is its one home: \
                     {sentence:?}",
                    r.rule
                ));
            }
        }
    }
    found
}

#[test]
fn spec_7_alone_states_each_moved_export_rule() {
    let (six, seven) = (spec_six(), spec_seven());
    let found: Vec<String> = RULES
        .iter()
        .flat_map(|r| one_home_findings(r, &six, &seven))
        .collect();
    assert!(found.is_empty(), "{found:#?}");
}

/// The text of `rule`'s whole paragraph, with its line end.
fn paragraph_line(rule: &Rule) -> String {
    format!("{}\n", rule.paragraph)
}

/// Each mutation of spec 6 or spec 7 that the one-home tests must turn red,
/// over the real text with one edit applied. The paraphrases are the bar of
/// #1572's slice 4d: the two named in the issue and one for each of G1 to
/// G5. A paraphrase this table does not hold is advisory, not a defect of
/// the bar.
#[test]
fn each_mutation_of_a_moved_export_rule_turns_a_one_home_test_red() {
    let (six, seven) = (spec_six(), spec_seven());
    let paraphrases: [(&Rule, &str); 7] = [
        (&EMITTERS_NEVER_CHAIN, "Emitters do not chain."),
        (
            &NO_VIEW_PRESENTS_AS_TOTAL,
            "No filtered view may look complete.",
        ),
        (
            &NO_WARRANT_NO_CONTENT,
            "An emitter with no place for the warrant withholds the node.",
        ),
        (
            &A_TRANSCRIPTION_CARRIES_ITS_PIN,
            "A transcribed node that is exported names the pin of its snapshot.",
        ),
        (
            &AN_EXPORTER_FAILS_CLOSED,
            "An exporter that cannot evaluate a filter writes no file at all.",
        ),
        (
            &A_WITHHOLDING_RULE_NEVER_SHIPS_ADVISORY,
            "A withholding rule is blocking from the day it ships.",
        ),
        (
            &THE_CLAIM,
            "A filtered export holds no withheld document and nothing derived from one.",
        ),
    ];
    for (rule, paraphrase) in paraphrases {
        let six = format!("{six}\n{paraphrase}\n");
        assert!(
            !one_home_findings(rule, &six, &seven).is_empty(),
            "spec 6 paraphrases {:?} as {paraphrase:?}, and no one-home test turns red",
            rule.rule
        );
    }
    for rule in RULES {
        let seven = seven.replacen(&paragraph_line(rule), "", 1);
        assert!(
            !one_home_findings(rule, &six, &seven).is_empty(),
            "spec 7 lost {:?}, and no one-home test turns red",
            rule.rule
        );
    }
    // A rule in the shape of a pointer is rule text: copied into spec 6, it
    // turns the fragment test and the run test red.
    let rule_as_pointer = "[Spec 4](04-assurance-model.md) states that an emitter reads \
                           the resolved lock and the graph of the corpus directly.";
    let anchor = paragraph_line(&A_TRANSCRIPTION_CARRIES_ITS_PIN);
    let seven = seven.replacen(&anchor, &format!("{anchor}\n{rule_as_pointer}\n"), 1);
    let six = format!("{six}\n{rule_as_pointer}\n");
    assert!(
        !copied_fragments(&six, &seven).is_empty(),
        "spec 6 copies a rule in pointer shape, and the fragment test stays green"
    );
    assert!(
        !shared_runs(&six, &seven).is_empty(),
        "spec 6 copies a rule in pointer shape, and the run test stays green"
    );
}

/// The pointer sentences of spec 7's export section drop, and a sentence in
/// the same shape that states a rule does not.
#[test]
fn a_pointer_drops_and_a_rule_in_pointer_shape_does_not() {
    let seven = spec_seven();
    let section = export_section(&seven).join("\n");
    let pointers: Vec<&str> = section
        .lines()
        .flat_map(|l| l.split(". "))
        .filter(|s| s.starts_with('[') && s.contains(") states "))
        .collect();
    assert!(
        pointers.len() >= 3,
        "spec 7's export section holds fewer pointer sentences than the three it had: \
         {pointers:#?}"
    );
    for p in &pointers {
        assert!(is_pointer(p), "a pointer of spec 7 does not drop: {p:?}");
    }
    for rule in [
        "[Spec 4](04-assurance-model.md) states that every emitter reads the lock directly",
        "[Spec 4](04-assurance-model.md) states the rule: every emitter reads the lock",
        "[Spec 4](04-assurance-model.md) states the rule that every emitter reads the \
         resolved lock and the graph directly, because a pipeline that routes one format \
         through another inherits every loss of every hop and declares none of them",
    ] {
        assert!(!is_pointer(rule), "a rule in pointer shape drops: {rule:?}");
    }
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

/// The line that opens spec 7's list of what a filtered export does not claim.
const NON_CLAIMS_LEAD: &str = "**Not claims,";

/// The number words a decision may count the non-claims with.
const NUMBER_WORDS: [&str; 10] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
];

/// The top-level list items that follow spec 7's `**Not claims,` lead.
fn non_claims(text: &str) -> Vec<String> {
    let mut lines = text.lines().skip_while(|l| !l.starts_with(NON_CLAIMS_LEAD));
    assert!(
        lines.next().is_some(),
        "spec 7 holds no line that opens {NON_CLAIMS_LEAD:?}"
    );
    lines
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| l.starts_with("- "))
        .map(str::to_owned)
        .collect()
}

/// True when the front matter of `text` says `status: current`.
fn is_current(text: &str) -> bool {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return false;
    }
    lines
        .take_while(|l| *l != "---")
        .any(|l| l.trim() == "status: current")
}

/// Each `<number word> non-claims` in `line`, as the number the word names.
fn stated_counts(line: &str) -> Vec<usize> {
    let lower = line.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|w| !w.is_empty())
        .collect();
    words
        .windows(2)
        .filter(|pair| pair[1] == "non-claims")
        .filter_map(|pair| NUMBER_WORDS.iter().position(|w| *w == pair[0]))
        .map(|i| i + 1)
        .collect()
}

#[test]
fn every_current_decision_that_counts_the_non_claims_of_a_filtered_export_counts_what_spec_7_lists()
{
    let listed = non_claims(&spec_seven()).len();
    assert!(
        listed > 0,
        "spec 7 lists no non-claim after {NON_CLAIMS_LEAD:?}"
    );
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/decisions");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read docs/decisions")
        .map(|e| e.expect("a directory entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    paths.sort();
    let mut found = 0;
    let mut wrong = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).expect("read a decision");
        if !is_current(&text) {
            continue;
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        for (n, line) in text.lines().enumerate() {
            for count in stated_counts(line) {
                found += 1;
                if count != listed {
                    wrong.push(format!("{name}:{} says {count}", n + 1));
                }
            }
        }
    }
    assert!(
        found > 0,
        "no current decision states a count of non-claims, so this test holds nothing"
    );
    assert!(
        wrong.is_empty(),
        "spec 7 lists {listed} non-claims, and these current decisions count otherwise: {wrong:#?}"
    );
}
