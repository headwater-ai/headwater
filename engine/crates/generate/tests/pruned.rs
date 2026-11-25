// SPDX-License-Identifier: Apache-2.0
//! The writer removes a marked rule file that the budget withholds, and the
//! removal is a write like any other (#2090).
//!
//! `agent_rules::emit` puts each withheld path in [`Plan::pruned`]. The cases
//! below build that plan by hand, so they hold the writer and the pass loop
//! of [`write_settled`] apart from the emitter, which `cli/tests/agent_rules.rs`
//! holds end to end.
//!
//! # Watched failing
//!
//! - With `Verdict::Removed` dropped from the `changed` test of
//!   `write_settled`, [`a_removal_alone_is_a_change_and_the_run_plans_again`]
//!   failed: the run planned once.
//! - With it dropped from the second test, the one that clears `withheld`,
//!   [`a_refusal_after_a_removal_does_not_say_the_run_wrote_nothing`] failed.
//! - With the removal tried before the refusal was read, as the first cut of
//!   `run` did, [`a_run_that_refuses_leaves_a_pruned_file_in_place`] failed:
//!   the file was gone and the report said `Withheld`.

use headwater_generate::{write, write_settled, Kind, Output, Plan, Report, Verdict};
use std::convert::Infallible;
use std::path::{Path, PathBuf};

/// The withheld rule file every case plants.
const PRUNED: &str = ".github/instructions/build.instructions.md";

/// A scratch root that removes itself.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(label: &str) -> Scratch {
    let root = std::env::temp_dir().join(format!(
        "headwater-generate-pruned-{}-{label}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the scratch root is made");
    Scratch(root)
}

fn marked(path: &str, kind: Kind) -> String {
    format!(
        "{}\n\nbody\n",
        headwater_mark::marker(kind.name(), path).expect("Markdown carries a marker")
    )
}

/// Writes a marked rule file at [`PRUNED`] under `root`.
fn plant(root: &Path) -> PathBuf {
    let path = root.join(PRUNED);
    std::fs::create_dir_all(path.parent().expect("the path has a parent"))
        .expect("the directory is made");
    std::fs::write(&path, marked(PRUNED, Kind::AgentRules)).expect("the rule file is planted");
    path
}

/// A plan that withholds [`PRUNED`] and writes nothing.
fn pruning() -> Plan {
    Plan {
        pruned: vec![PRUNED.to_string()],
        ..Plan::default()
    }
}

/// A plan that refuses before it writes: its one output climbs out of the
/// root.
fn refusing() -> Plan {
    Plan {
        outputs: vec![Output {
            path: "../escaped.md".to_string(),
            kind: Kind::ShelfIndex,
            committed: true,
            bytes: marked("../escaped.md", Kind::ShelfIndex),
        }],
        pruned: vec![PRUNED.to_string()],
        ..Plan::default()
    }
}

fn verdicts(report: &Report) -> Vec<(&str, &Verdict)> {
    report
        .wrote
        .iter()
        .map(|wrote| (wrote.path.as_str(), &wrote.verdict))
        .collect()
}

/// A pass that only removes a file changed the tree, so the loop plans again
/// over the tree it left, and that second pass finds nothing to do.
#[test]
fn a_removal_alone_is_a_change_and_the_run_plans_again() {
    let root = scratch("alone");
    let path = plant(&root.0);
    let mut turns = 0;
    let report = write_settled(&root.0, || {
        turns += 1;
        Ok::<_, Infallible>(pruning())
    })
    .unwrap_or_else(|never| match never {});

    assert!(!path.exists(), "the withheld rule file is removed");
    assert_eq!(turns, 2, "a removal is a change, so the run plans again");
    assert_eq!(verdicts(&report), [(PRUNED, &Verdict::Removed)]);
    assert!(!report.withheld, "the run removed a file");
    assert_eq!(report.unsettled, None, "the second pass changed nothing");
    assert!(!report.has_errors(), "{:?}", report.wrote);
}

/// A later pass that refuses does not make the run say it wrote nothing,
/// where the only thing an earlier pass did was remove a file.
#[test]
fn a_refusal_after_a_removal_does_not_say_the_run_wrote_nothing() {
    let root = scratch("then-refused");
    let path = plant(&root.0);
    let mut turns = 0;
    let report = write_settled(&root.0, || {
        turns += 1;
        Ok::<_, Infallible>(match turns {
            1 => pruning(),
            _ => refusing(),
        })
    })
    .unwrap_or_else(|never| match never {});

    assert!(!path.exists(), "the first pass removed the rule file");
    assert_eq!(turns, 2, "the removal made the run plan again");
    assert!(report.has_errors(), "the second pass refused");
    assert!(
        verdicts(&report).contains(&(PRUNED, &Verdict::Removed)),
        "{:?}",
        report.wrote
    );
    assert!(
        !report.withheld,
        "the run removed a file before it refused, so it did not write nothing"
    );
}

/// A run that refuses before it writes leaves a withheld rule file where it
/// was, as it leaves every output it would have written.
#[test]
fn a_run_that_refuses_leaves_a_pruned_file_in_place() {
    let root = scratch("refused");
    let path = plant(&root.0);
    let report = write(&root.0, &refusing());

    assert!(report.withheld, "the run refused before it wrote");
    assert!(path.is_file(), "a refused run removed the rule file");
    assert!(
        verdicts(&report).contains(&(PRUNED, &Verdict::Withheld)),
        "{:?}",
        report.wrote
    );
}
