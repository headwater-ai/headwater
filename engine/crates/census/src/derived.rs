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
//! A tree is asked only about the producers it holds, by
//! [`Producer::held_by`]. An adopter's tree holds the two verbs and neither the
//! script nor the engine workspace, so a report there names no command that
//! the tree cannot run.
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
//!
//! # Why the report takes a [`ColorMode`]
//!
//! `docs/interfaces/headwater-derived.md` states the promise every interface
//! contract of this binary states: the default senses whether each stream is a
//! terminal, and renders color only there. This verb broke that promise from
//! the day it shipped, and not by an oversight at a call site — the palette was
//! a module of `headwater-check`, which depends on this crate, so naming it
//! from here was a cycle cargo refuses. [#479](https://github.com/headwater-ai/headwater/issues/479)
//! moved the primitives to `headwater-paint`, a leaf, and this is the first
//! renderer below `headwater-check` to reach them.
//!
//! [`Population::render`] stays pure: the mode is a parameter, so the whole
//! report is a case table with no terminal in it. That purity is also what
//! [HW-OBL-0180](../../../../docs/obligations/0180-a-renderer-s-color-mode-is-wired-at-a-call-site-that-no-type-forbids-from-being-wrong.md)
//! records, because nothing in the type says a call site handed it the mode a
//! stream is actually in. `tools/engine/color-fixtures.sh` is what reads that,
//! by attaching a pseudo-terminal and counting escape bytes.

use headwater_paint::{paint, ColorMode, Role};
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

    /// Whether the tree at `root` holds this producer, so that it can be run there.
    ///
    /// The two verbs are held by every tree that has the engine. The figure
    /// script and the blessing run belong to the repository that maintains the
    /// engine, and a tree holds each one only where it carries the file that
    /// runs it: [`REFRESH_SCRIPT`] and [`ENGINE_MANIFEST`]. A producer that a
    /// tree does not hold claims no file of that tree, so `headwater derived`
    /// never names its command there and `headwater init --git` writes no line
    /// for it. This is the one predicate both read.
    pub fn held_by(self, root: &Path) -> bool {
        match self {
            Producer::Generate | Producer::TaxonomyResolve => true,
            Producer::FigureRefresh => root.join(REFRESH_SCRIPT).is_file(),
            Producer::RecordedFold => root.join(ENGINE_MANIFEST).is_file(),
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
            Shape::Fold => Treatment::Refuse,
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
                "a fold depends on every record, so a merge keeps the current side and conflicts, \
                 and it is rewritten rather than reconciled. A clone that ran \
                 `headwater init --git --git-config` overrides it with `merge=headwater-regenerate` \
                 in `info/attributes`, which also agrees"
            }
        }
    }
}

/// What git's attributes say about merging a path.
///
/// Inside a git repository this is the answer of `git check-attr merge
/// <path>`. `unset` (`-merge` and the `binary` macro) reads as
/// [`Treatment::Refuse`], because git keeps the current side of such a path
/// and records a conflict with no driver and no configuration. `unspecified`,
/// `set` and the built-in `text` driver read as [`Treatment::Unset`], because
/// each of them is a text merge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Treatment {
    /// `merge=union`: keep the lines of both sides.
    Union,
    /// `merge=headwater-regenerate`: refuse, and name the producer to rerun.
    ///
    /// A fold takes it only from a clone's own `$GIT_DIR/info/attributes`,
    /// which `headwater init --git --git-config` writes beside the driver
    /// config. Committed, it names a driver that an unconfigured clone and a
    /// forge do not define, and git reads an undefined driver as a text merge
    /// ([#1058](https://github.com/headwater-ai/headwater/issues/1058)).
    Regenerate,
    /// `-merge`: keep the current side and record a conflict, with no driver.
    ///
    /// What the committed `.gitattributes` declares for a fold, because git
    /// honors it in every clone whatever its configuration.
    Refuse,
    /// No merge attribute: git reconciles the lines like any other file.
    Unset,
    /// A merge driver this verb does not know, such as `merge=ours`.
    ///
    /// [`Population::drivers`] carries its name. The verb cannot say what the
    /// driver does, so it reports the driver rather than read it as no
    /// attribute.
    Unknown,
}

impl Treatment {
    /// Whether a merge of a path under this treatment stops rather than reconciles.
    pub fn stops_a_merge(self) -> bool {
        matches!(self, Treatment::Regenerate | Treatment::Refuse)
    }

    /// How this treatment is written in `.gitattributes`, for the report.
    pub fn declaration(self) -> &'static str {
        match self {
            Treatment::Union => "merge=union",
            Treatment::Regenerate => "merge=headwater-regenerate",
            Treatment::Refuse => "-merge",
            Treatment::Unset => "no merge attribute",
            Treatment::Unknown => "a merge driver this verb does not know",
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
    /// The disagreements are enumerated here rather than described in a
    /// comment, so that a shape or a treatment added later fails to compile
    /// until somebody says what its merge does. Each arm names the cost at the
    /// merge rather than restating the rule.
    pub fn disagreement(&self) -> Option<&'static str> {
        match (self.shape, self.treatment) {
            (Shape::IndependentLines, Treatment::Union) => None,
            (Shape::RecordPerEntity, Treatment::Unset) => None,
            (Shape::Fold, Treatment::Regenerate) => None,
            (Shape::Fold, Treatment::Refuse) => None,

            (Shape::IndependentLines, Treatment::Unset) => Some(
                "two branches that each appended a reading conflict on the last line, on every parallel append",
            ),
            (Shape::IndependentLines, Treatment::Regenerate) => Some(
                "no producer rewrites an append store, so the driver refuses the merge that `union` resolves correctly every time",
            ),
            (Shape::RecordPerEntity, Treatment::Regenerate) => Some(
                "the driver refuses the merge this record was decomposed to take, which is the whole return on decomposing it",
            ),
            (Shape::RecordPerEntity, Treatment::Refuse) => Some(
                "the merge keeps one side of every record and conflicts, which refuses the merge this record was decomposed to take",
            ),
            (Shape::IndependentLines, Treatment::Refuse) => Some(
                "two branches that each appended a reading conflict, and the merge keeps one side's readings and drops the other's",
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

            // A driver this verb does not know is not one of the six, because
            // the verb cannot say what the driver does with any shape.
            (_, Treatment::Unknown) => Some(
                "a merge driver this verb does not know decides the merge, so nothing here can say that the merge is sound",
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
    /// The producers the tree holds, in report order, by [`Producer::held_by`].
    pub held: Vec<Producer>,
    /// Every producer output, sorted, one entry per path.
    pub outputs: Vec<Output>,
    /// Every path git's attributes declare `-merge` or `merge=headwater-regenerate`.
    pub declared: Vec<String>,
    /// A producer writes a fold and no attribute covers it: a merge of it is silent.
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
    /// What the root-file reader could not read, sorted: each root
    /// `.gitattributes` pattern that carries a merge attribute and a glob, and
    /// each `.gitattributes` file below the root, by its path.
    ///
    /// Empty inside a git repository, where git expands its own patterns and
    /// reads every nested file. Outside one, the root-file reader expands no
    /// pattern and reads no nested file, so either can reach files it cannot
    /// see. Each is named rather than passed over, because a declaration that
    /// nothing reads looks exactly like a declaration that agrees.
    /// [`is_a_nested_attributes_file`] tells the two kinds apart.
    pub unreadable: Vec<String>,
    /// Every path that carries a merge driver this verb does not know, and
    /// the driver's name, as git reports it.
    pub drivers: Vec<(String, String)>,
    /// What git printed where it refused `git check-attr` inside a repository.
    ///
    /// The root-file reader answered in its place, which reads less than a
    /// merge does. So a refusal is a disagreement in its own right: the report
    /// names it and the verb exits 1, rather than fall back in silence.
    pub refused: Option<String>,
}

impl Population {
    /// Whether the tree, the producers and the shapes all agree.
    ///
    /// Five things can disagree, and each one is a reason to exit 1: a producer
    /// output with no attribute, a declared path with no producer, a shape whose
    /// attribute is not its treatment, a declaration this reader cannot
    /// expand, and a repository where git refused to give the attributes.
    pub fn agrees(&self) -> bool {
        self.undeclared.is_empty()
            && self.unproduced.is_empty()
            && self.unreadable.is_empty()
            && self.refused.is_none()
            && self.disagreements().next().is_none()
    }

    /// Every member whose merge attribute is not the treatment its shape takes.
    pub fn disagreements(&self) -> impl Iterator<Item = (&Member, &'static str)> {
        self.members
            .iter()
            .filter_map(|member| member.disagreement().map(|cost| (member, cost)))
    }

    /// The report a caller reads, painted for `mode`.
    ///
    /// Four roles, each chosen for what the reader does with the token rather
    /// than for how the line reads. The opening count is a [`Role::Heading`],
    /// the one line that says what the whole report is about. Each
    /// [`Producer::command`] is a [`Role::Verb`], because it is the command a
    /// reader retypes to rewrite that producer's outputs. Every path is a
    /// [`Role::Path`], on both sides of the disagreement, because a path is
    /// what a reader takes to an editor. The two disagreement headings are
    /// [`Role::Error`] and their agreement counterparts are plain: the verb
    /// exits non-zero on exactly those two conditions, so the color says the
    /// same thing the exit status does.
    pub fn render(&self, mode: ColorMode) -> String {
        let mut out = String::new();
        out.push_str(&paint(
            Role::Heading,
            &format!(
                "{} derived artifacts, computed from {} producers",
                self.outputs.len(),
                self.held.len()
            ),
            mode,
        ));
        out.push('\n');
        for producer in &self.held {
            let mine: Vec<&Output> = self
                .outputs
                .iter()
                .filter(|output| output.producer == *producer)
                .collect();
            out.push_str(&format!(
                "\n  {} — {}, {}\n",
                paint(Role::Verb, producer.command(), mode),
                producer.rule(),
                match mine.len() {
                    1 => "1 output".to_string(),
                    n => format!("{n} outputs"),
                }
            ));
            for output in mine {
                out.push_str(&format!("    {}\n", paint(Role::Path, &output.path, mode)));
            }
        }
        out.push('\n');
        match self.undeclared.is_empty() {
            true => out.push_str("no producer output merges as an ordinary file\n"),
            false => {
                out.push_str(&paint(
                    Role::Error,
                    "these are written by a producer and carry neither `-merge` nor `merge=headwater-regenerate`, \
                     so two branches that move one to the same value merge it silently:",
                    mode,
                ));
                out.push('\n');
                for output in &self.undeclared {
                    out.push_str(&format!(
                        "    {} — {}, rerun with `{}`\n",
                        paint(Role::Path, &output.path, mode),
                        output.producer.rule(),
                        paint(Role::Verb, output.producer.command(), mode)
                    ));
                }
            }
        }
        match self.unproduced.is_empty() {
            true => out.push_str("no declared path is without a producer\n"),
            false => {
                out.push_str(&paint(
                    Role::Error,
                    "these declare `merge=headwater-regenerate` and no producer writes them, \
                     so the declaration now refuses a merge of hand-written text:",
                    mode,
                ));
                out.push('\n');
                for path in &self.unproduced {
                    out.push_str(&format!("    {}\n", paint(Role::Path, path, mode)));
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
        if let Some(refusal) = &self.refused {
            out.push_str(
                "git refused `git check-attr` in this repository, so the merge \
                 attributes above are the root `.gitattributes` alone, and a nested \
                 file, `info/attributes` and `core.attributesFile` were not read:\n",
            );
            out.push_str(&format!("    {refusal}\n"));
        }
        if !self.drivers.is_empty() {
            out.push_str(
                "these carry a merge driver this verb does not know, so it cannot \
                 say what a merge of them does:\n",
            );
            for (path, driver) in &self.drivers {
                out.push_str(&format!("    {path} merge={driver}\n"));
            }
        }
        let (nested, patterns): (Vec<&String>, Vec<&String>) = self
            .unreadable
            .iter()
            .partition(|entry| is_a_nested_attributes_file(entry));
        if !nested.is_empty() {
            out.push_str(
                "these `.gitattributes` files are below the root, and this reader \
                 does not read them, so no shape of this tree was held against them:\n",
            );
            for file in nested {
                out.push_str(&format!("    {file}\n"));
            }
        }
        match patterns.is_empty() {
            true => {
                out.push_str("no merge attribute is behind a pattern this verb cannot expand\n");
            }
            false => {
                out.push_str(
                    "these carry a merge attribute behind a pattern this reader cannot \
                     expand, so no shape of this tree was held against them:\n",
                );
                for pattern in patterns {
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

/// The script that the figure producer runs, which a tree holds or does not.
pub const REFRESH_SCRIPT: &str = "tools/site/refresh-figures.sh";

/// The engine workspace that the blessing run needs, which a tree holds or does not.
pub const ENGINE_MANIFEST: &str = "engine/Cargo.toml";

/// The path the taxonomy resolver writes.
pub const LOCK: &str = ".headwater/taxonomy.lock";

/// The element the figure refresh substitutes into.
pub const FIGURE: &str = "data-figure=";

/// Compute the population of a tree, and hold it against that tree's attributes.
pub fn population(root: &Path) -> Population {
    let files = files_of(root);
    let held: Vec<Producer> = PRODUCERS
        .iter()
        .copied()
        .filter(|producer| producer.held_by(root))
        .collect();
    let blessing = held.contains(&Producer::RecordedFold);

    let mut outputs: Vec<Output> = Vec::new();
    for path in &files {
        if let Some(producer) = claimed_by(root, path).filter(|producer| held.contains(producer)) {
            outputs.push(Output {
                path: path.clone(),
                producer,
            });
        }
    }
    outputs.sort();

    let Attributes {
        found: attributes,
        unreadable,
        drivers,
        refused,
    } = attributes_of(root, &files);
    let declared: Vec<String> = attributes
        .iter()
        .filter(|(_, treatment)| treatment.stops_a_merge())
        .map(|(path, _)| path.clone())
        .collect();
    // Only a fold owes an attribute. A producer output that is one record per
    // entity merges as text and needs none (#1058, 1058-a).
    let undeclared: Vec<Output> = outputs
        .iter()
        .filter(|output| {
            shape_of(root, &output.path, Some(output.producer), blessing) == Some(Shape::Fold)
        })
        .filter(|output| !declared.contains(&output.path))
        .cloned()
        .collect();
    // The driver, and a literal `-merge` line of the root file, are held in
    // this direction. The `binary` macro and a pattern are not, because
    // `*.png binary` on hand-made images is no claim that a producer writes
    // them.
    let literal = literal_unsets(root);
    let unproduced: Vec<String> = attributes
        .iter()
        .filter(|(path, treatment)| {
            *treatment == Treatment::Regenerate
                || (*treatment == Treatment::Refuse && literal.contains(path))
        })
        .map(|(path, _)| path)
        .filter(|path| !outputs.iter().any(|output| output.path == **path))
        .cloned()
        .collect();

    let mut reported: Vec<String> = outputs.iter().map(|output| output.path.clone()).collect();
    reported.extend(attributes.iter().map(|(path, _)| path.clone()));
    reported.extend(
        files
            .iter()
            .filter(|path| blessing && is_recorded_fixture(path))
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
            let shape = shape_of(root, path, producer, blessing)?;
            let treatment = attributes
                .iter()
                .find(|(declared, _)| declared == path)
                .map(|(_, treatment)| *treatment)
                .unwrap_or(Treatment::Unset);
            Some(Member {
                path: path.clone(),
                shape,
                treatment,
                rebuild: rebuild_of(path, producer, blessing),
            })
        })
        .collect();

    Population {
        held,
        outputs,
        declared,
        undeclared,
        unproduced,
        members,
        unreadable,
        drivers,
        refused,
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
fn shape_of(root: &Path, path: &str, producer: Option<Producer>, blessing: bool) -> Option<Shape> {
    if blessing && is_recorded_fixture(path) && producer.is_none() {
        // A recorded fixture the fold rule did not claim is the decomposed one.
        return Some(Shape::RecordPerEntity);
    }
    match producer {
        // A generated file is one record per entity unless its opening states a
        // fold, which is the rule the recorded fixtures take. Since #1058 no
        // generated file of this repository states a count, and 1058-a
        // measured every one of them merging as text to what the producer
        // writes over the merged tree.
        //
        // A file whose marker line is its only line is a projection written
        // compactly, and one line cannot be one record per entity: any two
        // edits conflict on it, and the cure is to regenerate (#809).
        Some(Producer::Generate) => {
            let text = std::fs::read_to_string(root.join(path)).ok()?;
            return Some(match states_a_fold(&text) || is_one_marked_line(&text) {
                true => Shape::Fold,
                false => Shape::RecordPerEntity,
            });
        }
        // The lock carries a digest over its whole canonical text, a figure is
        // a count, and a recorded fold opens with one.
        Some(_) => return Some(Shape::Fold),
        None => {}
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
fn rebuild_of(path: &str, producer: Option<Producer>, blessing: bool) -> Option<&'static str> {
    match producer {
        Some(producer) => Some(producer.command()),
        // A decomposed fixture is written by the same blessing run that writes
        // the folded ones, and it is deliberately not a member of the
        // population. The command is the one thing a reader mid-conflict needs.
        None if blessing && is_recorded_fixture(path) => Some(Producer::RecordedFold.command()),
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
/// is a path or an anchor name, and it states no total anywhere. A generated
/// file is read the same way after its marker line.
fn states_a_fold(text: &str) -> bool {
    text.lines()
        .filter(|line| !line.contains("headwater:generated") && !line.trim().is_empty())
        .take(2)
        .any(|line| line.starts_with(|c: char| c.is_ascii_digit()) || line.contains("sha256:"))
}

/// Whether the only line of a file that is not blank is the one that carries the marker.
fn is_one_marked_line(text: &str) -> bool {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    matches!(
        (lines.next(), lines.next()),
        (Some(line), None) if line.contains("headwater:generated")
    )
}

/// Whether a path lies inside a fixture tree, which is another corpus.
fn in_a_fixture_tree(path: &str) -> bool {
    path.split('/').any(|component| component == "fixtures")
}

/// Every path that git's attributes declare `-merge` or `merge=headwater-regenerate`.
///
/// Read through [`merge_attributes`], so `headwater init --git` sees a
/// declaration in a nested `.gitattributes` and does not append a second one.
pub fn declared_paths(root: &Path) -> Vec<String> {
    merge_attributes(root)
        .into_iter()
        .filter(|(_, treatment)| treatment.stops_a_merge())
        .map(|(path, _)| path)
        .collect()
}

/// Each literal path of the root `.gitattributes` and the last merge attribute it sets there.
///
/// This is the committed half of a declaration, read by hand, so that
/// `headwater init --git` can tell a committed `-merge` from the driver that a
/// clone's own `info/attributes` supplies. Git answers with the override, which
/// wins, so its answer alone cannot say what the committed file holds. A later
/// line for one path wins over an earlier one, as it does in git.
pub fn root_declarations(root: &Path) -> Vec<(String, Treatment)> {
    let mut found: Vec<(String, Treatment)> = Vec::new();
    for (pattern, treatment) in declarations(root) {
        if is_a_pattern(&pattern) {
            continue;
        }
        let path = pattern.trim_start_matches('/').to_string();
        found.retain(|(seen, _)| *seen != path);
        found.push((path, treatment));
    }
    found
}

/// Every literal path that a root `.gitattributes` line gives `-merge`, by that word.
///
/// The `binary` macro also unsets the merge, and it is left out on purpose: it
/// is how a repository marks a file a person made, such as an image.
fn literal_unsets(root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(root.join(".gitattributes")) else {
        return Vec::new();
    };
    text.lines()
        .map(|line| line.trim().trim_start_matches('\u{feff}'))
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pattern = fields.next()?;
            (fields.any(|field| field == "-merge") && !is_a_pattern(pattern))
                .then(|| pattern.trim_start_matches('/').to_string())
        })
        .collect()
}

/// Every path of the tree that carries a merge attribute, and which one.
///
/// A path with no merge attribute is absent. [`attributes_of`] says where the
/// answer comes from.
pub fn merge_attributes(root: &Path) -> Vec<(String, Treatment)> {
    attributes_of(root, &files_of(root)).found
}

/// Every `.gitattributes` pattern that carries a merge attribute and a glob,
/// and every `.gitattributes` file below the root, by its path.
///
/// Empty inside a git repository, which expands its own patterns and reads
/// every nested file.
pub fn unreadable_patterns(root: &Path) -> Vec<String> {
    attributes_of(root, &files_of(root)).unreadable
}

/// The merge attributes of a tree, and what the verb could not read of them.
struct Attributes {
    /// Every path with a merge attribute, sorted, and the treatment it takes.
    found: Vec<(String, Treatment)>,
    /// Every pattern the root-file reader cannot expand, and every nested
    /// `.gitattributes` file it does not read.
    unreadable: Vec<String>,
    /// Every path with a driver this verb does not know, and the driver.
    drivers: Vec<(String, String)>,
    /// What git printed when it refused the question inside a repository.
    refused: Option<String>,
}

/// The merge attribute of each path, from git where there is a repository.
///
/// **Inside a git repository, git answers.** [`headwater_vcs::merge_attributes`]
/// asks `git check-attr` about every file of the walk, every literal path of
/// the root `.gitattributes` (so a declaration whose file is gone still
/// reaches the `unproduced` direction), and the lock. Git applies its own
/// precedence: nested `.gitattributes` files, `$GIT_DIR/info/attributes`, and
/// `core.attributesFile`. Nothing is unreadable here, because git expands its
/// own patterns.
///
/// A literal path of the root file is asked as git names it: a leading `/`
/// anchors a pattern to the root and is not part of the path, and git refuses
/// the whole run for a path that begins with one. A path that leaves the tree
/// through `..` is not asked at all. Git refuses it too, and no pattern that
/// names it can match a file of the tree, so git's answer for it is nothing.
///
/// **Outside a repository, the root `.gitattributes` alone is read**, as a
/// list of literal paths. A pattern with a glob is named as unreadable, and so
/// is each `.gitattributes` file of the walk below the root, by its path.
///
/// **Where git refuses the question inside a repository**, the root-file
/// reader answers, and [`Population::refused`] carries what git printed, so
/// the report says which reader answered and the verb exits 1.
fn attributes_of(root: &Path, files: &[String]) -> Attributes {
    let declarations = declarations(root);
    let mut candidates: Vec<String> = files.to_vec();
    candidates.extend(
        declarations
            .iter()
            .filter(|(pattern, _)| !is_a_pattern(pattern))
            .map(|(pattern, _)| pattern.trim_start_matches('/').to_string())
            .filter(|path| stays_inside_the_tree(path)),
    );
    candidates.push(LOCK.to_string());
    candidates.sort();
    candidates.dedup();

    attributes_from(
        headwater_vcs::merge_attributes(root, &candidates),
        declarations,
        files,
    )
}

/// Git's answer read into treatments, or the root file where git gave none.
///
/// Split from [`attributes_of`] so that a refusal, which no tree of a test can
/// provoke once the candidates are filtered, is still held by a case.
fn attributes_from(
    answer: Option<Result<Vec<(String, String)>, String>>,
    declarations: Vec<(String, Treatment)>,
    files: &[String],
) -> Attributes {
    let answers = match answer {
        Some(Ok(answers)) => answers,
        Some(Err(refusal)) => {
            return Attributes {
                refused: Some(refusal),
                ..root_file_reading(declarations, files)
            };
        }
        None => return root_file_reading(declarations, files),
    };

    let mut found = Vec::new();
    let mut drivers = Vec::new();
    for (path, value) in answers {
        let treatment = match value.as_str() {
            "unspecified" | "set" | "text" => continue,
            "unset" | "binary" => Treatment::Refuse,
            "union" => Treatment::Union,
            "headwater-regenerate" => Treatment::Regenerate,
            _ => {
                drivers.push((path.clone(), value));
                Treatment::Unknown
            }
        };
        found.push((path, treatment));
    }
    found.sort();
    found.dedup();
    drivers.sort();
    drivers.dedup();
    Attributes {
        found,
        unreadable: Vec::new(),
        drivers,
        refused: None,
    }
}

/// Whether a path, read relative to the root, stays inside the tree.
///
/// Git accepts `x/../y` and refuses a path that climbs above the root, so the
/// depth is counted component by component rather than by a search for `..`.
fn stays_inside_the_tree(path: &str) -> bool {
    let mut depth = 0usize;
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => match depth.checked_sub(1) {
                Some(up) => depth = up,
                None => return false,
            },
            _ => depth += 1,
        }
    }
    depth > 0
}

/// The merge attributes that the root `.gitattributes` states, read by hand.
///
/// A leading `/` of a literal path anchors it to the root and is not part of
/// the path, so it is removed, as it is for the question put to git. A nested
/// `.gitattributes` among `files` is not read. It is named in `unreadable`
/// instead, because git would read it and this reader cannot say what it
/// changes.
fn root_file_reading(declarations: Vec<(String, Treatment)>, files: &[String]) -> Attributes {
    let mut found: Vec<(String, Treatment)> = declarations
        .iter()
        .filter(|(pattern, _)| !is_a_pattern(pattern))
        .map(|(pattern, treatment)| (pattern.trim_start_matches('/').to_string(), *treatment))
        .collect();
    found.sort();
    found.dedup();
    let mut unreadable: Vec<String> = declarations
        .into_iter()
        .filter(|(pattern, _)| is_a_pattern(pattern))
        .map(|(pattern, _)| pattern)
        .chain(
            files
                .iter()
                .filter(|file| is_a_nested_attributes_file(file))
                .cloned(),
        )
        .collect();
    unreadable.sort();
    unreadable.dedup();
    Attributes {
        found,
        unreadable,
        drivers: Vec::new(),
        refused: None,
    }
}

/// Every file of a tree that the verb's walk visits, sorted.
fn files_of(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let ignored = headwater_vcs::ignored(root);
    collect(root, root, &ignored, &mut files);
    files.sort();
    files
}

/// Each line of the root `.gitattributes` that sets a merge attribute.
///
/// This is the reader for a tree outside a git repository. A comment line is
/// skipped, because the file explains the attribute in prose and quotes the
/// `git config` lines that install the driver.
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
                "-merge" | "binary" => Some(Treatment::Refuse),
                _ => None,
            })?;
            Some((pattern.to_string(), treatment))
        })
        .collect()
}

/// Whether an entry of [`Population::unreadable`] is a nested `.gitattributes`
/// file rather than a root pattern, read from its last component alone.
///
/// A walk path is a file name and not a pattern, so a directory such as
/// `br[1]` does not make it one: a test for glob characters here would drop
/// that file from the report in silence. The one entry this reads wrongly is a
/// root glob whose last component is `.gitattributes`, and that entry is still
/// named and is still a disagreement, only under the other heading.
pub fn is_a_nested_attributes_file(entry: &str) -> bool {
    entry.ends_with("/.gitattributes")
}

/// Whether a `.gitattributes` pattern reaches more than the path it spells.
///
/// Git's pattern language is the one `.gitignore` uses. This reader expands
/// none of it, so a pattern that carries any of these characters is reported
/// rather than matched, and a trailing `/` is a directory rather than a file.
fn is_a_pattern(pattern: &str) -> bool {
    pattern.ends_with('/') || pattern.contains(['*', '?', '[', ']'])
}

/// Every file of a tree, relative to its root, skipping what no producer
/// writes and every path `ignored` (git's own ignore rules, read once by the
/// caller) excludes.
///
/// `ignored` holds a whole directory as one entry ending in `/` where
/// everything under it is ignored — [`headwater_vcs::ignored`]'s own
/// contract — so a directory match here prunes the recursion rather than
/// filtering its contents one file at a time, and a gitignored tree this
/// walk would otherwise descend into (a locally built site, an editor's
/// scratch directory) is never read at all.
fn collect(dir: &Path, root: &Path, ignored: &[String], found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            // `.git` holds git's own state, `target` holds a build, and the
            // three below hold a dependency tree or a harness. None is a file
            // any producer of this repository writes. `.git` is never
            // something git's own ignore rules report (nothing ignores the
            // repository that holds it), and `git check-ignore` against this
            // tree today answers "not ignored" for the other four too — this
            // is an explicit fast path the fix does not depend on, not a list
            // the ignore-rule read makes redundant.
            if matches!(
                name.as_ref(),
                ".git" | "target" | "node_modules" | ".claude" | ".venv"
            ) {
                continue;
            }
            if ignored.contains(&format!("{relative}/")) {
                continue;
            }
            collect(&path, root, ignored, found);
        } else if !ignored.contains(&relative) {
            found.push(relative);
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

#[cfg(test)]
mod tests {
    use super::{Output, Population, Producer};
    use headwater_paint::ColorMode;

    /// One population that reaches every branch of the renderer: outputs under
    /// two different producers, and both directions of disagreement non-empty.
    ///
    /// A population that agrees would render two plain sentences where the two
    /// `Role::Error` headings go, so it could not tell a renderer that paints
    /// them from one that does not.
    fn disagreeing() -> Population {
        let generated = Output {
            path: "docs/spec/06-engine-architecture.md".to_string(),
            producer: Producer::Generate,
        };
        let lock = Output {
            path: ".headwater/taxonomy.lock".to_string(),
            producer: Producer::TaxonomyResolve,
        };
        Population {
            held: super::PRODUCERS.to_vec(),
            outputs: vec![lock, generated.clone()],
            declared: vec![
                ".headwater/taxonomy.lock".to_string(),
                "docs/handbook.md".to_string(),
            ],
            undeclared: vec![generated],
            unproduced: vec!["docs/handbook.md".to_string()],
            members: Vec::new(),
            unreadable: Vec::new(),
            drivers: Vec::new(),
            refused: None,
        }
    }

    /// `Plain` writes the whole report and not one escape byte.
    ///
    /// This is the arm every headless test of this repository already had, and
    /// on its own it passed a renderer that emitted no color under any
    /// condition — which is what `headwater derived` did until #479. It is here
    /// for the other direction: that turning the palette on moved no text.
    #[test]
    fn plain_writes_the_whole_report_and_no_escape_sequence() {
        let rendered = disagreeing().render(ColorMode::Plain);
        assert!(!rendered.contains('\x1b'), "{rendered:?}");
        assert!(rendered.starts_with("2 derived artifacts, computed from 4 producers\n"));
        assert!(rendered.contains("  headwater generate — carries the generated-file marker"));
        assert!(rendered.contains("    docs/spec/06-engine-architecture.md\n"));
        assert!(rendered.contains("these are written by a producer and carry neither"));
        assert!(rendered.contains("these declare `merge=headwater-regenerate` and no producer"));
        assert!(rendered.contains("    docs/handbook.md\n"));
    }

    /// `Ansi` paints the four roles the report uses, and changes no text.
    ///
    /// The assertion is per role rather than a count of escape bytes: a count
    /// passes when one token is painted and three are not, which is the
    /// half-wired renderer this case exists to refuse.
    #[test]
    fn ansi_paints_the_heading_the_verbs_the_paths_and_the_two_refusals() {
        let rendered = disagreeing().render(ColorMode::Ansi);
        for (role, expected) in [
            (
                "heading",
                "\x1b[1m2 derived artifacts, computed from 4 producers\x1b[0m",
            ),
            ("verb", "\x1b[1;32mheadwater generate\x1b[0m"),
            ("path", "\x1b[36mdocs/spec/06-engine-architecture.md\x1b[0m"),
            ("unproduced path", "\x1b[36mdocs/handbook.md\x1b[0m"),
            (
                "undeclared refusal",
                "\x1b[1;31mthese are written by a producer and carry neither",
            ),
            (
                "unproduced refusal",
                "\x1b[1;31mthese declare `merge=headwater-regenerate`",
            ),
        ] {
            assert!(
                rendered.contains(expected),
                "the {role} is unpainted under Ansi:\n{rendered}"
            );
        }
    }

    /// The two modes differ by escape sequences alone.
    ///
    /// Stripping every SGR sequence out of the painted report gives the plain
    /// report back, byte for byte. This is the unit-level half of what
    /// `strips_to_the_plain_bytes` asserts in `tools/engine/color-fixtures.sh`
    /// for the surfaces that fold, and it is what says a painted token did not
    /// gain or lose a byte beside it.
    #[test]
    fn stripping_the_escapes_gives_the_plain_report_back() {
        let population = disagreeing();
        let painted = population.render(ColorMode::Ansi);
        let mut stripped = String::with_capacity(painted.len());
        let mut rest = painted.as_str();
        while let Some(start) = rest.find('\x1b') {
            stripped.push_str(&rest[..start]);
            let after = &rest[start..];
            let end = after
                .find('m')
                .expect("every escape this renderer writes is an SGR sequence ending in m");
            rest = &after[end + 1..];
        }
        stripped.push_str(rest);
        assert_eq!(stripped, population.render(ColorMode::Plain));
    }

    /// A refusal of `git check-attr` inside a repository is a disagreement, and
    /// the report names it and what git printed.
    #[test]
    fn a_refusal_of_git_check_attr_is_carried_rather_than_read_as_no_repository() {
        let declarations = vec![("a.md".to_string(), super::Treatment::Union)];
        let refused = super::attributes_from(
            Some(Err("fatal: an invented refusal".to_string())),
            declarations.clone(),
            &[],
        );
        assert_eq!(
            refused.refused.as_deref(),
            Some("fatal: an invented refusal")
        );
        assert_eq!(refused.found, declarations);
        let outside = super::attributes_from(None, declarations, &[]);
        assert_eq!(outside.refused, None);
    }

    /// Where git refuses, the root-file reader answers, and it names each
    /// nested `.gitattributes` of the walk as it does outside a repository.
    #[test]
    fn a_refusal_of_git_check_attr_names_each_nested_attributes_file() {
        let files = [
            ".gitattributes",
            "a.md",
            "sub/.gitattributes",
            "sub/deep/.gitattributes",
        ]
        .map(String::from);
        let refused = super::attributes_from(
            Some(Err("fatal: an invented refusal".to_string())),
            Vec::new(),
            &files,
        );
        assert_eq!(
            refused.unreadable,
            ["sub/.gitattributes", "sub/deep/.gitattributes"].map(String::from)
        );
    }

    #[test]
    fn a_refusal_of_git_check_attr_is_named_and_does_not_agree() {
        let population = Population {
            refused: Some("fatal: an invented refusal".to_string()),
            ..Population::default()
        };
        assert!(!population.agrees());
        let rendered = population.render(ColorMode::Plain);
        assert!(
            rendered.contains("git refused `git check-attr`"),
            "{rendered}"
        );
        assert!(
            rendered.contains("    fatal: an invented refusal\n"),
            "{rendered}"
        );
    }

    /// A population that agrees prints its two agreement sentences plain under
    /// `Ansi`, because the verb exits zero on it.
    ///
    /// `Role::Error` says what the exit status says. A renderer that painted
    /// both branches red would tell a reader that an agreeing tree is a
    /// disagreeing one, and the two cases above cannot see that.
    #[test]
    fn an_agreeing_population_paints_no_refusal() {
        let rendered = Population::default().render(ColorMode::Ansi);
        assert!(rendered.contains("\nno producer output merges as an ordinary file\n"));
        assert!(rendered.contains("no declared path is without a producer\n"));
        assert!(!rendered.contains("\x1b[1;31m"), "{rendered:?}");
    }
}
