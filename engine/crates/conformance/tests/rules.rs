// SPDX-License-Identifier: Apache-2.0
//! What a conformance report may claim, and every way this crate refuses to let
//! it claim more.
//!
//! # Two grains, and the reason they are separate
//!
//! The refusals and the ladder are tested over text and hand-built verdicts,
//! with no corpus at all. That is deliberate. The claim these tests exist to
//! hold is that **a waiver cannot buy a rung**, and a test that had to build a
//! corpus to ask it would measure the readings and the ladder at once. A defect
//! in either would then read as a defect in the other.
//!
//! The readings are tested over this repository's own tree, because the four of
//! them read a pin, a lock, a census and a projection plan, and no fixture
//! smaller than a repository carries all four. Two of them are asserted as gaps
//! and two as met, and each assertion states the fact about this tree that
//! produces it.

use headwater_check::context::Date;
use headwater_conformance::{
    assemble, pin_check, read, waivers, Cover, DecidedBy, Identity, Installed, PinCheck, Reason,
    SetError, Verdict, Waiver,
};
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn at(text: &str) -> Date {
    Date::parse(text).expect("a date")
}

/// The pin statement for a report assembled in memory from hand-built verdicts.
///
/// Every identity below pins no digest, so `render` prints no pin line and this
/// value reaches nothing these cases assert. The case that does assert the line
/// calls [`pin_check`] over a real package directory instead.
fn no_pin() -> PinCheck {
    PinCheck::Unchecked(Installed::NoRecord)
}

/// A rendered report with every run of whitespace collapsed to one space.
///
/// The report is filled to a width, so a sentence of it is a sentence only
/// after this: `contains` over the raw render asks whether the words happen to
/// fall on one line, which is a question about the fill and not about what the
/// report says.
fn collapsed(rendered: &str) -> String {
    rendered.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A rule set with one tree rule and one attestation rule, over two rungs.
fn source(tree_rule: &str) -> String {
    format!(
        "\
conformance:
  format: 1
  rules:
    - name: {tree_rule}
      title: The lock is current
      decided_by: tree
      statement: it is
      remediation: run resolve
    - name: corpus.classified
      title: Everything is classified
      decided_by: tree
      statement: it is
      remediation: declare a kind
    - name: gates.required
      title: The gate is required
      decided_by: attestation
      statement: nothing here decides it
      remediation: record an attestation
  levels:
    - name: L0
      title: Pointed at
      rules: [{tree_rule}]
    - name: L1
      title: Classified
      rules: [corpus.classified]
"
    )
}

fn waiver(rule: &str, until: &str) -> Waiver {
    Waiver {
        rule: rule.to_string(),
        reason: Reason::AcceptedDeviation,
        owner: "a team".to_string(),
        until: at(until),
        note: None,
    }
}

// ---------------------------------------------------------------------------
// The refusals at read time
// ---------------------------------------------------------------------------

/// **The refusal this whole design rests on.** A package may declare a tree rule
/// that the consumer's engine holds no reading for, and the run ends rather than
/// skipping it. An engine that skipped it would report a level over a rule set
/// that the publisher and the consumer disagree about.
#[test]
fn a_tree_rule_this_engine_holds_no_reading_for_ends_the_run() {
    let refused = read(&source("acme.invented_rule"), "acme/taxonomy")
        .expect_err("this engine holds no reading for it");
    assert_eq!(
        refused,
        SetError::NoReading {
            rule: "acme.invented_rule".to_string(),
            package: "acme/taxonomy".to_string(),
        }
    );
    // The message names both, because a consumer who meets it has to know which
    // package asked for what.
    let rendered = refused.to_string();
    assert!(rendered.contains("acme.invented_rule"));
    assert!(rendered.contains("acme/taxonomy"));
    assert!(rendered.contains("requires_engine"));
}

/// An attestation rule needs no reading, so a name this engine never heard of is
/// fine there. Without this arm the test above would pass an implementation that
/// refused every unfamiliar name, and a publisher could then never add an
/// attestation rule at all.
#[test]
fn an_attestation_rule_needs_no_reading_and_any_name_is_taken() {
    let text = "\
conformance:
  format: 1
  rules:
    - name: acme.someone_signed_a_form
      title: Somebody signed a form
      decided_by: attestation
      statement: no tree decides it
      remediation: record an attestation
  levels: []
";
    let set = read(text, "acme/taxonomy").expect("an attestation rule needs no reading");
    assert_eq!(set.rules.len(), 1);
    assert_eq!(set.rules[0].decided_by, DecidedBy::Attestation);
}

/// A rung that names no rule is a rung every repository already stands on.
#[test]
fn a_level_with_no_rule_is_refused() {
    let text = "\
conformance:
  format: 1
  rules:
    - name: lock.current
      title: t
      decided_by: tree
      statement: s
      remediation: r
  levels:
    - name: L0
      title: Free
      rules: []
";
    assert_eq!(
        read(text, "acme/taxonomy").expect_err("an empty rung earns nothing"),
        SetError::EmptyLevel("L0".to_string())
    );
}

/// A rung that names a rule the set does not declare would report a level over
/// a requirement nothing evaluated, from the other direction.
#[test]
fn a_level_that_names_an_undeclared_rule_is_refused() {
    let text = "\
conformance:
  format: 1
  rules:
    - name: lock.current
      title: t
      decided_by: tree
      statement: s
      remediation: r
  levels:
    - name: L0
      title: Pointed at
      rules: [lock.current, pin.current]
";
    assert_eq!(
        read(text, "acme/taxonomy").expect_err("pin.current is not in this set"),
        SetError::UnknownInLevel {
            level: "L0".to_string(),
            rule: "pin.current".to_string(),
        }
    );
}

#[test]
fn a_format_this_engine_does_not_read_is_refused_rather_than_guessed_at() {
    let later = source("lock.current").replace("format: 1", "format: 9");
    assert!(matches!(
        read(&later, "acme/taxonomy"),
        Err(SetError::Format { .. })
    ));
}

#[test]
fn a_rule_that_states_no_decided_by_is_refused() {
    let text = "\
conformance:
  format: 1
  rules:
    - name: lock.current
      title: t
      statement: s
      remediation: r
  levels: []
";
    assert!(matches!(
        read(text, "acme/taxonomy"),
        Err(SetError::Malformed(_))
    ));
}

// ---------------------------------------------------------------------------
// The waiver reader, and its four required fields
// ---------------------------------------------------------------------------

/// Every field of a waiver is required, and each absence is its own refusal.
/// The four are asserted in one test because the property is the set: a waiver
/// missing any one of them is a deviation that nobody closes.
#[test]
fn a_waiver_missing_any_of_its_four_fields_is_refused() {
    let complete = "\
taxonomy:
  package: acme/taxonomy
  version: 1.0.0
corpus:
  root: docs
conformance:
  waivers:
    - rule: pin.current
      reason: accepted_deviation
      owner: a team
      until: 2027-01-01
";
    let tmp = std::env::temp_dir().join(format!("headwater-waiver-{}", std::process::id()));
    let write = |text: &str| {
        let dir = tmp.join(".headwater");
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        std::fs::write(dir.join("taxonomy.yml"), text).expect("write");
    };

    write(complete);
    let read_back = waivers(&tmp).expect("a complete waiver reads");
    assert_eq!(read_back.len(), 1);
    assert_eq!(read_back[0].rule, "pin.current");
    assert_eq!(read_back[0].reason, Reason::AcceptedDeviation);
    assert_eq!(read_back[0].owner, "a team");
    assert_eq!(read_back[0].until, at("2027-01-01"));

    for (field, line) in [
        ("rule", "      rule: pin.current\n"),
        ("reason", "      reason: accepted_deviation\n"),
        ("owner", "      owner: a team\n"),
        ("until", "      until: 2027-01-01\n"),
    ] {
        // The `rule:` key opens the sequence entry, so removing it needs the
        // dash to move onto the next key rather than leaving a bare item.
        let short = match field {
            "rule" => complete.replace("    - rule: pin.current\n", "    - reason: x\n"),
            _ => complete.replace(line, ""),
        };
        write(&short);
        let refused =
            waivers(&tmp).expect_err(&format!("a waiver with no `{field}` is not a waiver"));
        assert!(
            !refused.is_empty(),
            "a waiver with no `{field}` was taken silently"
        );
    }

    write(&complete.replace("2027-01-01", "next Tuesday"));
    assert!(
        waivers(&tmp).is_err(),
        "an expiry that is not a date was taken"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// **The probe an adopter is most likely to try, and it has to fail loudly.**
/// `waivers` is the only key the `conformance` block takes. A `level:` key reads
/// as a declaration and is not one, so an engine that ignored it would leave an
/// adopter holding a claim that nothing evaluated and nothing contradicted.
///
/// A silent drop here is the same defect as an engine that skipped a rule it
/// could not read, from the consumer's side rather than the publisher's.
#[test]
fn a_key_this_engine_does_not_read_ends_the_run_and_a_level_is_the_one_to_expect() {
    let tmp = std::env::temp_dir().join(format!("headwater-unknown-key-{}", std::process::id()));
    let dir = tmp.join(".headwater");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let write = |text: &str| std::fs::write(dir.join("taxonomy.yml"), text).expect("write");

    // The declaration an adopter would reach for to claim a rung.
    write(
        "\
taxonomy:
  package: acme/taxonomy
  version: 1.0.0
corpus:
  root: docs
conformance:
  level: L2
",
    );
    let refused = waivers(&tmp).expect_err("a level is not a thing an adopter declares");
    assert_eq!(refused.len(), 1);
    assert!(refused[0].contains("conformance.level"));
    assert!(
        refused[0].contains("derives the level from the rules that pass"),
        "the message has to say what does decide a level, or an adopter learns nothing"
    );

    // Any other unread key, so the refusal is about the closed set rather than
    // about the one word `level`.
    write(
        "\
taxonomy:
  package: acme/taxonomy
  version: 1.0.0
corpus:
  root: docs
conformance:
  target_rung: L2
",
    );
    assert!(waivers(&tmp).is_err(), "an unread key was taken silently");

    // And the block with only the key it does take still reads.
    write(
        "\
taxonomy:
  package: acme/taxonomy
  version: 1.0.0
corpus:
  root: docs
conformance:
  waivers:
    - rule: pin.current
      reason: accepted_deviation
      owner: a team
      until: 2027-01-01
",
    );
    assert_eq!(waivers(&tmp).expect("waivers alone reads").len(), 1);

    let _ = std::fs::remove_dir_all(&tmp);
}

/// A waiver naming a rule the package does not declare ends the run. The report
/// that would list it has nothing to list it under.
#[test]
fn a_waiver_against_no_rule_ends_the_run() {
    let set = read(&source("lock.current"), "acme/taxonomy").expect("it reads");
    let identity = Identity {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        digest: None,
    };
    let refused = assemble(
        &set,
        &[waiver("acme.no_such_rule", "2027-01-01")],
        &identity,
        &[],
        at("2026-08-14"),
        no_pin(),
    )
    .expect_err("no rule of the set carries that name");
    assert_eq!(refused.len(), 1);
    assert!(refused[0].contains("acme.no_such_rule"));
    assert!(refused[0].contains("acme/taxonomy"));
}

// ---------------------------------------------------------------------------
// Expiry
// ---------------------------------------------------------------------------

/// The expiry is inclusive of the day it names, and the day after it covers
/// nothing. **Both arms are asserted**, because a comparison with the wrong
/// direction would pass an implementation that honored a waiver forever.
#[test]
fn a_waiver_stands_on_the_day_it_names_and_lapses_the_day_after() {
    let one = waiver("pin.current", "2027-02-28");
    assert!(one.live(at("2026-08-14")));
    assert!(one.live(at("2027-02-27")));
    assert!(one.live(at("2027-02-28")), "the day named is covered");
    assert!(!one.live(at("2027-03-01")), "the day after is not");
}

/// An expired waiver is reported and it covers nothing, so a gate that was green
/// goes red with no file edited.
#[test]
fn an_expired_waiver_is_reported_rather_than_honored() {
    let set = read(&source("lock.current"), "acme/taxonomy").expect("it reads");
    let identity = Identity {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        digest: None,
    };
    let taken = vec![
        ("lock.current".to_string(), Verdict::Gap("it moved".into())),
        ("corpus.classified".to_string(), Verdict::Met),
    ];
    let held = [waiver("lock.current", "2027-02-28")];

    let live =
        assemble(&set, &held, &identity, &taken, at("2027-02-28"), no_pin()).expect("it assembles");
    assert!(matches!(live.readings[0].cover, Cover::Live(_)));
    assert_eq!(live.gate("L0"), Ok(true), "a live waiver passes the gate");

    let lapsed =
        assemble(&set, &held, &identity, &taken, at("2027-03-01"), no_pin()).expect("it assembles");
    assert!(matches!(lapsed.readings[0].cover, Cover::Expired(_)));
    assert_eq!(
        lapsed.gate("L0"),
        Ok(false),
        "an expired waiver covers nothing"
    );
    // It is reported rather than dropped. A waiver that vanished on its expiry
    // would leave nobody a record of the deviation that just came back.
    assert!(lapsed
        .render(headwater_check::paint::ColorMode::Plain)
        .contains("EXPIRED on 2027-02-28"));
    assert!(lapsed
        .render(headwater_check::paint::ColorMode::Plain)
        .contains("it covers nothing"));
}

// ---------------------------------------------------------------------------
// The ladder, and what a waiver cannot buy
// ---------------------------------------------------------------------------

/// **A waiver moves the exit status and never the level.** The same verdicts are
/// assembled twice, once with a live waiver over the failing rule and once with
/// none, and the reported level is identical. Only the gate moves.
///
/// This is the test that answers "what stops a level from becoming a score an
/// adopter games". An implementation that counted a waived rule as met would
/// pass every other test in this file.
#[test]
fn a_live_waiver_moves_the_gate_and_never_the_level() {
    let set = read(&source("lock.current"), "acme/taxonomy").expect("it reads");
    let identity = Identity {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        digest: None,
    };
    let taken = vec![
        ("lock.current".to_string(), Verdict::Gap("it moved".into())),
        ("corpus.classified".to_string(), Verdict::Met),
    ];

    let bare =
        assemble(&set, &[], &identity, &taken, at("2026-08-14"), no_pin()).expect("it assembles");
    let waived = assemble(
        &set,
        &[waiver("lock.current", "2027-02-28")],
        &identity,
        &taken,
        at("2026-08-14"),
        no_pin(),
    )
    .expect("it assembles");

    assert_eq!(bare.reached, None);
    assert_eq!(
        waived.reached, None,
        "a waiver bought a rung that nothing earned"
    );
    assert!(!waived.levels[0].reached);
    assert_eq!(waived.levels[0].met, 0);
    assert_eq!(waived.levels[0].gaps, 1);
    assert_eq!(waived.levels[0].waived, 1);

    assert_eq!(bare.gate("L0"), Ok(false));
    assert_eq!(waived.gate("L0"), Ok(true), "a waiver buys the gate");
}

/// A rung is reached only when every rule it requires is met, and a rung
/// includes every rung below it. So closing the second rung's own rule while the
/// first rung's is open reaches nothing.
#[test]
fn a_rung_includes_every_rung_below_it() {
    let set = read(&source("lock.current"), "acme/taxonomy").expect("it reads");
    assert_eq!(set.cumulative(0), vec!["lock.current".to_string()]);
    assert_eq!(
        set.cumulative(1),
        vec!["lock.current".to_string(), "corpus.classified".to_string()]
    );

    let identity = Identity {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        digest: None,
    };
    let upper_only = vec![
        ("lock.current".to_string(), Verdict::Gap("it moved".into())),
        ("corpus.classified".to_string(), Verdict::Met),
    ];
    let report = assemble(
        &set,
        &[],
        &identity,
        &upper_only,
        at("2026-08-14"),
        no_pin(),
    )
    .expect("it assembles");
    assert!(!report.levels[0].reached);
    assert!(
        !report.levels[1].reached,
        "L1 requires L0's rule as well as its own"
    );
    assert_eq!(report.reached, None);

    let both = vec![
        ("lock.current".to_string(), Verdict::Met),
        ("corpus.classified".to_string(), Verdict::Met),
    ];
    let climbed =
        assemble(&set, &[], &identity, &both, at("2026-08-14"), no_pin()).expect("it assembles");
    assert_eq!(climbed.reached, Some("L1".to_string()));
}

/// A rule that no tree decides is never met, so a rung that names one is never
/// reached. That is the same guarantee as the waiver one, for the other kind of
/// rule the package may declare.
#[test]
fn a_rung_that_names_an_undecidable_rule_is_never_reached() {
    let text = "\
conformance:
  format: 1
  rules:
    - name: gates.required
      title: The gate is required
      decided_by: attestation
      statement: nothing here decides it
      remediation: record an attestation
  levels:
    - name: L4
      title: Gated
      rules: [gates.required]
";
    let set = read(text, "acme/taxonomy").expect("it reads");
    let identity = Identity {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        digest: None,
    };
    let taken = vec![(
        "gates.required".to_string(),
        Verdict::NotDecided("record an attestation".into()),
    )];
    let report =
        assemble(&set, &[], &identity, &taken, at("2026-08-14"), no_pin()).expect("it assembles");
    assert!(!report.levels[0].reached);
    assert_eq!(report.reached, None);
    assert_eq!(report.levels[0].undecided, 1);
    assert_eq!(report.levels[0].met, 0);

    // The rung says what it waits on, and the rule says it is neither met nor
    // missing. A rung reported only as "not reached" would read as a gap
    // somebody can close, and this one nobody can.
    let rendered = report.render(headwater_check::paint::ColorMode::Plain);
    assert!(rendered.contains("L4 Gated — not reached, 0 of 1 rules met"));
    assert!(rendered.contains("waits on an attestation record"));
    // The report fills every line of prose to `render::WIDTH`, so this sentence
    // arrives over two lines and `It is` sits at the end of the first one. The
    // fragment asserted here lies wholly inside the second line. **A `contains`
    // over a filled report holds the words and nothing about the shape** — the
    // hold on the shape is the recorded block in `tests/render.rs`, which is
    // where this assertion would have to move if the width moved.
    assert!(rendered.contains("neither met nor missing"));
    assert!(
        !rendered.contains("gap"),
        "an undecidable rule was reported as a gap"
    );
}

/// A `--level` run that names a rung the package does not declare is an error
/// that names the rungs that exist. A run that treated it as unreached would
/// tell a caller who mistyped a name that their repository failed.
#[test]
fn a_level_the_package_does_not_declare_is_an_error_that_names_the_ones_it_does() {
    let set = read(&source("lock.current"), "acme/taxonomy").expect("it reads");
    let identity = Identity {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        digest: None,
    };
    let report =
        assemble(&set, &[], &identity, &[], at("2026-08-14"), no_pin()).expect("it assembles");
    let refused = report.gate("L9").expect_err("there is no L9");
    assert!(refused.contains("L9"));
    assert!(refused.contains("`L0`"));
    assert!(refused.contains("`L1`"));
}

/// The report says on every run that a level is not a measurement, including a
/// run that reaches the top rung. A sentence printed only on a failure is one
/// that the reader who most needs it never sees.
#[test]
fn the_report_says_what_a_level_is_not_even_when_every_rung_is_reached() {
    let set = read(&source("lock.current"), "acme/taxonomy").expect("it reads");
    let identity = Identity {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        digest: None,
    };
    let all = vec![
        ("lock.current".to_string(), Verdict::Met),
        ("corpus.classified".to_string(), Verdict::Met),
    ];
    let rendered = assemble(&set, &[], &identity, &all, at("2026-08-14"), no_pin())
        .expect("it assembles")
        .render(headwater_check::paint::ColorMode::Plain);
    assert!(rendered.contains("L1 reached"));
    // Same reason as above: the disclaimer is filled, and the break now falls
    // between `the` and `corpus,`. The fragment asserted here is the first
    // sentence, which lies wholly inside the first line of the block.
    assert!(rendered.contains("a level states what this repository wired up."));
    assert!(rendered.contains("acme/taxonomy 1.0.0"));
}

// ---------------------------------------------------------------------------
// The rule set this repository ships
// ---------------------------------------------------------------------------

/// The package this repository publishes reads, and this engine holds a reading
/// for every tree rule in it. A package whose own engine refuses its rule set
/// would be a package nobody can evaluate against.
#[test]
fn the_rule_set_this_repository_ships_reads_against_this_engine() {
    let root = repository_root();
    let consumer =
        headwater_resolve::package::consumer(&root).expect("the consumer declaration reads");
    let set = headwater_conformance::at(&root, &consumer).expect("the shipped rule set reads");
    assert!(!set.rules.is_empty());
    assert!(!set.levels.is_empty());
    for rule in &set.rules {
        assert!(!rule.title.is_empty(), "{} states no title", rule.name);
        assert!(
            !rule.remediation.is_empty(),
            "{} states no remediation, so a gap on it tells nobody what to do",
            rule.name
        );
        if rule.decided_by == DecidedBy::Tree {
            assert!(
                headwater_conformance::READINGS.contains(&rule.name.as_str()),
                "{} declares a tree rule this engine cannot read",
                rule.name
            );
        }
    }
}

/// Every waiver this repository declares names a rule the shipped set declares,
/// and every one carries an owner and an expiry. This is the arm that catches a
/// waiver committed against a rule that a later release renamed.
#[test]
fn every_waiver_this_repository_declares_names_a_shipped_rule() {
    let root = repository_root();
    let consumer =
        headwater_resolve::package::consumer(&root).expect("the consumer declaration reads");
    let set = headwater_conformance::at(&root, &consumer).expect("the shipped rule set reads");
    let held = waivers(&root).expect("the waivers read");
    for waiver in &held {
        assert!(
            set.rule(&waiver.rule).is_some(),
            "the waiver on `{}` names a rule {} does not declare",
            waiver.rule,
            consumer.package
        );
        assert!(!waiver.owner.is_empty());
    }
}

// ---------------------------------------------------------------------------
// #366's follow-up: a diverged installed package is not a rule set to trust
// ---------------------------------------------------------------------------

/// A scratch package under `.headwater/packages/`, with a manifest, a taxonomy source and
/// a conformance rule set naming two rules on one level — `lock.current` and
/// `corpus.classified`, both real readings — and a real release record over
/// all three files, computed the way `taxonomy publish` computes one.
fn diverged_package_root(label: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "headwater-conformance-rules-{}-{label}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join(".headwater/packages/acme-taxonomy");
    std::fs::create_dir_all(&dir).expect("the package directory");
    std::fs::write(
        dir.join("package.yml"),
        "package: acme/taxonomy\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  \
         conformance: conformance.yml\n",
    )
    .expect("the manifest");
    std::fs::write(
        dir.join("taxonomy.yml"),
        "taxonomy: acme/taxonomy\nversion: 1.0.0\n",
    )
    .expect("a source");
    std::fs::write(
        dir.join("conformance.yml"),
        "conformance:\n  format: 1\n  rules:\n    - name: lock.current\n      title: The lock is \
         current\n      decided_by: tree\n      statement: it is\n      remediation: run resolve\n    \
         - name: corpus.classified\n      title: Everything is classified\n      decided_by: tree\n      \
         statement: it is\n      remediation: declare a kind\n  levels:\n    - name: L0\n      title: \
         Pointed at\n      rules: [lock.current, corpus.classified]\n",
    )
    .expect("the rule set");

    let members = headwater_resolve::release::members(&dir).expect("the members");
    let release = headwater_resolve::release::Release {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        requires_engine: None,
        digest: headwater_resolve::release::digest_of(&members),
        members,
    };
    std::fs::write(
        dir.join(headwater_resolve::release::RECORD),
        headwater_resolve::release::render(&release),
    )
    .expect("the record");
    (root, dir)
}

fn taxonomy_consumer() -> headwater_resolve::package::Consumer {
    headwater_resolve::package::Consumer {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        bundles: Vec::new(),
        digest: None,
        overlay: None,
        corpus_root: "docs".to_string(),
        exclusions: Vec::new(),
    }
}

/// **The plain case: any hand edit to a vendored `conformance.yml` is refused
/// before a single rule is read.**
#[test]
fn a_hand_edited_conformance_file_is_refused_before_any_rule_is_read() {
    let (root, dir) = diverged_package_root("plain-edit");
    let consumer = taxonomy_consumer();
    assert!(
        headwater_conformance::at(&root, &consumer).is_ok(),
        "the freshly published rule set does not read"
    );

    let path = dir.join("conformance.yml");
    let mut text = std::fs::read_to_string(&path).expect("it reads");
    text.push_str("\n# hand-edited, never through `vendor`\n");
    std::fs::write(&path, text).expect("the hand edit writes");

    let refused = headwater_conformance::at(&root, &consumer)
        .expect_err("a diverged rule set must not read as one this run can trust");
    assert!(matches!(refused, SetError::Diverged(_)), "{refused}");
    let _ = std::fs::remove_dir_all(&root);
}

/// **The kill shot: stripping a rule out of the very level it would have
/// reported the tamper under does not let the level pass, because the whole
/// file is refused before any level is read at all.** Before `at` checked
/// divergence, this edit would have narrowed `L0` to `corpus.classified`
/// alone, and a run over the narrowed level would never have asked
/// `lock.current` anything.
#[test]
fn stripping_a_rule_out_of_its_own_level_does_not_let_the_level_pass() {
    let (root, dir) = diverged_package_root("kill-shot");
    let consumer = taxonomy_consumer();

    let path = dir.join("conformance.yml");
    let text = std::fs::read_to_string(&path).expect("it reads");
    assert!(text.contains("rules: [lock.current, corpus.classified]"));
    let narrowed = text.replace(
        "rules: [lock.current, corpus.classified]",
        "rules: [corpus.classified]",
    );
    std::fs::write(&path, narrowed).expect("the narrowed level writes");

    let refused = headwater_conformance::at(&root, &consumer)
        .expect_err("a level narrowed inside a diverged file must not be read as trustworthy");
    assert!(
        matches!(refused, SetError::Diverged(_)),
        "the refusal is not the divergence this test provoked: {refused}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The format guard on a rule set is a string inequality on the raw token. It
/// establishes what the file says, and never who published it or when. An
/// earlier token reaches the same arm as a later one, and a hand edit reaches it
/// with no engine anywhere near the file, so the message may name no party. The
/// token that mutates here is **lower** than the one this engine reads, which is
/// the case a message about a newer publisher gets exactly backwards.
#[test]
fn an_earlier_format_rule_set_is_refused_without_naming_a_publisher() {
    let text = source("lock.current");
    let earlier = text.replace("format: 1", "format: 0");
    assert_ne!(earlier, text, "the mutation substituted nothing:\n{text}");

    let refused =
        read(&earlier, "acme/taxonomy").expect_err("format 0 is not the format this engine reads");
    let message = refused.to_string();
    assert!(matches!(refused, SetError::Format { .. }), "{message}");

    let lowered = message.to_lowercase();
    assert!(
        !lowered.contains("newer") && !lowered.contains("older"),
        "the message names a direction the guard never compared: {message}"
    );
    assert!(
        !lowered.contains("published it") && !lowered.contains("wrote it"),
        "the message names a party the guard never read: {message}"
    );
    assert!(
        message.contains("`0`"),
        "the message drops the token it found: {message}"
    );
    assert!(
        message.contains("requires_engine"),
        "the message names no remedy: {message}"
    );
}

// ---------------------------------------------------------------------------
// #415's residual: the header prints a digest, and says whether it read it
// ---------------------------------------------------------------------------

/// **A digest the run never read is printed as one nobody read.**
///
/// `diverged_package_root` publishes a package declaring `lock.current` and
/// `corpus.classified` and **no `pin.current`**, which is what an adopter who
/// forks the rule set to drop a rule they have not wired up is left with. The
/// consumer below pins a digest that names no artifact anywhere.
///
/// Before this, the header printed that digest and the report said nothing
/// about it at all — over the whole rendered report,
/// `grep -ci 'digest\|release record\|pin'` returned 0 — and deleting the
/// release record out of the package left the report **byte-identical**. Both
/// halves of that symptom are asserted here, the second by rendering the same
/// report twice over a directory that loses its record between the two.
///
/// Nothing about a rung moves: `L0` reads `lock.current` and
/// `corpus.classified`, both met on both sides, and both reports still reach it.
#[test]
fn a_pinned_digest_that_no_rule_reads_is_printed_as_one_nothing_checked() {
    let (root, dir) = diverged_package_root("unchecked-pin");
    let mut consumer = taxonomy_consumer();
    let names_nothing =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string();
    consumer.digest = Some(names_nothing.clone());

    let set = headwater_conformance::at(&root, &consumer).expect("the rule set reads");
    assert!(
        set.rule("pin.current").is_none(),
        "the case stopped being the one it was written for: this set does read the pin"
    );
    let declared = headwater_resolve::release::at(&dir)
        .expect("the freshly published record")
        .digest;
    assert_ne!(
        declared, names_nothing,
        "the pin matches the installed artifact, so there is nothing unchecked here"
    );

    let taken = [
        ("lock.current".to_string(), Verdict::Met),
        ("corpus.classified".to_string(), Verdict::Met),
    ];
    let with_record = assemble(
        &set,
        &[],
        &Identity::of(&consumer),
        &taken,
        at("2026-08-14"),
        pin_check(&set, &root, &consumer),
    )
    .expect("it assembles");
    assert_eq!(
        with_record.reached.as_deref(),
        Some("L0"),
        "the rung this change must not move has moved"
    );
    let with_record = with_record.render(headwater_check::paint::ColorMode::Plain);

    assert!(
        with_record.contains(&names_nothing),
        "the header stopped printing the pinned digest:\n{with_record}"
    );
    assert!(
        collapsed(&with_record).contains("not checked by any rule of this set"),
        "the header prints a digest and says nothing about whether the run read it:\n\
         {with_record}"
    );
    assert!(
        with_record.contains(&declared),
        "and it does not print the digest the installed artifact declares of itself:\n\
         {with_record}"
    );

    // The second half. `drop-norecord` against `drop-clean`: a package that
    // carries no release record at all, and therefore no published artifact
    // behind it, once rendered a report byte-identical to the one above.
    std::fs::remove_file(dir.join(headwater_resolve::release::RECORD)).expect("the record goes");
    let without_record = assemble(
        &set,
        &[],
        &Identity::of(&consumer),
        &taken,
        at("2026-08-14"),
        pin_check(&set, &root, &consumer),
    )
    .expect("it assembles")
    .render(headwater_check::paint::ColorMode::Plain);

    assert_ne!(
        with_record, without_record,
        "a package with a release record and one with none render the same report"
    );
    assert!(
        collapsed(&without_record).contains(
            "the installed package carries no release record, so no published artifact \
                       stands behind it"
        ),
        "the report does not say that no published artifact stands behind the package:\n\
         {without_record}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The other state of the same line, and the two things that decide it.
///
/// A set that declares `pin.current` as a **tree** rule has a reading of the pin
/// and the header names it. Nothing is read off disk to answer that, which the
/// root below holds: it does not exist. Declaring the same name under
/// `decided_by: attestation` is not a reading — [`headwater_conformance::reading`]
/// is never called for one — so the header says the pin was not checked, and
/// over a root with no package on it that is all it can say.
#[test]
fn the_header_names_the_rule_that_read_the_pin_and_reads_no_tree_to_say_so() {
    let nowhere = Path::new("/nonexistent-headwater-415");
    let mut consumer = taxonomy_consumer();
    consumer.digest = Some("sha256:abc".to_string());

    let reads_it = read(&source("pin.current"), "acme/taxonomy").expect("it reads");
    assert_eq!(
        pin_check(&reads_it, nowhere, &consumer),
        PinCheck::By("pin.current".to_string()),
        "a declared tree reading of the pin is what the header names"
    );
    let rendered = assemble(
        &reads_it,
        &[],
        &Identity::of(&consumer),
        &[("pin.current".to_string(), Verdict::Met)],
        at("2026-08-14"),
        pin_check(&reads_it, nowhere, &consumer),
    )
    .expect("it assembles")
    .render(headwater_check::paint::ColorMode::Plain);
    assert!(
        collapsed(&rendered)
            .contains("checked against the installed release record by `pin.current`"),
        "the header does not name the rule that read the pin:\n{rendered}"
    );

    let attested = source("pin.current").replace(
        "    - name: pin.current\n      title: The lock is current\n      decided_by: tree",
        "    - name: pin.current\n      title: The lock is current\n      decided_by: attestation",
    );
    let attested = read(&attested, "acme/taxonomy").expect("it reads");
    assert_eq!(
        attested.rule("pin.current").expect("the rule").decided_by,
        DecidedBy::Attestation,
        "the substitution did nothing"
    );
    assert_eq!(
        pin_check(&attested, nowhere, &consumer),
        PinCheck::Unchecked(Installed::Absent),
        "a rule no tree decides read the pin"
    );
}
