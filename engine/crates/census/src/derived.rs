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
}

impl Population {
    /// Whether the tree and the producers agree in both directions.
    pub fn agrees(&self) -> bool {
        self.undeclared.is_empty() && self.unproduced.is_empty()
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
                PRODUCERS.len()
            ),
            mode,
        ));
        out.push('\n');
        for producer in PRODUCERS {
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
                    "these are written by a producer and carry no `merge=headwater-regenerate`, \
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

    let declared = declared_paths(root);
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

    Population {
        outputs,
        declared,
        undeclared,
        unproduced,
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
    let Ok(text) = std::fs::read_to_string(root.join(".gitattributes")) else {
        return Vec::new();
    };
    let mut declared: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter(|line| line.contains("merge=headwater-regenerate"))
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect();
    declared.sort();
    declared.dedup();
    declared
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
            outputs: vec![lock, generated.clone()],
            declared: vec![
                ".headwater/taxonomy.lock".to_string(),
                "docs/handbook.md".to_string(),
            ],
            undeclared: vec![generated],
            unproduced: vec!["docs/handbook.md".to_string()],
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
        assert!(rendered.contains("these are written by a producer and carry no"));
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
                "\x1b[1;31mthese are written by a producer and carry no",
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
