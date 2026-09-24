// SPDX-License-Identifier: Apache-2.0
//! Whether a generated shelf page stores a count of the documents it lists.
//!
//! # The defect this file was written against
//!
//! [#1058](https://github.com/headwater-ai/headwater/issues/1058).
//! [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
//! rules that a corpus-wide fold is derived and never stored. A shelf index
//! opened with "N documents on this shelf", and a sections page carried the
//! same count twice: once in that sentence and once in its front-matter
//! `summary`. Two branches that each add one document to the shelf both write
//! `N+1` on that one line. A text merge with no driver takes the line as an
//! identical change on both sides and writes `N+1`, where the merged tree holds
//! `N+2`. Nothing conflicts, and the page states a number true of neither
//! branch.
//!
//! # The two cases
//!
//! The first case reads every page the fixture tree generates and asserts that
//! no prose line holds the count of the rows the page lists. The second case
//! is the merge itself. Two branches add one document each, at positions in
//! the reading order that one unchanged row separates, and `git merge-file`
//! merges each page as text. The merged page must be the page that
//! `headwater generate` writes over the merged tree.
//!
//! A shelf page still keeps `merge=headwater-regenerate` after this change,
//! because its row order is a fold over the relation graph of the shelf and
//! not a record order. The second case places the two new documents where no
//! relation moves an existing row, so it measures the count alone.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{plan, Identity, Plan, Projections, Runs};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The pages the second case merges. Each one lists the `decisions` shelf.
const PAGES: [&str; 3] = [
    "generate/decisions/README.md",
    "generate/decisions/SECTIONS.md",
    "generate/archive/RETIRED.md",
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn identity() -> Identity {
    Identity {
        corpus_root: "generate".to_string(),
        exclusions: Vec::new(),
        package: "headwater/fixture".to_string(),
        version: "1.0.0".to_string(),
        lock: "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

/// The plan over the tree under `at`, whose corpus root is `at/generate`.
fn planned(at: &Path) -> Plan {
    let corpus = Corpus::new(at.to_path_buf(), "generate");
    let root = load_map(&fixtures_dir().join("generate.taxonomy.yml"));
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let relations = Declarations::read(&root).expect("the declarations read");
    let shape = Shape::read(&root).expect("the shape reads");
    let census: Census = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &census,
        &relations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let surface = Surface::over(&census, &graph, &shape, &taxonomy, &relations, &config);
    let projections = Projections::read(&root).expect("the projections read");
    plan(
        &surface,
        &census,
        &projections,
        &identity(),
        &Runs::default(),
        &[],
    )
}

fn written(plan: &Plan, path: &str) -> String {
    plan.outputs
        .iter()
        .find(|output| output.path == path)
        .unwrap_or_else(|| {
            panic!(
                "nothing written at {path}. Unwritten: {:?}",
                plan.unwritten
                    .iter()
                    .map(|entry| format!("{}: {}", entry.at, entry.reason))
                    .collect::<Vec<_>>()
            )
        })
        .bytes
        .clone()
}

/// A scratch directory for one case. `label` names the case, because cargo
/// runs the cases of one target as threads of one process.
fn scratch(label: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "headwater-generate-stored-count-{}-{label}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    copy_tree(&fixtures_dir().join("generate"), &at.join("generate"));
    at
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is created");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("the type reads").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("the file copies");
        }
    }
}

/// One plain decision that no relation places, so the reading order puts it
/// by its file name.
fn add_decision(at: &Path, number: u32) {
    let body = format!(
        "---\nid: DR-FIX-{number:04}\ntitle: Fixture decision {number}\nstatus: current\n\
         status_since: 2026-02-01\nsummary: a decision that no relation places\nprovenance:\n  \
         warrant: accepted\n  accepted_by: the fixture tree\n---\n\n# Fixture decision {number}\n\n\
         A row of the shelf and nothing else.\n"
    );
    std::fs::write(
        at.join(format!(
            "generate/decisions/{number:04}-fixture-decision.md"
        )),
        body,
    )
    .expect("the decision is written");
}

/// The prose lines of a page: every line that is not a row, a heading, a
/// fence or a front-matter member, except the `summary` member.
fn prose(bytes: &str) -> Vec<&str> {
    let mut in_front = false;
    let mut out = Vec::new();
    for (index, line) in bytes.lines().enumerate() {
        if line == "---" {
            // The first line opens a block, and the next fence closes it.
            in_front = index == 0;
            continue;
        }
        if in_front {
            if line.starts_with("summary:") {
                out.push(line);
            }
            continue;
        }
        if line.starts_with("- ")
            || line.starts_with('#')
            || line.starts_with('|')
            || line.starts_with("<!--")
            || line.starts_with("  ")
        {
            continue;
        }
        out.push(line);
    }
    out
}

/// The whole-word decimal numbers of a line.
fn numbers(line: &str) -> Vec<usize> {
    line.split(|c: char| !c.is_ascii_digit())
        .filter(|token| !token.is_empty() && !token.starts_with('0'))
        .filter_map(|token| token.parse().ok())
        .collect()
}

/// No prose line of a page that lists the shelf states how many it lists.
///
/// Five documents stand on the shelf, so a stored count is the number 5 and no
/// date or identifier of the fixture reads as that number.
#[test]
fn a_shelf_page_states_no_count_of_the_documents_it_lists() {
    let at = scratch("no-count");
    for number in [3, 5, 7] {
        add_decision(&at, number);
    }
    let plan = planned(&at);
    let _ = std::fs::remove_dir_all(&at);
    for page in PAGES {
        let bytes = written(&plan, page);
        for line in prose(&bytes) {
            assert!(
                !numbers(line).contains(&5),
                "{page} stores the count of the documents it lists, which a text merge gets \
                 wrong:\n{line}\n\n{bytes}"
            );
        }
    }
}

/// Two branches each add one document, and a text merge with no driver has to
/// write what `headwater generate` writes over the merged tree.
///
/// Before #1058 this failed on the count line: both branches wrote `6`, the
/// merge kept `6`, and the merged tree holds 7.
#[test]
fn a_driverless_merge_of_two_additions_writes_what_the_merged_tree_generates() {
    let base = scratch("base");
    for number in [3, 5, 7] {
        add_decision(&base, number);
    }
    let ours = scratch("ours");
    let theirs = scratch("theirs");
    let merged = scratch("merged");
    for at in [&ours, &theirs, &merged] {
        for number in [3, 5, 7] {
            add_decision(at, number);
        }
    }
    add_decision(&ours, 4);
    add_decision(&theirs, 6);
    add_decision(&merged, 4);
    add_decision(&merged, 6);

    let plans = [
        planned(&base),
        planned(&ours),
        planned(&theirs),
        planned(&merged),
    ];
    let pages = scratch("pages");
    for page in PAGES {
        let [b, o, t, m] = [0, 1, 2, 3].map(|i| written(&plans[i], page));
        let file = |name: &str, bytes: &str| {
            let path = pages.join(name);
            std::fs::write(&path, bytes).expect("the page is written");
            path
        };
        let (b_path, o_path, t_path) = (file("base", &b), file("ours", &o), file("theirs", &t));
        let output = Command::new("git")
            .args(["merge-file", "-p"])
            .arg(&o_path)
            .arg(&b_path)
            .arg(&t_path)
            .output()
            .expect("git merge-file runs");
        let text = String::from_utf8(output.stdout).expect("the merge is UTF-8");
        assert_eq!(
            output.status.code(),
            Some(0),
            "{page}: the two additions conflict as text, so this case measures nothing:\n{text}"
        );
        assert_eq!(
            text, m,
            "{page}: a text merge of two additions wrote a page that the merged tree does not \
             generate"
        );
    }
    for at in [&base, &ours, &theirs, &merged, &pages] {
        let _ = std::fs::remove_dir_all(at);
    }
}
