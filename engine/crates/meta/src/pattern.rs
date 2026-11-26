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
//! A shelf path admits a fifth form, and only a shelf path:
//!
//! - `{name}` **as a whole segment** matches exactly one segment, as `*` does,
//!   and binds the facet `name` to the text of that segment. A document on
//!   `docs/contexts/{context}/contracts/**` at
//!   `docs/contexts/billing/contracts/a.md` reads `billing` for `context`.
//!
//! Only [`Pattern::shelf`] parses the fifth form. [`Pattern::new`] reads
//! `{name}` as a literal segment, because a code-path anchor, a governed scope,
//! an exclusion and the scope ratchet are written in this language too, and
//! [HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)
//! keeps a value with no wildcard naming one path outright. If `Pattern::new`
//! learned the form, a `governs: src/{x}/**` anchor would change from a
//! literal into a wildcard with no edit to the anchor. A placeholder inside a
//! segment, as in `ctx-{context}`, is not the fifth form: `taxonomy validate`
//! refuses it on a shelf path, because nothing states where its value ends.
//!
//! Five forms and no more. Nearly every pattern this repository declares is a
//! literal prefix followed by `**`, and a language that admits more than the
//! corpus uses is a language with untested cases in it. The fifth form is here
//! because an adopter with one folder per bounded context asked for the folder
//! to state the context (#2101).
//!
//! A match costs at most one test of each pattern segment against each path
//! segment, and an overlap at most one comparison of each pair of segments,
//! because both searches remember their answer at each pair of positions.
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
//! beats `docs/spec/**` on the second. A bound segment ranks as a glob segment
//! with no literal character in it, so `docs/contexts/{context}/contracts/**`
//! and `docs/contexts/*/contracts/**` tie, and the determinism rule refuses the
//! two together. Two patterns that tie on both are a tie,
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
    /// `{name}` as a whole segment of a shelf path: any one segment, whose
    /// text is the value of the facet `name`. Only [`Pattern::shelf`] makes it.
    Bound(String),
}

impl Pattern {
    /// A pattern in the four-form language, in which `{name}` is literal.
    pub fn new(source: &str) -> Self {
        Self::parse(source, false)
    }

    /// A shelf path: the four forms and a whole `{name}` segment, which binds
    /// the facet `name`. See the module header for why only a shelf path
    /// reads the fifth form.
    pub fn shelf(source: &str) -> Self {
        Self::parse(source, true)
    }

    fn parse(source: &str, binds: bool) -> Self {
        let mut segments: Vec<Segment> = source
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| match segment {
                "**" => Segment::AnyDepth,
                _ if binds && bound_name(segment).is_some() => {
                    Segment::Bound(bound_name(segment).unwrap_or_default().to_string())
                }
                _ if segment.contains(['*', '?']) => Segment::Glob(segment.to_string()),
                _ => Segment::Literal(segment.to_string()),
            })
            .collect();
        // `**/**` matches the same set as `**`, so adjacent ones are one
        // segment. `source` keeps the pattern as declared.
        segments.dedup_by(|next, kept| {
            matches!(next, Segment::AnyDepth) && matches!(kept, Segment::AnyDepth)
        });
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
                Segment::Bound(_) => 0,
            })
            .sum();
        (leading, literal_chars)
    }

    /// Every facet this pattern binds, with the index of its segment in the
    /// pattern, in pattern order. Empty for a pattern [`Pattern::new`] made.
    pub fn bound(&self) -> Vec<(usize, &str)> {
        self.segments
            .iter()
            .enumerate()
            .filter_map(|(index, segment)| match segment {
                Segment::Bound(name) => Some((index, name.as_str())),
                _ => None,
            })
            .collect()
    }

    /// The value a path gives each bound facet, or `None` when the pattern
    /// does not match the path. A pattern that binds nothing answers an empty
    /// list for a path it matches.
    ///
    /// Where a `**` stands before a bound segment, more than one reading can
    /// match; the one returned lets each `**` take as few segments as it can,
    /// so the answer is the same on every run.
    pub fn bindings(&self, path: &str) -> Option<Vec<(String, String)>> {
        let path: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut out = Vec::new();
        bind_from(&self.segments, &path, &mut out).then_some(out)
    }

    /// Whether this pattern names one path outright: no `**` and no glob
    /// segment anywhere in it.
    ///
    /// [HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)
    /// keeps a value with no wildcard resolving exactly as it did before a
    /// pattern language reached the `code_path` anchor: a literal path is
    /// tested with a filesystem `exists`, and a search over the tree runs only
    /// for a pattern that actually holds one.
    pub fn is_literal(&self) -> bool {
        self.segments
            .iter()
            .all(|segment| matches!(segment, Segment::Literal(_)))
    }

    /// The leading run of literal segments, joined with `/`.
    ///
    /// This is where a search for this pattern's matches has to start: no path
    /// this pattern admits lies outside it, because every segment before the
    /// first wildcard is fixed. A pattern with no literal segment at all (one
    /// that opens with `**`) answers the empty string, and a search rooted
    /// there walks the whole tree the resolver was given — a cost this
    /// repository's own patterns never pay, because every one of them opens on
    /// a literal directory.
    pub fn literal_prefix(&self) -> String {
        self.segments
            .iter()
            .take_while(|segment| matches!(segment, Segment::Literal(_)))
            .map(|segment| match segment {
                Segment::Literal(text) => text.as_str(),
                _ => unreachable!("stopped at the first non-literal segment"),
            })
            .collect::<Vec<&str>>()
            .join("/")
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

/// The facet name of a whole `{name}` segment: a name with no brace, no
/// wildcard and no format specifier in it.
pub fn bound_name(segment: &str) -> Option<&str> {
    let name = segment.strip_prefix('{')?.strip_suffix('}')?;
    (!name.is_empty() && !name.contains(['{', '}', '*', '?', ':'])).then_some(name)
}

/// A match that records the text of each bound segment, trying the fewest
/// segments for each `**` first. Only [`Pattern::bindings`] calls it, on one
/// document at a time, so it searches without the table `matches_at` keeps.
fn bind_from(pattern: &[Segment], path: &[&str], out: &mut Vec<(String, String)>) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((Segment::AnyDepth, rest)) => (0..=path.len()).any(|taken| {
            let kept = out.len();
            let found = bind_from(rest, &path[taken..], out);
            if !found {
                out.truncate(kept);
            }
            found
        }),
        Some((head, rest)) => {
            let Some((segment, tail)) = path.split_first() else {
                return false;
            };
            if !segment_matches(head, segment) {
                return false;
            }
            let kept = out.len();
            if let Segment::Bound(name) = head {
                out.push((name.clone(), (*segment).to_string()));
            }
            let found = bind_from(rest, tail, out);
            if !found {
                out.truncate(kept);
            }
            found
        }
    }
}

/// Segment-wise intersection. `**` on either side searches over how many
/// segments of the other side it swallows, exactly as matching does.
///
/// The search is a table over the two positions `(i, j)`, filled from the ends
/// back, so each pair of segments is compared at most once. `down` holds
/// whether any row at or below `i` overlaps column `j`, and `across` whether
/// any column at or after `j` overlaps row `i`: those are the two disjunctions
/// a `**` on the right and on the left reads, kept so that neither is a loop.
fn overlaps_from(left: &[Segment], right: &[Segment]) -> bool {
    let (rows, columns) = (left.len(), right.len());
    let width = columns + 1;
    let mut both = vec![false; (rows + 1) * width];
    let mut down = vec![false; (rows + 2) * width];
    let mut across = vec![false; (rows + 1) * (width + 1)];
    for i in (0..=rows).rev() {
        for j in (0..=columns).rev() {
            let answer = match (left.get(i), right.get(j)) {
                (None, None) => true,
                // A trailing `**` matches zero segments, so it can meet the end.
                (None, Some(Segment::AnyDepth)) => both[i * width + j + 1],
                (Some(Segment::AnyDepth), None) => both[(i + 1) * width + j],
                (None, Some(_)) | (Some(_), None) => false,
                (Some(Segment::AnyDepth), _) => across[(i + 1) * (width + 1) + j],
                (_, Some(Segment::AnyDepth)) => down[i * width + j + 1],
                (Some(head), Some(other)) => {
                    both[(i + 1) * width + j + 1] && segments_overlap(head, other)
                }
            };
            both[i * width + j] = answer;
            down[i * width + j] = answer || down[(i + 1) * width + j];
            across[i * (width + 1) + j] = answer || across[i * (width + 1) + j + 1];
        }
    }
    both[0]
}

/// Two single-segment patterns, and whether one text satisfies both.
fn segments_overlap(left: &Segment, right: &Segment) -> bool {
    // A bound segment admits any one segment, so it overlaps as `*` does.
    let text = |segment: &Segment| match segment {
        Segment::Literal(text) | Segment::Glob(text) => text.clone(),
        Segment::Bound(_) => "*".to_string(),
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
/// The search is a table over the two character positions, filled from the
/// ends back, so it is quadratic where the plain recursion was exponential in
/// the stars.
fn glob_overlaps(left: &[char], right: &[char]) -> bool {
    let (rows, columns) = (left.len(), right.len());
    let width = columns + 1;
    let mut table = vec![false; (rows + 1) * width];
    for i in (0..=rows).rev() {
        for j in (0..=columns).rev() {
            let at = |row: usize, column: usize| table[row * width + column];
            let answer = match (left.get(i), right.get(j)) {
                (None, None) => true,
                (None, Some('*')) => at(i, j + 1),
                (Some('*'), None) => at(i + 1, j),
                (None, Some(_)) | (Some(_), None) => false,
                (Some('*'), _) => at(i + 1, j) || at(i, j + 1),
                (_, Some('*')) => at(i, j + 1) || at(i + 1, j),
                // `?` is one character of anything, so it meets whatever stands there.
                (Some('?'), Some(_)) | (Some(_), Some('?')) => at(i + 1, j + 1),
                (Some(one), Some(two)) => one == two && at(i + 1, j + 1),
            };
            table[i * width + j] = answer;
        }
    }
    table[0]
}

/// Segment matching, written out rather than compiled to a regular expression.
///
/// `**` is the only construct that needs a search, and it needs one because it
/// can consume any number of segments. A pattern with no `**` is a walk in step
/// with the path. A pattern with one is a search over the two positions
/// `(i, j)`, remembered in a table, so each pattern segment is tested against
/// each path segment at most once.
fn matches_from(pattern: &[Segment], path: &[&str]) -> bool {
    if !pattern
        .iter()
        .any(|segment| matches!(segment, Segment::AnyDepth))
    {
        return pattern.len() == path.len()
            && pattern
                .iter()
                .zip(path)
                .all(|(head, segment)| segment_matches(head, segment));
    }
    let mut memo = vec![None; (pattern.len() + 1) * (path.len() + 1)];
    matches_at(pattern, path, 0, 0, &mut memo)
}

/// Whether `pattern[i..]` matches `path[j..]`, through the table `memo`.
fn matches_at(
    pattern: &[Segment],
    path: &[&str],
    i: usize,
    j: usize,
    memo: &mut [Option<bool>],
) -> bool {
    let cell = i * (path.len() + 1) + j;
    if let Some(known) = memo[cell] {
        return known;
    }
    let answer = match pattern.get(i) {
        None => j == path.len(),
        // Zero segments consumed, or one and then the same `**` again. A
        // trailing `**` matches everything below it, including nothing.
        Some(Segment::AnyDepth) => {
            matches_at(pattern, path, i + 1, j, memo)
                || (j < path.len() && matches_at(pattern, path, i, j + 1, memo))
        }
        Some(head) => {
            j < path.len()
                && segment_matches(head, path[j])
                && matches_at(pattern, path, i + 1, j + 1, memo)
        }
    };
    memo[cell] = Some(answer);
    answer
}

/// One pattern segment that is not `**`, against one path segment.
fn segment_matches(head: &Segment, segment: &str) -> bool {
    match head {
        Segment::Literal(text) => text == segment,
        Segment::Glob(glob) => glob_matches(glob, segment),
        Segment::Bound(_) => true,
        Segment::AnyDepth => unreachable!("handled by the caller"),
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
    fn a_bound_segment_matches_one_segment_and_reads_its_value() {
        let pattern = Pattern::shelf("docs/contexts/{context}/contracts/**");
        assert_eq!(pattern.bound(), vec![(2, "context")]);
        assert!(pattern.matches("docs/contexts/billing/contracts/a.md"));
        assert!(!pattern.matches("docs/contexts/contracts/a.md"));
        assert!(!pattern.matches("docs/contexts/a/b/contracts/a.md"));
        assert_eq!(
            pattern.bindings("docs/contexts/billing/contracts/deep/a.md"),
            Some(vec![("context".to_string(), "billing".to_string())])
        );
        assert_eq!(pattern.bindings("docs/other/billing/contracts/a.md"), None);
        // Ranked as a glob segment with no literal character, so it ties `*`.
        assert_eq!(
            pattern.specificity(),
            Pattern::new("docs/contexts/*/contracts/**").specificity()
        );
        assert!(pattern.overlaps(&Pattern::new("docs/contexts/*/contracts/**")));
        // As `*` and not as any one text: it meets a glob a literal would miss.
        assert!(pattern.overlaps(&Pattern::new("docs/contexts/??/contracts/**")));
        assert!(pattern.overlaps(&Pattern::new("docs/contexts/billing/**")));
        assert!(!pattern.overlaps(&Pattern::new("docs/spec/**")));
        assert_eq!(pattern.literal_prefix(), "docs/contexts");
        assert!(!pattern.is_literal());
    }

    #[test]
    fn only_a_shelf_path_binds_and_every_other_pattern_reads_braces_literally() {
        // HW-DR-0074: a code-path anchor with no wildcard names one path.
        let anchor = Pattern::new("src/{x}/lib.rs");
        assert!(anchor.is_literal());
        assert!(anchor.bound().is_empty());
        assert!(anchor.matches("src/{x}/lib.rs"));
        assert!(!anchor.matches("src/y/lib.rs"));
        // A placeholder inside a segment is not the fifth form, even on a shelf.
        let partial = Pattern::shelf("docs/ctx-{context}/**");
        assert!(partial.bound().is_empty());
        assert!(!partial.matches("docs/ctx-billing/a.md"));
        assert_eq!(bound_name("{sequence:02d}"), None);
        assert_eq!(bound_name("{}"), None);
        assert_eq!(bound_name("{context}"), Some("context"));
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
    fn a_pattern_with_no_wildcard_is_literal_and_a_prefix_of_itself() {
        let pattern = Pattern::new(".claude/hooks");
        assert!(pattern.is_literal());
        assert_eq!(pattern.literal_prefix(), ".claude/hooks");
    }

    #[test]
    fn a_wildcard_pattern_is_not_literal_and_its_prefix_stops_before_the_wildcard() {
        for (source, prefix) in [
            (".claude/hooks/**", ".claude/hooks"),
            ("docs/spec/*.md", "docs/spec"),
            ("docs/**/README.md", "docs"),
            ("**", ""),
        ] {
            let pattern = Pattern::new(source);
            assert!(!pattern.is_literal(), "{source}");
            assert_eq!(pattern.literal_prefix(), prefix, "{source}");
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

    // The matcher before the memo, copied verbatim with its own parse, so that
    // the tables below compare the rewrite with the answers it replaced and not
    // with itself. The parse keeps every `**` segment, so a collapse in
    // `Pattern::new` is compared with the uncollapsed language.
    fn reference_segments(source: &str) -> Vec<Segment> {
        source
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| match segment {
                "**" => Segment::AnyDepth,
                _ if segment.contains(['*', '?']) => Segment::Glob(segment.to_string()),
                _ => Segment::Literal(segment.to_string()),
            })
            .collect()
    }

    fn reference_matches(pattern: &[Segment], path: &[&str]) -> bool {
        match pattern.split_first() {
            None => path.is_empty(),
            Some((Segment::AnyDepth, rest)) => {
                (0..=path.len()).any(|taken| reference_matches(rest, &path[taken..]))
            }
            Some((head, rest)) => match path.split_first() {
                None => false,
                Some((segment, tail)) => {
                    let matched = match head {
                        Segment::Literal(text) => text == segment,
                        Segment::Glob(glob) => glob_matches(glob, segment),
                        Segment::Bound(_) => true,
                        Segment::AnyDepth => unreachable!("handled above"),
                    };
                    matched && reference_matches(rest, tail)
                }
            },
        }
    }

    fn reference_overlaps(left: &[Segment], right: &[Segment]) -> bool {
        match (left.split_first(), right.split_first()) {
            (None, None) => true,
            (None, Some((Segment::AnyDepth, rest))) => reference_overlaps(&[], rest),
            (Some((Segment::AnyDepth, rest)), None) => reference_overlaps(rest, &[]),
            (None, Some(_)) | (Some(_), None) => false,
            (Some((Segment::AnyDepth, rest)), _) => {
                (0..=right.len()).any(|taken| reference_overlaps(rest, &right[taken..]))
            }
            (_, Some((Segment::AnyDepth, rest))) => {
                (0..=left.len()).any(|taken| reference_overlaps(&left[taken..], rest))
            }
            (Some((head, left_rest)), Some((other, right_rest))) => {
                let text = |segment: &Segment| match segment {
                    Segment::Literal(text) | Segment::Glob(text) => {
                        text.chars().collect::<Vec<char>>()
                    }
                    Segment::Bound(_) => vec!['*'],
                    Segment::AnyDepth => unreachable!("handled above"),
                };
                reference_glob_overlaps(&text(head), &text(other))
                    && reference_overlaps(left_rest, right_rest)
            }
        }
    }

    fn reference_glob_overlaps(left: &[char], right: &[char]) -> bool {
        match (left.split_first(), right.split_first()) {
            (None, None) => true,
            (None, Some(('*', rest))) => reference_glob_overlaps(&[], rest),
            (Some(('*', rest)), None) => reference_glob_overlaps(rest, &[]),
            (None, Some(_)) | (Some(_), None) => false,
            (Some(('*', rest)), _) => {
                reference_glob_overlaps(rest, right) || reference_glob_overlaps(left, &right[1..])
            }
            (_, Some(('*', rest))) => {
                reference_glob_overlaps(left, rest) || reference_glob_overlaps(&left[1..], right)
            }
            (Some(('?', left_rest)), Some((_, right_rest)))
            | (Some((_, left_rest)), Some(('?', right_rest))) => {
                reference_glob_overlaps(left_rest, right_rest)
            }
            (Some((one, left_rest)), Some((two, right_rest))) => {
                one == two && reference_glob_overlaps(left_rest, right_rest)
            }
        }
    }

    const TABLE_PATTERNS: &[&str] = &[
        "docs/spec/**",
        "docs/*.md",
        "docs/??-*.md",
        "docs/**/README.md",
        "docs/**",
        "docs/*/README.md",
        "docs/*-spec.md",
        "docs/evaluations/**",
        "docs/special-cases/**",
        "docs/spec/*.md",
        "docs/a?.md",
        "docs/abc.md",
        "docs/**/a.md",
        ".claude/hooks",
        ".claude/hooks/**",
        "a/**/**/b",
        "a/**/b",
        "a/**/**/**/b",
        "**",
        "**/**",
        "**/x/**",
        "**/**/x/**/**",
        "a/*/**",
        "a/**/*",
        "a/**/*/**/b",
        "?",
        "*.md",
        "*",
        "",
        "a",
        "a/b",
    ];

    const TABLE_PATHS: &[&str] = &[
        "",
        "a",
        "b",
        "x",
        "a/b",
        "a/x/b",
        "a/x/y/b",
        "a/b/b",
        "a/b/c",
        "x/a/b",
        "a/x",
        "x/y/x/z",
        "docs",
        "docs/README.md",
        "docs/spec",
        "docs/spec/02.md",
        "docs/spec/deep/a.md",
        "docs/spec/12-check-layer.md",
        "docs/12-check-layer.md",
        "docs/1-check-layer.md",
        "docs/evaluations/a.md",
        "docs/special-cases/a.md",
        "docs/taxonomies/design-spec/README.md",
        "docs/taxonomies/README.txt",
        "docs/ab.md",
        "docs/abc.md",
        "docs/a.md",
        "docs/x/a.md",
        ".claude/hooks",
        ".claude/hooks/write.sh",
        "README.md",
        "q",
    ];

    #[test]
    fn matching_keeps_the_answers_of_the_matcher_it_replaced() {
        for source in TABLE_PATTERNS {
            let pattern = Pattern::new(source);
            let reference = reference_segments(source);
            for path in TABLE_PATHS {
                let split: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                assert_eq!(
                    pattern.matches(path),
                    reference_matches(&reference, &split),
                    "{source} against {path}"
                );
            }
        }
    }

    #[test]
    fn overlap_keeps_the_answers_of_the_search_it_replaced() {
        for left in TABLE_PATTERNS {
            for right in TABLE_PATTERNS {
                assert_eq!(
                    Pattern::new(left).overlaps(&Pattern::new(right)),
                    reference_overlaps(&reference_segments(left), &reference_segments(right)),
                    "{left} and {right}"
                );
            }
        }
        for left in ["*", "a*", "*a", "*a*", "?*", "*?", "a?c", "**", "*b*", "ab"] {
            for right in ["*", "b*", "*b", "*a*b*", "??", "abc", "a", "", "*c", "a*?"] {
                let chars = |text: &str| text.chars().collect::<Vec<char>>();
                assert_eq!(
                    glob_overlaps(&chars(left), &chars(right)),
                    reference_glob_overlaps(&chars(left), &chars(right)),
                    "{left} and {right}"
                );
            }
        }
    }

    #[test]
    fn adjacent_any_depth_segments_match_the_same_set_as_one() {
        for (repeated, single) in [
            ("a/**/**/b", "a/**/b"),
            ("a/**/**/**/b", "a/**/b"),
            ("**/**", "**"),
            ("**/**/x/**/**", "**/x/**"),
        ] {
            let (repeated, single) = (Pattern::new(repeated), Pattern::new(single));
            for path in TABLE_PATHS {
                assert_eq!(
                    repeated.matches(path),
                    single.matches(path),
                    "{} and {} against {path}",
                    repeated.source(),
                    single.source()
                );
            }
            // The collapse leaves one `**` for the search to try, and it is
            // not visible to a reader of the pattern.
            assert_eq!(repeated.segments, single.segments);
            assert_eq!(repeated.specificity(), single.specificity());
            assert!(repeated.source().contains("**/**"));
        }
    }

    // The three timing tests below are the hostile input of #1969. Before the
    // memo, eight `**` segments took 5.2 s on one path and twenty did not finish.
    // The bound is one second in a debug build, which the memoized walk misses
    // by three orders of magnitude.
    fn many_any_depth(count: usize, last: &str) -> String {
        format!("docs/{}{last}", "**/".repeat(count))
    }

    #[test]
    fn twenty_any_depth_segments_match_a_deep_path_in_bounded_time() {
        let pattern = Pattern::new(&many_any_depth(20, "nomatch"));
        let path = format!("docs/{}file.md", "d/".repeat(39));
        assert_eq!(path.split('/').count(), 41);
        // The collapse makes the twenty one `**`, so the case that holds the
        // memo puts a literal that every path segment matches between them.
        let interleaved = Pattern::new(&format!("docs/{}nomatch", "**/x/".repeat(20)));
        let all_x = format!("docs/{}file.md", "x/".repeat(39));
        let started = std::time::Instant::now();
        assert!(!pattern.matches(&path));
        assert!(!interleaved.matches(&all_x));
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }

    #[test]
    fn twenty_any_depth_segments_on_both_sides_overlap_in_bounded_time() {
        // `**/x` between the stars keeps the collapse from making this trivial.
        let interleaved = |last: &str| format!("docs/{}{last}", "**/x/".repeat(20));
        let left = Pattern::new(&interleaved("left"));
        let right = Pattern::new(&interleaved("right"));
        let started = std::time::Instant::now();
        assert!(!left.overlaps(&right));
        assert!(!Pattern::new(&many_any_depth(20, "left"))
            .overlaps(&Pattern::new(&many_any_depth(20, "right"))));
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }

    #[test]
    fn thirty_stars_in_one_segment_overlap_in_bounded_time() {
        // Every text both globs admit would end in `a` and in `b` at once.
        let left = Pattern::new(&format!("docs/{}a", "*a".repeat(30)));
        let right = Pattern::new(&format!("docs/{}b", "*b".repeat(30)));
        let started = std::time::Instant::now();
        assert!(!left.overlaps(&right));
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }
}
