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

use headwater_census::Pattern;
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

/// The `governs` list under the `relations:` block of a document's front
/// matter, read for the fixed shape `headwater new` writes: an inline `[a, b]`
/// list or a block of `- a` lines. A `governs:` key anywhere else declares no
/// edge, and `headwater route` reads nothing from it, so it is not read here.
fn governs(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let front = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---").map(|(front, _)| front))
        .unwrap_or_else(|| panic!("{}: no front matter", path.display()));
    let unquote = |s: &str| s.trim().trim_matches('"').trim_matches('\'').to_string();
    let mut lines = front
        .lines()
        .skip_while(|line| line.trim_end() != "relations:")
        .skip(1)
        .take_while(|line| line.starts_with(' '));
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

/// The patterns of one `governs` entry, normalized the way the `source-tree`
/// resolver normalizes them. A block entry `- [a, b]` is a list anchor
/// ([HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md))
/// and gives each member. A value that does not normalize binds nothing, so
/// it reaches no crate and is left out.
fn patterns_of(entry: &str) -> Vec<String> {
    entry
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|member| member.trim().trim_matches('"').trim_matches('\''))
        .filter_map(|member| headwater_graph::anchors::normalize(member).ok())
        .collect()
}

/// Every pattern the `governs` list of the spec at `path` holds, each entry
/// normalized through `patterns_of`. The corpus case reads a spec through this
/// function and nothing else, so the case below holds what the corpus case
/// sees.
fn governed_patterns(path: &Path) -> BTreeSet<String> {
    governs(path)
        .iter()
        .flat_map(|entry| patterns_of(entry))
        .collect()
}

/// Each crate of `crates` outside `row` that `pattern` can reach: some path
/// under `engine/crates/<crate>/` that the pattern admits. So `engine/**`,
/// `engine/crates/**` and `./engine/crates/check/src/**` each reach `check`,
/// and `engine/crates/graph/src/**` reaches no crate but `graph`.
fn foreign_crates(pattern: &str, row: &[String], crates: &BTreeSet<String>) -> Vec<String> {
    let pattern = Pattern::new(pattern);
    crates
        .iter()
        .filter(|krate| !row.contains(krate))
        .filter(|krate| pattern.overlaps(&Pattern::new(&format!("engine/crates/{krate}/**"))))
        .cloned()
        .collect()
}

/// Every `.md` file under `dir` and its subdirectories except `README.md`,
/// as a path relative to `dir`.
fn specs_under(dir: &Path, prefix: &str, out: &mut BTreeSet<String>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("a directory entry reads");
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        if path.is_dir() {
            specs_under(&path, &format!("{prefix}{name}/"), out);
        } else if name.ends_with(".md") && name != "README.md" {
            out.insert(format!("{prefix}{name}"));
        }
    }
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
/// pattern per crate ([HW-DR-0074]). It governs no crate the row does not
/// name, because every crate belongs to exactly one subsystem (HW-DR-0098).
///
/// Watched failing two ways over the Taxonomy resolution row before it passed
/// (#1288): with the row linked and no spec on disk, and with the spec's
/// `governs` missing `engine/crates/hash/src/**` (the message named `hash`).
///
/// [HW-DR-0074]: ../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md
#[test]
fn a_row_that_links_a_subsystem_spec_is_governed_by_it() {
    let spec_six = root().join("docs/spec");
    let crates = workspace_crates();
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
        let governed = governed_patterns(&path);
        let foreign: Vec<String> = governed
            .iter()
            .flat_map(|pattern| {
                foreign_crates(pattern, &row.crates, &crates)
                    .into_iter()
                    .map(move |krate| format!("{pattern} reaches `{krate}`"))
            })
            .collect();
        assert!(
            foreign.is_empty(),
            "{link} governs a crate that its row of spec 6 '{HEADING}' does not name: {}",
            foreign.join("; ")
        );
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

/// Which crates a `governs` entry of the Graph build row reaches outside that
/// row. A spelling the resolver normalizes (`./`, `..`) and a pattern whose
/// literal prefix stops above one crate directory each reach the crates they
/// would govern, which is what `headwater route` reports for a file there.
///
/// Watched failing two ways before it passed (#1288): with `foreign_crates`
/// narrowed to `engine/crates/<crate>/src/**` (the `check/tests/**` row went
/// red), and with `overlaps` replaced by a match of `src/lib.rs` alone (the
/// `check/src/main.rs` row went red once the `check/tests/**` row was out).
#[test]
fn a_governs_pattern_that_reaches_outside_its_row_is_foreign() {
    let row = vec!["graph".to_string()];
    let crates: BTreeSet<String> = ["check", "graph", "hash"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let reach = |entry: &str| -> Vec<String> {
        patterns_of(entry)
            .iter()
            .flat_map(|pattern| foreign_crates(pattern, &row, &crates))
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect()
    };
    let cases: [(&str, &[&str]); 13] = [
        ("engine/crates/graph/src/**", &[]),
        ("engine/crates/check/tests/**", &["check"]),
        ("engine/crates/check/src/main.rs", &["check"]),
        ("engine/crates/hash/src/deep/mod.rs", &["hash"]),
        ("./engine/crates/graph/src/**", &[]),
        ("engine/crates/graph/tests/**", &[]),
        ("engine/crates/check/../graph/src/**", &[]),
        ("docs/spec/**", &[]),
        ("./engine/crates/check/src/**", &["check"]),
        ("engine/**", &["check", "hash"]),
        ("engine/crates/**", &["check", "hash"]),
        ("engine/crates/*/src/lib.rs", &["check", "hash"]),
        (
            "[engine/crates/graph/src/lib.rs, engine/crates/hash/src/lib.rs]",
            &["hash"],
        ),
    ];
    for (entry, expected) in cases {
        assert_eq!(
            reach(entry),
            expected,
            "the crates `{entry}` reaches outside its row"
        );
    }
}

/// The corpus case reads a spec's `governs` through `governed_patterns`, so a
/// spelling the resolver normalizes and a member of a list anchor each reach
/// the crate they would govern. The spec here is a temporary file written in
/// the shape `headwater new` writes, so the case holds the read path of the
/// corpus case and not a copy of it.
///
/// Watched failing before it passed (#1288): with `governed_patterns`
/// returning the raw entries and not calling `patterns_of`, it named the
/// unnormalized `./` entry and the list anchor.
#[test]
fn a_spec_read_through_governed_patterns_reaches_the_crates_it_names() {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "subsystem_map-governed_patterns-{}-{:?}-{}",
        std::process::id(),
        std::thread::current().id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("the temp dir is written");
    let spec = dir.join("spec.md");
    std::fs::write(
        &spec,
        "---\nid: HW-SPEC-temp\nrelations:\n  governs:\n    - ./engine/crates/check/src/**\n    - [engine/crates/graph/src/lib.rs, engine/crates/check/src/lib.rs]\n  traces_to:\n    - HW-SPEC-engine-architecture\n---\n\n# Temp\n",
    )
    .expect("the temp spec is written");
    let governed = governed_patterns(&spec);
    std::fs::remove_dir_all(&dir).expect("the temp dir is removed");
    let row = vec!["graph".to_string()];
    let crates: BTreeSet<String> = ["check", "graph", "hash"]
        .into_iter()
        .map(str::to_string)
        .collect();
    for pattern in [
        "engine/crates/check/src/**",
        "engine/crates/check/src/lib.rs",
    ] {
        assert!(
            governed.contains(pattern),
            "governed_patterns does not hold {pattern} (it holds {governed:?})"
        );
        assert_eq!(
            foreign_crates(pattern, &row, &crates),
            ["check"],
            "the crates `{pattern}` reaches outside its row"
        );
    }
}

/// Every subsystem spec on the shelf, in a subdirectory too, is linked from
/// exactly one row. With the case above, no spec on the shelf claims a crate
/// the map does not give it.
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
    let mut specs = BTreeSet::new();
    specs_under(&shelf, "", &mut specs);
    for spec in &specs {
        let count = linked.get(spec).copied().unwrap_or(0);
        assert_eq!(
            count, 1,
            "docs/subsystems/{spec} is linked from {count} rows of spec 6 '{HEADING}', not one"
        );
    }
}
