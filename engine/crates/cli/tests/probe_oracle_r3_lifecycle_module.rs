// SPDX-License-Identifier: Apache-2.0
//! The hidden oracle of the probe
//! HW-PROBE-a-session-renames-the-module-of-the-check-crate-that-reads-the-state-a-document-stands-in
//! (#1874, slice 4b), row 3 of HW-DR-0112.
//!
//! This file names that probe in its first lines, so `tools/probe/seal.sh`
//! removes it from every workspace, and `tools/probe/grading-tree.sh` puts it
//! back in the grading copy. The `r3-lifecycle-module` oracle under `oracles:`
//! in `.headwater/probe.yml` holds it and runs it with `-- --ignored`, and
//! `--check-clean` over the records that govern the module.
//!
//! **Why the case is ignored.** The pin has not moved the module, so the case
//! fails on this repository's own tree by design, and CI never runs it. It
//! builds and lints on every change all the same.
//!
//! "Builds" is the build of this target: it sits in `headwater-cli`, whose
//! dependencies hold `headwater-check` and every crate that names the module,
//! so a caller left on the old path fails to compile. The case then reads the
//! tree it was built in:
//!
//! 1. The old file is gone and the new file is there.
//! 2. `headwater route "change <new file>" --json` offers the record
//!    that governs the module as an anchored pointer whose `anchors` hold the new
//!    file. A subsystem governs the whole crate by a glob, so an anchored
//!    pointer alone would pass a session that left the record pointing at
//!    nothing, and the record is named.

use headwater_yaml::json::{count, field};
use std::path::{Path, PathBuf};
use std::process::Command;

const OLD: &str = "engine/crates/check/src/lifecycle_state.rs";
const NEW: &str = "engine/crates/check/src/standing.rs";
const GOVERNOR: &str = "HW-DR-0026";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn at(document: &str, steps: &[&str]) -> Option<String> {
    field(
        document,
        &steps.iter().map(|step| step.to_string()).collect::<Vec<_>>(),
    )
}

fn len(document: &str, steps: &[&str]) -> usize {
    count(
        document,
        &steps.iter().map(|step| step.to_string()).collect::<Vec<_>>(),
    )
    .unwrap_or(0)
}

/// Whether `headwater route "change <path>" --json` offers `governor` as a
/// pointer that a governs anchor on `path` reached.
fn anchored(root: &Path, path: &str, governor: &str) -> Result<(), String> {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["route", &format!("change {path}"), "--json", "--root"])
        .arg(root)
        .output()
        .expect("the binary runs");
    let route = String::from_utf8(output.stdout).expect("the route is UTF-8");
    for index in 0..len(&route, &["pointers"]) {
        let index = index.to_string();
        if at(&route, &["pointers", &index, "id"]).as_deref() != Some(governor) {
            continue;
        }
        if at(&route, &["pointers", &index, "evidence", "by"]).as_deref() != Some("anchor") {
            return Err(format!("{governor} reaches {path}, and not by a governs anchor"));
        }
        let anchors = len(&route, &["pointers", &index, "evidence", "anchors"]);
        if (0..anchors).any(|slot| {
            at(&route, &["pointers", &index, "evidence", "anchors", &slot.to_string()]).as_deref()
                == Some(path)
        }) {
            return Ok(());
        }
        return Err(format!("{governor} is anchored, and its anchors do not hold {path}"));
    }
    Err(format!(
        "`headwater route \"change {path}\"` offers no pointer to {governor}:\n{route}"
    ))
}

#[test]
#[ignore = "the hidden oracle of one probe: it fails on the pin by design"]
fn the_module_moved_and_its_record_governs_the_new_file() {
    let root = repository();
    assert!(!root.join(OLD).exists(), "{OLD} is still in the tree");
    assert!(root.join(NEW).is_file(), "{NEW} is not in the tree");
    if let Err(why) = anchored(&root, NEW, GOVERNOR) {
        panic!("{why}");
    }
}
