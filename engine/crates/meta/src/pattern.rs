// SPDX-License-Identifier: Apache-2.0
//! The path patterns a shelf and an exclusion are written in.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#kind-resolution) says
//! that kind resolution "matches the path against shelf patterns — the most
//! specific wins, and ties are a schema-validation error, not a runtime
//! coin-flip". It does not say what the pattern language is, and it does not say
//! what makes one pattern more specific than another. Both are decided here,
//! narrowly, and both are recorded as findings against the specification rather
//! than left as engine folklore.
//!
//! # Why the meta-schema owns this
//!
//! The language started in `headwater-census`, which was the first code to read
//! a shelf declaration. [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! then recorded that the ordering "belongs to the meta-schema", and the
//! determinism rule of `taxonomy validate` proved the point: that rule asks
//! whether two shelf patterns can match one path, and it runs before any corpus
//! is walked. Two implementations of one glob language would disagree the day
//! one of them learned a construct, so the language lives in one crate and both
//! callers read it here.
//!
//! # The language
//!
//! A pattern is a `/`-separated sequence of segments, matched against a path
//! that is relative to the repository root and written with `/`.
//!
//! - `**` **as a whole segment** matches zero or more segments.
//! - `*` matches any run of characters inside one segment, never `/`.
//! - `?` matches exactly one character inside one segment.
//! - Everything else is literal.
//!
//! Four forms and no more. Every pattern this repository declares is a literal
//! prefix followed by `**`, and a language that admits more than the corpus uses
//! is a language with untested cases in it.
//!
//! # Specificity
//!
//! Two shelves can match one path, and one of them has to win without a coin
//! flip. A pattern is more specific than another when it fixes more of the path:
//!
//! 1. first by the number of leading segments that hold no wildcard,
//! 2. then by the count of literal characters in the whole pattern.
//!
//! So `docs/spec/**` beats `docs/**` on the first test, and `docs/spec/*.md`
//! beats `docs/spec/**` on the second. Two patterns that tie on both are a tie,
//! and `headwater_census::resolve` reports it rather than picking one.
//!
//! # Overlap
//!
//! [`Pattern::overlaps`] answers a different question from [`Pattern::matches`]:
//! is there **any** path that both patterns match? Kind resolution never asks
//! it, because it holds a path in its hand. The determinism rule asks it with no
//! corpus at all, and a rule that waited for a corpus would report a schema
//! defect only on the day that somebody wrote the file which triggers it.

/// A compiled path pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    source: String,
    segments: Vec<Segment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    /// `**`: zero or more whole segments.
    AnyDepth,
    /// One segment, matched against a glob with `*` and `?` in it.
    Glob(String),
    Literal(String),
}

impl Pattern {
    pub fn new(source: &str) -> Self {
        let segments = source
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| match segment {
                "**" => Segment::AnyDepth,
                _ if segment.contains(['*', '?']) => Segment::Glob(segment.to_string()),
                _ => Segment::Literal(segment.to_string()),
            })
            .collect();
        Self {
            source: source.to_string(),
            segments,
        }
    }

    /// The pattern as it was declared, for a message that quotes it back.
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn matches(&self, path: &str) -> bool {
        let path: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        matches_from(&self.segments, &path)
    }

    /// How much of a path this pattern fixes. Ordered, so a larger value is
    /// more specific, and equal values are a tie.
    pub fn specificity(&self) -> (usize, usize) {
        let leading = self
            .segments
            .iter()
            .take_while(|segment| matches!(segment, Segment::Literal(_)))
            .count();
        let literal_chars = self
            .segments
            .iter()
            .map(|segment| match segment {
                Segment::AnyDepth => 0,
                Segment::Literal(text) => text.len(),
                Segment::Glob(text) => text.chars().filter(|c| !matches!(c, '*' | '?')).count(),
            })
            .sum();
        (leading, literal_chars)
    }

    /// Whether some path exists that both patterns match.
    ///
    /// This is emptiness of an intersection rather than a match, and it is
    /// decided over the two patterns with no path in hand. The recursion mirrors
    /// [`matches_from`], with a path segment replaced by a pattern segment at
    /// every step.
    pub fn overlaps(&self, other: &Pattern) -> bool {
        overlaps_from(&self.segments, &other.segments)
    }
}

/// Segment-wise intersection. `**` on either side searches over how many
/// segments of the other side it swallows, exactly as matching does.
fn overlaps_from(left: &[Segment], right: &[Segment]) -> bool {
    match (left.split_first(), right.split_first()) {
        (None, None) => true,
        // A trailing `**` matches zero segments, so it can meet the end.
        (None, Some((Segment::AnyDepth, rest))) => overlaps_from(&[], rest),
        (Some((Segment::AnyDepth, rest)), None) => overlaps_from(rest, &[]),
        (None, Some(_)) | (Some(_), None) => false,
        (Some((Segment::AnyDepth, rest)), _) => {
            (0..=right.len()).any(|taken| overlaps_from(rest, &right[taken..]))
        }
        (_, Some((Segment::AnyDepth, rest))) => {
            (0..=left.len()).any(|taken| overlaps_from(&left[taken..], rest))
        }
        (Some((head, left_rest)), Some((other, right_rest))) => {
            segments_overlap(head, other) && overlaps_from(left_rest, right_rest)
        }
    }
}

/// Two single-segment patterns, and whether one text satisfies both.
fn segments_overlap(left: &Segment, right: &Segment) -> bool {
    let text = |segment: &Segment| match segment {
        Segment::Literal(text) | Segment::Glob(text) => text.clone(),
        Segment::AnyDepth => unreachable!("handled by the caller"),
    };
    glob_overlaps(
        &text(left).chars().collect::<Vec<char>>(),
        &text(right).chars().collect::<Vec<char>>(),
    )
}

/// Whether two globs over one segment admit a common text.
///
/// A literal is the glob with no wildcard in it, so this one function covers
/// every pair. `*` consumes nothing or one character of the other side and
/// stays, which is the same search that [`glob_matches`] runs against a text.
fn glob_overlaps(left: &[char], right: &[char]) -> bool {
    match (left.split_first(), right.split_first()) {
        (None, None) => true,
        (None, Some(('*', rest))) => glob_overlaps(&[], rest),
        (Some(('*', rest)), None) => glob_overlaps(rest, &[]),
        (None, Some(_)) | (Some(_), None) => false,
        (Some(('*', rest)), _) => glob_overlaps(rest, right) || glob_overlaps(left, &right[1..]),
        (_, Some(('*', rest))) => glob_overlaps(left, rest) || glob_overlaps(&left[1..], right),
        // `?` is one character of anything, so it meets whatever stands there.
        (Some(('?', left_rest)), Some((_, right_rest)))
        | (Some((_, left_rest)), Some(('?', right_rest))) => glob_overlaps(left_rest, right_rest),
        (Some((one, left_rest)), Some((two, right_rest))) => {
            one == two && glob_overlaps(left_rest, right_rest)
        }
    }
}

/// Segment matching, written out rather than compiled to a regular expression.
///
/// `**` is the only construct that needs a search, and it needs one because it
/// can consume any number of segments. The recursion is over the pattern rather
/// than over the path, so its depth is the length of a declared pattern.
fn matches_from(pattern: &[Segment], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((Segment::AnyDepth, rest)) => {
            // Zero segments consumed, then one, then two. A trailing `**`
            // matches everything below it, including nothing.
            (0..=path.len()).any(|taken| matches_from(rest, &path[taken..]))
        }
        Some((head, rest)) => match path.split_first() {
            None => false,
            Some((segment, tail)) => {
                let matched = match head {
                    Segment::Literal(text) => text == segment,
                    Segment::Glob(glob) => glob_matches(glob, segment),
                    Segment::AnyDepth => unreachable!("handled above"),
                };
                matched && matches_from(rest, tail)
            }
        },
    }
}

/// `*` and `?` inside one segment.
fn glob_matches(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    // The classic two-cursor walk with one backtrack point, which is linear in
    // the common case and needs no allocation per candidate.
    let (mut p, mut t) = (0usize, 0usize);
    let (mut star, mut resume) = (None, 0usize);
    while t < text.len() {
        match pattern.get(p) {
            Some('*') => {
                star = Some(p);
                resume = t;
                p += 1;
            }
            Some('?') => {
                p += 1;
                t += 1;
            }
            Some(c) if *c == text[t] => {
                p += 1;
                t += 1;
            }
            _ => match star {
                Some(at) => {
                    p = at + 1;
                    resume += 1;
                    t = resume;
                }
                None => return false,
            },
        }
    }
    pattern[p..].iter().all(|c| *c == '*')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trailing_any_depth_matches_the_directory_and_everything_under_it() {
        let pattern = Pattern::new("docs/spec/**");
        assert!(pattern.matches("docs/spec/12-check-layer.md"));
        assert!(pattern.matches("docs/spec/deep/deeper/file.md"));
        assert!(pattern.matches("docs/spec"));
        assert!(!pattern.matches("docs/evaluations/a.md"));
        // The failure that shrinks a denominator: a prefix comparison would say
        // yes to a sibling directory whose name starts with the same letters.
        assert!(!pattern.matches("docs/special-cases/a.md"));
    }

    #[test]
    fn a_star_stays_inside_one_segment() {
        let pattern = Pattern::new("docs/*.md");
        assert!(pattern.matches("docs/README.md"));
        assert!(!pattern.matches("docs/spec/README.md"));
        assert!(Pattern::new("docs/??-*.md").matches("docs/12-check-layer.md"));
        assert!(!Pattern::new("docs/??-*.md").matches("docs/1-check-layer.md"));
    }

    #[test]
    fn any_depth_can_sit_in_the_middle() {
        let pattern = Pattern::new("docs/**/README.md");
        assert!(pattern.matches("docs/README.md"));
        assert!(pattern.matches("docs/taxonomies/design-spec/README.md"));
        assert!(!pattern.matches("docs/taxonomies/README.txt"));
    }

    #[test]
    fn overlap_finds_the_pairs_that_no_corpus_has_to_exist_to_show() {
        let overlap = |a: &str, b: &str| Pattern::new(a).overlaps(&Pattern::new(b));
        // The pair the determinism rule exists to refuse: both claim every file
        // under `docs/spec/`, and no file has to be there for that to be true.
        assert!(overlap("docs/**", "docs/spec/**"));
        assert!(overlap("docs/spec/**", "docs/spec/**"));
        assert!(overlap("docs/*/README.md", "docs/spec/**"));
        assert!(overlap("docs/??-*.md", "docs/*-spec.md"));
        // Siblings never meet, and a prefix comparison would say that they do.
        assert!(!overlap("docs/spec/**", "docs/evaluations/**"));
        assert!(!overlap("docs/spec/**", "docs/special-cases/**"));
        assert!(!overlap("docs/*.md", "docs/spec/*.md"));
        assert!(!overlap("docs/a?.md", "docs/abc.md"));
    }

    #[test]
    fn overlap_agrees_with_matching_on_every_path_a_pattern_admits() {
        // The property that makes the rule sound: if one path matches both, the
        // pair overlaps. The corpus here is small and written out, because the
        // failure this guards is a rule that stays quiet.
        let paths = [
            "docs/spec/02.md",
            "docs/spec/deep/a.md",
            "docs/README.md",
            "docs/evaluations/a.md",
        ];
        let patterns = ["docs/**", "docs/spec/**", "docs/*.md", "docs/**/a.md"];
        for left in patterns {
            for right in patterns {
                let (left, right) = (Pattern::new(left), Pattern::new(right));
                if paths
                    .iter()
                    .any(|path| left.matches(path) && right.matches(path))
                {
                    assert!(
                        left.overlaps(&right),
                        "{} and {} share a path and the overlap test missed it",
                        left.source(),
                        right.source()
                    );
                }
            }
        }
    }

    #[test]
    fn specificity_orders_the_patterns_this_repository_declares() {
        let all = Pattern::new("docs/**");
        let shelf = Pattern::new("docs/spec/**");
        let file = Pattern::new("docs/spec/*.md");
        assert!(shelf.specificity() > all.specificity());
        assert!(file.specificity() > shelf.specificity());
        // A tie is a tie, and the caller reports it rather than picking.
        assert_eq!(
            Pattern::new("docs/spec/**").specificity(),
            Pattern::new("docs/spec/**").specificity()
        );
    }
}
