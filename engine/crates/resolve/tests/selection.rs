// SPDX-License-Identifier: Apache-2.0
//! An incomplete bundle selection is told which bundle would complete it.
//!
//! The library here is synthetic on purpose. Nothing in these cases comes from
//! `headwater/standard`, from `docs/taxonomies/` or from any name this
//! repository ships, so what they measure is the derivation and not the
//! library. The one case that does read this repository's own package is named
//! for it and asserts over bundle names rather than over a version.
//!
//! Every case names its own scratch directory. `Scratch` keys the temporary
//! tree on the process id, `cargo` runs a target's cases as threads of one
//! process, and two cases sharing a name meet each other's `remove_dir_all` as
//! a `NotFound` out of `fs::copy` that reads like a missing fixture.

use headwater_resolve::selection;
use headwater_resolve::Consumer;
use std::path::PathBuf;

struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let at =
            std::env::temp_dir().join(format!("headwater-selection-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Self(at)
    }

    fn write(&self, relative: &str, text: &str) {
        let at = self.0.join(relative);
        std::fs::create_dir_all(at.parent().expect("the file has a parent"))
            .expect("the parent is made");
        std::fs::write(at, text).expect("the file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const PACKAGE: &str = "\
package: acme/fixture
version: 1.0.0
contents:
  taxonomy: taxonomy.yml
  bundles: bundles
";

const TAXONOMY: &str = "\
taxonomy: acme/fixture
version: 1.0.0
purposes:
  behavior: {intent: state what the system does}
  record: {intent: state what happened}
kinds:
  governed_document: {abstract: true}
  specification: {is_a: governed_document, purpose: behavior}
  note: {is_a: governed_document, purpose: record}
core:
  requires:
    - purpose: behavior
";

/// Reads `stewardship`, which nothing in the base declares.
const ALPHA: &str = "\
bundle: alpha
extends: acme/fixture@1.0.0
requires: []
add:
  kinds.alpha: {is_a: governed_document, purpose: stewardship}
";

/// Declares `stewardship`. This is the answer every positive case is about.
const BETA: &str = "\
bundle: beta
extends: acme/fixture@1.0.0
requires: []
add:
  purposes.stewardship: {intent: state who tends the corpus}
";

/// Declares nothing `alpha` reads. `gamma` is the discriminator: a message that
/// names it is a message that names any bundle at all rather than the one that
/// helps.
const GAMMA: &str = "\
bundle: gamma
extends: acme/fixture@1.0.0
requires: []
add:
  purposes.custody: {intent: state who holds the record}
  kinds.gamma: {is_a: governed_document, purpose: custody}
";

/// Reads `provenance`, which no bundle in this library declares.
const DELTA: &str = "\
bundle: delta
extends: acme/fixture@1.0.0
requires: []
add:
  kinds.delta: {is_a: governed_document, purpose: provenance}
";

/// Collides with the base, so a resolution that includes it fails for its own
/// reason. A candidate that cannot resolve is not advice.
const EPSILON: &str = "\
bundle: epsilon
extends: acme/fixture@1.0.0
requires: []
add:
  purposes.behavior: {intent: a second declaration of a name the base holds}
";

/// Reads one name `beta` declares and one name `gamma` declares, so no single
/// bundle completes a selection that holds it.
const ETA: &str = "\
bundle: eta
extends: acme/fixture@1.0.0
requires: []
add:
  kinds.eta_one: {is_a: governed_document, purpose: stewardship}
  kinds.eta_two: {is_a: governed_document, purpose: custody}
";

/// Removes a purpose the base declares, which `kinds.note` reads as a bare
/// string and no bundle in the library re-declares. Nothing can be added to
/// complete this.
///
/// The purpose removed is not one `core.requires` names, deliberately: removing
/// that one fails the core requirement inside `resolve` itself, so the refusal
/// under test would never be a dangling name.
const REMOVES: &str = "\
taxonomy: acme/adopter
extends: acme/fixture@1.0.0

remove:
  - purposes.record
";

/// Reads `stewardship` from the adopter's own overlay rather than from a
/// bundle. `beta` still supplies it, and only if the candidate is resolved
/// *before* the overlay rather than after it.
const READS: &str = "\
taxonomy: acme/adopter
extends: acme/fixture@1.0.0

add:
  kinds.house: {is_a: governed_document, purpose: stewardship}
";

fn library(scratch: &Scratch) {
    scratch.write("packages/acme-fixture/package.yml", PACKAGE);
    scratch.write("packages/acme-fixture/taxonomy.yml", TAXONOMY);
    scratch.write("packages/acme-fixture/bundles/alpha/bundle.yml", ALPHA);
    scratch.write("packages/acme-fixture/bundles/beta/bundle.yml", BETA);
    scratch.write("packages/acme-fixture/bundles/gamma/bundle.yml", GAMMA);
    scratch.write("packages/acme-fixture/bundles/delta/bundle.yml", DELTA);
    scratch.write("packages/acme-fixture/bundles/epsilon/bundle.yml", EPSILON);
    scratch.write("packages/acme-fixture/bundles/eta/bundle.yml", ETA);
}

fn consumer(bundles: &[&str], overlay: Option<&str>) -> Consumer {
    Consumer {
        package: "acme/fixture".to_string(),
        version: "1.0.0".to_string(),
        bundles: bundles.iter().map(|name| name.to_string()).collect(),
        digest: None,
        overlay: overlay.map(|path| path.to_string()),
        corpus_root: "docs".to_string(),
        exclusions: Vec::new(),
    }
}

/// The refusal a selection takes, as a reader meets it.
fn refusal(scratch: &Scratch, consumer: &Consumer) -> String {
    headwater_resolve::render_errors(&resolved(scratch, consumer).validate())
}

fn resolved(scratch: &Scratch, consumer: &Consumer) -> headwater_resolve::Resolution {
    let sources = headwater_resolve::package::sources(&scratch.0, consumer)
        .expect("the declared selection resolves its sources");
    headwater_resolve::resolve(&sources).expect("the declared selection resolves")
}

#[test]
fn a_selection_that_omits_a_bundle_another_bundle_needs_is_told_which_bundle_to_add() {
    let scratch = Scratch::new("omits");
    library(&scratch);
    let consumer = consumer(&["alpha"], None);

    let refused = refusal(&scratch, &consumer);
    assert!(
        refused.contains("reads `stewardship`, and no purpose of that name is declared"),
        "the case reproduces the refusal it is about: {refused}"
    );

    let advice = selection::advice(&scratch.0, &consumer).expect("a bundle in the library helps");
    assert_eq!(advice.dangling, 1);
    assert_eq!(advice.bundles.len(), 1, "one bundle helps, not two");
    assert_eq!(advice.bundles[0].bundle, "beta");
    assert_eq!(advice.bundles[0].names, vec!["stewardship".to_string()]);

    let text = advice.render();
    assert!(
        text.contains("beta"),
        "the message names the bundle: {text}"
    );
    assert!(
        !text.contains("gamma"),
        "the message names no bundle that does not help: {text}"
    );
    assert!(
        !text.contains("epsilon"),
        "the message names no bundle that does not resolve: {text}"
    );
    assert!(
        text.contains("stewardship"),
        "the message is about the dangling name: {text}"
    );
    assert!(
        text.contains("1 of the 1"),
        "the message counts what the bundle supplies: {text}"
    );
    assert!(
        text.contains("acme/fixture"),
        "the message names the package: {text}"
    );
    assert!(
        text.contains(".headwater/taxonomy.yml"),
        "the message names the file to edit: {text}"
    );
}

#[test]
fn a_candidate_that_does_not_resolve_is_skipped_and_the_refusal_is_unchanged() {
    let scratch = Scratch::new("skipped");
    library(&scratch);
    let consumer = consumer(&["alpha"], None);

    // `epsilon` collides with the base. The advice path resolves it as a
    // candidate and must drop what that resolution returns rather than let it
    // reach the reader in place of the refusal the reader asked about.
    let before = refusal(&scratch, &consumer);
    let advice = selection::advice(&scratch.0, &consumer).expect("a bundle in the library helps");
    let after = refusal(&scratch, &consumer);

    assert_eq!(before, after, "the advice path leaves the refusal alone");
    assert!(
        !before.contains("epsilon"),
        "the refusal is the one under test"
    );
    assert!(!advice.render().contains("collides"));
    assert_eq!(advice.bundles.len(), 1);
    assert_eq!(advice.bundles[0].bundle, "beta");
}

#[test]
fn a_selection_no_bundle_in_the_library_can_complete_names_no_bundle() {
    let scratch = Scratch::new("nobundle");
    library(&scratch);
    let consumer = consumer(&["delta"], None);

    let before = refusal(&scratch, &consumer);
    assert!(
        before.contains("reads `provenance`, and no purpose of that name is declared"),
        "the case reproduces a dangling-name refusal: {before}"
    );

    assert_eq!(
        selection::advice(&scratch.0, &consumer),
        None,
        "no bundle declares `provenance`, so the message names none"
    );

    let after = refusal(&scratch, &consumer);
    assert_eq!(before, after, "the refusal is unchanged");
}

#[test]
fn a_purpose_the_consumers_own_overlay_removed_names_no_bundle() {
    let scratch = Scratch::new("removes");
    library(&scratch);
    scratch.write("adopter.yml", REMOVES);
    let consumer = consumer(&[], Some("adopter.yml"));

    let before = refusal(&scratch, &consumer);
    assert!(
        before.contains("reads `record`, and no purpose of that name is declared"),
        "the case reproduces the refusal it is about: {before}"
    );

    assert_eq!(
        selection::advice(&scratch.0, &consumer),
        None,
        "no bundle re-declares what the overlay removed, so the message names none"
    );

    let after = refusal(&scratch, &consumer);
    assert_eq!(before, after, "the refusal is unchanged");
}

#[test]
fn a_name_the_consumers_own_overlay_reads_is_told_which_bundle_declares_it() {
    let scratch = Scratch::new("reads");
    library(&scratch);
    scratch.write("adopter.yml", READS);
    let consumer = consumer(&[], Some("adopter.yml"));

    // The candidate is resolved between the selected bundles and the overlay,
    // which is the order `package::selected` fixes for a real selection. A
    // candidate resolved after the overlay would answer a question no consumer
    // can ask.
    let advice = selection::advice(&scratch.0, &consumer).expect("a bundle in the library helps");
    assert_eq!(advice.bundles.len(), 1);
    assert_eq!(advice.bundles[0].bundle, "beta");
    assert_eq!(advice.bundles[0].names, vec!["stewardship".to_string()]);
}

#[test]
fn no_bundle_that_supplies_part_of_what_is_missing_is_presented_as_sufficient() {
    let scratch = Scratch::new("part");
    library(&scratch);
    let consumer = consumer(&["eta"], None);

    let advice = selection::advice(&scratch.0, &consumer).expect("two bundles in the library help");
    assert_eq!(advice.dangling, 2);
    assert_eq!(advice.supplied, 2);
    assert_eq!(advice.bundles.len(), 2, "no one bundle completes this");
    assert_eq!(advice.bundles[0].bundle, "beta");
    assert_eq!(advice.bundles[0].names, vec!["stewardship".to_string()]);
    assert_eq!(advice.bundles[1].bundle, "gamma");
    assert_eq!(advice.bundles[1].names, vec!["custody".to_string()]);

    let text = advice.render();
    assert!(
        text.contains("Bundles `acme/fixture` ships that this repository did not select declare 2"),
        "the message is plural and counts the union: {text}"
    );
    assert!(
        text.contains("`beta` declares 1 of the 2: `stewardship`"),
        "each bundle is named with what it alone supplies: {text}"
    );
    assert!(
        text.contains("`gamma` declares 1 of the 2: `custody`"),
        "each bundle is named with what it alone supplies: {text}"
    );
}

#[test]
fn a_selection_that_resolves_is_told_nothing() {
    let scratch = Scratch::new("complete");
    library(&scratch);
    let consumer = consumer(&["alpha", "beta"], None);

    let taxonomy = resolved(&scratch, &consumer).taxonomy;
    assert!(
        headwater_resolve::rules::dangling(&taxonomy).is_empty(),
        "the selection leaves no name dangling"
    );
    assert_eq!(selection::advice(&scratch.0, &consumer), None);
}

/// The corroboration, over this repository's own package.
///
/// It pins bundle names and the dangling names they supply, never a version:
/// the version moves and the closure does not. `find_version` is what keeps the
/// consumer's pin honest without this file carrying a copy of it.
#[test]
fn the_shipped_triple_is_told_to_add_evidence_and_obligation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let Some(version) = headwater_resolve::package::find_version(&root, "headwater/standard")
    else {
        return;
    };

    let mut consumer = consumer(&["design-spec", "decision-record", "standards-spec"], None);
    consumer.package = "headwater/standard".to_string();
    consumer.version = version;

    let advice = selection::advice(&root, &consumer)
        .expect("the shipped triple is an incomplete selection, and a shipped bundle completes it");
    let text = advice.render();
    assert!(
        text.contains("evidence-and-obligation"),
        "the message names the bundle that declares the dangling names: {text}"
    );
    assert!(
        !text.contains("brd-prd"),
        "the message names no bundle that does not help: {text}"
    );
    assert!(
        !text.contains("diataxis"),
        "the message names no bundle that does not help: {text}"
    );
}
