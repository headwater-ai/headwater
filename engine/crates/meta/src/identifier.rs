// SPDX-License-Identifier: Apache-2.0
//! The template language an `identifier_scheme` writes its `pattern` in.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md) writes three
//! placeholder forms and no others: `{namespace}`, `{slug}` and `{seq:04d}`.
//! [`crate::pattern::Pattern`] is the glob language a *shelf path* is written
//! in, where `*` and `**` mean runs of path segments, so it is the wrong matcher
//! for this and this module is the other one. The two are siblings: both are
//! small languages the meta-schema writes, and both live here so that no
//! consumer of the meta-schema carries a second reading of one.
//!
//! # Why the grammar is here and not in the crate that mints
//!
//! It was in `headwater_check::identifier`, which is where the generated check
//! `identifier.pattern.not_met` lives, and `headwater_resolve` cannot see that
//! crate. So `taxonomy validate`'s identifier-integrity rule carried its own,
//! weaker reading of the same string — a scan for the text before the first
//! placeholder — and reported that it could not decide what the parser beside it
//! could. One grammar, read two ways, is the defect that
//! [#210](https://github.com/headwater-ai/headwater/issues/210) names. Both
//! readers now sit above this module.
//!
//! # The three forms, and what each one means
//!
//! Only one of them is a constraint the specification states.
//!
//! * `{namespace}` is the declared `namespace`, exactly. That is the one
//!   lexical requirement of a pattern, and `identifier integrity` on the
//!   resolved taxonomy is what makes it
//!   ([spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema)).
//!   The invariant core cannot: its identifier form names a scheme rather
//!   than a property of one. A namespace is cheap at minting and
//!   unrecoverable once the identifier is in somebody else's ticket, and
//!   [Q25](../../../../docs/spec/09-decisions.md#q25--where-the-namespace-goes-in-an-identifier-and-who-declares-it)
//!   puts it first in the rendered form.
//! * `{seq:04d}` is exactly four decimal digits, and `{seq:0Nd}` is exactly N.
//!   The width is in the declaration, so a reader takes it rather than
//!   assuming one.
//! * `{slug}` is a free token, and the specification says nothing else about
//!   it. So it is required to be present and not empty, and nothing is required
//!   of its alphabet. A rule that admitted only lower-case words would be this
//!   engine inventing a lexical constraint that no declaration makes, over the
//!   one part of an identifier spec 2 leaves to the author.
//!
//! Matching is left to right, and each placeholder is bounded by what follows
//! it. `{namespace}` and `{seq:0Nd}` have a known width or a known value, so
//! they need no lookahead. `{slug}` does not, so it is read only in terminal
//! position: a template that puts a free token before another segment does not
//! say where the token ends, and two readings of `HW-SPEC-a-b` would be equally
//! defensible. Such a pattern is [`Unreadable`], and every caller reports the
//! reason rather than picking one of the two.
//!
//! # What that restriction buys, and it is the whole of [`Template::disjoint`]
//!
//! Because a `{slug}` is terminal, every template denotes a language of exactly
//! this shape: a fixed-length run of positions, each of which is either one
//! literal character or any decimal digit, followed either by the end of the
//! string or by one or more free characters. Nothing else is expressible. So
//! whether two templates admit a common string is decided by comparing their
//! runs position by position and then comparing their lengths, which is what
//! [`Template::disjoint`] does. It is exact in both directions: it never calls
//! two overlapping schemes disjoint, and it never calls two disjoint schemes
//! overlapping.

/// One piece of a template, in declaration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    /// Characters that appear as written. The scheme's prefix is one of these.
    Literal(String),
    /// `{namespace}`: the declared namespace, and no other string.
    Namespace,
    /// `{seq:0Nd}`: exactly N decimal digits.
    Sequence { width: usize },
    /// `{slug}`: a free, non-empty token, read to the end of the identifier.
    Slug,
}

impl Segment {
    /// What a report calls this segment when it is the one that failed.
    fn describe(&self, namespace: &str) -> String {
        match self {
            Segment::Literal(text) => format!("the literal `{text}`"),
            Segment::Namespace => format!("the namespace `{namespace}`"),
            Segment::Sequence { width } => format!("{width} digits"),
            Segment::Slug => "a slug".to_string(),
        }
    }
}

/// A template no reader here can read, and the reason a caller states.
///
/// The same value is the reason a minter refuses, the reason the generated
/// check skips, and the reason `taxonomy validate` refuses the scheme. A scheme
/// whose pattern this module cannot read is a scheme under which no identifier
/// can be checked, none should be issued, and nothing can be said about what it
/// admits. One sentence says all three.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable(pub String);

impl std::fmt::Display for Unreadable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A parsed `pattern`, with the namespace it was declared beside.
#[derive(Clone, Debug)]
pub struct Template {
    segments: Vec<Segment>,
    namespace: String,
}

/// What a template asks a minter for, beyond what the declaration already says.
///
/// A literal and the namespace come from the scheme. These two do not: a slug
/// is a name somebody chooses and a sequence is an allocation over the corpus.
/// So a caller reads this before it mints, and it supplies what the list names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Slug,
    Sequence { width: usize },
}

/// The outcome of matching one identifier against one template.
enum Match {
    Admitted,
    /// The segment that stopped the match, and what was left of the identifier
    /// when it did.
    Refused {
        at: Segment,
        rest: String,
    },
    /// Every segment matched and characters were left over.
    Trailing {
        rest: String,
    },
}

/// One position of a template's fixed-length run.
///
/// Two values and not three: a literal character and the namespace both fix a
/// character, so the namespace contributes [`Class::Char`] per character and no
/// variant of its own. A reader that kept the namespace as a third class would
/// be asking whether two schemes share a namespace, which is a question the
/// caller has already answered by grouping them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Char(char),
    Digit,
}

impl Class {
    /// Whether one character can satisfy both positions at once.
    fn meets(self, other: Class) -> bool {
        match (self, other) {
            (Class::Char(one), Class::Char(two)) => one == two,
            (Class::Char(one), Class::Digit) | (Class::Digit, Class::Char(one)) => {
                one.is_ascii_digit()
            }
            (Class::Digit, Class::Digit) => true,
        }
    }
}

impl Template {
    /// Read a `pattern` into segments, or say why it cannot be read.
    pub fn parse(pattern: &str, namespace: &str) -> Result<Self, Unreadable> {
        let mut segments: Vec<Segment> = Vec::new();
        let mut literal = String::new();
        let mut rest = pattern;

        while let Some(open) = rest.find('{') {
            literal.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else {
                return Err(Unreadable(format!(
                    "the pattern `{pattern}` opens a placeholder that it does not close"
                )));
            };
            let name = &after[..close];
            if !literal.is_empty() {
                segments.push(Segment::Literal(std::mem::take(&mut literal)));
            }
            segments.push(read_placeholder(name, pattern)?);
            rest = &after[close + 1..];
        }
        literal.push_str(rest);
        if !literal.is_empty() {
            segments.push(Segment::Literal(literal));
        }

        // A free token is bounded only by the end of the identifier. See the
        // module comment: a template that puts one before another segment does
        // not state where it ends, and `disjoint` below rests on the same fact.
        if let Some(position) = segments.iter().position(|part| part == &Segment::Slug) {
            if position + 1 != segments.len() {
                return Err(Unreadable(format!(
                    "the pattern `{pattern}` writes `{{slug}}` before another segment, and a free token has no stated end"
                )));
            }
        }
        if segments.is_empty() {
            return Err(Unreadable(
                "the scheme declares no `pattern`, so nothing states the shape of its identifiers"
                    .to_string(),
            ));
        }
        if namespace.is_empty() {
            return Err(Unreadable(
                "the scheme declares no `namespace`, and the namespace is the one part of a pattern that an overlay may not change".to_string(),
            ));
        }
        Ok(Template {
            segments,
            namespace: namespace.to_string(),
        })
    }

    /// What this template asks a minter for, in the order the pattern writes it.
    pub fn needs(&self) -> Vec<Needs> {
        self.segments
            .iter()
            .filter_map(|segment| match segment {
                Segment::Slug => Some(Needs::Slug),
                Segment::Sequence { width } => Some(Needs::Sequence { width: *width }),
                Segment::Literal(_) | Segment::Namespace => None,
            })
            .collect()
    }

    /// Fill the segments, in the same reading that [`Template::matches`] uses.
    ///
    /// A caller supplies a slug and a sequence, and the template decides which
    /// of the two it writes. Two refusals, and each one is a string this
    /// template admits no identifier for. A slug the pattern asks for and the
    /// caller does not hold has no substitute, and a sequence past the declared
    /// width would produce an identifier that the rule above then refuses.
    pub fn mint(&self, slug: Option<&str>, sequence: Option<u64>) -> Result<String, Unreadable> {
        let mut minted = String::new();
        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => minted.push_str(text),
                Segment::Namespace => minted.push_str(&self.namespace),
                Segment::Slug => match slug {
                    Some(slug) if !slug.is_empty() => minted.push_str(slug),
                    _ => {
                        return Err(Unreadable(format!(
                            "`{}` writes `{{slug}}`, and this run holds no name to put there",
                            self.render()
                        )))
                    }
                },
                Segment::Sequence { width } => {
                    let value = sequence.unwrap_or(0);
                    let written = format!("{value:0width$}");
                    if written.len() != *width {
                        return Err(Unreadable(format!(
                            "the next sequence value is {value}, and `{}` holds {width} digits",
                            self.render()
                        )));
                    }
                    minted.push_str(&written);
                }
            }
        }
        Ok(minted)
    }

    /// Whether this template admits an identifier, with no report of where it
    /// stopped. The minter's own test, and a check needs the detail.
    pub fn admits(&self, identifier: &str) -> bool {
        matches!(self.matches(identifier), Match::Admitted)
    }

    /// Why this template does not admit an identifier, or `None` where it does.
    ///
    /// The sentence a finding puts after the colon. It is here rather than at
    /// the check, because what stopped a match is a fact about the template and
    /// the reader that walked it, and a second walk at the check would be the
    /// second reading of one grammar that this module exists to remove.
    pub fn refusal(&self, identifier: &str) -> Option<String> {
        match self.matches(identifier) {
            Match::Admitted => None,
            Match::Refused { at, rest } => Some(match rest.is_empty() {
                true => format!("it ends before {}", at.describe(&self.namespace)),
                false => format!(
                    "`{rest}` is where {} was expected",
                    at.describe(&self.namespace)
                ),
            }),
            Match::Trailing { rest } => Some(format!("`{rest}` is left over at the end")),
        }
    }

    /// The sequence value an identifier carries, for a template that writes one.
    ///
    /// `None` for a template with no sequence segment, and for an identifier
    /// this template does not admit. Reconcile-first allocation is the one
    /// caller: it reads the highest value the corpus already spent.
    pub fn sequence_of(&self, identifier: &str) -> Option<u64> {
        if !self.admits(identifier) {
            return None;
        }
        let mut rest = identifier;
        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => rest = rest.strip_prefix(text.as_str())?,
                Segment::Namespace => rest = rest.strip_prefix(self.namespace.as_str())?,
                Segment::Slug => return None,
                Segment::Sequence { width } => {
                    let head = rest.get(..*width)?;
                    return head.parse::<u64>().ok();
                }
            }
        }
        None
    }

    /// The template as a person reads it, for the remediation line.
    pub fn render(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.clone(),
                Segment::Namespace => self.namespace.clone(),
                Segment::Sequence { width } => "0".repeat(*width),
                Segment::Slug => "<slug>".to_string(),
            })
            .collect()
    }

    /// Whether no string satisfies both templates. Exact, in both directions.
    ///
    /// This is what `taxonomy validate`'s identifier-integrity rule asks, and
    /// the answer is a consequence of the grammar rather than an approximation
    /// of it. Every template is a fixed-length run of positions followed either
    /// by the end of the string or by one or more free characters, so:
    ///
    /// - a position the two runs share admits a common character or it does not,
    ///   and [`Class::meets`] decides that;
    /// - past the shorter run, the shorter template either has ended, in which
    ///   case the lengths have to agree, or it is open, in which case it absorbs
    ///   whatever the longer one requires.
    ///
    /// Two consequences are worth stating, because both are cases the scan this
    /// replaced got wrong in opposite directions. `HW-DR-{seq:04d}` and
    /// `HW-DR-{seq:06d}` are **disjoint**: no string is both exactly four digits
    /// and exactly six. `HW-SPEC-{slug}` and `HW-SPEC-{seq:04d}` **overlap**:
    /// `{slug}` is a free token, so it admits `0042` like any other.
    pub fn disjoint(&self, other: &Template) -> bool {
        let (mine, yours) = (self.run(), other.run());
        let overlap = mine.len().min(yours.len());
        if !(0..overlap).all(|at| mine[at].meets(yours[at])) {
            return true;
        }
        let overlaps = match (self.is_open(), other.is_open()) {
            // Both extend freely past their runs, so one long enough string
            // satisfies whichever run is longer and both free tails.
            (true, true) => true,
            // Neither extends. The two runs agree over their overlap, so a
            // common string exists exactly when they are the same length.
            (false, false) => mine.len() == yours.len(),
            // One extends. Its free tail is at least one character, so the
            // closed one has to be strictly longer for its own end to land
            // inside that tail.
            (true, false) => yours.len() > mine.len(),
            (false, true) => mine.len() > yours.len(),
        };
        !overlaps
    }

    /// The fixed-length run this template constrains, one position per character.
    fn run(&self) -> Vec<Class> {
        let mut out = Vec::new();
        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => out.extend(text.chars().map(Class::Char)),
                Segment::Namespace => out.extend(self.namespace.chars().map(Class::Char)),
                Segment::Sequence { width } => {
                    out.extend(std::iter::repeat_n(Class::Digit, *width));
                }
                // Terminal by construction, and not part of the run.
                Segment::Slug => {}
            }
        }
        out
    }

    /// Whether a free, non-empty tail follows the run.
    fn is_open(&self) -> bool {
        self.segments.last() == Some(&Segment::Slug)
    }

    fn matches(&self, identifier: &str) -> Match {
        let mut rest = identifier;
        for segment in &self.segments {
            let consumed = match segment {
                Segment::Literal(text) => rest.strip_prefix(text.as_str()),
                Segment::Namespace => rest.strip_prefix(self.namespace.as_str()),
                Segment::Sequence { width } => digits(rest, *width),
                // Terminal by construction, and a free token is not empty.
                Segment::Slug => match rest.is_empty() {
                    true => None,
                    false => Some(""),
                },
            };
            match consumed {
                Some(remainder) => rest = remainder,
                None => {
                    return Match::Refused {
                        at: segment.clone(),
                        rest: rest.to_string(),
                    }
                }
            }
        }
        match rest.is_empty() {
            true => Match::Admitted,
            false => Match::Trailing {
                rest: rest.to_string(),
            },
        }
    }
}

/// Exactly `width` decimal digits at the front, and the rest.
///
/// Fixed width rather than "at least one digit", because that is what a
/// declaration writing `{seq:04d}` states. `DR-HW-42` is not the identifier
/// `DR-HW-0042` written short: it is a second string, and an index that held
/// both would resolve one name to two documents.
fn digits(text: &str, width: usize) -> Option<&str> {
    let head = text.get(..width)?;
    match head.bytes().all(|byte| byte.is_ascii_digit()) {
        true => Some(&text[width..]),
        false => None,
    }
}

/// One placeholder, in the three forms spec 2 writes.
fn read_placeholder(name: &str, pattern: &str) -> Result<Segment, Unreadable> {
    if name == "namespace" {
        return Ok(Segment::Namespace);
    }
    if name == "slug" {
        return Ok(Segment::Slug);
    }
    // `seq:04d`, and any width. The `d` is Python's format language, which is
    // what spec 2 borrowed the form from.
    if let Some(spec) = name.strip_prefix("seq:0") {
        if let Some(width) = spec.strip_suffix('d').and_then(|w| w.parse::<usize>().ok()) {
            if width > 0 {
                return Ok(Segment::Sequence { width });
            }
        }
    }
    Err(Unreadable(format!(
        "the pattern `{pattern}` writes the placeholder `{{{name}}}`, and spec 2 states `{{namespace}}`, `{{slug}}` and `{{seq:0Nd}}`"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(pattern: &str, namespace: &str) -> Template {
        Template::parse(pattern, namespace).expect("a readable pattern")
    }

    fn disjoint(one: (&str, &str), two: (&str, &str)) -> bool {
        let (a, b) = (template(one.0, one.1), template(two.0, two.1));
        let forward = a.disjoint(&b);
        // The relation is symmetric, and a caller compares each pair once.
        assert_eq!(forward, b.disjoint(&a), "`{}` against `{}`", one.0, two.0);
        forward
    }

    /// The nine schemes this repository declares, pairwise. Every pair is
    /// disjoint, and that is the state `taxonomy validate` reports on every run.
    #[test]
    fn a_type_token_between_two_schemes_makes_them_disjoint() {
        assert!(disjoint(
            ("{namespace}-SPEC-{slug}", "HW"),
            ("{namespace}-REG-{slug}", "HW")
        ));
        assert!(disjoint(
            ("{namespace}-DR-{seq:04d}", "HW"),
            ("{namespace}-OBL-{seq:04d}", "HW")
        ));
    }

    /// The accepting case the scan this replaced could not reach.
    ///
    /// Two schemes that differ only in the width of their sequence share every
    /// literal character, so a comparison of literal prefixes refuses them. No
    /// string is both exactly four digits and exactly six, so they are disjoint.
    #[test]
    fn two_sequences_of_different_widths_are_disjoint_under_one_prefix() {
        assert!(disjoint(
            ("{namespace}-DR-{seq:04d}", "HW"),
            ("{namespace}-DR-{seq:06d}", "HW")
        ));
        // And the same width is the same language.
        assert!(!disjoint(
            ("{namespace}-DR-{seq:04d}", "HW"),
            ("{namespace}-DR-{seq:04d}", "HW")
        ));
    }

    /// The refusing case, which the scan also reached and which must not be lost.
    ///
    /// A `{slug}` is a free token, so it admits `0042` like any other string.
    /// Two schemes that differ only in whether their local part is a slug or a
    /// sequence therefore **overlap**, whatever the sequence's width.
    #[test]
    fn a_slug_admits_a_sequence_so_the_two_are_not_disjoint() {
        assert!(!disjoint(
            ("{namespace}-SPEC-{slug}", "HW"),
            ("{namespace}-SPEC-{seq:04d}", "HW")
        ));
        assert!(!disjoint(
            ("{namespace}-SPEC-{slug}", "HW"),
            ("{namespace}-SPEC-{seq:09d}", "HW")
        ));
    }

    /// A slug swallows a longer scheme whose text starts the same way.
    #[test]
    fn an_early_free_token_absorbs_a_longer_scheme() {
        assert!(!disjoint(
            ("{namespace}-{slug}", "HW"),
            ("{namespace}-DR-{seq:04d}", "HW")
        ));
        // And it does not absorb one that differs before the token begins.
        assert!(disjoint(
            ("{namespace}X-{slug}", "HW"),
            ("{namespace}-DR-{seq:04d}", "HW")
        ));
    }

    /// A digit position and a literal position meet only where the literal is a
    /// digit. This is the arm a comparison of literal text alone cannot make.
    #[test]
    fn a_digit_position_meets_a_literal_one_only_on_a_digit() {
        assert!(!disjoint(
            ("{namespace}-{seq:02d}-X", "HW"),
            ("{namespace}-42-X", "HW")
        ));
        assert!(disjoint(
            ("{namespace}-{seq:02d}-X", "HW"),
            ("{namespace}-4a-X", "HW")
        ));
    }

    /// Two closed templates of different lengths admit no common string, even
    /// where every shared position agrees.
    #[test]
    fn two_closed_templates_of_different_lengths_are_disjoint() {
        assert!(disjoint(
            ("{namespace}-DR-{seq:04d}", "HW"),
            ("{namespace}-DR-{seq:04d}X", "HW")
        ));
    }

    /// A namespace is compared character by character like any literal, so two
    /// schemes in two namespaces are disjoint whenever the namespaces are.
    /// The caller groups by namespace before it asks, and this is what makes
    /// that grouping a shortcut rather than a rule of its own.
    #[test]
    fn two_namespaces_that_differ_make_two_schemes_disjoint() {
        assert!(disjoint(
            ("{namespace}-SPEC-{slug}", "HW"),
            ("{namespace}-SPEC-{slug}", "XX")
        ));
    }

    /// Every claim [`Template::disjoint`] makes about a pair is a claim about
    /// strings, so a disagreement between it and [`Template::admits`] is a
    /// defect in one of the two, and this is the differential that would report
    /// it.
    ///
    /// The search space is `HW-` followed by every word of up to five
    /// characters over four letters, which is chosen to be wide enough that
    /// every overlapping pair below has a witness inside it — the longest is
    /// `HW-A-00`. So the reading is exact for these patterns in both
    /// directions: an "overlapping" verdict with no witness fails, and a
    /// "disjoint" verdict with one fails.
    #[test]
    fn disjointness_agrees_with_what_the_two_templates_admit() {
        let patterns = [
            "{namespace}-A-{slug}",
            "{namespace}-A-{seq:01d}",
            "{namespace}-A-{seq:02d}",
            "{namespace}-{slug}",
            "{namespace}-{seq:01d}-A",
            "{namespace}-AA",
            "{namespace}-A0",
        ];
        // Every pattern above fixes `HW-`, so a word that does not start with
        // it is a word neither template admits and the search skips it.
        let mut words: Vec<String> = vec!["HW-".to_string()];
        let mut frontier: Vec<String> = words.clone();
        for _ in 0..5 {
            let mut next = Vec::new();
            for word in &frontier {
                for letter in ['A', '0', '1', '-'] {
                    next.push(format!("{word}{letter}"));
                }
            }
            words.extend(next.iter().cloned());
            frontier = next;
        }

        for (index, one) in patterns.iter().enumerate() {
            for two in &patterns[index..] {
                let (a, b) = (template(one, "HW"), template(two, "HW"));
                let witness = words.iter().find(|word| a.admits(word) && b.admits(word));
                assert_eq!(
                    a.disjoint(&b),
                    witness.is_none(),
                    "`{one}` against `{two}`: disjoint says {}, and the witness is {witness:?}",
                    a.disjoint(&b)
                );
            }
        }
    }

    fn admits(pattern: &str, namespace: &str, identifier: &str) -> bool {
        template(pattern, namespace).admits(identifier)
    }

    #[test]
    fn a_namespace_placeholder_is_the_declared_namespace_and_no_other_string() {
        assert!(admits("{namespace}-SPEC-{slug}", "HW", "HW-SPEC-glossary"));
        // The one lexical requirement, and `identifier integrity` is what makes
        // it. Another owner's namespace, and no namespace at all.
        assert!(!admits("{namespace}-SPEC-{slug}", "HW", "XX-SPEC-glossary"));
        assert!(!admits("{namespace}-SPEC-{slug}", "HW", "-SPEC-glossary"));
        // The same two under a type-first pattern, which this engine still
        // reads and which no taxonomy of this repository writes. The namespace
        // here is somebody else's for the same reason the pattern is.
        assert!(admits("SPEC-{namespace}-{slug}", "XX", "SPEC-XX-glossary"));
        assert!(!admits("SPEC-{namespace}-{slug}", "XX", "SPEC-YY-glossary"));
        assert!(!admits("SPEC-{namespace}-{slug}", "XX", "SPEC--glossary"));
    }

    #[test]
    fn a_sequence_placeholder_is_the_width_the_declaration_writes() {
        assert!(admits("{namespace}-DR-{seq:04d}", "ACME", "ACME-DR-0042"));
        // Not the same identifier written short. See `digits`.
        assert!(!admits("{namespace}-DR-{seq:04d}", "ACME", "ACME-DR-42"));
        assert!(!admits("{namespace}-DR-{seq:04d}", "ACME", "ACME-DR-00042"));
        assert!(!admits("{namespace}-DR-{seq:04d}", "ACME", "ACME-DR-004x"));
        assert!(admits("{namespace}-DR-{seq:03d}", "ACME", "ACME-DR-042"));
    }

    #[test]
    fn a_slug_is_a_free_token_and_nothing_here_states_its_alphabet() {
        assert!(admits("{namespace}-SPEC-{slug}", "HW", "HW-SPEC-two-words"));
        assert!(admits(
            "{namespace}-SPEC-{slug}",
            "HW",
            "HW-SPEC-Mixed_Case9"
        ));
        // Present and not empty is the whole of the requirement.
        assert!(!admits("{namespace}-SPEC-{slug}", "HW", "HW-SPEC-"));
    }

    #[test]
    fn the_literal_prefix_has_to_be_there() {
        assert!(!admits("{namespace}-SPEC-{slug}", "HW", "HW-REG-glossary"));
        assert!(!admits("{namespace}-SPEC-{slug}", "HW", "HW-glossary"));
    }

    /// A pattern this module cannot read is a reason a caller states, and never
    /// a finding against a document.
    #[test]
    fn a_pattern_this_module_cannot_read_says_so() {
        for (pattern, namespace) in [
            // A free token before another segment: two readings, and the
            // declaration does not choose.
            ("SPEC-{slug}-{namespace}", "HW"),
            // A placeholder spec 2 does not write.
            ("{namespace}-SPEC-{uuid}", "HW"),
            // A width the format language does not give.
            ("{namespace}-DR-{seq:d}", "HW"),
            ("{namespace}-DR-{seq:00d}", "HW"),
            // An unclosed placeholder.
            ("SPEC-{namespace", "HW"),
            // The namespace is required, and a scheme without one states
            // nothing an identifier can be held to.
            ("{namespace}-SPEC-{slug}", ""),
            // No pattern at all.
            ("", "HW"),
        ] {
            assert!(
                Template::parse(pattern, namespace).is_err(),
                "`{pattern}` read as a template"
            );
        }
    }

    /// The report quotes the template back with the namespace filled in, so a
    /// reader compares two strings rather than a string and a form.
    #[test]
    fn a_rendered_template_is_the_shape_an_identifier_takes() {
        assert_eq!(
            template("{namespace}-SPEC-{slug}", "HW").render(),
            "HW-SPEC-<slug>"
        );
        assert_eq!(
            template("{namespace}-DR-{seq:04d}", "ACME").render(),
            "ACME-DR-0000"
        );
    }

    /// The sentence a finding puts after the colon, in each of its three shapes.
    #[test]
    fn a_refusal_names_the_segment_that_stopped_the_match() {
        let one = template("{namespace}-DR-{seq:04d}", "HW");
        assert_eq!(one.refusal("HW-DR-0042"), None);
        assert_eq!(
            one.refusal("HW-DR-004x"),
            Some("`004x` is where 4 digits was expected".to_string())
        );
        assert_eq!(
            one.refusal("HW-DR-"),
            Some("it ends before 4 digits".to_string())
        );
        assert_eq!(
            one.refusal("HW-DR-0042X"),
            Some("`X` is left over at the end".to_string())
        );
    }
}
