// SPDX-License-Identifier: Apache-2.0
//! Which files of this repository a producer writes, computed from the producers.
//!
//! # The defect this answers
//!
//! [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
//! rules that an artifact holding a fold declares `merge=headwater-regenerate`.
//! Which artifacts those are was answered by hand every time it was asked, and
//! every hand answer has been wrong. It was published as five and measured
//! seven, answered as three and as seventeen in one afternoon, and
//! [#676](https://github.com/headwater-ai/headwater/issues/676) was filed on
//! that measurement. While the issue sat open the same defect re-instantiated
//! itself: the decision record's own consequence clause said six, the merged
//! tree gate's header said twenty-one, and `.gitattributes` held twenty-four.
//! Three hand statements of one population, all in one tree, none agreeing.
//!
//! So this module holds no list. Each producer of this repository states the
//! files it writes by a rule of its own, and the population is the union over
//! those rules. A producer output added to a tree changes the answer with no
//! edit here.
//!
//! # The four producers, and each one's own rule
//!
//! | producer | its rule |
//! |---|---|
//! | `headwater generate` | the file carries the generated-file marker, which is [`headwater_mark::carries_marker`] and the same predicate the verb refuses to overwrite on |
//! | `headwater taxonomy resolve` | the lock path, which the verb writes and nothing else does |
//! | `tools/site/refresh-figures.sh` | a page under `site/` carrying a `data-figure` element, which is the set the script substitutes into |
//! | a recorded corpus fixture | a `corpus.*` fixture of an engine crate whose opening states a fold: a count over the corpus, or a digest over the whole canonical text |
//!
//! The fourth is the one that needs its rule stated, because most recorded
//! fixtures are **not** members. HW-DR-0049 decomposed `corpus.census` and
//! `corpus.graph` into one record per entity precisely so that they merge, and
//! a decomposed artifact must not declare the driver. What separates them is
//! the shape of the artifact rather than the identity of its producer, so the
//! rule reads the artifact's opening rather than a list of file names.
//!
//! # What a fixture tree is not
//!
//! A file under a `fixtures/` directory that carries the marker belongs to
//! another corpus, rooted at that fixture tree, written by a run this
//! repository's producers never make. The marker rule therefore stops at a
//! `fixtures/` component. The recorded-fold rule reaches into `fixtures/` on
//! purpose, because a recorded fixture is an output of *this* repository's test
//! run rather than a document of a nested corpus.
//!
//! # The shape of a record, which is computed for the same reason
//!
//! [*The shapes a record takes*](../../../../docs/evaluations/what-a-check-can-know.md#the-shapes-a-record-takes)
//! states five shapes and the treatment each one takes at a merge. Four of them
//! name a file. [#809](https://github.com/headwater-ai/headwater/issues/809)
//! asked for a facet that declares the shape of each member, and no member can
//! carry one: most are generated documents, which carry no front matter at all,
//! and the rest are a lock, three pages, two recorded fixtures and two stores of
//! readings, none of which is a document. A path-to-shape table would be this
//! population written by hand under a second name, which is the defect above.
//!
//! So a shape is computed, exactly as membership is: from the structure of the
//! artifact, and from the rule of the producer that writes it. [`Shape`] carries
//! the three rules and [`Shape::row`] carries the evaluation's own words for
//! each one.
//!
//! # Why the reported set is wider than the population
//!
//! The population is what a producer writes, and three of the four file-shaped
//! rows have no member in it. The two append-only stores are written by no
//! producer at all. The two decomposed recorded fixtures are excluded by the
//! fold rule, which is the point of the fold rule. A report that covered the
//! population alone would leave three rows with no instance forever, and a row
//! with no instance stops being read in silence.
//!
//! [`Population::members`] therefore covers the population, every path
//! `.gitattributes` gives a merge attribute, and every decomposed recorded
//! fixture. [`Member::disagreement`] holds each member's shape against the merge
//! attribute it carries, and names what the merge costs where the two disagree.

use std::path::{Path, PathBuf};

/// A thing that writes files of this repository, and the command that reruns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Producer {
    /// The projection layer, enumerable by the marker it writes.
    Generate,
    /// The taxonomy resolver, which writes the committed lock.
    TaxonomyResolve,
    /// The figure substitution over the hand-built pages.
    FigureRefresh,
    /// A recorded fixture of an engine crate that states a fold.
    RecordedFold,
}

impl Producer {
    /// What to run to rewrite this producer's outputs.
    pub fn command(self) -> &'static str {
        match self {
            Producer::Generate => "headwater generate",
            Producer::TaxonomyResolve => "headwater taxonomy resolve",
            Producer::FigureRefresh => "sh tools/site/refresh-figures.sh",
            Producer::RecordedFold => "HEADWATER_BLESS=1 cargo test",
        }
    }

    /// The rule by which this producer's output set is enumerated.
    pub fn rule(self) -> &'static str {
        match self {
            Producer::Generate => "carries the generated-file marker",
            Producer::TaxonomyResolve => "is the committed taxonomy lock",
            Producer::FigureRefresh => "is a page under site/ carrying a data-figure element",
            Producer::RecordedFold => "is a recorded corpus fixture whose opening states a fold",
        }
    }
}

/// The shape of a record, which decides whether a merge of it is sound.
///
/// The variants are the rows of [*The shapes a record takes*](../../../../docs/evaluations/what-a-check-can-know.md#the-shapes-a-record-takes)
/// that name a file. The fifth row of that table is the shape of an identifier
/// space rather than of a file, and `identity.duplicate` is the rule that reads
/// it, so nothing here has a variant for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    /// Row 1. Every line is one complete record that depends on no other line.
    IndependentLines,
    /// Row 2. A recorded corpus fixture that was decomposed so that it merges.
    RecordPerEntity,
    /// Rows 3 and 4. A producer writes it from the corpus, so it holds a fold.
    ///
    /// One variant covers two rows because nothing in the structure of an
    /// artifact separates a fold over the records from a fold over everything,
    /// and the separation buys nothing here: the two rows take one merge
    /// attribute. The evaluation gives them different *cures*, which is a
    /// judgment a person makes, and this verb reports rather than chooses.
    Fold,
}

impl Shape {
    /// The evaluation table's own words for the row or rows this shape covers.
    ///
    /// The words are that table's and not a paraphrase of it, because two
    /// vocabularies for one set of shapes is how a citation goes quiet.
    /// `the_report_uses_the_words_of_the_evaluation_table` holds every one of
    /// these against the table itself.
    ///
    /// Only [`Shape::Fold`] names two, and the reason is in its own comment.
    pub fn rows(self) -> &'static [&'static str] {
        match self {
            Shape::IndependentLines => &["A line that depends on nothing"],
            Shape::RecordPerEntity => &["One record for each entity, in a fixed order"],
            Shape::Fold => &["A fold over those records", "A fold over everything"],
        }
    }

    /// The rows above as the report prints them, and no second copy of them.
    pub fn row(self) -> String {
        self.rows().join(" / ")
    }

    /// The merge attribute this shape takes, in the words `.gitattributes` uses.
    pub fn treatment(self) -> Treatment {
        match self {
            Shape::IndependentLines => Treatment::Union,
            Shape::RecordPerEntity => Treatment::Unset,
            Shape::Fold => Treatment::Regenerate,
        }
    }

    /// Why that treatment, for a reader who met this report mid-merge.
    pub fn why(self) -> &'static str {
        match self {
            Shape::IndependentLines => {
                "a reading depends on no other reading, so both sides may be kept"
            }
            Shape::RecordPerEntity => {
                "two branches write two different lines, so an ordinary conflict is the correct report and no attribute is wanted"
            }
            Shape::Fold => {
                "a fold depends on every record, so it is rewritten rather than reconciled"
            }
        }
    }
}

/// What `.gitattributes` says about merging a path.
///
/// The three values `git check-attr merge <path>` can report over this tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Treatment {
    /// `merge=union`: keep the lines of both sides.
    Union,
    /// `merge=headwater-regenerate`: refuse, and name the producer to rerun.
    Regenerate,
    /// No merge attribute: git reconciles the lines like any other file.
    Unset,
}

impl Treatment {
    /// How this treatment is written in `.gitattributes`, for the report.
    pub fn declaration(self) -> &'static str {
        match self {
            Treatment::Union => "merge=union",
            Treatment::Regenerate => "merge=headwater-regenerate",
            Treatment::Unset => "no merge attribute",
        }
    }
}

/// The shapes, in report order. A list of rules rather than of paths.
pub const SHAPES: &[Shape] = &[Shape::IndependentLines, Shape::RecordPerEntity, Shape::Fold];

/// One reported path: its shape, the attribute it carries, and what rebuilds it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
    /// The path, relative to the repository root, with `/` separators.
    pub path: String,
    /// The shape computed from the artifact and from its producer's rule.
    pub shape: Shape,
    /// The merge attribute `.gitattributes` gives this path.
    pub treatment: Treatment,
    /// What rewrites this file, where anything does.
    ///
    /// This is not the same question as membership. A decomposed recorded
    /// fixture is written by a blessing run and is deliberately not a member of
    /// the population, and a reader resolving a conflict in one still needs the
    /// command. So the command is attributed where the file is rebuildable,
    /// and membership stays the narrower question it was.
    pub rebuild: Option<&'static str>,
}

impl Member {
    /// What the merge of this path costs, where its attribute is not its treatment.
    ///
    /// The six disagreements are enumerated here rather than described in a
    /// comment, so that a shape or a treatment added later fails to compile
    /// until somebody says what its merge does. Each arm names the cost at the
    /// merge rather than restating the rule.
    pub fn disagreement(&self) -> Option<&'static str> {
        match (self.shape, self.treatment) {
            (Shape::IndependentLines, Treatment::Union) => None,
            (Shape::RecordPerEntity, Treatment::Unset) => None,
            (Shape::Fold, Treatment::Regenerate) => None,

            (Shape::IndependentLines, Treatment::Unset) => Some(
                "two branches that each appended a reading conflict on the last line, on every parallel append",
            ),
            (Shape::IndependentLines, Treatment::Regenerate) => Some(
                "no producer rewrites an append store, so the driver refuses the merge that `union` resolves correctly every time",
            ),
            (Shape::RecordPerEntity, Treatment::Regenerate) => Some(
                "the driver refuses the merge this record was decomposed to take, which is the whole return on decomposing it",
            ),
            (Shape::RecordPerEntity, Treatment::Union) => Some(
                "two record streams interleave out of the fixed order, into a file no producer writes and no reader can bless",
            ),
            (Shape::Fold, Treatment::Unset) => Some(
                "two branches that move the fold to one value merge it in silence, into a value that was true on neither",
            ),
            (Shape::Fold, Treatment::Union) => Some(
                "two folds interleave into a value true of nothing, and no conflict is reported: the worst of the six",
            ),
        }
    }
}

/// One file, and the producer whose rule found it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Output {
    /// The path, relative to the repository root, with `/` separators.
    pub path: String,
    /// Which producer's rule claimed it.
    pub producer: Producer,
}

/// The computed population, and both directions of its disagreement with the tree.
#[derive(Clone, Debug, Default)]
pub struct Population {
    /// Every producer output, sorted, one entry per path.
    pub outputs: Vec<Output>,
    /// Every path `.gitattributes` declares `merge=headwater-regenerate`.
    pub declared: Vec<String>,
    /// A producer writes it and no attribute covers it: a merge of it is silent.
    pub undeclared: Vec<Output>,
    /// An attribute covers it and no producer writes it: a merge of hand-written
    /// text is refused, which `.gitattributes` names the worse of the two.
    pub unproduced: Vec<String>,
    /// Every reported path, with its shape and the attribute it carries.
    ///
    /// Wider than [`Population::outputs`], because three of the four
    /// file-shaped rows of the evaluation table have no member in the
    /// population. The module header says why that matters.
    pub members: Vec<Member>,
    /// A `.gitattributes` pattern that carries a merge attribute and a glob.
    ///
    /// This reader expands no pattern, so such a line reaches files it cannot
    /// enumerate. It is named rather than passed over, because a declaration
    /// that nothing reads looks exactly like a declaration that agrees.
    pub unreadable: Vec<String>,
}

impl Population {
    /// Whether the tree, the producers and the shapes all agree.
    ///
    /// Four things can disagree, and each one is a reason to exit 1: a producer
    /// output with no attribute, a declared path with no producer, a shape whose
    /// attribute is not its treatment, and a declaration this reader cannot
    /// expand.
    pub fn agrees(&self) -> bool {
        self.undeclared.is_empty()
            && self.unproduced.is_empty()
            && self.unreadable.is_empty()
            && self.disagreements().next().is_none()
    }

    /// Every member whose merge attribute is not the treatment its shape takes.
    pub fn disagreements(&self) -> impl Iterator<Item = (&Member, &'static str)> {
        self.members
            .iter()
            .filter_map(|member| member.disagreement().map(|cost| (member, cost)))
    }

    /// The report a caller reads.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{} derived artifacts, computed from {} producers\n",
            self.outputs.len(),
            PRODUCERS.len()
        ));
        for producer in PRODUCERS {
            let mine: Vec<&Output> = self
                .outputs
                .iter()
                .filter(|output| output.producer == *producer)
                .collect();
            out.push_str(&format!(
                "\n  {} — {}, {}\n",
                producer.command(),
                producer.rule(),
                match mine.len() {
                    1 => "1 output".to_string(),
                    n => format!("{n} outputs"),
                }
            ));
            for output in mine {
                out.push_str(&format!("    {}\n", output.path));
            }
        }
        out.push('\n');
        match self.undeclared.is_empty() {
            true => out.push_str("no producer output merges as an ordinary file\n"),
            false => {
                out.push_str(
                    "these are written by a producer and carry no `merge=headwater-regenerate`, \
                     so two branches that move one to the same value merge it silently:\n",
                );
                for output in &self.undeclared {
                    out.push_str(&format!(
                        "    {} — {}, rerun with `{}`\n",
                        output.path,
                        output.producer.rule(),
                        output.producer.command()
                    ));
                }
            }
        }
        match self.unproduced.is_empty() {
            true => out.push_str("no declared path is without a producer\n"),
            false => {
                out.push_str(
                    "these declare `merge=headwater-regenerate` and no producer writes them, \
                     so the declaration now refuses a merge of hand-written text:\n",
                );
                for path in &self.unproduced {
                    out.push_str(&format!("    {path}\n"));
                }
            }
        }
        out.push_str(&self.render_shapes());
        out
    }

    /// The second half of the report: what shape each path is, and what that costs.
    ///
    /// Split out because the first half is asserted by substring in several
    /// cases and a fold point moved inside it breaks every one of them.
    fn render_shapes(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\n{} paths hold a record whose shape decides its merge\n",
            self.members.len()
        ));
        for shape in SHAPES {
            let mine: Vec<&Member> = self
                .members
                .iter()
                .filter(|member| member.shape == *shape)
                .collect();
            out.push_str(&format!(
                "\n  {} — takes `{}`, because {}\n",
                shape.row(),
                shape.treatment().declaration(),
                shape.why()
            ));
            match mine.is_empty() {
                true => out.push_str("    (no path of this tree)\n"),
                false => {
                    for member in mine {
                        out.push_str(&format!("    {}", member.path));
                        if let Some(rebuild) = member.rebuild {
                            out.push_str(&format!(" — rerun with `{rebuild}`"));
                        }
                        out.push('\n');
                    }
                }
            }
        }

        out.push('\n');
        let disagreements: Vec<(&Member, &'static str)> = self.disagreements().collect();
        match disagreements.is_empty() {
            true => out.push_str("no path carries a merge attribute its shape does not take\n"),
            false => {
                out.push_str(
                    "these carry a merge attribute that their shape does not take, and \
                     each line says what the merge of it costs:\n",
                );
                for (member, cost) in disagreements {
                    out.push_str(&format!(
                        "    {} — {}, and it declares {}: {}\n",
                        member.path,
                        member.shape.row(),
                        member.treatment.declaration(),
                        cost
                    ));
                    out.push_str(&format!(
                        "      it takes `{}`",
                        member.shape.treatment().declaration()
                    ));
                    match member.rebuild {
                        Some(rebuild) => out.push_str(&format!(", and `{rebuild}` rebuilds it\n")),
                        None => out.push_str(", and no producer of this repository rebuilds it\n"),
                    }
                }
            }
        }
        match self.unreadable.is_empty() {
            true => out.push_str("every merge attribute of `.gitattributes` names one path\n"),
            false => {
                out.push_str(
                    "these carry a merge attribute behind a pattern this reader cannot \
                     expand, so no shape of this tree was held against them:\n",
                );
                for pattern in &self.unreadable {
                    out.push_str(&format!("    {pattern}\n"));
                }
            }
        }
        out
    }
}

/// The producers, in report order. The only list here, and it is of rules.
pub const PRODUCERS: &[Producer] = &[
    Producer::Generate,
    Producer::TaxonomyResolve,
    Producer::FigureRefresh,
    Producer::RecordedFold,
];

/// The path the taxonomy resolver writes.
pub const LOCK: &str = ".headwater/taxonomy.lock";

/// The element the figure refresh substitutes into.
pub const FIGURE: &str = "data-figure=";

/// Compute the population of a tree, and hold it against that tree's attributes.
pub fn population(root: &Path) -> Population {
    let mut files = Vec::new();
    collect(root, root, &mut files);
    files.sort();

    let mut outputs: Vec<Output> = Vec::new();
    for path in &files {
        if let Some(producer) = claimed_by(root, path) {
            outputs.push(Output {
                path: path.clone(),
                producer,
            });
        }
    }
    outputs.sort();

    let attributes = merge_attributes(root);
    let declared: Vec<String> = attributes
        .iter()
        .filter(|(_, treatment)| *treatment == Treatment::Regenerate)
        .map(|(path, _)| path.clone())
        .collect();
    let undeclared: Vec<Output> = outputs
        .iter()
        .filter(|output| !declared.contains(&output.path))
        .cloned()
        .collect();
    let unproduced: Vec<String> = declared
        .iter()
        .filter(|path| !outputs.iter().any(|output| output.path == **path))
        .cloned()
        .collect();

    let mut reported: Vec<String> = outputs.iter().map(|output| output.path.clone()).collect();
    reported.extend(attributes.iter().map(|(path, _)| path.clone()));
    reported.extend(
        files
            .iter()
            .filter(|path| is_recorded_fixture(path))
            .cloned(),
    );
    reported.extend(record_streams(root, &files));
    reported.sort();
    reported.dedup();

    let members: Vec<Member> = reported
        .iter()
        .filter_map(|path| {
            let producer = outputs
                .iter()
                .find(|output| output.path == *path)
                .map(|output| output.producer);
            let shape = shape_of(root, path, producer)?;
            let treatment = attributes
                .iter()
                .find(|(declared, _)| declared == path)
                .map(|(_, treatment)| *treatment)
                .unwrap_or(Treatment::Unset);
            Some(Member {
                path: path.clone(),
                shape,
                treatment,
                rebuild: rebuild_of(path, producer),
            })
        })
        .collect();

    Population {
        outputs,
        declared,
        undeclared,
        unproduced,
        members,
        unreadable: unreadable_patterns(root),
    }
}

/// Every store of readings of this tree, by the structure of the file.
///
/// A store that `.gitattributes` does not name is the one disagreement the
/// widened set would otherwise still miss, and it costs a conflict on every
/// parallel append. So the rule reads the tree rather than the declaration.
///
/// It stops at a `fixtures/` component, for the reason the marker rule does: a
/// store under a fixture tree belongs to another corpus and is an input of a
/// test rather than a record this repository appends to.
fn record_streams(root: &Path, files: &[String]) -> Vec<String> {
    files
        .iter()
        .filter(|path| !in_a_fixture_tree(path))
        .filter(|path| {
            std::fs::read_to_string(root.join(path)).is_ok_and(|text| is_a_record_stream(&text))
        })
        .cloned()
        .collect()
}

/// The shape of a path, and none for a path no rule here recognizes.
///
/// The three rules are read in the order below, and the order matters once. A
/// producer output is a fold before it is anything else, because a producer
/// writes it from the corpus. Reading the record-stream rule first would let a
/// producer output that happened to be one line per record be reported as
/// `union`-safe, and `union` on a fold is the worst failure of the six.
fn shape_of(root: &Path, path: &str, producer: Option<Producer>) -> Option<Shape> {
    if is_recorded_fixture(path) && producer.is_none() {
        // A recorded fixture the fold rule did not claim is the decomposed one.
        return Some(Shape::RecordPerEntity);
    }
    if producer.is_some() {
        return Some(Shape::Fold);
    }
    let text = std::fs::read_to_string(root.join(path)).ok()?;
    is_a_record_stream(&text).then_some(Shape::IndependentLines)
}

/// Whether every line of a file is one complete record.
///
/// This is the structure `headwater capture` and `headwater taxonomy audit`
/// read: a store of readings in JSON Lines, where one line is one reading and a
/// line that will not parse is reported rather than trusted. A file with no
/// line at all is not a record stream, because an empty file is every shape.
fn is_a_record_stream(text: &str) -> bool {
    let mut lines = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .peekable();
    lines.peek().is_some()
        && lines.all(|line| {
            let line = line.trim();
            line.starts_with('{') && line.ends_with('}')
        })
}

/// What rewrites a path, where anything of this repository does.
fn rebuild_of(path: &str, producer: Option<Producer>) -> Option<&'static str> {
    match producer {
        Some(producer) => Some(producer.command()),
        // A decomposed fixture is written by the same blessing run that writes
        // the folded ones, and it is deliberately not a member of the
        // population. The command is the one thing a reader mid-conflict needs.
        None if is_recorded_fixture(path) => Some(Producer::RecordedFold.command()),
        None => None,
    }
}

/// Which producer's rule claims a path, and none for a file nobody writes.
///
/// A path is claimed by at most one producer. No two rules here overlap: the
/// marker rule stops at a `fixtures/` component, the lock carries no marker,
/// and a page under `site/` is neither.
fn claimed_by(root: &Path, path: &str) -> Option<Producer> {
    if path == LOCK {
        return Some(Producer::TaxonomyResolve);
    }
    let text = std::fs::read_to_string(root.join(path)).ok()?;
    if path.starts_with("site/") && path.ends_with(".html") && text.contains(FIGURE) {
        return Some(Producer::FigureRefresh);
    }
    if is_recorded_fixture(path) {
        return states_a_fold(&text).then_some(Producer::RecordedFold);
    }
    if in_a_fixture_tree(path) {
        return None;
    }
    headwater_mark::carries_marker(path, &text).then_some(Producer::Generate)
}

/// A `corpus.*` fixture of an engine crate, which a blessing run writes.
fn is_recorded_fixture(path: &str) -> bool {
    path.starts_with("engine/crates/")
        && path.contains("/fixtures/")
        && path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("corpus."))
}

/// Whether an artifact's opening states a fold.
///
/// HW-DR-0049's own words for what a fold is: "a count over the whole corpus,
/// or a digest over the whole canonical text". A recorded artifact that states
/// one opens with it. A decomposed artifact opens with its first record, which
/// is a path or an anchor name, and it states no total anywhere.
fn states_a_fold(text: &str) -> bool {
    text.lines()
        .take(2)
        .any(|line| line.starts_with(|c: char| c.is_ascii_digit()) || line.contains("sha256:"))
}

/// Whether a path lies inside a fixture tree, which is another corpus.
fn in_a_fixture_tree(path: &str) -> bool {
    path.split('/').any(|component| component == "fixtures")
}

/// Every path `.gitattributes` declares `merge=headwater-regenerate`.
///
/// A comment line is skipped, because the file explains the attribute in prose
/// and quotes the `git config` lines that install the driver.
pub fn declared_paths(root: &Path) -> Vec<String> {
    merge_attributes(root)
        .into_iter()
        .filter(|(_, treatment)| *treatment == Treatment::Regenerate)
        .map(|(path, _)| path)
        .collect()
}

/// Every path `.gitattributes` gives a merge attribute, and which one.
///
/// A comment line is skipped, because the file explains the attribute in prose
/// and quotes the `git config` lines that install the driver. A pattern this
/// reader cannot expand is skipped here and reported by [`unreadable_patterns`].
pub fn merge_attributes(root: &Path) -> Vec<(String, Treatment)> {
    let mut found: Vec<(String, Treatment)> = declarations(root)
        .into_iter()
        .filter(|(pattern, _)| !is_a_pattern(pattern))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// Every `.gitattributes` pattern that carries a merge attribute and a glob.
pub fn unreadable_patterns(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = declarations(root)
        .into_iter()
        .filter(|(pattern, _)| is_a_pattern(pattern))
        .map(|(pattern, _)| pattern)
        .collect();
    found.sort();
    found.dedup();
    found
}

/// Each `.gitattributes` line that sets a merge attribute, pattern and value.
fn declarations(root: &Path) -> Vec<(String, Treatment)> {
    let Ok(text) = std::fs::read_to_string(root.join(".gitattributes")) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pattern = fields.next()?;
            let treatment = fields.find_map(|field| match field {
                "merge=union" => Some(Treatment::Union),
                "merge=headwater-regenerate" => Some(Treatment::Regenerate),
                _ => None,
            })?;
            Some((pattern.to_string(), treatment))
        })
        .collect()
}

/// Whether a `.gitattributes` pattern reaches more than the path it spells.
///
/// Git's pattern language is the one `.gitignore` uses. This reader expands
/// none of it, so a pattern that carries any of these characters is reported
/// rather than matched, and a trailing `/` is a directory rather than a file.
fn is_a_pattern(pattern: &str) -> bool {
    pattern.ends_with('/') || pattern.contains(['*', '?', '[', ']'])
}

/// Every file of a tree, relative to its root, skipping what no producer writes.
fn collect(dir: &Path, root: &Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            // `.git` holds git's own state, `target` holds a build, and the
            // three below hold a dependency tree or a harness. None is a file
            // any producer of this repository writes.
            if matches!(
                name.as_ref(),
                ".git" | "target" | "node_modules" | ".claude" | ".venv"
            ) {
                continue;
            }
            collect(&path, root, found);
        } else if let Ok(relative) = path.strip_prefix(root) {
            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// The repository root, for a caller holding any path inside one.
pub fn root_of(start: &Path) -> Option<PathBuf> {
    let mut here = start.to_path_buf();
    loop {
        if here.join(".gitattributes").exists() {
            return Some(here);
        }
        if !here.pop() {
            return None;
        }
    }
}
