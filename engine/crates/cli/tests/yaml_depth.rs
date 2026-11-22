// SPDX-License-Identifier: Apache-2.0
//! Front matter nested past the loader's bound, through the binary (#1966).
//!
//! The yaml crate's case table holds where the bound is and what the refusal
//! says. This file holds what the issue exists for: a source nested 20,000
//! deep in block style killed the process with a signal and printed nothing,
//! because the loader recursed once for each level and overflowed the stack.
//! `check --strict` and `generate` must each end with an exit status, and the
//! report must name the file and a `line:col` in it.

mod common;
use common::{Ran, Root};

/// Far past the bound, and deep enough that the unbounded recursion overflowed
/// the stack of a debug test binary before the fix.
const LEVELS: usize = 20_000;

const NAME: &str = "9999-nested-past-the-bound.md";

fn deep_front_matter() -> String {
    format!(
        "---\nx:\n  {}1\n---\n# Nested past the bound\n\nThe front matter above nests too deep to load.\n",
        "- ".repeat(LEVELS)
    )
}

/// Each case takes its own label: the root directory is keyed on the process
/// and the label, and the cases of one file run as threads of one process.
fn root(label: &str) -> Root {
    Root::shaped(label, |at| {
        let path = at.join("docs/decisions").join(NAME);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        std::fs::write(path, deep_front_matter()).expect("the document writes");
    })
}

/// The process ended with a status rather than a signal, and the report names
/// the file with the position of the refusal.
fn assert_reported(verb: &str, ran: &Ran) {
    assert!(
        matches!(ran.code, Some(0) | Some(1)),
        "`{verb}` did not end with a status (a signal reads as None): {:?}\nstderr:\n{}",
        ran.code,
        tail(&ran.err)
    );
    let both = format!("{}\n{}", ran.out, ran.err);
    let named = both
        .lines()
        .find(|line| line.contains(NAME) && line.contains("nest"));
    let line = named.unwrap_or_else(|| {
        panic!(
            "`{verb}` did not name {NAME} with the depth refusal:\nstdout:\n{}\nstderr:\n{}",
            tail(&ran.out),
            tail(&ran.err)
        )
    });
    // `3:` is the line of the front matter that opens the deep list: the
    // opening `---` is line 1 and `x:` is line 2.
    assert!(
        line.contains(":3:"),
        "`{verb}` named the file without the line and column of the refusal: {line}"
    );
}

fn tail(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(20)..].join("\n")
}

#[test]
fn check_strict_reports_front_matter_nested_past_the_bound() {
    let root = root("yaml-depth-check");
    let ran = root.run(&["check", "--strict"]);
    assert_reported("check --strict", &ran);
}

#[test]
fn generate_reports_front_matter_nested_past_the_bound() {
    let root = root("yaml-depth-generate");
    let ran = root.run(&["generate"]);
    assert_reported("generate", &ran);
}
