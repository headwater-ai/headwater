// SPDX-License-Identifier: Apache-2.0
//! `headwater taxonomy audit` counts the governed scope over what git does
//! not ignore (#951, owner ruling 2026-09-25: "local and CI must agree, and
//! nobody governs a cache").
//!
//! The graph build counts the scope with nothing ignored, because spec 12
//! keeps git off the check loop, and the audit verb drops the ignored entries
//! afterward. This target holds that second step through the binary, so a
//! verb that stopped dropping them prints a different figure here.

mod common;
use common::Root;
use std::process::Command;

/// The line of the scope section that names `pattern`.
fn line_for<'a>(out: &'a str, pattern: &str) -> &'a str {
    out.lines()
        .find(|line| line.contains(&format!("`{pattern}`")) && line.contains("in scope"))
        .unwrap_or_else(|| panic!("no scope line for `{pattern}` in:\n{out}"))
}

#[test]
fn the_audit_leaves_a_file_git_ignores_out_of_the_governed_scope() {
    let root = Root::new("scope-audit-ignored");
    let cache = root.at.join("tools/__pycache__/stub.cpython-312.pyc");
    std::fs::create_dir_all(cache.parent().expect("a parent")).expect("the cache is made");
    std::fs::write(&cache, "").expect("the cache writes");
    std::fs::write(root.at.join(".gitignore"), "__pycache__/\n").expect("the ignore file writes");

    // Outside a git repository nothing is ignored, so the cache counts.
    let before = root.run(&["taxonomy", "audit", "--now", "2026-09-25"]);
    assert_eq!(before.code, Some(0), "{before:?}");
    let line = line_for(&before.out, "tools/**");
    assert!(line.contains(" 2 in scope"), "{line}");

    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root.at)
        .status()
        .expect("git runs");
    assert!(init.success());

    let after = root.run(&["taxonomy", "audit", "--now", "2026-09-25"]);
    assert_eq!(after.code, Some(0), "{after:?}");
    let line = line_for(&after.out, "tools/**");
    assert!(line.contains(" 1 in scope"), "{line}");
    assert!(!after.out.contains("__pycache__"), "{}", after.out);
}
