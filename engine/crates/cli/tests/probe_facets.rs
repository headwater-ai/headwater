// SPDX-License-Identifier: Apache-2.0
//! `headwater new probe` over this repository's own taxonomy, for the `action`
//! facet that HW-DR-0112 adds (#1873).
//!
//! The scaffold case table already holds the generic refusal of a closed-set
//! value outside its set. What it cannot hold is that this repository's probe
//! kind requires `action`, that the set is the eleven rows of the grid, and
//! that `new` therefore asks for one and refuses `wibble`. A root here copies
//! `.headwater/overlay.yml` as it is, so an edit that drops `action` from the
//! kind's `require` or from the overlay turns these cases red.

mod common;

use common::Root;

const TITLE: &str = "A session finds the rule a widget breaks";

fn scaffold(root: &Root, action: Option<&str>) -> common::Ran {
    let action = action.map(|value| format!("action={value}"));
    let mut arguments = vec![
        "new",
        "probe",
        "--title",
        TITLE,
        "--facet",
        "probe_category=discovery",
        "--facet",
        "expectation=opened",
    ];
    if let Some(action) = &action {
        arguments.push("--facet");
        arguments.push(action);
    }
    root.run(&arguments)
}

fn probes(root: &Root) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(root.at.join("docs/probes"))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.file_name().is_some_and(|name| name != "README.md"))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn new_probe_refuses_an_action_outside_the_eleven() {
    let root = Root::new("probe-facets-wibble");
    let ran = scaffold(&root, Some("wibble"));
    assert_ne!(
        ran.code,
        Some(0),
        "an action outside the set was written: {ran:?}"
    );
    assert!(
        ran.err.contains("action") && ran.err.contains("wibble"),
        "the refusal names the facet and the value: {ran:?}"
    );
    for slug in [
        "ask_or_locate",
        "change_governed_code",
        "ungoverned_control",
    ] {
        assert!(
            ran.err.contains(slug),
            "the refusal prints the closed set, and `{slug}` is missing: {ran:?}"
        );
    }
    assert!(probes(&root).is_empty(), "a refused run wrote a probe");
}

#[test]
fn new_probe_writes_an_action_from_the_eleven() {
    let root = Root::new("probe-facets-ask");
    let ran = scaffold(&root, Some("ask_or_locate"));
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let written = probes(&root);
    assert_eq!(written.len(), 1, "{written:?}");
    let text = std::fs::read_to_string(&written[0]).expect("the probe reads");
    assert!(
        text.lines().any(|line| line == "action: ask_or_locate"),
        "the probe carries the action it was given:\n{text}"
    );
}

/// The kind requires `action`, a closed set with no engine role, so `new` asks
/// for it: it writes nothing and names `--facet action=<value>`. A probe written
/// with no action would be one `headwater probe plan` refuses.
#[test]
fn new_probe_asks_for_an_action() {
    let root = Root::new("probe-facets-ask-for");
    let ran = scaffold(&root, None);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err.contains("requires the facet `action`") && ran.err.contains("--facet action="),
        "the refusal asks for the action: {ran:?}"
    );
    assert!(probes(&root).is_empty(), "a run that asked wrote a probe");
}

/// A root whose taxonomy is this repository's and which carries its
/// `.headwater/probe.yml`, so `probe plan` reads past its declaration.
fn planned(label: &str) -> Root {
    Root::shaped(label, |at| {
        std::fs::copy(
            common::repository().join(".headwater/probe.yml"),
            at.join(".headwater/probe.yml"),
        )
        .expect("the probe declaration copies");
    })
}

/// `--action` and `--moment` are read against the sets the taxonomy declares,
/// and a value outside them is a caller's error at status 1 with the set
/// printed. A value inside them plans, and status 0 holds whatever the plan
/// says.
#[test]
fn probe_plan_refuses_an_action_or_a_moment_outside_the_taxonomy() {
    let root = planned("probe-facets-plan");
    let ran = root.run(&["probe", "plan", "--action", "wibble"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err.contains("`--action wibble`")
            && ran.err.contains("ask_or_locate")
            && ran.err.contains("ungoverned_control"),
        "the refusal prints the set: {ran:?}"
    );
    let ran = root.run(&["probe", "plan", "--moment", "bogus"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err.contains("`--moment bogus`") && ran.err.contains("intent, read, write, review"),
        "the refusal prints the set: {ran:?}"
    );
    for (flag, value) in [("--action", "ask_or_locate"), ("--moment", "write")] {
        let ran = root.run(&["probe", "plan", flag, value]);
        assert_eq!(ran.code, Some(0), "{flag} {value}: {ran:?}");
    }
}
