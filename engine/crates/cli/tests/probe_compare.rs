// SPDX-License-Identifier: Apache-2.0
//! `headwater probe compare`, end to end over the committed screening
//! transcripts of 2026-10-18 (#1820).
//!
//! The library test `crates/probe/tests/compare.rs` holds the renderer to the
//! published rows from counts. This file holds the path from a transcript to
//! those counts: both transcripts are graded the way `probe grade` grades one,
//! and the pooled line has to come out as the screening evaluation printed it.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn screening(scenario: u32) -> String {
    format!("docs/probe-runs/additive-screening-of-2026-10-18-scenario-{scenario}.md")
}

fn compare(arguments: &[&str]) -> std::process::Output {
    let root = repository();
    Command::new(env!("CARGO_BIN_EXE_headwater"))
        .arg("probe")
        .arg("compare")
        .args(arguments)
        .arg("--root")
        .arg(&root)
        .current_dir(&root)
        .stdin(Stdio::null())
        .output()
        .expect("the binary runs")
}

fn pooled(treated: u32, control: u32) -> String {
    let output = compare(&[&screening(treated), &screening(control)]);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(
        output.status.code(),
        Some(0),
        "compare exits 0 on a comparison; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
        .lines()
        .find(|line| line.starts_with("pooled: "))
        .unwrap_or_else(|| panic!("no pooled line in:\n{stdout}"))
        .to_string()
}

#[test]
fn scenario_1_against_scenario_0_is_the_published_row() {
    assert_eq!(
        pooled(1, 0),
        "pooled: 39/72 against 11/72, +38.9 points [+23.7, +51.7]"
    );
}

#[test]
fn scenario_3_against_scenario_1_is_the_published_row() {
    assert_eq!(
        pooled(3, 1),
        "pooled: 40/72 against 39/72, +1.4 points [-14.5, +17.2]"
    );
}

#[test]
fn each_probe_both_sides_ran_has_a_line_and_the_cells_sum_to_the_pool() {
    let output = compare(&[&screening(1), &screening(0)]);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 5, "four probes and the pool:\n{stdout}");
    let (mut treated, mut control) = ((0, 0), (0, 0));
    for line in &lines[..4] {
        // `<probe>: <x1>/<n1> against <x0>/<n0>, ...`
        let counts = line
            .split_once(": ")
            .and_then(|(_, rest)| rest.split_once(','))
            .map(|(counts, _)| counts)
            .unwrap_or_else(|| panic!("a probe line has counts: {line}"));
        let (one, other) = counts.split_once(" against ").expect("two sides");
        for (side, total) in [(one, &mut treated), (other, &mut control)] {
            let (x, n) = side.split_once('/').expect("a count");
            total.0 += x.parse::<usize>().expect("a number");
            total.1 += n.parse::<usize>().expect("a number");
        }
    }
    assert_eq!((treated, control), ((39, 72), (11, 72)), "{stdout}");
}

#[test]
fn a_missing_operand_or_an_unreadable_transcript_exits_1() {
    let screening_1 = screening(1);
    let one = compare(&[&screening_1]);
    assert_eq!(one.status.code(), Some(1));
    let missing = compare(&[&screening_1, "docs/probe-runs/no-such-transcript.md"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
}
