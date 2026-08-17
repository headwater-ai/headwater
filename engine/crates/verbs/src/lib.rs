// SPDX-License-Identifier: Apache-2.0
//! The command surface of this binary, in one list.
//!
//! # Four copies of this list disagreed, and three of them were wrong
//!
//! [#257](https://github.com/headwater-ai/headwater/issues/257) recorded the
//! measurement. The `match verb.as_slice()` arms of the binary were the truth.
//! The message a caller reads on an unknown verb listed fifteen names where the
//! arms carried seventeen, and it omitted `probe` and `query`. The `USAGE`
//! synopsis that `--help` prints omitted `query` and `taxonomy migrate`. Spec 6
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
//! [`VERBS`] is the dispatch table. The binary resolves the first word against
//! it before it enters the arms, so **an arm this list does not carry never
//! runs**. That inverts the old direction: the list was downstream of the arms
//! and drifted from them, and now the arms are downstream of the list.
//!
//! `engine/crates/cli/tests/verbs.rs` closes the other direction. It reads the
//! arms out of `main.rs` and holds them against this list, so a name here with
//! no arm fails the suite. Neither half is a rule of the check layer, and
//! neither one had to be: the surface of a binary is not corpus content
//! ([Q29](../../../../docs/decisions/0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md)).
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

/// One verb of the command surface.
///
/// `words` holds the second words a verb takes, and it is empty for a verb that
/// takes none. The distinction is the one the arms make: `headwater sweep` is
/// refused and `headwater sweep plan` runs, so `sweep` names two command lines
/// and no bare one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verb {
    /// The first word, as a caller types it.
    pub name: &'static str,
    /// The second words this verb takes, in the order the arms carry them.
    pub words: &'static [&'static str],
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
                .map(|word| format!("{BINARY} {} {word}", self.name))
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
/// The order is the order of the arms, which is the order `--help` prints. It
/// is not alphabetical, and a generated index keeps it, so that a reader who
/// runs `--help` and a reader who opens the index meet the same sequence.
pub const VERBS: &[Verb] = &[
    Verb {
        name: "check",
        words: &[],
    },
    Verb {
        name: "gate",
        words: &[],
    },
    Verb {
        name: "route",
        words: &[],
    },
    Verb {
        name: "explain",
        words: &[],
    },
    Verb {
        name: "mcp",
        words: &[],
    },
    Verb {
        name: "new",
        words: &[],
    },
    Verb {
        name: "capture",
        words: &[],
    },
    Verb {
        name: "sweep",
        words: &["plan", "report"],
    },
    Verb {
        name: "probe",
        words: &["plan", "record", "grade", "stale"],
    },
    Verb {
        name: "generate",
        words: &[],
    },
    Verb {
        name: "import",
        words: &[],
    },
    Verb {
        name: "export",
        words: &[],
    },
    Verb {
        name: "init",
        words: &[],
    },
    Verb {
        name: "infer",
        words: &[],
    },
    Verb {
        name: "conformance",
        words: &[],
    },
    // Listed in spec 6, and no document states what an expression is. The verb
    // states that wait when a caller types it, which is what #146 requires of a
    // declared name. It is a verb of this surface for exactly that reason: a
    // wait a caller cannot discover is a wait nobody reads.
    Verb {
        name: "query",
        words: &[],
    },
    Verb {
        name: "taxonomy",
        words: &[
            "validate", "resolve", "audit", "publish", "vendor", "diff", "migrate",
        ],
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
        Some(verb) => join(verb.words),
        None => String::new(),
    }
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
    use super::{join, parse, words_of, Verb, VERBS};

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

    /// A verb with second words names no bare command line, because the arms
    /// refuse one. The index prints what a caller may type, so this is the
    /// difference that makes it honest.
    #[test]
    fn a_verb_with_second_words_names_one_form_for_each_and_no_bare_one() {
        let verb = Verb {
            name: "sweep",
            words: &["plan", "report"],
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
}
