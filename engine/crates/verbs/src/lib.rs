// SPDX-License-Identifier: Apache-2.0
//! The command surface of this binary, in one list.
//!
//! # Four copies of this list disagreed, and three of them were wrong
//!
//! [#257](https://github.com/headwater-ai/headwater/issues/257) recorded the
//! measurement. The `match verb.as_slice()` arms of the binary were the truth.
//! The message a caller reads on an unknown verb listed fifteen names where the
//! arms carried seventeen, and it omitted `probe` and `query`. The `USAGE`
//! synopsis that `--help` printed omitted `query` and `taxonomy migrate`. Spec 6
//! kept a fourth copy as the CLI grammar.
//!
//! The one that costs the most is `query`. `headwater query "anything"` exits 1
//! and states the wait that [#146](https://github.com/headwater-ai/headwater/issues/146)
//! requires of a declared name. A caller who cannot discover the name never
//! types it, so a refusal that only a reader of the source can reach is a
//! refusal nobody reads.
//!
//! # What this crate changes, and the one property that matters
//!
//! [`VERBS`] is the surface this binary answers to, and it is the copy every
//! other reader takes. `headwater generate` writes the verb index out of it,
//! and every message that enumerates what a caller may type next is written out
//! of it. Nothing in the binary looks a first word up here to decide a verb:
//! `engine/crates/cli/src/lib.rs` declares the parse and `clap` builds a command
//! tree from that declaration, so the tree is what dispatches.
//!
//! `engine/crates/cli/tests/verbs.rs` is what holds the two together, in **both
//! directions**. It walks that command tree to its leaves and compares them with
//! this list: a name here that the parser does not answer to fails, and a
//! command line the parser answers to that is not here fails, and each failure
//! names the command line. A third direction needs no case at all, because
//! `dispatch` in `main.rs` matches the parser's enum exhaustively — a verb in
//! the parser with no arm behind it does not compile. Neither half is a rule of
//! the check layer, and neither one had to be: the surface of a binary is not
//! corpus content
//! ([Q29](../../../../docs/decisions/0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md)).
//!
//! # The words a caller reads are here, and not in the parser
//!
//! [`Verb`] carries a group, a summary and a description, and [`Word`] carries
//! the same three for a second word. `engine/crates/cli/src/lib.rs` reads them
//! onto the command tree and `engine/crates/generate/src/verb_index.rs` renders
//! them into the verb index, so one edit here moves the first screen of
//! `headwater --help`, the long help of one verb, and a committed artifact
//! together.
//!
//! A summary written into the parser instead would be the fifth hand-kept copy
//! of the verb list that #257 was filed about, which is
//! [#321](https://github.com/headwater-ai/headwater/issues/321)'s reason for
//! putting them here. The descriptions are the prose the `USAGE` literal
//! carried until
//! [HW-DR-0033](../../../../docs/decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md)
//! deleted it, recovered from that literal rather than rewritten
//! ([#333](https://github.com/headwater-ai/headwater/issues/333)).
//!
//! # Why it is a crate of its own and not a module of the binary
//!
//! `headwater generate` writes an index of this surface, so `headwater-generate`
//! reads this list. The binary already depends on the generator, so the
//! generator cannot depend on the binary. A crate below both is the only
//! position from which one list serves both, and a dependency the compiler
//! enforces is stronger than a comment asking for an edit in two places.

/// The name a caller types.
///
/// The index writes it in front of every verb, and a contract is matched to a
/// verb by the whole command line rather than by the bare word, because
/// `headwater check` is what the document is called and `check` is not.
pub const BINARY: &str = "headwater";

/// One line under the name, on the first screen of `headwater --help`.
pub const TAGLINE: &str = "a documentation corpus, governed and checked like code";

/// A second word of a grouped verb.
///
/// `headwater sweep plan` and `headwater sweep report` are two command lines
/// and `headwater sweep` is none, so a second word carries its own summary and
/// its own description exactly as a verb does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Word {
    /// The second word, as a caller types it.
    pub name: &'static str,
    /// One line, for the list a group's help prints.
    pub summary: &'static str,
    /// The long form, for `headwater <verb> <word> --help`.
    pub description: &'static str,
}

/// One verb of the command surface.
///
/// `words` holds the second words a verb takes, and it is empty for a verb that
/// takes none. The distinction is the one the parser makes: `headwater sweep`
/// is refused and `headwater sweep plan` runs, so `sweep` names two command
/// lines and no bare one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verb {
    /// The first word, as a caller types it.
    pub name: &'static str,
    /// The heading this verb prints under on the first screen.
    ///
    /// Verbs of one group are contiguous in [`VERBS`], and a case below holds
    /// them there, so the order of the groups is the order they first appear
    /// and no second list of group names exists.
    pub group: &'static str,
    /// One line, for the first screen and for the verb index.
    pub summary: &'static str,
    /// The long form, for `headwater <verb> --help` and `headwater help <verb>`.
    pub description: &'static str,
    /// The second words this verb takes, in the order the parser answers to.
    pub words: &'static [Word],
}

impl Verb {
    /// The command lines this verb answers to, in full.
    ///
    /// One line for a verb with no second word, and one per second word
    /// otherwise. A verb index prints these, and no other list of them exists.
    pub fn forms(&self) -> Vec<String> {
        match self.words.is_empty() {
            true => vec![format!("{BINARY} {}", self.name)],
            false => self
                .words
                .iter()
                .map(|word| format!("{BINARY} {} {}", self.name, word.name))
                .collect(),
        }
    }

    /// What a document that describes this verb is called.
    ///
    /// The whole command line, and never the bare word. A contract of the
    /// corpus carries this string in the facet that plays the `name` role, and
    /// that equality is the whole of the join between a verb and its
    /// description.
    pub fn described_as(&self) -> String {
        format!("{BINARY} {}", self.name)
    }
}

/// Every verb this binary dispatches.
///
/// The order is the order the first screen prints, and a generated index keeps
/// it, so that a reader who runs `--help` and a reader who opens the index meet
/// the same sequence. It is not alphabetical: it is grouped, and the verbs of
/// one group are contiguous.
pub const VERBS: &[Verb] = &[
    Verb {
        name: "check",
        group: "Checking a corpus",
        summary: "run the pipeline over the corpus, against the committed lock",
        description: "Run the pipeline over the corpus, against the taxonomy in the committed lock.",
        words: &[],
    },
    Verb {
        name: "gate",
        group: "Checking a corpus",
        summary: "hold an earlier run's read set against the tree in front of it",
        description: "Hold the read set of an earlier run against the tree in front of it, and report whether the verdicts of that run carry to this one. It reads only what the set lists, so it reports the reach of its own answer and never reports that a corpus is green. It exits non-zero on a verdict that does not carry, which is the signal to run the checks again.",
        words: &[],
    },
    Verb {
        name: "conformance",
        group: "Checking a corpus",
        summary: "evaluate this repository against a package's conformance rules",
        description: "Evaluate this repository against the conformance rules the taxonomy package ships, and report the level that the passing rules reach. A level states what this repository wired up: it measures nothing about the corpus, no key declares one, and a waiver moves the exit status and never the level. A rule this engine holds no reading for ends the run rather than being skipped. Without `--level` it exits 0 whatever it finds.",
        words: &[],
    },
    Verb {
        name: "route",
        group: "Reading a corpus",
        summary: "resolve a task description to the documents that govern it",
        description: "Resolve a task description to the documents that govern it, as pointers. Where nothing matches it offers no pointer and says which of the two reasons applies, so a caller can tell \"no purpose answers this\" from \"the corpus declares none\". It always prints a report and always exits 0.",
        words: &[],
    },
    Verb {
        name: "explain",
        group: "Reading a corpus",
        summary: "why a document is the kind it is, and what it serves",
        description: "Why a document is the kind it is, what it serves, and what is consequently required of it.",
        words: &[],
    },
    // Listed in spec 6, and no document states what an expression is. The verb
    // states that wait when a caller types it, which is what #146 requires of a
    // declared name. It is a verb of this surface for exactly that reason: a
    // wait a caller cannot discover is a wait nobody reads.
    Verb {
        name: "query",
        group: "Reading a corpus",
        summary: "listed in spec 6, and no document says what an expression is",
        description: "Listed in spec 6, and no document states what an expression is, so this engine implements none. It states that wait and exits non-zero. It is here because a wait a caller cannot discover is a wait nobody reads. `route` and `explain` are the reads that exist.",
        words: &[],
    },
    Verb {
        name: "capture",
        group: "Reading a corpus",
        summary: "read the capture-cost store back",
        description: "Read the capture-cost store back: the assisted fraction over every reading it holds, the same by kind, and how far the authoring verb reaches into the corpus. It names no person and no agent. Where more than one taxonomy produced the readings it still reports one fraction over all of them, names every taxonomy that contributed, and says the number is not a trend.",
        words: &[],
    },
    Verb {
        name: "mcp",
        group: "Reading a corpus",
        summary: "serve the reads and one run of the checks to an agent",
        description: "Serve the reads of this binary, and one run of the checks, to an agent over the Model Context Protocol, on standard input and output. It registers spec 5's query class, and with `--write` the working-tree write class beside it. The corpus is walked once before it starts and the clock is read once, so every call answers about the same tree at the same date, and the `check` tool returns the bytes `check --format` returns. A call that moves a byte of that tree ends the server rather than answering from a walk it made stale.",
        words: &[],
    },
    Verb {
        name: "new",
        group: "Writing a corpus",
        summary: "scaffold a document of a kind",
        description: "Scaffold a document of a kind: the placement its shelf dictates, the front matter its facets require, the sections its contract requires, an identifier under its scheme, and the edges the taxonomy assigns to a scaffold. It writes no generated-file marker, because what it writes is an authored document from the moment it lands and every check reads it. It decides everything before it writes anything, and it never overwrites a document. Every run that writes a document appends one capture-cost reading to the store, and a run whose reading did not land exits non-zero.",
        words: &[],
    },
    Verb {
        name: "infer",
        group: "Writing a corpus",
        summary: "report the debt this taxonomy raises over this corpus",
        description: "Report the debt this taxonomy raises over this corpus as an adoption payload: `(document, rule)` pairs under tasks that each carry an owner and an expiry. It prints the payload and writes nothing without `--write`.",
        words: &[],
    },
    Verb {
        name: "generate",
        group: "Writing a corpus",
        summary: "write every projection the taxonomy declares",
        description: "Write every projection the taxonomy declares, and report every one it does not write with the reason. It refuses to overwrite a file that carries no generated-file marker.",
        words: &[],
    },
    Verb {
        name: "import",
        group: "Writing a corpus",
        summary: "write the edges a committed snapshot declares",
        description: "Read a snapshot that somebody already fetched and committed, and write the edges it declares into the documents at their near ends. The snapshot is checked against a digest and a channel that a person wrote into `.headwater/taxonomy.yml`, and an import with neither is refused rather than recorded. Without `--write` it reports the edges and touches nothing. A wrong imported edge would produce a correct check result over a wrong graph, so every link is refused whole rather than reported as a finding.",
        words: &[],
    },
    Verb {
        name: "export",
        group: "Writing a corpus",
        summary: "emit a declared export profile through an emitter target",
        description: "Emit one declared export profile through one emitter target, with the loss set the target declares and the projection census that holds the output against the graph. With `--format` it writes the artifact to standard output, which is what a consumer outside this repository asks for. Without one it writes every declared export to the path its taxonomy names, and `--check` holds those to regeneration.",
        words: &[],
    },
    Verb {
        name: "sweep",
        group: "Sampling, which never gates",
        summary: "the two halves of the coherence sweep",
        description: "The two deterministic halves of the coherence sweep, which is a sampler and never a check. No model is reached from this binary, both halves exit 0 whatever they find, and neither writes a byte of the corpus.",
        words: &[
            Word {
                name: "plan",
                summary: "write the briefing an agent reads",
                description: "Write the briefing an agent reads: the slice, what the graph already declares about it, and the file to write back.",
            },
            Word {
                name: "report",
                summary: "read the file an agent wrote back, and say what holds",
                description: "Read the file an agent wrote back and say what this engine could confirm about it — that every quotation is in the document it names, that every path is a classified document, and that no proposed edge is one the graph already carries.",
            },
        ],
    },
    Verb {
        name: "probe",
        group: "Sampling, which never gates",
        summary: "the four parts of the probe harness",
        description: "The four deterministic parts of the probe harness, which is a sampler and never a check. Nothing here reaches a model and nothing here writes a transcript. A transcript that an agent wrote about its own session is a self-report, which spec 5 refuses, so the recorder observes a session from outside it and is not in this repository.",
        words: &[
            Word {
                name: "plan",
                summary: "fix the run identity, and project against a ceiling",
                description: "Fix the six members of the run identity that exist before a run, and project the sessions against the ceiling that `.headwater/probe.yml` declares for the tier. It refuses a run above it, and it refuses one whose probes name an oracle this engine does not carry or a predicate over no document.",
            },
            Word {
                name: "record",
                summary: "read a transcript a recorder wrote, and confirm its shape",
                description: "Read a transcript that a recorder wrote and confirm the taxonomy, the completeness of the run identity, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. It exits 0 whatever it finds.",
            },
            Word {
                name: "grade",
                summary: "the one part of this engine that returns a verdict",
                description: "The one component of this engine that returns a verdict, and three properties are why it may: its inputs carry no prose, every satisfied verdict names the event that satisfied it, and six conditions return no verdict where a green one would be free. It reports a rate over the sessions that reached a verdict, with the count that did not beside it.",
            },
            Word {
                name: "stale",
                summary: "which recorded results a change voided",
                description: "Hold the read set of every committed transcript against the tree in front of it and report which recorded results a change voided. A read set is the probes of the selection, the documents they examine and the documents a session was observed to open, so an edit anywhere else voids nothing. It exits 0 on every answer, because a result going stale is a fact about a measurement rather than a status a build reads.",
            },
        ],
    },
    Verb {
        name: "init",
        group: "The taxonomy",
        summary: "scaffold the consumer declaration and the overlay",
        description: "Scaffold the consumer declaration and the overlay for a repository that has neither, and print the questions that no tree answers. It refuses to overwrite a binding.",
        words: &[],
    },
    Verb {
        name: "taxonomy",
        group: "The taxonomy",
        summary: "read, write, publish and compare a taxonomy package",
        description: "Read, write, publish and compare the taxonomy this repository takes. Everything downstream reads `.headwater/taxonomy.lock` and never the sources, so `resolve` is what carries a change to the sources into a run.",
        words: &[
            Word {
                name: "validate",
                summary: "resolve the sources and report every rule of spec 2",
                description: "Resolve the sources and report every rule of spec 2's list, and what each one did not decide. Writes nothing.",
            },
            Word {
                name: "resolve",
                summary: "write `.headwater/taxonomy.lock`",
                description: "Write `.headwater/taxonomy.lock`. It is written only when the taxonomy validates, so a lock is a validated taxonomy.",
            },
            Word {
                name: "audit",
                summary: "measure the taxonomy against the corpus",
                description: "Measure the taxonomy against the corpus: edge counts and staleness by the creator each relation declares, relation drift by family, facet differentiation, the discriminator distribution of a heterogeneous shelf, and state dwell. It reports findings about the schema and never about a document, it gates nothing, and it always exits 0. One bar is declared and the rest of the readings are distributions with no verdict beside them.",
            },
            Word {
                name: "publish",
                summary: "write the artifact of a package into a directory",
                description: "Write the artifact of a package into a directory, with a release record over it: every file, the digest of its bytes, and one digest over that list. It prints the digest, which is the number the release notes state and a consumer pins. It reads every migration payload the manifest declares before it writes a file, and refuses one that the taxonomy under publication contradicts.",
            },
            Word {
                name: "vendor",
                summary: "check a fetched artifact against the pinned digest",
                description: "Check an artifact that somebody already fetched against the digest this repository pinned, and install it under `packages/`. It refuses an artifact that is not the pinned one, and it names every file that moved. Nothing here fetches: no crate of this engine depends on the network, so the verb takes the path of a directory and never a location.",
            },
            Word {
                name: "diff",
                summary: "measure what a published artifact would do to this corpus",
                description: "Measure what a published artifact would do to this corpus, across the six compatibility dimensions of spec 2. It resolves the artifact under this repository's own overlays and runs every phase twice over one tree, so a difference is attributable to the schema rather than to two publishes of one package differing in trivia. Where the artifact ships a migration payload for the move, it reports what each step reaches in this corpus and every document that stopped validating under no step. It writes nothing, and it fails only when it could not measure.",
            },
            Word {
                name: "migrate",
                summary: "apply the migration payload a published artifact ships",
                description: "Apply the migration payload a published artifact ships, to this corpus. It takes the path of a directory somebody already fetched, because this engine fetches nothing. Without `--apply` it reports every file each step would write and writes nothing.",
            },
        ],
    },
    // A verb that reads no corpus, and the only one.
    //
    // A harness hands a hook one JSON object on standard input, and the hook
    // needs three words of it: which moment fired, which path a tool is about
    // to touch, and whether this stop is a second stop. Reading those needed an
    // interpreter that nothing else in a session requires, and
    // [HW-OBL-0146](../../../../docs/obligations/0146-the-stop-hook-reads-its-re-entry-guard-with-an-interpreter-it-does-not-require-so-a-machine-with-no-python3-re-blocks-the-same-turn.md)
    // records what that cost: on a machine with no `python3` the review
    // position could not read its re-entry guard, and it re-blocked the same
    // turn with no way out.
    //
    // The [hook contract](../../../../docs/spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind)
    // forbids a `headwater hook <moment>` verb, and the reason it gives is
    // drift: two entry points to one answer are two answers. This verb is
    // outside that reason rather than inside an exception to it. It answers
    // nothing about a corpus, so there is no second answer for it to drift
    // from, and it names no moment, so no position of a harness is spelled
    // anywhere in this binary.
    // [HW-DR-0055](../../../../docs/decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md)
    // is the ruling.
    Verb {
        name: "json",
        group: "Reading a wire format, and no corpus",
        summary: "read one member of the JSON object on standard input",
        description: "Read the JSON object on standard input. This verb reads no corpus and answers no question about one. It is here because a harness hands a hook a JSON payload, and a hook that read it for itself would need an interpreter that nothing else in a session requires.",
        words: &[
            Word {
                name: "field",
                summary: "one member, addressed by a path of keys",
                description: "Print one member of the object on standard input, addressed by a path of keys. `headwater json field tool_input file_path` reads the `file_path` member of the `tool_input` member. A string is printed with its escapes resolved, a number as it was written, and a boolean as `true` or `false`. It prints nothing and exits non-zero where the read reaches no scalar, which is one answer for six states: the document will not parse, a step of the path is not an object, the key is absent, the member is an array, the member is an object, or the member is null. A caller that told those apart would act on the shape of a message it did not write.",
            },
            Word {
                name: "count",
                summary: "how many elements the array or the object at a path holds",
                description: "Print how many elements the array or the object at that path holds. It is the read `field` cannot do. An empty array and an absent member both give a caller nothing back, and they are different facts about the message. It prints nothing and exits non-zero where the path reaches no array and no object.",
            },
            Word {
                name: "quote",
                summary: "standard input as one JSON string literal",
                description: "Read standard input and write it back as one JSON string literal, quoted and escaped. A caller needs this to put a path or a report inside the JSON object it writes back to a harness. It is the writer that every JSON this engine emits is written with.",
            },
        ],
    },
    // `help` is a verb of this table and a variant of the parser, rather than
    // the subcommand `clap` injects during `build()`. The injected one carries a
    // copy of the whole command tree under it — `headwater help sweep plan` and
    // forty-two more — and no such command line is in this table, so enabling it
    // would fail the walk in `engine/crates/cli/tests/verbs.rs` or force an
    // exclusion into it. One variant with one positional is the shape that adds
    // the command line #321 asks for and leaves that walk exact.
    Verb {
        name: "help",
        group: "Getting help",
        summary: "the long description of one verb, or this screen",
        description: "Print the long description of one verb, or this screen when no verb follows. `headwater help check`, `headwater check --help` and `headwater check -h` print the same text, and a second word follows its verb: `headwater help taxonomy diff`.",
        words: &[],
    },
    // `completions` is in this group because a completion script is how a
    // caller finds a verb without reading the help at all. It is the second
    // reader of the command tree that is not a caller: `engine/crates/cli/tests/verbs.rs`
    // walks that tree to hold it against this table, and `clap_complete` walks
    // the same tree to write a script. Neither one carries a copy of this list.
    Verb {
        name: "completions",
        group: "Getting help",
        summary: "write the completion script of one shell",
        description: "Write the completion script for one shell on standard output, and write no file. The four names are `bash`, `zsh`, `fish` and `powershell`, and a fifth is refused with the four printed. The script is generated from the command tree this binary parses with, so it carries the verbs, the second words and the flags this binary answers to rather than a list somebody keeps. Where the script goes is the shell's convention rather than this engine's, so redirect it there: `headwater completions bash > f && . f` is the form the tests use.",
        words: &[],
    },
];

/// The verb a first word names, and `None` for a word this binary does not
/// carry.
pub fn parse(word: &str) -> Option<&'static Verb> {
    VERBS.iter().find(|verb| verb.name == word)
}

/// Every first word, as a message prints them: `` `a`, `b` and `c` ``.
pub fn listed() -> String {
    join(&VERBS.iter().map(|verb| verb.name).collect::<Vec<_>>())
}

/// Every second word of one verb, as a message prints them.
pub fn words_of(name: &str) -> String {
    match parse(name) {
        Some(verb) => join(&verb.words.iter().map(|word| word.name).collect::<Vec<_>>()),
        None => String::new(),
    }
}

/// Every group, in the order [`VERBS`] first names each one.
///
/// No second list of group names exists: the order is read off the table. A
/// group misspelled in one entry would otherwise become a second heading with
/// one verb under it, and the case below refuses that arrangement outright.
pub fn groups() -> Vec<&'static str> {
    let mut seen: Vec<&'static str> = Vec::new();
    for verb in VERBS {
        if !seen.contains(&verb.group) {
            seen.push(verb.group);
        }
    }
    seen
}

/// `` `a`, `b` and `c` ``: the form every message in this binary uses.
fn join(words: &[&str]) -> String {
    let quoted: Vec<String> = words.iter().map(|word| format!("`{word}`")).collect();
    match quoted.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::{groups, join, parse, words_of, Verb, Word, VERBS};

    #[test]
    fn a_word_no_verb_carries_resolves_to_nothing() {
        assert_eq!(parse("wibble"), None);
        assert_eq!(parse("check").map(|verb| verb.name), Some("check"));
    }

    #[test]
    fn no_two_verbs_carry_one_name() {
        let mut names: Vec<&str> = VERBS.iter().map(|verb| verb.name).collect();
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(before, names.len(), "two verbs carry one name");
    }

    /// A verb with second words names one form for each and no bare one,
    /// because the parser refuses a bare one. The index prints what a caller
    /// may type, so this is the difference that makes it honest.
    #[test]
    fn a_verb_with_second_words_names_one_form_for_each_and_no_bare_one() {
        let verb = Verb {
            name: "sweep",
            group: "Sampling",
            summary: "s",
            description: "d",
            words: &[
                Word {
                    name: "plan",
                    summary: "s",
                    description: "d",
                },
                Word {
                    name: "report",
                    summary: "s",
                    description: "d",
                },
            ],
        };
        assert_eq!(
            verb.forms(),
            vec![
                "headwater sweep plan".to_string(),
                "headwater sweep report".to_string()
            ]
        );
        assert_eq!(verb.described_as(), "headwater sweep");
        let bare = Verb {
            name: "check",
            group: "Checking",
            summary: "s",
            description: "d",
            words: &[],
        };
        assert_eq!(bare.forms(), vec!["headwater check".to_string()]);
    }

    #[test]
    fn a_list_of_one_carries_no_conjunction() {
        assert_eq!(join(&[]), "");
        assert_eq!(join(&["one"]), "`one`");
        assert_eq!(join(&["one", "two"]), "`one` and `two`");
        assert_eq!(join(&["one", "two", "three"]), "`one`, `two` and `three`");
        assert_eq!(words_of("check"), "");
        assert_eq!(words_of("sweep"), "`plan` and `report`");
    }

    /// Every verb says what it is, in both lengths.
    ///
    /// The first screen prints the summary and `headwater <verb> --help` prints
    /// the description, so an empty one of either is a verb a caller meets with
    /// nothing against it. `engine/crates/cli/tests/help.rs` holds the same
    /// property over the command tree the parser builds, where a flag is caught
    /// too; this one holds it at the source and names the verb.
    #[test]
    fn every_verb_and_every_second_word_says_what_it_is() {
        for verb in VERBS {
            assert!(!verb.summary.is_empty(), "`{}` has no summary", verb.name);
            assert!(
                !verb.description.is_empty(),
                "`{}` has no description",
                verb.name
            );
            assert!(!verb.group.is_empty(), "`{}` has no group", verb.name);
            for word in verb.words {
                assert!(
                    !word.summary.is_empty(),
                    "`{} {}` has no summary",
                    verb.name,
                    word.name
                );
                assert!(
                    !word.description.is_empty(),
                    "`{} {}` has no description",
                    verb.name,
                    word.name
                );
            }
        }
    }

    /// The order of the groups is the order they first appear, so the verbs of
    /// one group have to be contiguous for that order to mean anything.
    ///
    /// It also refuses the failure a free-text group is open to: a group
    /// misspelled in one entry becomes a second heading with one verb under it,
    /// and this case names both the group and the verb rather than leaving a
    /// reader to compare a help screen by eye.
    #[test]
    fn the_verbs_of_one_group_are_contiguous() {
        let mut seen: Vec<&str> = Vec::new();
        let mut previous = "";
        for verb in VERBS {
            if verb.group == previous {
                continue;
            }
            assert!(
                !seen.contains(&verb.group),
                "`{}` reopens the group `{}`, which is already closed",
                verb.name,
                verb.group
            );
            seen.push(verb.group);
            previous = verb.group;
        }
        assert_eq!(seen, groups());
    }
}
