// SPDX-License-Identifier: Apache-2.0
//! Spec 6's subsystem map names every engine crate once, and a subsystem spec
//! it links governs the crates of its row.
//!
//! [HW-DR-0098](../../../../docs/decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)
//! rules that every crate under `engine/crates/` belongs to exactly one
//! subsystem, and that spec 6 keeps the map from each stage to its subsystem
//! spec. The map is a table of prose, so a crate added next month drifts from
//! it with nothing to say so. These cases are that review, read off the
//! directory listing, the table under `### Subsystems` in
//! `docs/spec/06-engine-architecture.md`, and the front matter of each file
//! under `docs/subsystems/`. The shape follows `verbs.rs`, which holds the
//! same part's CLI grammar block against `VERBS`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const HEADING: &str = "### Subsystems";

/// The repository root, from this crate's manifest directory.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// One body row of the table: the subsystem cell's link target, if any, and
/// the code spans of the crates cell.
struct Row {
    subsystem: String,
    link: Option<String>,
    crates: Vec<String>,
}

/// Every directory under `engine/crates/` that holds a `Cargo.toml`.
fn workspace_crates() -> BTreeSet<String> {
    let dir = root().join("engine/crates");
    std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.expect("a directory entry reads").path())
        .filter(|path| path.join("Cargo.toml").is_file())
        .map(|path| {
            path.file_name()
                .expect("a crate directory has a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// The text between each pair of backticks in `cell`.
fn code_spans(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// The target of the first Markdown link in `cell`.
fn link_target(cell: &str) -> Option<String> {
    let (_, after) = cell.split_once("](")?;
    let (target, _) = after.split_once(')')?;
    Some(target.to_string())
}

fn cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

/// The body rows of the one table under `### Subsystems` in spec 6. The
/// columns are found by their header names `subsystem` and `crates`, so a
/// column added later for the reason of a row moves nothing here.
fn spec_six_rows() -> Vec<Row> {
    let path = root().join("docs/spec/06-engine-architecture.md");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (_, after_heading) = text
        .split_once(&format!("\n{HEADING}\n"))
        .unwrap_or_else(|| panic!("{}: no '{HEADING}' heading", path.display()));
    let section = after_heading
        .split("\n#")
        .next()
        .expect("split yields at least one piece");
    let mut table = section
        .lines()
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'));
    let header = cells(
        table
            .next()
            .unwrap_or_else(|| panic!("{}: no table under '{HEADING}'", path.display())),
    );
    let column = |name: &str| {
        header
            .iter()
            .position(|cell| cell.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| {
                panic!(
                    "{}: the table under '{HEADING}' has no '{name}' column",
                    path.display()
                )
            })
    };
    let (subsystem, crates) = (column("subsystem"), column("crates"));
    table
        .skip(1)
        .map(|line| {
            let row = cells(line);
            Row {
                subsystem: row[subsystem].clone(),
                link: link_target(&row[subsystem]),
                crates: code_spans(&row[crates]),
            }
        })
        .collect()
}

/// The `governs` list of a document's front matter, read for the fixed shape
/// `headwater new` writes: an inline `[a, b]` list or a block of `- a` lines.
fn governs(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let front = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---").map(|(front, _)| front))
        .unwrap_or_else(|| panic!("{}: no front matter", path.display()));
    let unquote = |s: &str| s.trim().trim_matches('"').trim_matches('\'').to_string();
    let mut lines = front.lines();
    while let Some(line) = lines.next() {
        let Some(value) = line.trim_start().strip_prefix("governs:") else {
            continue;
        };
        let value = value.trim();
        if let Some(inline) = value.strip_prefix('[') {
            return inline
                .trim_end_matches(']')
                .split(',')
                .map(unquote)
                .filter(|s| !s.is_empty())
                .collect();
        }
        return lines
            .map_while(|line| line.trim_start().strip_prefix("- ").map(unquote))
            .collect();
    }
    Vec::new()
}

/// Every crate directory is in the map, and in one row only.
///
/// Watched failing three ways before it passed: with no `### Subsystems`
/// heading, with the `graph` row's crate removed (the message named
/// `graph`), and with `hash` written into a second row.
#[test]
fn every_crate_is_in_exactly_one_row_of_spec_6s_subsystem_map() {
    let mut rows_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in spec_six_rows() {
        for krate in row.crates {
            rows_of
                .entry(krate)
                .or_default()
                .push(row.subsystem.clone());
        }
    }
    let twice: Vec<String> = rows_of
        .iter()
        .filter(|(_, rows)| rows.len() > 1)
        .map(|(krate, rows)| format!("`{krate}` in {}", rows.join(" and ")))
        .collect();
    assert!(
        twice.is_empty(),
        "spec 6 '{HEADING}' writes a crate in more than one row: {}",
        twice.join("; ")
    );
    let mapped: BTreeSet<String> = rows_of.into_keys().collect();
    let crates = workspace_crates();
    let missing: Vec<&String> = crates.difference(&mapped).collect();
    assert!(
        missing.is_empty(),
        "engine/crates/ has a crate that no row of spec 6 '{HEADING}' names: {missing:?}"
    );
    let unknown: Vec<&String> = mapped.difference(&crates).collect();
    assert!(
        unknown.is_empty(),
        "spec 6 '{HEADING}' names a crate that engine/crates/ does not have: {unknown:?}"
    );
}

/// A row that links a subsystem spec names a file that exists, and that file
/// governs `engine/crates/<crate>/src/**` for each crate of the row, one
/// pattern per crate ([HW-DR-0074]).
///
/// [HW-DR-0074]: ../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md
#[test]
fn a_row_that_links_a_subsystem_spec_is_governed_by_it() {
    let spec_six = root().join("docs/spec");
    let mut linked = 0;
    for row in spec_six_rows() {
        let Some(link) = row.link else { continue };
        linked += 1;
        let path = spec_six.join(&link);
        assert!(
            path.is_file(),
            "spec 6 '{HEADING}' row {} links {link}, which does not exist",
            row.subsystem
        );
        let governed: BTreeSet<String> = governs(&path).into_iter().collect();
        for krate in &row.crates {
            let pattern = format!("engine/crates/{krate}/src/**");
            assert!(
                governed.contains(&pattern),
                "{link} is the spec of the row that names `{krate}`, and its `governs` does not hold {pattern} (it holds {governed:?})"
            );
        }
    }
    assert!(
        linked > 0,
        "no row of spec 6 '{HEADING}' links a subsystem spec"
    );
}

/// Every subsystem spec is linked from exactly one row, so no spec on the
/// shelf claims crates the map does not give it.
#[test]
fn a_subsystem_spec_is_named_by_a_row() {
    let shelf = root().join("docs/subsystems");
    let mut linked: BTreeMap<String, usize> = BTreeMap::new();
    for row in spec_six_rows() {
        if let Some(link) = row.link {
            if let Some(file) = link.strip_prefix("../subsystems/") {
                *linked.entry(file.to_string()).or_default() += 1;
            }
        }
    }
    let specs: BTreeSet<String> = std::fs::read_dir(&shelf)
        .unwrap_or_else(|e| panic!("{}: {e}", shelf.display()))
        .map(|entry| entry.expect("a directory entry reads").file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".md") && name != "README.md")
        .collect();
    for spec in &specs {
        let count = linked.get(spec).copied().unwrap_or(0);
        assert_eq!(
            count, 1,
            "docs/subsystems/{spec} is linked from {count} rows of spec 6 '{HEADING}', not one"
        );
    }
}
