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
//! # A file this engine wrote is a third thing, and it is none of the six
//!
//! A projection lands inside the corpus root — a shelf index has to land on its
//! shelf — so the walk reaches it like any other file. It carries no front
//! matter, because its first line is the generated-file marker, so the six
//! outcomes above would report it as an untyped document of the shelf it landed
//! on, against every contract that shelf's kind requires. The artifact this
//! engine wrote would become a finding against the corpus, on every run.
//!
//! None of the three outcomes an author could act on is true of it.
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#lifecycle)
//! exempts a generated projection from acceptance and holds it to regeneration
//! instead, so `untyped` is wrong: nobody is going to type it. `excluded` is
//! wrong: no declaration excludes it, and the corpus root is where it belongs.
//! `typed` is wrong twice over, because it would then be routed to checks whose
//! findings name the wrong author — the content of a generated file is a
//! function of the emitter, and an author cannot fix it in the file.
//!
//! So a marked file gets a row of its own, with the reason on it, in the way an
//! excluded path does. **The census accounts for it and judges nothing about
//! it**, which is the division this module keeps everywhere: `headwater generate
//! --check` holds a generated file to the bytes its emitter produces now, and
//! that verb is the only one that reports drift in one. Two reports of one fact
//! send its author to two places.
//!
//! # A generated file can still be a node, and the reason above says nothing
//! # against it
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#projections) gives
//! one reason no check reads a generated document: its content is a function of
//! the emitter, and an author cannot repair it in the file. **That reason is
//! about checks. It says nothing about identity.** The identifier and the kind
//! of a generated file are a function of the declaration that writes it, and a
//! wrong one is repaired in the declaration, which is what `generate --check`
//! already holds.
//!
//! The consequence of reading it as a statement about identity was that a
//! projection could only write a file that no document points at. The one
//! property that makes an artifact worth generating, that many documents depend
//! on it, was the property that made it ungeneratable.
//!
//! So a marked file that carries front matter keeps its own outcome **and**
//! carries what it parsed. [`Outcome::Generated`] holds the projection kind the
//! marker claims and, separately, the document kind the block resolved.
//! [`Outcome::node`] is what the identifier index and the edge builder read, and
//! it is the one definition of "this row is a node of the graph".
//!
//! Nothing about the check side moves. A generated row is not
//! [`Outcome::Typed`], no check instantiates over it, and the coverage account
//! counts it under `generated` as before.
//!
//! The marker is a **claim**, and this module cannot test it: testing it needs
//! the projection declarations, which the census does not read. The generator
//! tests it, by reading this census beside its own plan and reporting a marked
//! file that no declaration writes. That test is what stops the marker from
//! being a line an author can add to exempt a document from every check.
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
//! 4. the bytes will not read as text;
//! 5. the first line marks the file as this engine's own output;
//! 6. the front matter will not load — a defect in the file, and it outranks
//!    every remaining outcome because it is the only one that is nobody's
//!    ordinary corpus state;
//! 7. the front-matter block carries the marker as a member, which is where a
//!    generated Markdown document that declares an identity puts it;
//! 8. no shelf claims the path, or two claim it equally;
//! 9. there is no front-matter block, on a path that a shelf does claim;
//! 10. kind resolution ran on the front matter, and it reports its own outcome.
//!
//! Step 7 sits where step 5 does, and for the same reason: a statement about
//! who wrote a file outranks what the engine would work out from its content.
//! It sits below step 6 because a block that will not load is a block whose
//! members nobody can read, so the question it asks has no answer there. Kind
//! resolution still runs for a row that reaches step 7, because a node needs a
//! kind, and its outcome rides on the row rather than replacing it.
//!
//! Step 7 sits above step 8 on purpose. A file with no front matter and no shelf
//! has two things wrong with it, and the shelf is the one that decides whether
//! the other matters: a path that no shelf claims is a file the taxonomy has no
//! opinion about, and telling its author to add front matter would be advice
//! about a document this corpus has not said it wants.
//!
//! Step 5 sits above step 6 for the reason step 2 sits where it does: a
//! statement about who wrote a file outranks what the engine would work out from
//! its content. The two cases are in fact disjoint, because a marker on the
//! first line is a first line that is not `---`, so such a file has no block to
//! fail to load. The order is fixed anyway, because a row has one outcome and a
//! reader should not have to derive which.
//!
//! **Step 3 sits above step 5, and that is a boundary rather than an oversight.**
//! A generated file in a format that is not Markdown is already accounted for as
//! `not a document`, which is true of it and reports nothing against it. Moving
//! the marker test above the extension test would mean reading every file under
//! the corpus root — including every image and every archive — to ask a question
//! whose answer changes no verdict.

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
    /// The document this row read, when it read one.
    ///
    /// The census is the denominator, and the graph build is the next phase
    /// over it ([spec 6](../../../../docs/spec/06-engine-architecture.md)). A
    /// phase that re-opened each file would read a corpus that the census had
    /// already read, and the two accounts could then differ — by an edit
    /// between the two passes, or by one rule drifting from the other. So the
    /// row carries what it parsed rather than the path to parse again.
    ///
    /// It is `None` where nothing parsed: an excluded file, a file that is not
    /// Markdown, an unwalkable entry, and a file the engine could not read.
    pub document: Option<Box<headwater_doc::Document>>,
    /// The digest of the bytes this row read, and `None` where it read none.
    ///
    /// This is the content hash that
    /// [spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
    /// puts in a cache key and in the read set of a run: "the content hash of
    /// every document and edge that an instance read". It is taken here, of
    /// the bytes this walk read, for the reason the row carries the document
    /// it parsed. A phase that opened the file again to hash it could hash a
    /// different file from the one that was checked, and the cache key would
    /// then name a state that no run evaluated.
    ///
    /// It is over the bytes rather than over the parsed document, because the
    /// bytes are the input. A digest over a parse would need a canonical form
    /// of the parse, and two documents that differ only where the parser
    /// discards would then share a key.
    pub digest: Option<String>,
}

/// What became of one file. Closed, and matched exhaustively everywhere.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// A kind, and the derivation that produced it.
    Typed {
        kind: String,
        derivation: Box<Resolution>,
    },
    /// This engine wrote the file, and the file says so.
    Generated {
        /// The projection kind the marker names, and it is **what the file
        /// claims rather than what is true**: anything can write the marker,
        /// and nothing here reads the projection declarations to test the
        /// claim. It is `None` where the marker names nothing, which is what a
        /// marker somebody wrote by hand usually looks like.
        projection: Option<String>,
        /// The document kind the front matter resolved, for a generated file
        /// that declares an identity. `None` for one that declares no front
        /// matter, which every projection that writes a list produces.
        ///
        /// This is the half that makes a generated file a node. It is a second
        /// field rather than a second outcome, because the two facts are
        /// independent: the marker says who wrote the file and the block says
        /// what the file is.
        kind: Option<String>,
        /// The derivation that produced `kind`, which a finding about an edge
        /// of this document anchors to.
        derivation: Option<Box<Resolution>>,
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
        .map(|entry| {
            let read = outcome_of(&entry, taxonomy);
            Row {
                outcome: read.outcome,
                document: read.document,
                digest: read.digest,
                path: entry.path,
            }
        })
        .collect();
    Census { rows }
}

/// What one file gave this walk: an outcome, the document if it parsed, and
/// the digest of the bytes if any were read.
struct Read {
    outcome: Outcome,
    document: Option<Box<headwater_doc::Document>>,
    digest: Option<String>,
}

fn outcome_of(entry: &walk::Entry, taxonomy: &Taxonomy) -> Read {
    match &entry.kind {
        EntryKind::Symlink { target } => {
            return unread(Outcome::Unwalkable(Unwalkable::Symlink {
                target: target.clone(),
            }))
        }
        EntryKind::UnreadableDirectory { error } => {
            return unread(Outcome::Unwalkable(Unwalkable::UnreadableDirectory {
                error: error.clone(),
            }))
        }
        EntryKind::UnreadableName => {
            return unread(Outcome::Unwalkable(Unwalkable::UnreadableName))
        }
        EntryKind::File => {}
    }

    if let Some(exclusion) = &entry.excluded_by {
        return unread(Outcome::Excluded {
            pattern: exclusion.pattern.source().to_string(),
            reason: exclusion.reason.clone(),
        });
    }

    // The extension test is exact and lower case. A `.MD` file is a row that
    // says "not a document", which is a visible statement an author can act on;
    // a case-insensitive test would be a rule that behaves differently on two
    // filesystems, and the census would then disagree with itself across
    // machines.
    if !entry.path.ends_with(".md") {
        return unread(Outcome::NotADocument);
    }

    let bytes = match std::fs::read(&entry.on_disk) {
        Ok(bytes) => bytes,
        Err(error) => return unread(Outcome::Unreadable(Unreadable::Io(error.to_string()))),
    };
    // One read of the file, and the digest of exactly those bytes. Everything
    // after this line works on what is already in hand.
    let digest = headwater_hash::digest(&bytes);
    let Ok(source) = String::from_utf8(bytes) else {
        return hashed(Outcome::Unreadable(Unreadable::NotText), digest);
    };

    // Step 5. Before the parse, because a marked file has no front matter to
    // parse and because who wrote a file outranks what its content would say.
    // The predicate is `headwater-mark`'s and not this module's: the generator
    // writes the marker, this reads it, and a second copy of the rule here is
    // how the two would come to disagree about which files this engine wrote.
    if headwater_mark::carries_marker(&entry.path, &source) && !marked_in_front_matter(&source) {
        return hashed(
            Outcome::Generated {
                projection: headwater_mark::kind_named(&entry.path, &source),
                kind: None,
                derivation: None,
            },
            digest,
        );
    }

    // The one distinction this module exists to keep: a file with no block is
    // an untyped document, and a file whose block will not load is a file the
    // engine could not read.
    let parsed = match headwater_doc::parse(&source) {
        Ok(document) => Some(document),
        Err(errors) => match errors.iter().any(|e| e.reason == Reason::NoFrontMatter) {
            true => None,
            false => return hashed(Outcome::Unreadable(Unreadable::FrontMatter(errors)), digest),
        },
    };

    let Some(document) = parsed else {
        // Step 1 needs a path and nothing else, so it still runs.
        return hashed(
            match resolve::shelf_for(&entry.path, taxonomy) {
                resolve::ShelfMatch::Matched { .. } => Outcome::Untyped(Untyped::NoFrontMatter),
                resolve::ShelfMatch::Stopped(stop) => Outcome::Untyped(Untyped::Unresolved {
                    reason: stop.reason.clone(),
                    derivation: Box::new(stop.into_resolution()),
                }),
            },
            digest,
        );
    };

    let resolution = resolve::resolve(&entry.path, &document.facets, taxonomy);
    // Step 7. The block loaded, so its members are readable, and the marker is
    // one of them. Kind resolution ran first because a node needs a kind, and
    // its outcome rides on the row rather than replacing it.
    let outcome = match headwater_mark::carries_marker(&entry.path, &source) {
        true => Outcome::Generated {
            projection: headwater_mark::kind_named(&entry.path, &source),
            kind: match &resolution.outcome {
                resolve::Outcome::Typed(kind) => Some(kind.clone()),
                resolve::Outcome::Untyped(_) => None,
            },
            derivation: Some(Box::new(resolution.clone())),
        },
        false => match &resolution.outcome {
            resolve::Outcome::Typed(kind) => Outcome::Typed {
                kind: kind.clone(),
                derivation: Box::new(resolution.clone()),
            },
            resolve::Outcome::Untyped(reason) => Outcome::Untyped(Untyped::Unresolved {
                reason: reason.clone(),
                derivation: Box::new(resolution.clone()),
            }),
        },
    };
    Read {
        outcome,
        document: Some(Box::new(document)),
        digest: Some(digest),
    }
}

/// Whether the marker sits inside a front-matter block rather than above one.
///
/// The two positions produce two different rows: a marker on the first line is
/// a file with no block to parse, and a marker inside the block is a file whose
/// block this census reads. `headwater_mark` owns both rules and answers one
/// question about them, so this asks it the question twice rather than stating
/// a third rule of its own: a file whose first line carries the marker is the
/// first case, and a marked file whose first line does not is the second.
fn marked_in_front_matter(source: &str) -> bool {
    // The path is a Markdown one by the time this runs, and only a Markdown
    // path admits both positions. The name is fixed here rather than taken,
    // because the answer is about the first line and not about the file.
    !headwater_mark::carries_marker("a.md", source.lines().next().unwrap_or_default())
}

/// An outcome the walk reached without reading a byte of the file.
fn unread(outcome: Outcome) -> Read {
    Read {
        outcome,
        document: None,
        digest: None,
    }
}

/// An outcome over bytes that were read and did not become a document.
fn hashed(outcome: Outcome, digest: String) -> Read {
    Read {
        outcome,
        document: None,
        digest: Some(digest),
    }
}

impl Outcome {
    /// The kind this row resolved and the derivation that produced it, for a
    /// row that the graph may hold as a node.
    ///
    /// One definition, read by the identifier index and by the edge builder. A
    /// second reading of "which rows are nodes" is how the two would come to
    /// disagree, and a graph whose node set and edge set disagree reports a
    /// correct verdict about a corpus that nobody has.
    ///
    /// Two outcomes answer. A typed document is the ordinary one. A generated
    /// document that declared an identity is the other, and the reason it
    /// answers is that spec 6 excuses it from *checks* rather than from
    /// identity. See the module comment.
    pub fn node(&self) -> Option<(&str, Option<&Resolution>)> {
        match self {
            Outcome::Typed { kind, derivation } => Some((kind, Some(derivation))),
            Outcome::Generated {
                kind: Some(kind),
                derivation,
                ..
            } => Some((kind, derivation.as_deref())),
            _ => None,
        }
    }

    /// The class this outcome counts under. One word per class, and the set is
    /// the census's own vocabulary: a report that invents a class per message
    /// cannot be counted.
    pub fn class(&self) -> &'static str {
        match self {
            Outcome::Typed { .. } => "typed",
            Outcome::Generated { .. } => "generated",
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
            // What holds the file, rather than what this row will not do to it.
            // A reader who finds a generated file in a report is asking which
            // verb owns it, and the answer is the same for every one of them.
            Outcome::Generated { projection, .. } => match projection {
                Some(kind) => format!(
                    "`{kind}`, and `headwater generate --check` holds it rather than this census"
                ),
                None => "`headwater generate --check` holds it rather than this census".to_string(),
            },
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
        const CLASSES: [&str; 7] = [
            "typed",
            "generated",
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
    fn a_row_carries_the_document_it_read_and_no_other_row_carries_one() {
        // The graph build reads front matter and prose links off these rows.
        // A phase that opened the file again would be a second read of one
        // corpus, and two reads can disagree.
        let census = take(&corpus(), &taxonomy());
        for row in &census.rows {
            let parsed = row.document.is_some();
            match &row.outcome {
                Outcome::Typed { .. } => assert!(parsed, "{} is typed and empty", row.path),
                Outcome::Untyped(Untyped::NoFrontMatter) => {
                    assert!(!parsed, "{} has no block to carry", row.path);
                }
                Outcome::Excluded { .. }
                | Outcome::NotADocument
                | Outcome::Unwalkable(_)
                | Outcome::Unreadable(_) => {
                    assert!(!parsed, "{} carries a document nothing read", row.path);
                }
                // A generated row carries a document exactly when it declared
                // one. The kind rides on the document rather than on the
                // marker, so a marked file with no block resolves none, and a
                // resolved kind is what puts the row on the node shelf.
                Outcome::Generated { kind, .. } => {
                    assert!(
                        kind.is_none() || parsed,
                        "{} resolved a kind from a block nothing read",
                        row.path
                    );
                    assert_eq!(
                        kind.is_some(),
                        row.outcome.node().is_some(),
                        "{} disagrees with itself about being a node",
                        row.path
                    );
                }
                Outcome::Untyped(Untyped::Unresolved { .. }) => {}
            }
        }
    }

    /// A row carries a digest exactly when the walk read the file.
    ///
    /// The check layer keys a cache on this number, and a row that carried one
    /// without reading bytes would key a result on a hash of nothing. A row
    /// that read bytes and dropped the digest costs the other way: every
    /// instance over it is a permanent cache miss.
    #[test]
    fn a_row_carries_a_digest_exactly_when_it_read_the_file() {
        let census = take(&corpus(), &taxonomy());
        for row in &census.rows {
            let read = !matches!(
                &row.outcome,
                Outcome::Excluded { .. }
                    | Outcome::NotADocument
                    | Outcome::Unwalkable(_)
                    | Outcome::Unreadable(Unreadable::Io(_))
            );
            assert_eq!(
                row.digest.is_some(),
                read,
                "{} is {} and its digest is {:?}",
                row.path,
                row.outcome.class(),
                row.digest
            );
        }

        // And it is the digest of the file, rather than of anything the census
        // derived from it.
        let row = census
            .rows
            .iter()
            .find(|row| row.path == "walk/spec/typed-by-discriminator.md")
            .expect("the fixture");
        let bytes = std::fs::read(fixtures().join(&row.path)).expect("the fixture reads");
        assert_eq!(row.digest, Some(headwater_hash::digest(&bytes)));
    }

    /// The outcome this module gained, on the file that motivated it.
    ///
    /// `walk/spec/generated-index.md` sits on a shelf whose kind is chosen by a
    /// discriminator, and it carries no front matter, no identifier and no
    /// section. Every one of those is a contract that shelf's kind requires. It
    /// is a generated row rather than an untyped one, so nothing is reported
    /// against it and nobody is told to type it.
    #[test]
    fn a_marked_file_on_a_shelf_is_generated_and_not_untyped() {
        let census = take(&corpus(), &taxonomy());
        let row = census
            .rows
            .iter()
            .find(|row| row.path == "walk/spec/generated-index.md")
            .expect("the fixture");
        assert!(
            matches!(row.outcome, Outcome::Generated { .. }),
            "{:?}",
            row.outcome
        );
        // The shelf does claim the path, which is what makes the row load
        // bearing: the alternative outcome was available and was not taken.
        assert!(matches!(
            resolve::shelf_for(&row.path, &taxonomy()),
            resolve::ShelfMatch::Matched { .. }
        ));
        let Outcome::Generated {
            projection,
            kind,
            derivation,
        } = &row.outcome
        else {
            unreachable!()
        };
        assert_eq!(projection.as_deref(), Some("shelf_index"));
        // No block, so no kind and no derivation, and so no node. This is the
        // shape every projection that writes a list produces.
        assert_eq!(kind.as_deref(), None);
        assert!(derivation.is_none());
        assert!(row.outcome.node().is_none());
        assert!(row.document.is_none());
    }

    /// A generated file that declares an identity is generated **and** a node.
    ///
    /// The two facts are independent. The marker says who wrote the file, and
    /// [spec 6](../../../../docs/spec/06-engine-architecture.md#projections)
    /// gives one consequence of it: no check reads the file, because an author
    /// cannot repair its content there. The block says what the file is, and
    /// the graph reads that. A reading of the first as an answer to the second
    /// is what left a projection able to write only a file that nothing cites.
    #[test]
    fn a_marked_file_that_declares_an_identity_is_generated_and_a_node() {
        let census = take(&corpus(), &taxonomy());
        let row = census
            .rows
            .iter()
            .find(|row| row.path == "walk/spec/generated-register.md")
            .expect("the fixture");

        let Outcome::Generated {
            projection,
            kind,
            derivation,
        } = &row.outcome
        else {
            panic!("{:?}", row.outcome)
        };
        assert_eq!(projection.as_deref(), Some("shelf_sections"));
        assert_eq!(kind.as_deref(), Some("decision_register"));
        assert!(derivation.is_some());

        // The row carries what it parsed, which is what the index and the edge
        // builder read. Before this, a marked row carried no document at all.
        assert!(row.document.is_some());
        assert_eq!(
            row.outcome.node().map(|(kind, _)| kind),
            Some("decision_register")
        );

        // And it is still not typed, so no check instantiates over it and the
        // coverage account counts it where it counted it before.
        assert!(!matches!(row.outcome, Outcome::Typed { .. }));
        assert_eq!(row.outcome.class(), "generated");
    }

    /// The false positive that would cost the most.
    ///
    /// A document that names the marker in its prose is an authored document,
    /// and this repository's own specification is one. Classifying it as
    /// generated would drop it out of every check silently, which is the shape
    /// of defect [spec 4](../../../../docs/spec/04-assurance-model.md) exists to
    /// prevent.
    #[test]
    fn a_document_that_quotes_the_marker_below_the_first_line_stays_typed() {
        let census = take(&corpus(), &taxonomy());
        let row = census
            .rows
            .iter()
            .find(|row| row.path == "walk/spec/quotes-the-marker.md")
            .expect("the fixture");
        assert!(
            matches!(&row.outcome, Outcome::Typed { kind, .. } if kind == "design_spec"),
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
