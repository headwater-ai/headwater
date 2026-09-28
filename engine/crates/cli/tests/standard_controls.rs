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

/// The lifecycle rules that a control of the package names, one row each:
/// the rule, the one obligation its control discharges, the posture, and the
/// promotion record that states why the posture stands.
const LIFECYCLE: [(&str, &str, &str, &str); 6] = [
    (
        "lifecycle.transition.not_permitted",
        "OB-LIFE-1",
        "blocking",
        "final_posture",
    ),
    (
        "lifecycle.state.not_admitted",
        "OB-LIFE-2",
        "blocking",
        "final_posture",
    ),
    (
        "lifecycle.dependency.on_terminal",
        "OB-LIFE-3",
        "advisory",
        "permanently_advisory",
    ),
    (
        "lifecycle.deletion.not_permitted",
        "OB-LIFE-4",
        "blocking",
        "final_posture",
    ),
    (
        "lifecycle.dependency.on_initial",
        "OB-LIFE-5",
        "advisory",
        "permanently_advisory",
    ),
    (
        "lifecycle.state.not_set_by_edge",
        "OB-LIFE-6",
        "blocking",
        "final_posture",
    ),
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
        .map(|(rule, ..)| rule)
        .filter(|rule| flat.contains(&format!(" {rule} reaches ")))
        .collect();
    assert!(
        unbound.is_empty(),
        "these lifecycle rules reach no single obligation of headwater/standard: {unbound:?}"
    );
}

/// The declaration of the one control of the maintained package whose
/// mechanism is `check:<rule>`. Controls sit one to a block, a blank line
/// separates two blocks, and each block opens with its identifier.
fn control_of<'a>(taxonomy: &'a str, rule: &str) -> &'a str {
    let at = taxonomy
        .find("\ncontrols:\n")
        .expect("the package declares controls");
    let mechanism = format!("    mechanism: check:{rule}\n");
    let blocks: Vec<&str> = taxonomy[at..]
        .split("\n\n")
        .filter(|block| block.contains(&mechanism))
        .collect();
    assert_eq!(blocks.len(), 1, "one control names `{rule}`: {blocks:?}");
    blocks[0]
}

/// The posture a merge meets is part of what the package promises, and the
/// register counts postures without naming the control, so a control moved
/// from `blocking` to `advisory` changed no line the case above reads (the
/// verifier's finding on #1212). This case reads each lifecycle control where
/// the maintained package declares it.
#[test]
fn each_lifecycle_control_declares_its_obligation_posture_and_promotion() {
    let source = common::repository().join("taxonomy-source/headwater-standard/taxonomy.yml");
    let taxonomy = std::fs::read_to_string(&source).expect("the maintained package reads");
    for (rule, obligation, posture, promotion) in LIFECYCLE {
        let control = control_of(&taxonomy, rule);
        for line in [
            format!("    discharges: [{obligation}]\n"),
            format!("    posture: {posture}\n"),
            format!("      {promotion}:\n"),
        ] {
            assert!(
                control.contains(&line),
                "the control of `{rule}` does not declare `{}`:\n{control}",
                line.trim()
            );
        }
    }
}
