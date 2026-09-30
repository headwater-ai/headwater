// SPDX-License-Identifier: Apache-2.0
//! Every rule the engine carries reaches one obligation of `headwater/standard`
//! (#1212, #1492).
//!
//! A rule reaches an obligation only through a control of the package that
//! names it, and nothing tied a new rule to its control. So
//! `lifecycle.state.not_set_by_edge` landed with #1198 and reached no
//! obligation, and only a reader of the printed register saw it, and twelve
//! rules had done the same by the time HW-OBL-0170 counted them. This target
//! copies the maintained package into a root, runs `headwater check` over it
//! and reads the register block. The first case below fails on every rule that
//! reaches no single obligation and names each one, so a new rule that ships
//! without its control fails here.
//!
//! The lifecycle table holds all seven lifecycle rules the engine carries, and
//! the second case reads the obligation, posture and promotion each control of
//! that family declares, which the register counts without naming.

mod common;
use common::Root;

/// The lifecycle rules that a control of the package names, one row each:
/// the rule, the one obligation its control discharges, the posture, and the
/// promotion record that states why the posture stands.
const LIFECYCLE: [(&str, &str, &str, &str); 7] = [
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
    ),    (
        "lifecycle.state.set_twice",
        "OB-LIFE-7",
        "advisory",
        "permanently_advisory",
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

/// The rules that may reach no single obligation of `headwater/standard`, each
/// with the reason it stands outside the package. A rule is named here one at
/// a time and never by a pattern, so a new rule that ships without a control
/// fails the case below and names itself (#1492).
const EXCEPTIONS: &[(&str, &str)] = &[];

/// Every rule the engine carries reaches exactly one obligation of the
/// maintained package, except the rules [`EXCEPTIONS`] names (#1492,
/// HW-OBL-0170). The register prints a line for each rule that reaches none
/// or several, so the set of rules on those lines is the set this case holds.
/// A register-line count would not name the rule, and a list of the bound
/// rules would not see a new one.
#[test]
fn every_rule_reaches_one_obligation_of_the_standard_package() {
    let root = Root::shaped("standard-controls-every-rule", |_| {});
    let ran = root.run(&["check"]);
    let flat = ran.out.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains("so it names none")
            || flat.contains("every rule this engine carries reaches one obligation"),
        "the register block is not in the output, so this case reads nothing: {ran:?}"
    );
    let words: Vec<&str> = flat.split(' ').collect();
    let mut unbound: Vec<&str> = words
        .windows(2)
        .filter(|pair| pair[1] == "reaches" && pair[0].contains('.'))
        .map(|pair| pair[0])
        .filter(|rule| rule.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'))
        .collect();
    unbound.sort_unstable();
    unbound.dedup();
    let mut expected: Vec<&str> = EXCEPTIONS.iter().map(|(rule, _)| *rule).collect();
    expected.sort_unstable();
    let unexpected: Vec<&str> = unbound
        .iter()
        .copied()
        .filter(|rule| !expected.contains(rule))
        .collect();
    let stale: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|rule| !unbound.contains(rule))
        .collect();
    assert!(
        unexpected.is_empty(),
        "these rules reach no single obligation of headwater/standard, so the register names none for each: {unexpected:?}"
    );
    assert!(
        stale.is_empty(),
        "these exceptions now reach one obligation, so remove them from EXCEPTIONS: {stale:?}"
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
