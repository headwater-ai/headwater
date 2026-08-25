// SPDX-License-Identifier: Apache-2.0
//! A re-import at a moved revision updates the one entry it already wrote, and
//! does not append a second one beside it. #384.
//!
//! `import::write::declares` reads "same target, a different revision" as
//! absent on purpose — that is what routes a moved-revision edge into
//! [`headwater_import::Plan::to_write`] at all, so a re-import over a moved
//! pin still has something to write. What this file pins down is what happens
//! once that edge reaches the splice: the one entry for `12345` updates in
//! place, at `verified_revision: 8`, rather than sitting beside the
//! `verified_revision: 7` entry the first import wrote.

mod chain;

use chain::{declaration, import, publish, tree, Scratch, PAYLOAD};

/// The document the import writes its edge into.
const DECLARING: &str = "docs/spec/00-first.md";

/// Advance the snapshot's item to a second revision, republish the record over
/// the new bytes, and answer with the digest a declaration now has to pin.
///
/// Copied from `drift.rs::advance` rather than shared: that file does not
/// export it, and the three lines are the whole of what this file needs from
/// it.
fn advance(scratch: &Scratch) -> String {
    scratch.write(
        "imports/ado/snapshot.yml",
        &PAYLOAD.replace("revision: \"7\"", "revision: \"8\""),
    );
    publish(scratch)
}

/// Every entry of `audited_by` in the declaring document that targets `12345`,
/// with the `verified_revision` each one carries.
fn entries_for_12345(text: &str) -> Vec<Option<String>> {
    let parsed = headwater_doc::parse(text).expect("the document still parses");
    let items = parsed
        .facets
        .get("relations")
        .and_then(|block| block.value.as_map())
        .and_then(|block| block.get("audited_by"))
        .and_then(|targets| targets.value.as_seq())
        .expect("the relation is declared");

    items
        .iter()
        .filter_map(|item| {
            let map = item.value.as_map()?;
            let to = map.get("to")?.value.as_scalar()?;
            if to.text != "12345" {
                return None;
            }
            let revision = map
                .get("verified_revision")
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone());
            Some(revision)
        })
        .collect()
}

/// **The decisive case.** A re-import after the snapshot's revision moved
/// leaves exactly one entry for the target, at the new revision — not two.
#[test]
fn a_reimport_at_a_moved_revision_replaces_the_edge_rather_than_duplicating_it() {
    let scratch = Scratch::new("reimport-moved-revision");
    let digest = tree(&scratch);
    let pinned = declaration(&digest);
    import(&scratch, &pinned);

    let text = std::fs::read_to_string(scratch.path().join(DECLARING)).expect("it is there");
    let before = entries_for_12345(&text);
    assert_eq!(
        before,
        vec![Some("7".to_string())],
        "the first import should write exactly one entry at revision 7: {text}"
    );

    let advanced = declaration(&advance(&scratch));
    import(&scratch, &advanced);

    let text = std::fs::read_to_string(scratch.path().join(DECLARING)).expect("it is there");
    let after = entries_for_12345(&text);
    assert_eq!(
        after,
        vec![Some("8".to_string())],
        "a re-import at a moved revision must replace the entry, not duplicate it: {text}"
    );
}

/// **Done-when clause 2, confirmed explicitly.** A second import over an
/// *unchanged* snapshot writes nothing — `import::write::declares` already
/// reads that edge as present, so no edge reaches the splice at all and the
/// document on disk is byte-identical before and after.
#[test]
fn a_reimport_over_an_unchanged_snapshot_writes_nothing() {
    let scratch = Scratch::new("reimport-unchanged");
    let digest = tree(&scratch);
    let pinned = declaration(&digest);
    import(&scratch, &pinned);

    let before = std::fs::read_to_string(scratch.path().join(DECLARING)).expect("it is there");

    // Same digest, same declaration: nothing moved upstream. `plan.edges`
    // still lists the edge the snapshot pins — that is `Plan`'s report of
    // every link, present or not — but `to_write()` is what a run actually
    // splices, and `declares` must have read this one as already there.
    let plan = import(&scratch, &pinned);
    assert_eq!(
        plan.to_write().len(),
        0,
        "an unchanged snapshot has nothing left to write: {:#?}",
        plan.edges
    );

    let after = std::fs::read_to_string(scratch.path().join(DECLARING)).expect("it is there");
    assert_eq!(
        before, after,
        "a re-import over an unchanged snapshot must write nothing"
    );
}
