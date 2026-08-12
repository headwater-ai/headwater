// SPDX-License-Identifier: Apache-2.0
//! The census: every file under the corpus root, and what became of it.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md) states what it is
//! for: "the census fixes the denominator for coverage before any check runs.
//! Thus a document that failed to classify is visibly unchecked, not silently
//! absent." [Spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! states the obligation it discharges: OB-COV-1, every file under the corpus
//! root is classified, or reported as unclassifiable.
//!
//! # The outcomes are a closed set, and two of them are easy to conflate
//!
//! A file with no front-matter block is an **untyped document**. It is an
//! ordinary state of a corpus: somebody has written a file that nobody has typed
//! yet, and the whole first-contact story of
//! [spec 7](../../../../docs/spec/07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy)
//! is about a corpus in that state.
//!
//! A file whose block will not load is a file the engine **could not read**.
//! That is a defect in the file, and it needs a different sentence to its
//! author. Collapsing the two lets a malformed file hide inside the untyped
//! count, and OB-COV-1..3 read that count.
//!
//! The parser draws the line already and this module keeps it:
//! `Reason::NoFrontMatter` is the first case, and every other reason is the
//! second.
//!
//! # Precedence, stated because a row has one outcome
//!
//! A file can be several things at once — excluded *and* unparseable, on no
//! shelf *and* not Markdown. One row carries one outcome, so the order is fixed
//! and it is this:
//!
//! 1. the walk itself failed (a symlink, an unreadable directory, a name that is
//!    not UTF-8);
//! 2. a declared exclusion claims the path, and the corpus's own statement about
//!    a file outranks anything the engine would work out about it;
//! 3. the file is not Markdown, so it is not a document;
//! 4. the bytes will not read as text, or the front matter will not load — a
//!    defect in the file, and it outranks every remaining outcome because it is
//!    the only one that is nobody's ordinary corpus state;
//! 5. no shelf claims the path, or two claim it equally;
//! 6. there is no front-matter block, on a path that a shelf does claim;
//! 7. kind resolution ran on the front matter, and it reports its own outcome.
//!
//! Step 5 sits above step 6 on purpose. A file with no front matter and no shelf
//! has two things wrong with it, and the shelf is the one that decides whether
//! the other matters: a path that no shelf claims is a file the taxonomy has no
//! opinion about, and telling its author to add front matter would be advice
//! about a document this corpus has not said it wants.

use crate::resolve::{self, Resolution};
use crate::shelves::Taxonomy;
use crate::walk::{self, Corpus, EntryKind};
use headwater_doc::{ParseError, Reason};

/// Every file under the corpus root, in path order.
#[derive(Clone, Debug)]
pub struct Census {
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Row {
    /// Relative to the repository root, with `/` separators.
    pub path: String,
    pub outcome: Outcome,
}

/// What became of one file. Closed, and matched exhaustively everywhere.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// A kind, and the derivation that produced it.
    Typed {
        kind: String,
        derivation: Box<Resolution>,
    },
    /// Read, and carrying no kind.
    Untyped(Untyped),
    /// The engine could not read the file.
    Unreadable(Unreadable),
    /// A declared exclusion claims the path, and states why.
    Excluded { pattern: String, reason: String },
    /// Not a Markdown file. A document is a Markdown file that opens with a
    /// block of facets, so this file is not one.
    NotADocument,
    /// The walk reached the name and could not turn it into a file to read.
    Unwalkable(Unwalkable),
}

#[derive(Clone, Debug)]
pub enum Untyped {
    /// No `---` block. An ordinary corpus state, and never a defect on its own.
    NoFrontMatter,
    /// Resolution ran and stopped, and it says where.
    Unresolved {
        reason: resolve::Untyped,
        derivation: Box<Resolution>,
    },
}

#[derive(Clone, Debug)]
pub enum Unreadable {
    /// The bytes are not UTF-8.
    NotText,
    /// The front-matter block would not load.
    FrontMatter(Vec<ParseError>),
    /// The file would not open.
    Io(String),
}

#[derive(Clone, Debug)]
pub enum Unwalkable {
    /// A symlink, which the walk never follows. See [`crate::walk`].
    Symlink { target: String },
    /// A directory whose contents are missing from this census.
    UnreadableDirectory { error: String },
    /// A name that is not UTF-8, so no pattern can match it.
    UnreadableName,
}

/// How much of the census to write out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    /// Every row. Right for a fixture tree, whose whole point is every row.
    EveryRow,
    /// The totals, and every row that is not a typed document.
    ///
    /// A real corpus adds a typed document most weeks, and a recorded file that
    /// changes on every commit is a file that nobody reads. The totals still
    /// account for every file, so a shrinking denominator is still visible —
    /// which is the property that matters.
    Exceptions,
}

/// Take the census.
pub fn take(corpus: &Corpus, taxonomy: &Taxonomy) -> Census {
    let rows = walk::walk(corpus)
        .into_iter()
        .map(|entry| Row {
            outcome: outcome_of(&entry, taxonomy),
            path: entry.path,
        })
        .collect();
    Census { rows }
}

fn outcome_of(entry: &walk::Entry, taxonomy: &Taxonomy) -> Outcome {
    match &entry.kind {
        EntryKind::Symlink { target } => {
            return Outcome::Unwalkable(Unwalkable::Symlink {
                target: target.clone(),
            })
        }
        EntryKind::UnreadableDirectory { error } => {
            return Outcome::Unwalkable(Unwalkable::UnreadableDirectory {
                error: error.clone(),
            })
        }
        EntryKind::UnreadableName => return Outcome::Unwalkable(Unwalkable::UnreadableName),
        EntryKind::File => {}
    }

    if let Some(exclusion) = &entry.excluded_by {
        return Outcome::Excluded {
            pattern: exclusion.pattern.source().to_string(),
            reason: exclusion.reason.clone(),
        };
    }

    // The extension test is exact and lower case. A `.MD` file is a row that
    // says "not a document", which is a visible statement an author can act on;
    // a case-insensitive test would be a rule that behaves differently on two
    // filesystems, and the census would then disagree with itself across
    // machines.
    if !entry.path.ends_with(".md") {
        return Outcome::NotADocument;
    }

    let source = match std::fs::read(&entry.on_disk) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => return Outcome::Unreadable(Unreadable::NotText),
        },
        Err(error) => return Outcome::Unreadable(Unreadable::Io(error.to_string())),
    };

    // The one distinction this module exists to keep: a file with no block is
    // an untyped document, and a file whose block will not load is a file the
    // engine could not read.
    let parsed = match headwater_doc::parse(&source) {
        Ok(document) => Some(document),
        Err(errors) if errors.iter().any(|e| e.reason == Reason::NoFrontMatter) => None,
        Err(errors) => return Outcome::Unreadable(Unreadable::FrontMatter(errors)),
    };

    let Some(document) = parsed else {
        // Step 1 needs a path and nothing else, so it still runs.
        return match resolve::shelf_for(&entry.path, taxonomy) {
            resolve::ShelfMatch::Matched { .. } => Outcome::Untyped(Untyped::NoFrontMatter),
            resolve::ShelfMatch::Stopped(stop) => Outcome::Untyped(Untyped::Unresolved {
                reason: stop.reason.clone(),
                derivation: Box::new(stop.into_resolution()),
            }),
        };
    };

    let resolution = resolve::resolve(&entry.path, &document.facets, taxonomy);
    match &resolution.outcome {
        resolve::Outcome::Typed(kind) => Outcome::Typed {
            kind: kind.clone(),
            derivation: Box::new(resolution.clone()),
        },
        resolve::Outcome::Untyped(reason) => Outcome::Untyped(Untyped::Unresolved {
            reason: reason.clone(),
            derivation: Box::new(resolution.clone()),
        }),
    }
}

impl Outcome {
    /// The class this outcome counts under. One word per class, and the set is
    /// the census's own vocabulary: a report that invents a class per message
    /// cannot be counted.
    pub fn class(&self) -> &'static str {
        match self {
            Outcome::Typed { .. } => "typed",
            Outcome::Untyped(_) => "untyped",
            Outcome::Unreadable(_) => "unreadable",
            Outcome::Excluded { .. } => "excluded",
            Outcome::NotADocument => "not a document",
            Outcome::Unwalkable(_) => "unwalkable",
        }
    }

    /// The rest of the row: what this outcome says about this file.
    pub fn detail(&self) -> String {
        match self {
            Outcome::Typed { kind, .. } => kind.clone(),
            Outcome::Untyped(Untyped::NoFrontMatter) => {
                "no front matter, so nobody has typed this file".to_string()
            }
            Outcome::Untyped(Untyped::Unresolved { reason, .. }) => reason.to_string(),
            Outcome::Unreadable(Unreadable::NotText) => "the bytes are not UTF-8".to_string(),
            Outcome::Unreadable(Unreadable::FrontMatter(errors)) => errors
                .iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; "),
            Outcome::Unreadable(Unreadable::Io(error)) => error.clone(),
            // The pattern only. The reason is prose, it is the same prose on
            // every file the rule covers, and a report that repeats it once per
            // row is a report nobody finishes reading. [`Census::render`] prints
            // each reason once, above the rows.
            Outcome::Excluded { pattern, .. } => pattern.clone(),
            Outcome::NotADocument => "not a Markdown file".to_string(),
            Outcome::Unwalkable(Unwalkable::Symlink { target }) => {
                format!("a symlink to `{target}`, which the walk does not follow")
            }
            Outcome::Unwalkable(Unwalkable::UnreadableDirectory { error }) => {
                format!("a directory this walk could not read: {error}")
            }
            Outcome::Unwalkable(Unwalkable::UnreadableName) => {
                "a name that is not UTF-8, so no pattern can match it".to_string()
            }
        }
    }
}

impl Census {
    /// Counts per outcome class, in the order the classes are declared, so that
    /// two reports line up column by column.
    pub fn counts(&self) -> Vec<(&'static str, usize)> {
        const CLASSES: [&str; 6] = [
            "typed",
            "untyped",
            "unreadable",
            "excluded",
            "not a document",
            "unwalkable",
        ];
        CLASSES
            .iter()
            .map(|class| {
                (
                    *class,
                    self.rows
                        .iter()
                        .filter(|r| r.outcome.class() == *class)
                        .count(),
                )
            })
            .collect()
    }

    /// Counts per resolved kind, in name order.
    pub fn kinds(&self) -> Vec<(String, usize)> {
        let mut names: Vec<&str> = self
            .rows
            .iter()
            .filter_map(|row| match &row.outcome {
                Outcome::Typed { kind, .. } => Some(kind.as_str()),
                _ => None,
            })
            .collect();
        names.sort_unstable();
        let mut counted: Vec<(String, usize)> = Vec::new();
        for name in names {
            match counted.last_mut() {
                Some((last, count)) if last == name => *count += 1,
                _ => counted.push((name.to_string(), 1)),
            }
        }
        counted
    }

    /// Each declared exclusion that claimed at least one file, in pattern order,
    /// with its reason and the number of files it covers.
    pub fn exclusions(&self) -> Vec<(String, String, usize)> {
        let mut found: Vec<(String, String, usize)> = Vec::new();
        for row in &self.rows {
            let Outcome::Excluded { pattern, reason } = &row.outcome else {
                continue;
            };
            match found.iter_mut().find(|(known, _, _)| known == pattern) {
                Some((_, _, count)) => *count += 1,
                None => found.push((pattern.clone(), reason.clone(), 1)),
            }
        }
        found.sort_by(|a, b| a.0.cmp(&b.0));
        found
    }

    /// The census as text.
    ///
    /// The totals come first and they account for every row, whatever `detail`
    /// then prints. That order is the point: a reader who checks nothing else
    /// can still see the denominator.
    pub fn render(&self, detail: Detail) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{} files under the corpus root\n",
            self.rows.len()
        ));
        for (class, count) in self.counts() {
            if count > 0 {
                out.push_str(&format!("  {count:5} {class}\n"));
            }
        }
        for (kind, count) in self.kinds() {
            out.push_str(&format!("  {count:5} typed {kind}\n"));
        }

        // Each exclusion once, with the count it covers. A rule that covers
        // more of the corpus than its author expected is visible here and
        // nowhere else in the report.
        let exclusions = self.exclusions();
        if !exclusions.is_empty() {
            out.push_str("\nexclusions\n");
            for (pattern, reason, count) in exclusions {
                out.push_str(&format!("  {count:5} `{pattern}`\n        {reason}\n"));
            }
        }

        let rows: Vec<&Row> = match detail {
            Detail::EveryRow => self.rows.iter().collect(),
            Detail::Exceptions => self
                .rows
                .iter()
                .filter(|row| !matches!(row.outcome, Outcome::Typed { .. }))
                .collect(),
        };
        if !rows.is_empty() {
            out.push('\n');
            for row in rows {
                out.push_str(&format!(
                    "{}\n  {}: {}\n",
                    row.path,
                    row.outcome.class(),
                    row.outcome.detail()
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::walk::Exclusion;
    use std::path::Path;

    fn fixtures() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    fn taxonomy() -> Taxonomy {
        let source = std::fs::read_to_string(fixtures().join("walk.taxonomy.yml"))
            .expect("the walk taxonomy");
        let root = headwater_yaml::load(&source).expect("it loads");
        Taxonomy::read(root.value.as_map().expect("a mapping")).expect("it reads")
    }

    fn corpus() -> Corpus {
        Corpus::new(fixtures(), "walk").excluding(vec![Exclusion::new(
            "walk/excluded/**",
            "a declared exclusion, so that the fixture tree has one",
        )])
    }

    #[test]
    fn every_row_falls_in_exactly_one_class() {
        let census = take(&corpus(), &taxonomy());
        let counted: usize = census.counts().iter().map(|(_, n)| n).sum();
        assert_eq!(
            counted,
            census.rows.len(),
            "a row fell outside the closed set, so the denominator is wrong"
        );
    }

    #[test]
    fn a_file_with_no_front_matter_is_untyped_and_not_unreadable() {
        let census = take(&corpus(), &taxonomy());
        let row = census
            .rows
            .iter()
            .find(|row| row.path.ends_with("no-front-matter.md"))
            .expect("the fixture");
        assert!(
            matches!(row.outcome, Outcome::Untyped(Untyped::NoFrontMatter)),
            "{:?}",
            row.outcome
        );
    }

    #[test]
    fn a_block_that_will_not_load_is_unreadable_and_not_untyped() {
        let census = take(&corpus(), &taxonomy());
        let row = census
            .rows
            .iter()
            .find(|row| row.path == "walk/spec/unterminated.md")
            .expect("the fixture");
        assert!(
            matches!(row.outcome, Outcome::Unreadable(Unreadable::FrontMatter(_))),
            "{:?}",
            row.outcome
        );
    }

    #[test]
    fn a_declared_exclusion_outranks_what_the_engine_would_work_out() {
        // The same broken document, once on a shelf and once under an
        // exclusion. The corpus's own statement about a file wins, and the row
        // says which rule made the statement.
        let census = take(&corpus(), &taxonomy());
        let row = census
            .rows
            .iter()
            .find(|row| row.path == "walk/excluded/unterminated.md")
            .expect("the fixture");
        assert!(
            matches!(row.outcome, Outcome::Excluded { .. }),
            "{:?}",
            row.outcome
        );
    }
}
