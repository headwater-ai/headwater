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
    if path.starts_with("site/")
        && path.ends_with(".html")
        && text.contains(FIGURE)
    {
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
