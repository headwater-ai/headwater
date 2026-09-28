// SPDX-License-Identifier: Apache-2.0
//! Six lifecycle rules each reach one obligation of `headwater/standard` (#1212).
//!
//! A rule reaches an obligation only through a control of the package that
//! names it, and nothing tied a new rule to its control. So
//! `lifecycle.state.not_set_by_edge` landed with #1198 and reached no
//! obligation, and only a reader of the printed register saw it. This target
//! copies the maintained package into a root, runs `headwater check` over it
//! and reads the register block, so a lifecycle rule that loses its control
//! fails here and names itself. A new lifecycle rule is held here only once
//! somebody adds its row.
//!
//! The table holds six of the seven lifecycle rules the engine carries, and
//! no rule of another family. `lifecycle.state.set_twice` came with #1198 as
//! well and still reaches no obligation. Other rules reach none over this
//! repository too (HW-OBL-0170). A row for any of them would be red for a
//! reason that is not this issue's, so each gets its row with its control.

mod common;
use common::Root;

/// The lifecycle rules that a control of the package names, one row each.
const LIFECYCLE: [&str; 6] = [
    "lifecycle.transition.not_permitted",
    "lifecycle.state.not_admitted",
    "lifecycle.dependency.on_terminal",
    "lifecycle.deletion.not_permitted",
    "lifecycle.dependency.on_initial",
    "lifecycle.state.not_set_by_edge",
];

#[test]
fn every_lifecycle_rule_reaches_one_obligation_of_the_standard_package() {
    let root = Root::shaped("standard-controls", |_| {});
    let ran = root.run(&["check"]);
    // The register wraps a line at the report's width, and a rule bound to
    // two obligations prints a line long enough to wrap, so the case reads the
    // output with each run of whitespace folded to one space.
    let flat = ran.out.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains("so it names none")
            || flat.contains("every rule this engine carries reaches one obligation"),
        "the register block is not in the output, so this case reads nothing: {ran:?}"
    );
    // The register prints a line for a rule only where the rule reaches no
    // obligation or more than one, and each such line opens with the rule.
    let unbound: Vec<&str> = LIFECYCLE
        .into_iter()
        .filter(|rule| flat.contains(&format!(" {rule} reaches ")))
        .collect();
    assert!(
        unbound.is_empty(),
        "these lifecycle rules reach no single obligation of headwater/standard: {unbound:?}"
    );
}
