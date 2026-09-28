// SPDX-License-Identifier: Apache-2.0
//! Each lifecycle rule reaches one obligation of `headwater/standard` (#1212).
//!
//! A rule reaches an obligation only through a control of the package that
//! names it, and nothing tied a new rule to its control. So
//! `lifecycle.state.not_set_by_edge` landed with #1198 and reached no
//! obligation, and only a reader of the printed register saw it. This target
//! copies the maintained package into a root, runs `headwater check` over it
//! and reads the register block, so a lifecycle rule that loses its control,
//! or arrives without one, fails here and names itself.
//!
//! The table holds the lifecycle rules alone. Other rules the engine carries
//! reach no obligation over this repository too (HW-OBL-0170), and a total
//! assertion would be red on them for a reason that is not this one.

mod common;
use common::Root;

/// Every rule of the lifecycle family the engine carries.
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
    assert!(
        ran.out.contains("reaches no obligation") || ran.out.contains("every rule this engine carries reaches one obligation"),
        "the register block is not in the output, so this case reads nothing: {ran:?}"
    );
    let unbound: Vec<&str> = LIFECYCLE
        .into_iter()
        .filter(|rule| {
            ran.out.lines().any(|line| {
                let line = line.trim_start();
                line.starts_with(&format!("{rule} reaches ")) && line.ends_with("so it names none")
            })
        })
        .collect();
    assert!(
        unbound.is_empty(),
        "these lifecycle rules reach no single obligation of headwater/standard: {unbound:?}"
    );
}
