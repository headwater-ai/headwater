// SPDX-License-Identifier: Apache-2.0
//! `check --path`: the findings a report prints and the strict exit, narrowed
//! to the paths a reader chose, after the run and never before it.
//!
//! An adopter keeps one folder per bounded context and asked for a report of
//! one context alone
//! ([#2101](https://github.com/headwater-ai/headwater/issues/2101)). The
//! owner's ruling 3 on that issue fixes the shape: a repeatable
//! `--path <pattern>` that narrows the printed findings and the `--strict`
//! exit, while every document is still evaluated, and the report says what
//! the filter held back.
//!
//! **Why after the run.** Every instance of every check is created and
//! decided as it is without the filter: coverage, the cache and its keys, the
//! read set and the register do not change. A rule can read a document outside
//! the filter to decide a finding inside it, as a relation does, so a filter
//! applied before the run would change a verdict and not only a report. So
//! [`narrow`] takes a finished [`Run`] and removes findings from
//! [`Run::findings`] alone. Every reading that decides an exit,
//! `Run::fails_strict` and `Run::blocking` among them, reads that list, so the
//! strict exit follows with no second code path.
//!
//! **What stays corpus-wide.** The adoption ledger, the suppression inventory,
//! the register, coverage and the read set describe the run and not one
//! context, and they are not narrowed. That reopens when an adopter reports
//! that the `not reported` block of a filtered run is noise for one context.
//!
//! **How a pattern reads.** A pattern is written relative to the corpus root,
//! as a finding path and a shelf path are, and parsed with
//! [`Pattern::new`]. A `{name}` segment is literal here, because only a shelf
//! path binds a facet. No path is normalized, so `./docs/**` matches nothing.
//! A finding on `.headwater/taxonomy.lock` is filtered as any other is, so
//! `--path docs/**` holds it back.
//!
//! **A pattern that matches nothing.** A pattern that matches no census row and
//! no finding is in [`Filtered::unmatched`], and a strict run fails on it. A
//! mistyped `--path` in a CI step would otherwise hold back every finding and
//! pass `--strict` for good, which is the reason
//! [`crate::scope_unmatched`] gives for a governed scope pattern.

use headwater_meta::pattern::Pattern;

use crate::Run;

/// What a `--path` filter did to one run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filtered {
    /// Each pattern as the command line wrote it, in that order.
    pub patterns: Vec<String>,
    /// The count of live findings the filter removed.
    pub held_back: usize,
    /// Each pattern that matches no census row path and no finding path, in
    /// command-line order.
    pub unmatched: Vec<String>,
}

impl Filtered {
    /// The text report's block: the patterns, the count held back, and each
    /// pattern that matched nothing.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "filter: {}",
            self.patterns
                .iter()
                .map(|pattern| format!("--path {pattern}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let _ = writeln!(
            out,
            "  held back: {} finding{} outside the filter",
            self.held_back,
            match self.held_back {
                1 => "",
                _ => "s",
            }
        );
        for pattern in &self.unmatched {
            let _ = writeln!(
                out,
                "  unmatched: {pattern} matches no file and no finding"
            );
        }
        out
    }
}

/// Whether any of `patterns` admits `path`.
pub fn admits(patterns: &[Pattern], path: &str) -> bool {
    patterns.iter().any(|pattern| pattern.matches(path))
}

/// Parse each pattern as a filter pattern, never as a shelf path.
pub fn parse(patterns: &[String]) -> Vec<Pattern> {
    patterns.iter().map(|source| Pattern::new(source)).collect()
}

/// `run` with only the findings a pattern admits, and [`Run::filter`] set.
///
/// An empty `patterns` is no filter: the run comes back as it went in, with
/// `filter` left as it was. `paths` are the census row paths, which decide
/// whether a pattern matched anything at all.
pub fn narrow<'a>(
    mut run: Run,
    patterns: &[String],
    paths: impl IntoIterator<Item = &'a str>,
) -> Run {
    if patterns.is_empty() {
        return run;
    }
    let parsed = parse(patterns);
    let mut hit = vec![false; parsed.len()];
    for path in paths {
        for (at, pattern) in parsed.iter().enumerate() {
            if !hit[at] && pattern.matches(path) {
                hit[at] = true;
            }
        }
    }
    for finding in &run.findings {
        for (at, pattern) in parsed.iter().enumerate() {
            if !hit[at] && pattern.matches(&finding.path) {
                hit[at] = true;
            }
        }
    }
    let before = run.findings.len();
    run.findings
        .retain(|finding| admits(&parsed, &finding.path));
    run.filter = Some(Filtered {
        patterns: patterns.to_vec(),
        held_back: before - run.findings.len(),
        unmatched: patterns
            .iter()
            .zip(&hit)
            .filter(|(_, hit)| !**hit)
            .map(|(pattern, _)| pattern.clone())
            .collect(),
    });
    run
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shelf_binding_segment_is_literal_in_a_filter() {
        let parsed = parse(&["docs/{context}/**".to_string()]);
        assert!(!admits(&parsed, "docs/billing/a.md"));
        assert!(admits(&parsed, "docs/{context}/a.md"));
    }

    #[test]
    fn no_path_is_normalized() {
        let parsed = parse(&["./docs/**".to_string()]);
        assert!(!admits(&parsed, "docs/a.md"));
    }

    #[test]
    fn the_block_counts_in_the_singular_and_the_plural() {
        let one = Filtered {
            patterns: vec!["a/**".into(), "b/**".into()],
            held_back: 1,
            unmatched: vec!["b/**".into()],
        };
        assert_eq!(
            one.render(),
            "filter: --path a/** --path b/**\n  held back: 1 finding outside the filter\n  unmatched: b/** matches no file and no finding\n"
        );
        let none = Filtered {
            held_back: 0,
            unmatched: vec![],
            ..one
        };
        assert!(none.render().contains("held back: 0 findings outside"));
    }
}
